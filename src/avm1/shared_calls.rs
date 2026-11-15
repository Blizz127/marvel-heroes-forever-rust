//! Opted-in same-Environment calls use one operand/register store.
//!
//! Static contract: docs/SAVED_SCRIPT_CALLS.md. Supplied selected-Environment
//! witnesses are conditional inputs, never discoveries from receiver names or
//! function source. Generic diagnostic calls retain their independent gate.
use super::*;
use std::collections::BTreeSet;

/// Explicit current native-state assertion for this bounded call profile.
/// It supplies neither a fresh-instance claim nor callback/reentry clearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavedScriptCallProfile {
    EmptyDynamicNoPendingException,
}

#[derive(Clone)]
pub(super) struct SharedCallContext {
    pub(super) target: ObjectId,
    pub(super) bindings: Arc<BTreeSet<ObjectId>>,
}

impl Vm {
    /// Admit calls only after the caller supplies the current empty dynamic bank
    /// and absence of pending native exception cleanup. Existing snapshots do
    /// not imply either fact. This is one-way and rejected after a body failure.
    pub fn enable_saved_script_calls(
        &mut self,
        environment: SavedEnvironmentId,
        profile: SavedScriptCallProfile,
    ) -> Result<()> {
        self.saved_idle()?;
        let env = self.saved_environment(environment)?;
        if env.halted || env.script_profile.is_some() || !env.dynamic.is_empty() {
            return self.fail(ErrorKind::Unsupported("saved script call profile state"));
        }
        self.saved_owner_profile(env.state.owner())?;
        self.saved_environments[environment.0]
            .as_mut()
            .unwrap()
            .script_profile = Some(profile);
        Ok(())
    }

    /// Supply the selected native Environment identity for this exact function.
    /// Structural target/scope checks constrain that assertion; they do not
    /// establish it from a method name or captured source movie alone.
    pub fn bind_saved_script_environment(
        &mut self,
        environment: SavedEnvironmentId,
        callable: ObjectId,
    ) -> Result<()> {
        self.saved_idle()?;
        let env = self.saved_environment(environment)?;
        if env.halted || env.script_profile.is_none() {
            return self.fail(ErrorKind::Unsupported(
                "saved script call profile not enabled",
            ));
        }
        self.check_shared_function(callable, env.state.target())?;
        let env = self.saved_environments[environment.0].as_mut().unwrap();
        Arc::make_mut(&mut env.script_bindings).insert(callable);
        Ok(())
    }

    /// Retained diagnostic tail after an unsupported body stops. A halted
    /// Environment cannot resume or reinterpret this as a normal native return.
    pub fn saved_environment_dynamic_registers(
        &self,
        environment: SavedEnvironmentId,
    ) -> Result<&[Value]> {
        Ok(&self.saved_environment(environment)?.dynamic)
    }

    fn check_shared_function(&self, id: ObjectId, target: ObjectId) -> Result<ScriptFunction> {
        self.check_object(id)?;
        let Kind::Script(function) = &self.objects[id.0].kind else {
            return self.fail(ErrorKind::Unsupported(
                "shared call requires ScriptFunction",
            ));
        };
        if function.target != target || function.scopes.as_slice() != [target, self.global] {
            return self.fail(ErrorKind::Unsupported(
                "shared call selected Environment/capture profile",
            ));
        }
        let instruction = self.code[function.code.0]
            .instruction_at(function.offset)
            .ok_or_else(|| self.error(ErrorKind::InvalidCode(function.code)))?;
        let Operands::Function(definition) = &instruction.operands else {
            return self.fail(ErrorKind::InvalidCode(function.code));
        };
        let (Some(count), Some(flags)) = (definition.register_count, definition.flags) else {
            return self.fail(ErrorKind::Unsupported(
                "shared type1 arguments object profile",
            ));
        };
        // No arguments graph, Super, root/parent/global preload or conflicting
        // flags is invented by this first setup profile.
        if !matches!(flags, 0x28 | 0x29 | 0x2a) {
            return self.fail(ErrorKind::Unsupported("shared Function2 setup flags"));
        }
        if flags & 1 != 0 && count <= 1 {
            return self.fail(ErrorKind::InvalidRegister(1));
        }
        for parameter in &definition.parameters {
            if let Some(index) = parameter.register.filter(|index| *index != 0) {
                if index >= count {
                    return self.fail(ErrorKind::InvalidRegister(index));
                }
            }
        }
        Ok(function.clone())
    }

