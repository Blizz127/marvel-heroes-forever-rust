//! Synthetic checks of the explicitly installed Object/Function graph.
use super::*;

fn setup() -> Vm {
    Vm::new(15, Limits::default()).unwrap()
}

fn object(vm: &Vm, id: ObjectId, key: &str) -> ObjectId {
    let Value::Object(value) = vm.get(id, key).unwrap() else {
        panic!("expected object: {key}")
    };
    value
}

#[test]
fn bootstrap_original_graph_keeps_thirteen_cold_method_parents_and_stable_function() {
    let mut vm = setup();
    let oc = vm.install_object_constructor().unwrap();
    let op = object(&vm, oc, "prototype");
    let fp = object(&vm, oc, "__proto__");
    let fc = object(&vm, fp, "constructor");
    assert_eq!(vm.function_constructor, Some(fc));
    assert_eq!(object(&vm, fc, "prototype"), fp);
    assert_eq!(object(&vm, fc, "__proto__"), fp);
    assert_eq!(object(&vm, op, "constructor"), oc);
    assert_eq!(object(&vm, fp, "__proto__"), op);
    assert_eq!(vm.get(op, "__proto__").unwrap(), Value::Undefined);
    assert_eq!(vm.own_property_flags(op, "__proto__").unwrap(), None);
    assert_eq!(vm.objects[op.0].properties.len(), 9);
    assert_eq!(vm.objects[fp.0].properties.len(), 6);
    assert_eq!(vm.objects[oc.0].properties.len(), 3);
    assert_eq!(vm.objects[fc.0].properties.len(), 2);
    assert_eq!(vm.own_flags(oc, &text("registerClass")), 7);
    let mut methods = vec![object(&vm, oc, "registerClass")];
    for name in [
        "addProperty",
        "hasOwnProperty",
        "watch",
        "unwatch",
        "isPropertyEnumerable",
        "isPrototypeOf",
        "toString",
        "valueOf",
    ] {
        assert_eq!(vm.own_flags(op, &text(name)), 1);
        methods.push(object(&vm, op, name));
    }
    for name in ["apply", "call", "toString", "valueOf"] {
        assert_eq!(vm.own_flags(fp, &text(name)), 1);
        methods.push(object(&vm, fp, name));
    }
    methods.sort();
    methods.dedup();
    assert_eq!(methods.len(), 13);
    for id in methods {
        assert_eq!(vm.objects[id.0].prototype, None);
        assert!(matches!(
            vm.objects[id.0].properties.get(&text("__proto__")),
            Some(Property::NativeMarker(
                NativeMarker::BootstrapFunctionParent
            ))
        ));
        assert_eq!(vm.own_flags(id, &text("__proto__")), 3);
        assert_eq!(vm.get(id, "__proto__").unwrap(), Value::Undefined);
        assert_eq!(vm.get(id, "call").unwrap(), Value::Undefined);
        assert_eq!(vm.get(id, "apply").unwrap(), Value::Undefined);
        assert!(vm.set(id, "__resolve", Value::Undefined).is_err());
    }
    assert_eq!(vm.get(vm.global(), "Function").unwrap(), Value::Undefined);
    let before = vm.object_count();
    assert_eq!(vm.install_function_conversion().unwrap(), fc);
    assert_eq!(vm.object_count(), before);
    assert_eq!(vm.install_function_conversion().unwrap(), fc);
    assert_eq!(object(&vm, vm.global(), "Function"), fc);
    assert_eq!(vm.own_flags(fc, &text("prototype")), 3);
    assert_eq!(vm.own_flags(fp, &text("constructor")), 3);
    assert_eq!(vm.own_flags(vm.global(), &text("Object")), 1);
    assert_eq!(vm.own_flags(vm.global(), &text("Function")), 1);
}

#[test]
fn bootstrap_does_not_backfill_existing_functions_and_later_functions_use_warm_parent() {
    let mut vm = setup();
    let prior = vm.native(400).unwrap();
    let prior_keys = vm.objects[prior.0].properties.len();
    vm.install_object_constructor().unwrap();
    assert_eq!(vm.objects[prior.0].prototype, None);
    assert_eq!(vm.objects[prior.0].properties.len(), prior_keys);
    let later = vm.native(401).unwrap();
    assert_eq!(vm.objects[later.0].prototype, vm.function_prototype);
    assert!(matches!(vm.get(later, "apply").unwrap(), Value::Object(_)));
    // One empty original-format DefineFunction record, synthetic code only.
    let code = vm
        .register(Arc::new(
            Program::parse(&[0x9b, 6, 0, b'f', 0, 0, 0, 0, 0, 0]).unwrap(),
        ))
        .unwrap();
    vm.execute(code, vm.root(), &mut RejectHost).unwrap();
    let function = object(&vm, vm.root(), "f");
    assert_eq!(vm.objects[function.0].prototype, vm.function_prototype);
}

