//! Bounded native Function.prototype primitive callbacks.
//!
//! `docs/REA_FUNCTION_PRIMITIVES.md` binds the original callback objects to
//! 0x14018c7c0 (toString) and 0x140196710 (valueOf). These are explicit calls,
//! not a general object-to-primitive conversion policy. The native result
//! destination is fresh Undefined; no old native FunctionRef wrapper is copied.

use super::{ErrorKind, Kind, Result, Value, Vm};

#[derive(Debug, Clone, Copy)]
pub(super) enum FunctionPrimitive {
    ToString,
    ValueOf,
}

impl Vm {
    pub(super) fn invoke_function_primitive(
        &mut self,
        method: FunctionPrimitive,
        this: Value,
    ) -> Result<Value> {
        match method {
            FunctionPrimitive::ToString => {
                // The callback obtains this builtin string from the ACTIVE
                // Environment's context. Neither its receiver nor arguments
                // select that context, and direct Vm::call supplies none.
                let Some(target) = self.current_target else {
                    return self.fail(ErrorKind::Unsupported(
                        "Function.toString requires current Environment",
                    ));
                };
                if !self.is_bound_builtin_context_target(target)? {
                    return self.fail(ErrorKind::Unsupported(
                        "Function.toString requires bound builtin context",
                    ));
                }
                // Native returns a retained builtin string. Charging each
                // returned value is this executor's existing engineering budget,
                // not a claim that native allocates another string per call.
                self.allocate_text("[type Function]".encode_utf16().collect())
                    .map(Value::String)
            }
            FunctionPrimitive::ValueOf => {
                let Value::Object(receiver) = this else {
                    // A native NULL this-interface becomes tag6/pointer0, which
                    // is neither this VM's Null nor Undefined. Primitive boxing
                    // and display-interface conversion are separate contracts.
                    return self.fail(ErrorKind::Unsupported(
                        "Function.valueOf receiver representation",
                    ));
                };
                self.check_object(receiver)?;
                match &self.objects[receiver.0].kind {
                    // These ordinary native callback objects use the proved
                    // common interface 142514430/+60 getter 18c580. Specialized
                    // constructor and method kinds deliberately remain gated.
                    Kind::Native(_)
                    | Kind::UnsupportedNative(_)
                    | Kind::ObjectConstructor
                    | Kind::FunctionConversion
                    | Kind::AddProperty
                    | Kind::RegisterClass
                    | Kind::FunctionMethod(_)
                    | Kind::FunctionPrimitive(_)
                    | Kind::ObjectPrimitive(_) => {}
                    Kind::Script(function) => {
                        // Actual type1/type2 getters preserve the function
                        // object but clear the auxiliary captured-frame ref.
                        // The representation is equivalent only in this profile.
                        if function.scopes.as_slice() != [function.target, self.global] {
                            return self.fail(ErrorKind::Unsupported(
                                "Function.valueOf captured activation/With profile",
                            ));
                        }
                    }
                    Kind::Plain if Some(receiver) == self.object_prototype => {}
                    Kind::FunctionPrototype if Some(receiver) == self.function_prototype => {}
                    _ => {
                        return self.fail(ErrorKind::Unsupported(
                            "Function.valueOf native receiver category/getter profile",
                        ));
                    }
                }
                // Category23 functions return the same object with fresh
                // auxiliary fields cleared; cached OP/FP category6 uses the
                // ordinary tag6 fallback. Both admitted paths preserve this ID.
                Ok(Value::Object(receiver))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super as vm;
    use super::super::super::Program;
    use super::super::{Limits, ObjectId, Property, RejectHost, ValueCallability, text};
    use super::*;
    use std::sync::Arc;

    fn setup() -> Vm {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm
    }

    fn method(vm: &Vm, name: &str) -> Value {
        vm.get(vm.function_prototype.unwrap(), name).unwrap()
    }

    fn named_call(vm: &mut Vm, receiver: Value, name: &str) -> vm::Result<vm::Execution> {
        vm.set(vm.root(), "receiver", receiver).unwrap();
        let mut bytes = vm::tests::push(&[Value::Number(0.0), Value::string("receiver")]);
        bytes.push(0x1c);
        bytes.extend(vm::tests::push(&[Value::string(name)]));
        bytes.extend([0x52, 0]);
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        vm.execute(code, vm.root(), &mut RejectHost)
    }

    #[test]
    fn function_primitives_named_to_string_requires_active_bound_context() {
        let mut vm = setup();
        let receiver = vm.native(801).unwrap();
        let callback = method(&vm, "toString");
        assert!(matches!(
            vm.call(callback, Value::Object(receiver), &[], &mut RejectHost)
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported("Function.toString requires current Environment")
        ));
        let error = named_call(&mut vm, Value::Object(receiver), "toString").unwrap_err();
        assert!(matches!(
            error.kind,
            ErrorKind::Unsupported("Function.toString requires bound builtin context")
        ));
        assert_eq!(error.opcode, Some(0x52));
        vm.bind_builtin_context_target(vm.root()).unwrap();
        let before = vm.object_count();
        let result = named_call(&mut vm, Value::Object(receiver), "toString").unwrap();
        assert_eq!(result.stack, vec![Value::string("[type Function]")]);
        assert_eq!(result.host_calls, 0);
        assert_eq!(vm.object_count(), before);
    }

    #[test]
    fn function_primitives_to_string_ignores_valid_receiver_and_arguments_without_coercion() {
        let mut vm = setup();
        vm.bind_builtin_context_target(vm.root()).unwrap();
        let token = vm
            .register_deferred_builtin(vm.global(), "Untouched", vm::BuiltinFactory::Unsupported, 1)
            .unwrap();
        let callback = method(&vm, "toString");
        let count = vm.object_count();
        for receiver in [
            Value::Undefined,
            Value::Null,
            Value::Bool(true),
            Value::Number(f64::NAN),
            Value::string("not source"),
            token.clone(),
        ] {
            vm.current_target = Some(vm.root());
            let before = vm.string_units;
            assert_eq!(
                vm.invoke(
                    callback.clone(),
                    receiver,
                    &[token.clone()],
                    &mut RejectHost
                )
                .unwrap(),
                Value::string("[type Function]")
            );
            assert_eq!(vm.string_units - before, 15);
        }
        assert_eq!(vm.get(vm.global(), "Untouched").unwrap(), token);
        assert_eq!(vm.object_count(), count);
        assert_eq!(vm.host_calls, 0);
    }

    #[test]
    fn function_primitives_value_of_preserves_only_proved_native_and_prototype_identities() {
        let mut vm = setup();
        let callback = method(&vm, "valueOf");
        let native = vm.native(802).unwrap();
        let op = vm.object_prototype.unwrap();
        let fp = vm.function_prototype.unwrap();
        let oc = vm.object_constructor.unwrap();
        let fc = vm.function_constructor.unwrap();
        let Value::Object(value_of) = callback.clone() else {
            panic!()
        };
        let mut ids = vec![native, op, fp, oc, fc, value_of];
        for owner in [oc, op, fp] {
            for property in vm.objects[owner.0].properties.values() {
                if let Property::Data(Value::Object(id)) = property {
                    if vm.classify_callability(&Value::Object(*id)).unwrap()
                        == ValueCallability::Callable
                    {
                        ids.push(*id);
                    }
                }
            }
        }
        let before = (vm.object_count(), vm.properties, vm.string_units);
        for id in ids {
            let result = vm
                .call(
                    callback.clone(),
                    Value::Object(id),
                    &[Value::Null, Value::Number(9.0)],
                    &mut RejectHost,
                )
                .unwrap();
            assert_eq!(result.value, Value::Object(id));
            assert_eq!(result.host_calls, 0);
        }
        assert_eq!((vm.object_count(), vm.properties, vm.string_units), before);
        let plain = vm.object().unwrap();
        let array = vm.install_array_constructor().unwrap();
        for receiver in [
            Value::Object(plain),
            Value::Object(array),
            Value::Undefined,
            Value::Null,
            Value::Number(2.0),
        ] {
            assert!(matches!(
                vm.call(callback.clone(), receiver, &[], &mut RejectHost)
                    .unwrap_err()
                    .kind,
                ErrorKind::Unsupported(_)
            ));
        }
        assert!(matches!(
            vm.call(
                callback,
                Value::Object(ObjectId(usize::MAX)),
                &[],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::InvalidObject(_)
        ));
    }

    fn define(name: &str, body: &[u8]) -> Vec<u8> {
        let mut payload = name.as_bytes().to_vec();
        payload.extend([0, 0, 0]);
        payload.extend((body.len() as u16).to_le_bytes());
        let mut bytes = vec![0x9b];
        bytes.extend((payload.len() as u16).to_le_bytes());
        bytes.extend(payload);
        bytes.extend(body);
        bytes
    }

    #[test]
    fn function_primitives_script_value_of_keeps_top_level_and_rejects_captured_activation() {
        let mut vm = setup();
        let mut nested = define("", &[0]);
        nested.push(0x3e);
        let code = vm
            .register(Arc::new(Program::parse(&define("outer", &nested)).unwrap()))
            .unwrap();
        vm.execute(code, vm.root(), &mut RejectHost).unwrap();
        let outer = vm.get(vm.root(), "outer").unwrap();
        let result = named_call(&mut vm, outer.clone(), "valueOf").unwrap();
        assert_eq!(result.stack, vec![outer.clone()]);
        let closure = vm
            .call(outer, Value::Object(vm.root()), &[], &mut RejectHost)
            .unwrap()
            .value;
        let error = vm
            .call(method(&vm, "valueOf"), closure, &[], &mut RejectHost)
            .unwrap_err();
        assert!(matches!(
            error.kind,
            ErrorKind::Unsupported("Function.valueOf captured activation/With profile")
        ));
        assert_eq!(vm.host_calls, 0);
    }

    #[test]
    fn function_primitives_budget_failure_preserves_state_and_error_origin() {
        let mut vm = setup();
        vm.bind_builtin_context_target(vm.root()).unwrap();
        vm.current_target = Some(vm.root());
        vm.current_code = Some(vm::CodeId(0));
        vm.current = Some((1234, 0x52));
        vm.limits.string_units = vm.string_units + 14;
        let before = (vm.object_count(), vm.properties, vm.string_units);
        let error = vm
            .invoke(method(&vm, "toString"), Value::Null, &[], &mut RejectHost)
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Budget("allocated string units"));
        assert_eq!(error.origin.unwrap().source_offset, 1234);
        assert_eq!((vm.object_count(), vm.properties, vm.string_units), before);
        assert_eq!(vm.depth, 0);
        assert_eq!(vm.current_target, Some(vm.root()));
        assert_eq!(vm.host_calls, 0);
        // Other Object.prototype callbacks remain separate unsupported natives.
        let object_method = vm.get(vm.object_prototype.unwrap(), "watch").unwrap();
        assert!(matches!(
            vm.call(
                object_method,
                Value::Object(vm.object_prototype.unwrap()),
                &[],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::UnsupportedNativeFunction(_)
        ));
        assert_eq!(
            vm.own_flags(vm.function_prototype.unwrap(), &text("valueOf")),
            1
        );
    }
}
