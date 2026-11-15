//! Focused interpreter integration checks, with synthetic action bytes only.
use super::*;

fn setup(current_byte: Option<u8>) -> Vm {
    let mut vm = Vm::new(15, Limits::default()).unwrap();
    vm.install_object_constructor().unwrap();
    vm.install_array_constructor().unwrap();
    vm.bind_builtin_context_target(vm.root()).unwrap();
    vm.bind_global_extensions(GlobalExtensionsProfile {
        context_target: vm.root(),
        current_byte,
    })
    .unwrap();
    vm
}

fn assignment(value: bool) -> (Vec<u8>, usize) {
    let mut bytes = tests::push(&[Value::string("_global")]);
    bytes.push(0x1c);
    bytes.extend(tests::push(&[
        Value::string("gfxExtensions"),
        Value::Bool(value),
    ]));
    let offset = bytes.len();
    bytes.push(0x4f);
    (bytes, offset)
}

fn execute(vm: &mut Vm, bytes: &[u8]) -> (CodeId, Result<Execution>) {
    let code = vm
        .register(Arc::new(Program::parse(bytes).unwrap()))
        .unwrap();
    (code, vm.execute(code, vm.root(), &mut RejectHost))
}

#[test]
fn global_extensions_original_integer_literal_category_reaches_native_assignment() {
    // Synthetic opcode sequence uses raw Push subtype7, as the unchanged
    // original13554 does. It does not replace an Integer literal with Bool.
    for integer in [1i32, 0, -1, i32::MIN] {
        let mut vm = setup(None);
        let mut bytes = tests::push(&[Value::string("_global")]);
        bytes.push(0x1c);
        let mut payload = vec![0];
        payload.extend(b"gfxExtensions\0");
        payload.push(7);
        payload.extend(integer.to_le_bytes());
        bytes.push(0x96);
        bytes.extend((payload.len() as u16).to_le_bytes());
        bytes.extend(payload);
        let assignment_offset = bytes.len();
        bytes.extend([0x4f, 0]);
        let (code, result) = execute(&mut vm, &bytes);
        assert!(result.unwrap().stack.is_empty());
        let enabled = integer != 0;
        let info = vm.global_extensions_info().unwrap();
        assert_eq!(info.current_byte, Some(if enabled { 1 } else { 2 }));
        assert_eq!(info.marker_written, Some(true));
        assert_eq!(
            info.last_write,
            Some(CodeOrigin {
                code,
                source_offset: assignment_offset
            })
        );
        assert_eq!(
            vm.get(vm.global(), "gfxExtensions").unwrap(),
            Value::Bool(enabled)
        );
        assert_eq!(
            vm.get(vm.global(), "gfxVersion").unwrap(),
            if enabled {
                Value::string("4.2.23")
            } else {
                Value::Undefined
            }
        );
        assert_eq!(vm.host_calls, 0);
    }
}

#[test]
fn global_extensions_opcode_preserves_effects_and_deferred_mouse_host_error() {
    let mut vm = setup(None);
    vm.register_deferred_builtin(vm.global(), "Mouse", BuiltinFactory::Mouse, 1)
        .unwrap();
    assert_eq!(vm.global_extensions_info().unwrap().current_byte, None);
    let (mut bytes, assignment_offset) = assignment(true);
    bytes.extend(tests::push(&[Value::Number(0.0), Value::string("Mouse")]));
    bytes.push(0x1c);
    bytes.extend(tests::push(&[Value::string("hide")]));
    let call_offset = bytes.len();
    bytes.extend([0x52, 0]);
    let (code, result) = execute(&mut vm, &bytes);
    let error = result.unwrap_err();
    assert!(matches!(error.kind, ErrorKind::Host(ref message)
        if message == "Mouse.hide user-event host is unbound"));
    assert_eq!(error.source_offset, Some(call_offset));
    assert_eq!(error.opcode, Some(0x52));
    let info = vm.global_extensions_info().unwrap();
    assert_eq!(info.current_byte, Some(1));
    assert_eq!(info.marker_written, Some(true));
    assert_eq!(
        info.last_write,
        Some(CodeOrigin {
            code,
            source_offset: assignment_offset
        })
    );
    assert_eq!(
        vm.get(vm.global(), "gfxExtensions").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        vm.get(vm.global(), "gfxVersion").unwrap(),
        Value::string("4.2.23")
    );
    assert!(matches!(
        vm.objects[vm.global.0]
            .properties
            .get(&text("gfxExtensions")),
        Some(Property::NativeMarker(NativeMarker::GlobalExtensions))
    ));
    assert_eq!(vm.own_flags(vm.global(), &text("gfxExtensions")), 0);
    assert_eq!(vm.own_flags(vm.global(), &text("gfxVersion")), 0);
    let mouse = vm.mouse_objects.unwrap();
    assert_eq!(
        vm.get(vm.global(), "Mouse").unwrap(),
        Value::Object(mouse.constructor)
    );
    assert_eq!(vm.own_flags(vm.global(), &text("Mouse")), 1);
    assert_eq!(vm.host_calls, 1);
}