#[test]
fn bootstrap_function_own_native_methods_remain_distinct_and_explicitly_unsupported() {
    let mut vm = setup();
    vm.install_object_constructor().unwrap();
    let fp = vm.function_prototype.unwrap();
    let op = vm.object_prototype.unwrap();
    for name in ["toString", "valueOf"] {
        let native = object(&vm, fp, name);
        assert_ne!(native, object(&vm, op, name));
        assert_eq!(
            vm.classify_callability(&Value::Object(native)).unwrap(),
            ValueCallability::Callable
        );
        assert!(matches!(
            vm.call(
                Value::Object(native),
                Value::Object(fp),
                &[],
                &mut RejectHost
            )
            .unwrap_err()
            .kind,
            ErrorKind::UnsupportedNativeFunction(_)
        ));
        assert_eq!(vm.host_calls, 0);
    }
    for prototype in [op, fp] {
        assert!(vm.set(prototype, "constructor", Value::Null).is_err());
    }
    assert!(!vm.delete_member(op, &text("constructor")).unwrap());
    // FunctionPrototype deletion remains outside the generic deletion profile.
    assert!(vm.delete_member(fp, &text("constructor")).is_err());
    assert!(
        vm.apply_property_flags(&[
            Value::Object(op),
            Value::string("constructor"),
            Value::Number(0.0),
            Value::Number(7.0)
        ])
        .is_err()
    );
    assert_eq!(vm.own_flags(op, &text("constructor")), 3);
}

#[test]
fn bootstrap_budget_rejection_does_not_publish_partial_graph_or_relink_existing_values() {
    let mut complete = setup();
    complete.install_object_constructor().unwrap();
    let full = (
        complete.object_count(),
        complete.properties,
        complete.string_units,
    );
    for limit in 0..3 {
        let mut vm = setup();
        let before = (vm.object_count(), vm.properties, vm.string_units);
        match limit {
            0 => vm.limits.objects = full.0 - 1,
            1 => vm.limits.properties = full.1 - 1,
            _ => vm.limits.string_units = full.2 - 1,
        }
        assert!(matches!(
            vm.install_object_constructor().unwrap_err().kind,
            ErrorKind::Budget(_)
        ));
        assert_eq!((vm.object_count(), vm.properties, vm.string_units), before);
        assert!(vm.object_constructor.is_none());
        assert!(vm.object_prototype.is_none());
        assert!(vm.function_prototype.is_none());
        assert!(vm.function_constructor.is_none());
        assert!(vm.function_conversion.is_none());
        assert_eq!(vm.get(vm.global(), "Object").unwrap(), Value::Undefined);
        vm.limits = Limits::default();
        vm.install_object_constructor().unwrap();
        assert_eq!((vm.object_count(), vm.properties, vm.string_units), full);
    }
}

#[test]
fn bootstrap_corrected_graph_remains_usable_by_bounded_deferred_factories() {
    let mut vm = setup();
    vm.install_object_constructor().unwrap();
    vm.install_function_conversion().unwrap();
    vm.install_array_constructor().unwrap();
    vm.bind_builtin_context_target(vm.root()).unwrap();
    vm.install_flash_geom_point_rectangle().unwrap();
    vm.register_deferred_builtin(vm.global(), "Mouse", BuiltinFactory::Mouse, 1)
        .unwrap();
    vm.register_deferred_builtin(vm.global(), "String", BuiltinFactory::String, 1)
        .unwrap();
    vm.current_target = Some(vm.root());
    let geom = vm.geom_namespace.unwrap().geom;
    for (namespace, key) in [
        (vm.global(), "Mouse"),
        (vm.global(), "String"),
        (geom, "Point"),
        (geom, "Rectangle"),
    ] {
        let Value::DeferredBuiltin(name) = vm.get(namespace, key).unwrap() else {
            panic!()
        };
        let ctor = vm.resolve_deferred_builtin(&name).unwrap();
        assert_eq!(vm.objects[ctor.0].prototype, vm.function_prototype);
    }
    let ei = vm.build_external_interface(47).unwrap();
    assert_eq!(
        vm.objects[ei.constructor.0].prototype,
        vm.function_prototype
    );
}