    pub(super) fn shared_call_method(
        &mut self,
        frame: &mut Frame,
        host: &mut impl Host,
    ) -> Result<()> {
        let Some(context) = frame.shared_calls.clone() else {
            return self.fail(ErrorKind::Unsupported("saved shared call profile not enabled"));
        };
        if self.current_target != Some(context.target) || frame.target != context.target {
            return self.fail(ErrorKind::Unsupported("shared caller Environment identity"));
        }
        let Value::String(key) = self.peek(frame, 0)? else {
            return self.fail(ErrorKind::Unsupported(
                "shared named CallMethod String name",
            ));
        };
        self.shared_local_name(&key)?;
        let receiver = self.peek(frame, 1)?;
        let Value::Object(owner) = receiver else {
            return self.fail(ErrorKind::Unsupported(
                "shared named CallMethod ordinary receiver",
            ));
        };
        self.check_object(owner)?;
        if !matches!(self.objects[owner.0].kind, Kind::Plain) {
            return self.fail(ErrorKind::Unsupported(
                "shared named CallMethod ordinary receiver",
            ));
        }
        // Exact existing own data; no lazy materialization, getter or resolver.
        let Some(Property::Data(Value::Object(id))) = self.objects[owner.0].properties.get(&key)
        else {
            return self.fail(ErrorKind::Unsupported(
                "shared method requires own data function",
            ));
        };
        let id = *id;
        if !context.bindings.contains(&id) {
            return self.fail(ErrorKind::Unsupported(
                "shared function Environment is not bound",
            ));
        }
        let function = self.check_shared_function(id, context.target)?;
        let Value::Number(count) = self.peek(frame, 2)? else {
            return self.fail(ErrorKind::Unsupported(
                "shared CallMethod exact Number count",
            ));
        };
        if !count.is_finite() || count < 0.0 || count.fract() != 0.0 {
            return self.fail(ErrorKind::InvalidArgumentCount);
        }
        if count > self.limits.arguments as f64 {
            return self.fail(ErrorKind::Budget("arguments"));
        }
        let count = count as usize;
        if count.saturating_add(3) > frame.store.borrow().len() {
            return self.fail(ErrorKind::StackUnderflow);
        }
        if !frame.store.borrow().dynamic.is_empty() {
            return self.fail(ErrorKind::Unsupported(
                "shared Function2 outer register allocation",
            ));
        }
        if self.depth >= self.limits.call_depth {
            return self.fail(ErrorKind::Budget("function call depth"));
        }
        let Kind::AttachedOwner(clip) = self.objects[context.target.0].kind else {
            return self.fail(ErrorKind::Unsupported("shared selected Environment owner"));
        };
        let (_, flags) = self.saved_owner_profile(clip)?;
        if flags & 0x10 != 0 {
            return self.fail(ErrorKind::Unsupported(
                "shared selected Environment target bit4",
            ));
        }
        let mut args = Vec::with_capacity(count);
        for index in 0..count {
            args.push(self.peek(frame, index + 3)?);
        }
        let program = Arc::clone(&self.code[function.code.0]);
        let Operands::Function(definition) =
            &program.instruction_at(function.offset).unwrap().operands
        else {
            unreachable!()
        };
        let activation = self.object()?;
        let bank = frame
            .store
            .borrow_mut()
            .enter_function2(definition.register_count.unwrap())
            .map_err(|error| self.store_error(error))?;
        let mut callee = Frame {
            store: Rc::clone(&frame.store),
            registers: bank,
            shared_calls: Some(context),
            scopes: vec![activation, function.target, self.global],
            pool: Arc::clone(&function.pool),
            target: function.target,
            this: Value::Object(owner),
            this_visible: definition.flags.unwrap() & 2 == 0,
            locals: activation,
            kind: FrameKind::Ordinary,
        };
        // From this point any returned error retains all changed operands and
        // registers. It does not manufacture native normal-return cleanup.
        for (index, parameter) in definition.parameters.iter().enumerate() {
            if let Some(register) = parameter.register.filter(|index| *index != 0) {
                if let Some(value) = args.get(index) {
                    self.write_register(&mut callee, register, value.clone())?;
                }
            } else {
                let key = self.decode_text(&parameter.name)?;
                self.shared_local_name(&key)?;
                self.set_key(
                    activation,
                    key,
                    args.get(index).cloned().unwrap_or(Value::Undefined),
                )?;
            }
        }
        if callee.this_visible {
            self.set(activation, "this", Value::Object(owner))?;
        }
        if definition.flags.unwrap() & 1 != 0 {
            self.write_register(&mut callee, 1, Value::Object(owner))?;
        }
        let caller = (self.current, self.current_code, self.current_target);
        self.depth += 1;
        let result = self
            .run(function.code, &definition.body, &mut callee, host)
            .and_then(|flow| match flow {
                Flow::Return(value) => Ok(value),
                Flow::End => Ok(Value::Undefined),
                Flow::Jump(target) => self.fail(ErrorKind::InvalidBranch(target as i128)),
            });
        self.depth -= 1;
        (self.current, self.current_code, self.current_target) = caller;
        let result = result?;
        frame
            .store
            .borrow_mut()
            .leave_function2(bank)
            .map_err(|error| self.store_error(error))?;
        frame
            .store
            .borrow_mut()
            .finish_call(count + 2, result)
            .map_err(|error| self.store_error(error))
    }