#[test]
fn global_extensions_opcode_readonly_final_write_keeps_earlier_native_effects() {
    let mut vm = Vm::new(15, Limits::default()).unwrap();
    vm.raw_member(
        vm.global(),
        text("gfxExtensions"),
        Property::Data(Value::Bool(false)),
        4,
    )
    .unwrap();
    vm.raw_member(
        vm.global(),
        text("gfxVersion"),
        Property::Data(Value::string("supplied")),
        7,
    )
    .unwrap();
    vm.bind_builtin_context_target(vm.root()).unwrap();
    vm.bind_global_extensions(GlobalExtensionsProfile {
        context_target: vm.root(),
        current_byte: None,
    })
    .unwrap();
    for value in [true, false] {
        let (mut bytes, _) = assignment(value);
        bytes.push(0);
        let (_, result) = execute(&mut vm, &bytes);
        assert!(result.unwrap().stack.is_empty());
        let info = vm.global_extensions_info().unwrap();
        assert_eq!(info.current_byte, Some(if value { 1 } else { 2 }));
        assert_eq!(info.marker_written, Some(false));
        assert!(matches!(
            vm.objects[vm.global.0]
                .properties
                .get(&text("gfxExtensions")),
            Some(Property::Data(Value::Bool(false)))
        ));
        assert_eq!(
            vm.get(vm.global(), "gfxExtensions").unwrap(),
            Value::Bool(value)
        );
        assert_eq!(
            vm.get(vm.global(), "gfxVersion").unwrap(),
            Value::string("4.2.23")
        );
        assert_eq!(vm.own_flags(vm.global(), &text("gfxVersion")), 7);
        assert_eq!(vm.own_flags(vm.global(), &text("gfxExtensions")), 4);
    }
}

#[test]
fn global_extensions_getter_presence_uses_supplied_byte_not_stored_value() {
    for (byte, value, found) in [
        (0, Value::Undefined, false),
        (1, Value::Bool(true), true),
        (2, Value::Bool(false), true),
        (255, Value::Bool(false), true),
    ] {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.set(
            vm.global(),
            "gfxExtensions",
            Value::string("unrelated stored value"),
        )
        .unwrap();
        vm.bind_builtin_context_target(vm.root()).unwrap();
        vm.bind_global_extensions(GlobalExtensionsProfile {
            context_target: vm.root(),
            current_byte: Some(byte),
        })
        .unwrap();
        vm.current_target = Some(vm.root());
        assert_eq!(vm.get(vm.global(), "gfxExtensions").unwrap(), value);
        assert_eq!(
            vm.has_key(vm.global(), &text("gfxExtensions")).unwrap(),
            found
        );
        assert_eq!(
            vm.descriptor(vm.global(), &text("gfxExtensions"))
                .unwrap()
                .is_some(),
            found
        );
    }
    let mut vm = setup(None);
    vm.current_target = Some(vm.root());
    assert!(vm.get(vm.global(), "gfxExtensions").is_err());
    assert!(vm.has_key(vm.global(), &text("gfxExtensions")).is_err());
    assert!(vm.descriptor(vm.global(), &text("gfxExtensions")).is_err());
}

#[test]
fn global_extensions_central_protection_and_unbound_ordinary_model_stay_distinct() {
    let mut ordinary = Vm::new(15, Limits::default()).unwrap();
    ordinary
        .set(ordinary.global(), "gfxExtensions", Value::Bool(true))
        .unwrap();
    assert_eq!(
        ordinary.get(ordinary.global(), "gfxExtensions").unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        ordinary.get(ordinary.global(), "gfxVersion").unwrap(),
        Value::Undefined
    );
    assert!(ordinary.global_extensions_info().is_none());

    let mut vm = setup(Some(1));
    vm.current_target = Some(vm.root());
    let before = (vm.object_count(), vm.properties, vm.string_units);
    for name in ["gfxVersion", "__proto__", "__resolve"] {
        assert!(vm.set(vm.global(), name, Value::Null).is_err());
    }
    for name in ["gfxExtensions", "gfxVersion", "__proto__", "__resolve"] {
        assert!(vm.delete_member(vm.global(), &text(name)).is_err());
        assert!(vm.own_property_flags(vm.global(), name).is_err());
    }
    assert!(vm.own_enumerable_names(vm.global()).is_err());
    assert!(vm.enumerate_names(vm.global()).is_err());
    assert!(
        vm.apply_property_flags(&[Value::Object(vm.global()), Value::Null, Value::Number(1.0)])
            .is_err()
    );
    assert_eq!((vm.object_count(), vm.properties, vm.string_units), before);
    let alias = vm.alloc(Kind::Plain, Some(vm.global())).unwrap();
    assert!(vm.get(alias, "gfxExtensions").is_err());
    assert!(vm.has_key(alias, &text("gfxExtensions")).is_err());
    assert!(vm.descriptor(alias, &text("gfxExtensions")).is_err());
}

#[test]
fn global_extensions_marker_exception_never_admits_foreign_namespace_marker() {
    let mut vm = setup(Some(1));
    vm.current_target = Some(vm.root());
    vm.set(vm.global(), "gfxExtensions", Value::Bool(true))
        .unwrap();
    vm.deferred_namespace_profile(vm.global()).unwrap();
    let foreign = vm.alloc(Kind::Plain, None).unwrap();
    vm.raw_member(
        foreign,
        text("gfxExtensions"),
        Property::NativeMarker(NativeMarker::GlobalExtensions),
        0,
    )
    .unwrap();
    assert!(vm.deferred_namespace_profile(foreign).is_err());
    vm.raw_member(
        vm.global(),
        text("wrongKey"),
        Property::NativeMarker(NativeMarker::GlobalExtensions),
        0,
    )
    .unwrap();
    assert!(vm.deferred_namespace_profile(vm.global()).is_err());
}
