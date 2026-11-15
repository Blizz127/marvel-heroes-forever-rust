//! Explicit Object.prototype callbacks from the owned native client.
//!
//! `docs/REA_OBJECT_PRIMITIVES.md` binds toString192760 and valueOf197410.
//! This is not general object-to-primitive coercion. Only concrete receiver
//! categories with retained evidence are admitted; `Kind::Plain` alone is not
//! native class evidence for a root, activation, or script-created prototype.

use super::{ErrorKind, Kind, ObjectId, Result, Value, Vm};

#[derive(Debug, Clone, Copy)]
pub(super) enum ObjectPrimitive {
    ToString,
    ValueOf,
}

#[derive(Clone, Copy)]
enum ReceiverCategory {
    Object,
    Function,
}

impl Vm {
    fn object_primitive_category(&self, receiver: ObjectId) -> Result<ReceiverCategory> {
        self.check_object(receiver)?;
        let object = &self.objects[receiver.0];
        match &object.kind {
            // The common native callback interface and actual ScriptFunction
            // interface have a proved category23 and FunctionRef getter.
            Kind::Native(_)
            | Kind::UnsupportedNative(_)
            | Kind::ObjectConstructor
            | Kind::FunctionConversion
            | Kind::AddProperty
            | Kind::RegisterClass
            | Kind::FunctionMethod(_)
            | Kind::FunctionPrimitive(_)
            | Kind::ObjectPrimitive(_)
            | Kind::Script(_) => Ok(ReceiverCategory::Function),
            Kind::Plain
                if object.native_object_instance || Some(receiver) == self.object_prototype =>
            {
                Ok(ReceiverCategory::Object)
            }
            Kind::FunctionPrototype if Some(receiver) == self.function_prototype => {
                Ok(ReceiverCategory::Object)
            }
            _ => self.fail(ErrorKind::Unsupported(
                "Object primitive native receiver category profile",
            )),
        }
    }