    fn shared_local_name(&self, key: &Text) -> Result<()> {
        self.simple_name(key)?;
        if key.is_empty()
            || key.iter().any(|unit| *unit == 0 || *unit > 127)
            || [
                "__proto__",
                "__constructor__",
                "__resolve",
                "_root",
                "_global",
                "_parent",
                "super",
            ]
            .iter()
            .any(|name| key.as_ref() == text(name).as_ref())
        {
            return self.fail(ErrorKind::Unsupported("shared call ordinary ASCII name"));
        }
        Ok(())
    }

    /// Prevent unrelated VM APIs from accidentally supplying private-stack
    /// callbacks or source-wrapper semantics inside the new shared route.
    pub(super) fn shared_script_instruction(
        &self,
        frame: &Frame,
        instruction: &super::super::Instruction,
    ) -> Result<()> {
        match &instruction.operands {
            Operands::Push(values)
                if values.iter().any(|value| {
                    matches!(value, Literal::Constant8(_) | Literal::Constant16(_))
                }) =>
            {
                self.fail(ErrorKind::Unsupported(
                    "shared script source constant-pool state",
                ))
            }
            Operands::Push(_) | Operands::StoreRegister(_) | Operands::Branch(_) => Ok(()),
            Operands::None
                if matches!(instruction.opcode,
                0 | 0x0b..=0x0e | 0x10..=0x12 | 0x17 | 0x18 | 0x1c | 0x1d |
                0x3c | 0x3e | 0x3f | 0x41 | 0x44 | 0x47 | 0x48 | 0x4a..=0x4d |
                0x50..=0x52 | 0x60..=0x68) =>
            {
                if matches!(instruction.opcode, 0x1c | 0x1d | 0x3c | 0x41) {
                    let depth = usize::from(matches!(instruction.opcode, 0x1d | 0x3c));
                    let Value::String(key) = self.peek(frame, depth)? else {
                        return self
                            .fail(ErrorKind::Unsupported("shared activation variable name"));
                    };
                    self.shared_local_name(&key)?;
                    if matches!(instruction.opcode, 0x1c | 0x1d)
                        && !matches!(
                            self.objects[frame.locals.0].properties.get(&key),
                            Some(Property::Data(_))
                        )
                    {
                        return self.fail(ErrorKind::Unsupported(
                            "shared activation own data variable",
                        ));
                    }
                }
                Ok(())
            }
            _ => self.fail(ErrorKind::Unsupported("shared script instruction profile")),
        }
    }
}
