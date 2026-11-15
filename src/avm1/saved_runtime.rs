//! Explicit saved-action installations and adopted persistent Environments.
//!
//! Native contracts: `docs/SAVED_EVENT_EXECUTION.md`. This module owns retained
//! wrappers and lends supplied state to the central saved Frame. It does not
//! discover initialization, split masks, queue events, or invoke dynamic handlers.
//! Admission errors leave state usable. A returned body error restores all
//! retained state and halts that Environment; it is never an implicit retry.

use super::saved_state::{
    SavedActionId, SavedActionSource, SavedActionState, SavedEnvironmentState, SavedMode,
};
use super::{
    AttachedClipId, ErrorKind, Flow, Frame, FrameKind, Host, Kind, ObjectId, Result, Value, Vm,
};
use super::{ExecutionStore, RegisterBank, SavedScriptCallProfile, SharedCallContext};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SavedEnvironmentId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavedActionInfo {
    pub owner: AttachedClipId,
    pub source: SavedActionSource,
    pub original_mask: u32,
    pub mode: SavedMode,
    /// Installed-source-relative offset, not a file-absolute opcode coordinate.
    pub first_pool_offset: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavedExecution {
    Completed {
        instructions: usize,
        host_calls: usize,
    },
    /// Native mode-4 target bit-4 gate: no body work or state loan occurred.
    TargetBit4Skipped,
}

#[derive(Debug)]
pub(super) struct InstalledAction {
    owner: AttachedClipId,
    state: SavedActionState,
}
#[derive(Debug)]
pub(super) struct InstalledEnvironment {
    pub(super) state: SavedEnvironmentState,
    pub(super) halted: bool,
    pub(super) script_profile: Option<SavedScriptCallProfile>,
    pub(super) script_bindings: Arc<BTreeSet<ObjectId>>,
    pub(super) dynamic: Vec<Value>,
}

impl Vm {
    /// Install a distinct retained wrapper. A split-key table must reuse the
    /// returned ID; calling this again intentionally creates independent state.
    pub fn install_saved_action(
        &mut self,
        owner: AttachedClipId,
        source: SavedActionSource,
        original_mask: u32,
    ) -> Result<SavedActionId> {
        self.saved_idle()?;
        self.saved_owner_profile(owner)?;
        self.saved_source_profile(source)?;
        if self.code[source.code.0]
            .instructions
            .first()
            .is_none_or(|instruction| instruction.opcode == 0)
        {
            return self.fail(ErrorKind::Unsupported(
                "saved installer requires nonempty source with nonzero first opcode",
            ));
        }
        if !matches!(original_mask, 1 | 0x40000 | 0x40001) {
            return self.fail(ErrorKind::Unsupported("saved installer event mask profile"));
        }
        if self.saved_actions.len() >= self.limits.programs {
            return self.fail(ErrorKind::Budget("saved action wrappers"));
        }
        let state = SavedActionState::new(source, original_mask).map_err(|_| {
            self.error(ErrorKind::Unsupported(
                "saved action source span outside installer profile",
            ))
        })?;
        let id = SavedActionId(self.saved_actions.len());
        self.saved_actions
            .push(Some(InstalledAction { owner, state }));
        Ok(id)
    }

    /// Adopt one explicit current Environment per owner. No stack/register
    /// defaults or prior-handler success are inferred. All supplied values stay
    /// private after adoption; the public inspection method borrows immutably.
    pub fn register_saved_environment(
        &mut self,
        state: SavedEnvironmentState,
    ) -> Result<SavedEnvironmentId> {
        self.saved_idle()?;
        let (target, _) = self.saved_owner_profile(state.owner())?;
        if target != state.target() {
            return self.fail(ErrorKind::Unsupported(
                "saved Environment owner/target mismatch",
            ));
        }
        if self.saved_environment_owners.contains_key(&state.owner()) {
            return self.fail(ErrorKind::Unsupported("duplicate saved Environment owner"));
        }
        if self.saved_environments.len() >= self.limits.objects {
            return self.fail(ErrorKind::Budget("saved Environments"));
        }
        if state.max_stack_values() > self.limits.stack_values
            || state.stack().len() > state.max_stack_values()
        {
            return self.fail(ErrorKind::Budget("saved Environment operand stack"));
        }
        let mut units = 0usize;
        for value in state.stack().iter().chain(state.registers()) {
            self.check_value(value)?;
            if let Value::String(value) = value {
                units = units.checked_add(value.len()).ok_or_else(|| {
                    self.error(ErrorKind::Budget("saved Environment string units"))
                })?;
            }
        }
        if units > self.limits.string_units.saturating_sub(self.string_units) {
            return self.fail(ErrorKind::Budget("saved Environment string units"));
        }
        let owner = state.owner();
        let id = SavedEnvironmentId(self.saved_environments.len());
        self.string_units += units;
        self.saved_environments.push(Some(InstalledEnvironment {
            state,
            halted: false,
            script_profile: None,
            script_bindings: Arc::new(BTreeSet::new()),
            dynamic: Vec::new(),
        }));
        self.saved_environment_owners.insert(owner, id);
        Ok(id)
    }

    pub fn saved_action_info(&self, id: SavedActionId) -> Result<SavedActionInfo> {
        let action = self.saved_action(id)?;
        Ok(SavedActionInfo {
            owner: action.owner,
            source: action.state.source(),
            original_mask: action.state.original_mask(),
            mode: action.state.mode(),
            first_pool_offset: action.state.first_pool_offset(),
        })
    }
    pub fn saved_environment_state(
        &self,
        id: SavedEnvironmentId,
    ) -> Result<&SavedEnvironmentState> {
        Ok(&self.saved_environment(id)?.state)
    }
    pub fn saved_environment_halted(&self, id: SavedEnvironmentId) -> Result<bool> {
        Ok(self.saved_environment(id)?.halted)
    }

    /// Execute only a pure Construct/onLoad mode-4 installation. The result
    /// destination is intentionally discarded; persistent state remains owned by
    /// the arenas. This does not claim event eligibility/order or dynamic calls.
    pub fn execute_saved_action(
        &mut self,
        id: SavedActionId,
        environment: SavedEnvironmentId,
        host: &mut impl Host,
    ) -> Result<SavedExecution> {
        self.saved_idle()?;
        let action = self.saved_action_info(id)?;
        let env = self.saved_environment(environment)?;
        if env.halted {
            return self.fail(ErrorKind::Unsupported("saved Environment is halted"));
        }
        if action.owner != env.state.owner() {
            return self.fail(ErrorKind::Unsupported(
                "saved action/Environment owner mismatch",
            ));
        }
        if !matches!(action.original_mask, 1 | 0x40000) || action.mode != SavedMode::Mode4 {
            return self.fail(ErrorKind::Unsupported(
                "saved execution requires pure Construct/onLoad mode4",
            ));
        }
        let (target, flags) = self.saved_owner_profile(action.owner)?;
        if env.state.target() != target {
            return self.fail(ErrorKind::Unsupported(
                "saved Environment owner/target mismatch",
            ));
        }
        if self.saved_environment_owners.get(&action.owner) != Some(&environment) {
            return self.fail(ErrorKind::Unsupported(
                "saved Environment owner registration mismatch",
            ));
        }
        if env.state.max_stack_values() > self.limits.stack_values
            || env.state.stack().len() > env.state.max_stack_values()
        {
            return self.fail(ErrorKind::Budget("saved Environment operand stack"));
        }
        self.saved_source_profile(action.source)?;
        if self.limits.call_depth == 0 {
            return self.fail(ErrorKind::Budget("function call depth"));
        }
        if self.limits.instructions == 0 {
            return self.fail(ErrorKind::Budget("instructions"));
        }
        let program = Arc::clone(&self.code[action.source.code.0]);
        let old_context = (
            self.current,
            self.current_code,
            self.current_target,
            self.depth,
        );
        self.reset_execution();
        if flags & 0x10 != 0 {
            (
                self.current,
                self.current_code,
                self.current_target,
                self.depth,
            ) = old_context;
            return Ok(SavedExecution::TargetBit4Skipped);
        }

        self.saved_active = true;
        // The arenas were checked while idle. Only this method removes entries;
        // no call, callback, or fallible work occurs between admission and take.
        let InstalledAction {
            owner,
            state: action_state,
        } = self.saved_actions[id.0].take().unwrap();
        let mut env = self.saved_environments[environment.0].take().unwrap();
        let (stack, registers, max_stack_values) = env.state.take_execution();
        // Successful admission proved the supplied stack ceiling. No dynamic
        // bank is inferred for calls: those need the separate explicit profile.
        let store = ExecutionStore::new(stack, registers, max_stack_values)
            .expect("admitted saved state obeys its stack ceiling");
        let shared_calls = env.script_profile.map(|_| SharedCallContext {
            target,
            bindings: Arc::clone(&env.script_bindings),
        });
        let mut frame = Frame {
            store: Rc::new(RefCell::new(store)),
            registers: RegisterBank::Fixed,
            shared_calls,
            scopes: vec![target, self.global],
            pool: Arc::clone(action_state.pool()),
            target,
            this: Value::DisplayClip(owner),
            this_visible: true,
            // Central Saved branches implement the null activation fence and
            // own opcode rules. This field is not an ordinary local activation.
            locals: target,
            kind: FrameKind::Saved {
                action: action_state,
            },
        };
        self.depth = 1;
        let result = match self.run(action.source.code, &program, &mut frame, host) {
            Ok(Flow::End | Flow::Return(_)) => {
                if frame.target != target {
                    self.fail(ErrorKind::Unsupported(
                        "saved execution changed Environment target",
                    ))
                } else {
                    Ok(SavedExecution::Completed {
                        instructions: self.instructions,
                        host_calls: self.host_calls,
                    })
                }
            }
            Ok(Flow::Jump(destination)) => self.fail(ErrorKind::InvalidBranch(destination as i128)),
            Err(error) => Err(error),
        };
        // Participating frames only borrow the Rc lexically. Returned errors
        // drop those control frames but retain the one store's partial contents.
        let store = Rc::try_unwrap(frame.store)
            .expect("saved execution returns sole ownership of its store")
            .into_inner();
        let FrameKind::Saved {
            action: action_state,
            ..
        } = frame.kind
        else {
            unreachable!("central run preserves saved Frame kind")
        };
        env.state.restore_execution(store.stack, store.fixed);
        env.dynamic = store.dynamic;
        env.halted = result.is_err();
        self.saved_actions[id.0] = Some(InstalledAction {
            owner,
            state: action_state,
        });
        self.saved_environments[environment.0] = Some(env);
        self.saved_active = false;
        (
            self.current,
            self.current_code,
            self.current_target,
            self.depth,
        ) = old_context;
        result
    }

    pub(super) fn saved_idle(&self) -> Result<()> {
        if self.saved_active || self.depth != 0 {
            return self.fail(ErrorKind::Unsupported(
                "saved execution entry is already active",
            ));
        }
        Ok(())
    }
    fn saved_action(&self, id: SavedActionId) -> Result<&InstalledAction> {
        self.saved_actions
            .get(id.0)
            .and_then(Option::as_ref)
            .ok_or_else(|| self.error(ErrorKind::Unsupported("invalid or borrowed saved action")))
    }
    pub(super) fn saved_environment(
        &self,
        id: SavedEnvironmentId,
    ) -> Result<&InstalledEnvironment> {
        self.saved_environments
            .get(id.0)
            .and_then(Option::as_ref)
            .ok_or_else(|| {
                self.error(ErrorKind::Unsupported(
                    "invalid or borrowed saved Environment",
                ))
            })
    }
    fn saved_source_profile(&self, source: SavedActionSource) -> Result<()> {
        let Some(program) = self.code.get(source.code.0) else {
            return self.fail(ErrorKind::InvalidCode(source.code));
        };
        if program.span != source.span {
            return self.fail(ErrorKind::Unsupported(
                "saved action source does not match registered Program span",
            ));
        }
        Ok(())
    }
    pub(super) fn saved_owner_profile(&self, owner: AttachedClipId) -> Result<(ObjectId, u32)> {
        self.check_attached(owner)?;
        let info = self.attached_info(owner)?;
        self.check_object(info.owner)?;
        if !matches!(self.objects[info.owner.0].kind, Kind::AttachedOwner(id) if id == owner) {
            return self.fail(ErrorKind::Unsupported(
                "saved owner requires live eligible case3 target",
            ));
        }
        let flags = self.attached_event_state(owner)?.target_flags;
        if flags & 0x80 == 0 {
            return self.fail(ErrorKind::Unsupported(
                "saved owner requires live eligible case3 target",
            ));
        }
        if !(7..=19).contains(&info.environment_header_version)
            || self.loaded_movie_targets.get(&info.owner) != Some(&info.environment_header_version)
        {
            return self.fail(ErrorKind::Unsupported(
                "saved owner has unsupported Environment version",
            ));
        }
        let Kind::AttachedOwner(root) = self.objects[self.root.0].kind else {
            return self.fail(ErrorKind::Unsupported(
                "saved execution requires one adopted attached root",
            ));
        };
        if self.attached_owner(root)? != self.root
            || self.attached_event_state(root)?.target_flags & 0x80 == 0
        {
            return self.fail(ErrorKind::Unsupported(
                "saved execution requires one adopted attached root",
            ));
        }
        let mut cursor = Some(owner);
        for _ in 0..self.limits.prototype_depth {
            let Some(current) = cursor else { break };
            if current == root {
                return Ok((info.owner, flags));
            }
            cursor = self.attached_info(current)?.parent;
        }
        self.fail(ErrorKind::Unsupported(
            "saved execution requires one adopted attached root",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::Program;
    use super::super::{
        AttachedClipSource, AttachedClipSpec, AttachedEventState, CodeId, CodeOrigin,
        DisplayTargetId, Limits, LoadedMovieSourceId, RejectHost,
    };
    use super::*;

    fn fixture() -> (Vm, AttachedClipId, SavedEnvironmentId) {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        let owner = vm
            .create_attached_movie_clip(AttachedClipSpec {
                target: DisplayTargetId(81),
                source: AttachedClipSource {
                    movie: LoadedMovieSourceId(3),
                    sprite_id: 11,
                    header_version: 15,
                },
                parent: None,
                event_state: AttachedEventState {
                    target_flags: 0x5200_0180,
                    load_flags: 0,
                    source_flags: 0,
                },
            })
            .unwrap();
        vm.adopt_attached_root(owner).unwrap();
        // Explicit synthetic prior state. This is not a native initialization
        // receipt or a claim that real handlers begin with these values.
        let state = SavedEnvironmentState::from_supplied(
            owner,
            vm.attached_owner(owner).unwrap(),
            vec![Value::Number(17.0)],
            [
                Value::Null,
                Value::Bool(true),
                Value::Number(4.0),
                Value::Undefined,
            ],
            8,
        )
        .unwrap();
        let environment = vm.register_saved_environment(state).unwrap();
        (vm, owner, environment)
    }
    fn source(vm: &mut Vm, bytes: &[u8]) -> SavedActionSource {
        let program = Arc::new(Program::parse_at(bytes, 500).unwrap());
        let span = program.span;
        SavedActionSource {
            code: vm.register(program).unwrap(),
            span,
        }
    }
    fn record(opcode: u8, payload: &[u8]) -> Vec<u8> {
        let mut bytes = vec![opcode];
        bytes.extend((payload.len() as u16).to_le_bytes());
        bytes.extend(payload);
        bytes
    }
    fn pool(value: &str) -> Vec<u8> {
        let mut payload = vec![1, 0];
        payload.extend(value.as_bytes());
        payload.push(0);
        record(0x88, &payload)
    }
    fn successful(result: SavedExecution) {
        assert!(matches!(
            result,
            SavedExecution::Completed { host_calls: 0, .. }
        ));
    }

    #[test]
    fn saved_runtime_keeps_bank_stack_first_pool_and_distinct_installations() {
        let (mut vm, owner, env) = fixture();
        let mut bytes = pool("one");
        bytes.extend(record(0x96, &[8, 0]));
        bytes.extend(record(0x87, &[2]));
        bytes.push(0x17);
        bytes.extend(pool("ignored"));
        bytes.extend(record(0x96, &[4, 2]));
        let src = source(&mut vm, &bytes);
        let first = vm.install_saved_action(owner, src, 1).unwrap();
        let second = vm.install_saved_action(owner, src, 0x40000).unwrap();
        assert_ne!(first, second);
        for repetition in 1..=2 {
            successful(
                vm.execute_saved_action(first, env, &mut RejectHost)
                    .unwrap(),
            );
            let state = vm.saved_environment_state(env).unwrap();
            assert_eq!(state.stack().len(), repetition + 1);
            assert_eq!(state.stack()[0], Value::Number(17.0));
            assert!(
                state.stack()[1..]
                    .iter()
                    .all(|v| *v == Value::string("one"))
            );
            assert_eq!(state.registers()[1], Value::Bool(true));
            assert_eq!(state.registers()[2], Value::string("one"));
            assert_eq!(
                vm.saved_action_info(first).unwrap().first_pool_offset,
                Some(0)
            );
            assert_eq!(
                vm.saved_action_info(second).unwrap().first_pool_offset,
                None
            );
        }
        let retained = Arc::clone(vm.saved_action(first).unwrap().state.pool());
        successful(
            vm.execute_saved_action(second, env, &mut RejectHost)
                .unwrap(),
        );
        assert!(!Arc::ptr_eq(
            &retained,
            vm.saved_action(second).unwrap().state.pool()
        ));
        assert!(!vm.saved_environment_halted(env).unwrap());
        assert!(vm.attached_info(owner).unwrap().storage.is_none());
    }

    #[test]
    fn saved_runtime_restores_error_state_then_halts_without_retry() {
        let (mut vm, owner, env) = fixture();
        let mut bytes = pool("saved");
        bytes.extend(record(0x96, &[8, 0]));
        bytes.extend(record(0x87, &[3]));
        bytes.push(0x2c); // Explicit unsupported FSCommand2 after state changes.
        let src = source(&mut vm, &bytes);
        let action = vm.install_saved_action(owner, src, 1).unwrap();
        let before = (Some((41, 0x17)), Some(CodeId(900)), Some(vm.root()), 0);
        (vm.current, vm.current_code, vm.current_target, vm.depth) = before;
        let error = vm
            .execute_saved_action(action, env, &mut RejectHost)
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::UnsupportedOpcode(0x2c));
        assert_eq!(
            error.origin,
            Some(CodeOrigin {
                code: src.code,
                source_offset: 500 + bytes.len() - 1,
            })
        );
        assert_eq!(
            (vm.current, vm.current_code, vm.current_target, vm.depth),
            before
        );
        assert!(!vm.saved_active);
        assert_eq!(
            vm.saved_action_info(action).unwrap().first_pool_offset,
            Some(0)
        );
        let state = vm.saved_environment_state(env).unwrap();
        assert_eq!(
            state.stack(),
            &[Value::Number(17.0), Value::string("saved")]
        );
        assert_eq!(state.registers()[3], Value::string("saved"));
        assert!(vm.saved_environment_halted(env).unwrap());
        let count = vm.instruction_count();
        let error = vm
            .execute_saved_action(action, env, &mut RejectHost)
            .unwrap_err();
        assert_eq!(
            error.kind,
            ErrorKind::Unsupported("saved Environment is halted")
        );
        assert_eq!(vm.instruction_count(), count);
        assert_eq!(vm.saved_environment_state(env).unwrap().stack().len(), 2);
    }

    #[test]
    fn saved_runtime_admission_failures_leave_environment_usable() {
        let (mut vm, owner, env) = fixture();
        let src = source(&mut vm, &[0x96, 0, 0, 0]);
        let combined = vm.install_saved_action(owner, src, 0x40001).unwrap();
        let normal = vm.install_saved_action(owner, src, 1).unwrap();
        assert_eq!(
            vm.saved_action_info(combined).unwrap().mode,
            SavedMode::Mode3
        );
        assert!(
            vm.execute_saved_action(SavedActionId(999), env, &mut RejectHost)
                .is_err()
        );
        assert!(
            vm.execute_saved_action(combined, env, &mut RejectHost)
                .is_err()
        );
        assert!(!vm.saved_environment_halted(env).unwrap());
        let mut wrong = src;
        wrong.span.offset += 1;
        assert!(vm.install_saved_action(owner, wrong, 1).is_err());
        assert_eq!(vm.saved_actions.len(), 2);
        let depth = vm.limits.call_depth;
        vm.limits.call_depth = 0;
        assert_eq!(
            vm.execute_saved_action(normal, env, &mut RejectHost)
                .unwrap_err()
                .kind,
            ErrorKind::Budget("function call depth")
        );
        vm.limits.call_depth = depth;
        vm.saved_active = true;
        assert_eq!(
            vm.execute_saved_action(normal, env, &mut RejectHost)
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported("saved execution entry is already active")
        );
        vm.saved_active = false;
        assert!(!vm.saved_environment_halted(env).unwrap());
        successful(
            vm.execute_saved_action(normal, env, &mut RejectHost)
                .unwrap(),
        );
        assert_eq!(
            vm.saved_environment_state(env).unwrap().stack(),
            &[Value::Number(17.0)]
        );
        assert_eq!(
            vm.saved_environment_state(env).unwrap().registers()[2],
            Value::Number(4.0)
        );
    }

    #[test]
    fn saved_runtime_target_bit4_skip_is_zero_work_without_state_loan_or_halt() {
        let (mut vm, owner, env) = fixture();
        let src = source(&mut vm, &[0x2c]);
        let action = vm.install_saved_action(owner, src, 1).unwrap();
        let mut flags = vm.attached_event_state(owner).unwrap();
        flags.target_flags |= 0x10;
        vm.update_attached_event_state(owner, flags).unwrap();
        let before = (Some((29, 0x17)), Some(CodeId(40)), Some(vm.root()), 0);
        (vm.current, vm.current_code, vm.current_target, vm.depth) = before;
        vm.instructions = 77;
        vm.host_calls = 2;
        assert_eq!(
            vm.execute_saved_action(action, env, &mut RejectHost)
                .unwrap(),
            SavedExecution::TargetBit4Skipped
        );
        assert_eq!(vm.instructions, 0);
        assert_eq!(vm.host_calls, 0);
        assert_eq!(
            (vm.current, vm.current_code, vm.current_target, vm.depth),
            before
        );
        assert_eq!(
            vm.saved_environment_state(env).unwrap().stack(),
            &[Value::Number(17.0)]
        );
        assert_eq!(
            vm.saved_environment_state(env).unwrap().registers()[2],
            Value::Number(4.0)
        );
        assert_eq!(
            vm.saved_action_info(action).unwrap().first_pool_offset,
            None
        );
        assert!(!vm.saved_environment_halted(env).unwrap());
        assert!(!vm.saved_active);
        flags.target_flags &= !0x10;
        vm.update_attached_event_state(owner, flags).unwrap();
        assert!(
            vm.execute_saved_action(action, env, &mut RejectHost)
                .is_err()
        );
        assert!(vm.saved_environment_halted(env).unwrap());
    }
}