    pub(super) fn invoke_object_primitive(
        &mut self,
        method: ObjectPrimitive,
        this: Value,
    ) -> Result<Value> {
        let Value::Object(receiver) = this else {
            return self.fail(ErrorKind::Unsupported(
                "Object primitive receiver representation",
            ));
        };
        let category = self.object_primitive_category(receiver)?;
        match method {
            ObjectPrimitive::ToString => {
                let Some(target) = self.current_target else {
                    return self.fail(ErrorKind::Unsupported(
                        "Object.toString requires current Environment",
                    ));
                };
                if !self.is_bound_builtin_context_target(target)? {
                    return self.fail(ErrorKind::Unsupported(
                        "Object.toString requires bound builtin context",
                    ));
                }
                // Native retains this context's builtin node. Charging returned
                // UTF-16 units is the existing Rust engineering budget, not a
                // native per-call allocation claim. No receiver property or
                // FunctionRef conversion is involved, even for a closure.
                let literal = match category {
                    ReceiverCategory::Object => "[object Object]",
                    ReceiverCategory::Function => "[type Function]",
                };
                self.allocate_text(literal.encode_utf16().collect())
                    .map(Value::String)
            }
            ObjectPrimitive::ValueOf => {
                if let Kind::Script(function) = &self.objects[receiver.0].kind {
                    // Fresh native return construction clears wrapper auxiliary
                    // fields; the same ID is sound only when no captured frame
                    // is encoded by this VM's scopes representation.
                    if function.scopes.as_slice() != [function.target, self.global] {
                        return self.fail(ErrorKind::Unsupported(
                            "Object.valueOf captured activation/With profile",
                        ));
                    }
                }
                Ok(Value::Object(receiver))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super as vm;
    use super::super::super::Program;
    use super::super::{Host, HostCall, Limits, RejectHost, text};
    use super::*;
    use std::sync::Arc;

    fn setup() -> Vm {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.bind_builtin_context_target(vm.root()).unwrap();
        vm
    }

    fn method(vm: &Vm, name: &str) -> Value {
        vm.get(vm.object_prototype.unwrap(), name).unwrap()
    }

    fn execute(vm: &mut Vm, bytes: &[u8]) -> vm::Result<vm::Execution> {
        let code = vm
            .register(Arc::new(Program::parse(bytes).unwrap()))
            .unwrap();
        vm.execute(code, vm.root(), &mut RejectHost)
    }

    fn constructed(vm: &mut Vm, opcode: u8) -> ObjectId {
        let mut bytes = vm::tests::push(&[Value::Number(0.0)]);
        if opcode == 0x40 {
            bytes.extend(vm::tests::push(&[Value::string("Object")]));
        }
        bytes.extend([opcode, 0]);
        let result = execute(vm, &bytes).unwrap();
        let Value::Object(id) = result.stack[0] else {
            panic!()
        };
        id
    }

    fn named_call_bytes() -> Vec<u8> {
        let mut bytes = vm::tests::push(&[Value::Number(0.0), Value::string("receiver")]);
        bytes.push(0x1c);
        bytes.extend(vm::tests::push(&[Value::string("toString")]));
        bytes.extend([0x52, 0]);
        bytes
    }

    #[test]
    fn object_primitives_exact_object_construction_admits_inherited_callbacks() {
        let mut vm = setup();
        for opcode in [0x43, 0x40] {
            let receiver = constructed(&mut vm, opcode);
            assert!(vm.objects[receiver.0].native_object_instance);
            vm.set(vm.root(), "receiver", Value::Object(receiver))
                .unwrap();
            let result = execute(&mut vm, &named_call_bytes()).unwrap();
            assert_eq!(result.stack, vec![Value::string("[object Object]")]);
            assert_eq!(result.host_calls, 0);
            let result = vm
                .call(
                    method(&vm, "valueOf"),
                    Value::Object(receiver),
                    &[Value::Number(4.0), Value::Null],
                    &mut RejectHost,
                )
                .unwrap();
            assert_eq!(result.value, Value::Object(receiver));
            assert_eq!(result.host_calls, 0);
        }
        let op = vm.object_prototype.unwrap();
        let fp = vm.function_prototype.unwrap();
        for receiver in [op, fp] {
            vm.current_target = Some(vm.root());
            assert_eq!(
                vm.invoke(
                    method(&vm, "toString"),
                    Value::Object(receiver),
                    &[],
                    &mut RejectHost
                )
                .unwrap(),
                Value::string("[object Object]")
            );
            assert_eq!(
                vm.call(
                    method(&vm, "valueOf"),
                    Value::Object(receiver),
                    &[],
                    &mut RejectHost
                )
                .unwrap()
                .value,
                Value::Object(receiver)
            );
        }
    }

    #[test]
    fn object_primitives_origin_survives_metadata_and_hash_changes_without_certifying_plain() {
        let mut vm = setup();
        let receiver = constructed(&mut vm, 0x43);
        vm.set(receiver, "x", Value::Bool(true)).unwrap();
        assert!(vm.delete_member(receiver, &text("x")).unwrap());
        assert!(!vm.objects[receiver.0].native_hash_profile);
        vm.set(receiver, "constructor", Value::Undefined).unwrap();
        vm.set(receiver, "__proto__", Value::Null).unwrap();
        let before = (vm.object_count(), vm.properties);
        vm.current_target = Some(vm.root());
        assert_eq!(
            vm.invoke(
                method(&vm, "toString"),
                Value::Object(receiver),
                &[],
                &mut RejectHost
            )
            .unwrap(),
            Value::string("[object Object]")
        );
        assert_eq!(
            vm.call(
                method(&vm, "valueOf"),
                Value::Object(receiver),
                &[],
                &mut RejectHost
            )
            .unwrap()
            .value,
            Value::Object(receiver)
        );
        assert_eq!((vm.object_count(), vm.properties), before);
        let plain = vm.object().unwrap();
        vm.set(
            plain,
            "constructor",
            Value::Object(vm.object_constructor.unwrap()),
        )
        .unwrap();
        vm.set(
            plain,
            "__proto__",
            Value::Object(vm.object_prototype.unwrap()),
        )
        .unwrap();
        vm.objects[plain.0].native_hash_profile = true;
        for id in [plain, vm.root(), vm.global()] {
            assert!(!vm.objects[id.0].native_object_instance);
            assert!(matches!(
                vm.call(
                    method(&vm, "valueOf"),
                    Value::Object(id),
                    &[],
                    &mut RejectHost
                )
                .unwrap_err()
                .kind,
                ErrorKind::Unsupported("Object primitive native receiver category profile")
            ));
        }
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
    fn object_primitives_function_string_does_not_drop_closure_but_value_of_rejects_it() {
        let mut vm = setup();
        let mut nested = define("", &[0]);
        nested.push(0x3e);
        execute(&mut vm, &define("outer", &nested)).unwrap();
        let outer = vm.get(vm.root(), "outer").unwrap();
        let closure = vm
            .call(
                outer.clone(),
                Value::Object(vm.root()),
                &[],
                &mut RejectHost,
            )
            .unwrap()
            .value;
        let native = vm.native(910).unwrap();
        for receiver in [
            outer.clone(),
            closure.clone(),
            Value::Object(native),
            method(&vm, "toString"),
        ] {
            vm.current_target = Some(vm.root());
            assert_eq!(
                vm.invoke(
                    method(&vm, "toString"),
                    receiver,
                    &[Value::Undefined],
                    &mut RejectHost
                )
                .unwrap(),
                Value::string("[type Function]")
            );
        }
        assert_eq!(
            vm.call(method(&vm, "valueOf"), outer.clone(), &[], &mut RejectHost)
                .unwrap()
                .value,
            outer
        );
        assert!(matches!(
            vm.call(method(&vm, "valueOf"), closure, &[], &mut RejectHost)
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported("Object.valueOf captured activation/With profile")
        ));
        assert_eq!(vm.host_calls, 0);
    }

    struct ShadowHost;
    impl Host for ShadowHost {
        fn call(&mut self, call: HostCall<'_>) -> std::result::Result<Value, String> {
            assert_eq!(call.binding, 911);
            Ok(Value::string("shadowed"))
        }
        fn command(&mut self, _: ObjectId, _: vm::HostCommand) -> std::result::Result<(), String> {
            Err("unexpected synthetic host command".into())
        }
    }

    #[test]
    fn object_primitives_named_lookup_uses_actual_shadowing_callable() {
        let mut vm = setup();
        let receiver = constructed(&mut vm, 0x43);
        let callback = vm.native(911).unwrap();
        vm.set(receiver, "toString", Value::Object(callback))
            .unwrap();
        vm.set(vm.root(), "receiver", Value::Object(receiver))
            .unwrap();
        let code = vm
            .register(Arc::new(Program::parse(&named_call_bytes()).unwrap()))
            .unwrap();
        let result = vm.execute(code, vm.root(), &mut ShadowHost).unwrap();
        assert_eq!(result.stack, vec![Value::string("shadowed")]);
        assert_eq!(result.host_calls, 1);
    }

    #[test]
    fn object_primitives_context_budget_and_receiver_gates_preserve_state() {
        let mut vm = setup();
        let receiver = constructed(&mut vm, 0x43);
        assert!(matches!(
            vm.call(
                method(&vm, "toString"),
                Value::Object(receiver),
                &[],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::Unsupported("Object.toString requires current Environment")
        ));
        let unbound = vm.object().unwrap();
        vm.current_target = Some(unbound);
        assert!(matches!(
            vm.invoke(
                method(&vm, "toString"),
                Value::Object(receiver),
                &[],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::Unsupported("Object.toString requires bound builtin context")
        ));
        vm.current_target = Some(vm.root());
        vm.current = Some((500, 0x52));
        vm.current_code = Some(vm::CodeId(0));
        vm.limits.string_units = vm.string_units + 14;
        let before = (vm.object_count(), vm.properties, vm.string_units);
        let error = vm
            .invoke(
                method(&vm, "toString"),
                Value::Object(receiver),
                &[],
                &mut RejectHost,
            )
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Budget("allocated string units"));
        assert_eq!(error.origin.unwrap().source_offset, 500);
        assert_eq!((vm.object_count(), vm.properties, vm.string_units), before);
        let array = vm.array(&[]).unwrap();
        for receiver in [
            Value::Object(array),
            Value::Null,
            Value::Undefined,
            Value::Number(1.0),
            Value::string("object"),
        ] {
            assert!(matches!(
                vm.call(method(&vm, "valueOf"), receiver, &[], &mut RejectHost)
                    .unwrap_err()
                    .kind,
                ErrorKind::Unsupported(_)
            ));
        }
        assert!(matches!(
            vm.call(
                method(&vm, "valueOf"),
                Value::Object(ObjectId(usize::MAX)),
                &[],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::InvalidObject(_)
        ));
        assert_eq!(vm.depth, 0);
        assert_eq!(vm.host_calls, 0);
    }
}
