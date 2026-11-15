//! Fresh ExternalInterface graph, without namespace installation or invocation.
//!
//! Native setup is documented in `docs/EXTERNAL_INTERFACE_BINDING.md`. The
//! private `rea-external-interface/final.bundle.json` records
//! `ev_1babf1ae6b6d390b7a93ebfbdf326636e240b408c9eb09c32ed3b48f51199040`
//! and `ev_0c83febe079b3472a36975647fa99a31c1eb1b11ea55b137a933fca5efc108a4`
//! establish factory 0x1401f2280, its method table, cached parents and constructor
//! metadata. This original helper models that fresh graph only. VM resource
//! failure rollback is an engineering policy, not a claim about native OOM.

use super::{ErrorKind, Kind, NativeMarker, ObjectId, Property, Result, Value, Vm, text};

/// Four distinct objects produced by one fresh factory invocation.
///
/// The constructor owns `call` and `addCallback`; those methods are not installed
/// on the separate prototype. These IDs do not imply a namespace binding,
/// materialization cache entry, active movie context, or installed host handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalInterfaceObjects {
    pub constructor: ObjectId,
    pub prototype: ObjectId,
    pub call: ObjectId,
    pub add_callback: ObjectId,
}

impl Vm {
    /// Build the native-backed fresh ExternalInterface object graph.
    ///
    /// Requires existing cached Object and Function prototypes. Public bindings
    /// are not consulted. `call_binding` identifies a host callback but neither
    /// installs one nor chooses its movie/receiver. The available marker remains
    /// explicit until the VM has a supported current-context getter.
    ///
    /// Prototype links use this VM's existing fresh metadata representation;
    /// the dedicated object kinds keep unsupported native mutations separate.
    /// A returned error leaves existing objects and allocation counters intact.
    /// No successful result is cached: repeated calls build distinct graphs.
    pub fn build_external_interface(
        &mut self,
        call_binding: u32,
    ) -> Result<ExternalInterfaceObjects> {
        let (Some(function_parent), Some(object_parent)) =
            (self.function_prototype, self.object_prototype)
        else {
            return self.fail(ErrorKind::Unsupported(
                "ExternalInterface requires cached Object and Function prototypes",
            ));
        };
        self.check_object(function_parent)?;
        self.check_object(object_parent)?;

        // Only newly allocated objects are written below. No existing parent,
        // namespace or cache is mutated before the complete graph is returned.
        let object_count = self.objects.len();
        let property_count = self.properties;
        let string_units = self.string_units;
        let result = (|| {
            let constructor =
                self.alloc(Kind::ExternalInterfaceConstructor, Some(function_parent))?;
            self.raw_member(
                constructor,
                text("available"),
                Property::NativeMarker(NativeMarker::ExternalAvailable),
                0,
            )?;

            // Preserve the concrete native table's installation order.
            let add_callback = self.alloc(
                Kind::UnsupportedNative("flash.external.ExternalInterface.addCallback"),
                Some(function_parent),
            )?;
            self.raw_member(
                constructor,
                text("addCallback"),
                Property::Data(Value::Object(add_callback)),
                7,
            )?;
            let call = self.alloc(Kind::Native(call_binding), Some(function_parent))?;
            self.raw_member(
                constructor,
                text("call"),
                Property::Data(Value::Object(call)),
                7,
            )?;

            let prototype = self.alloc(Kind::ExternalInterfacePrototype, Some(object_parent))?;
            self.raw_member(
                prototype,
                text("constructor"),
                Property::Data(Value::Object(constructor)),
                3,
            )?;
            self.raw_member(
                constructor,
                text("prototype"),
                Property::Data(Value::Object(prototype)),
                3,
            )?;

            Ok(ExternalInterfaceObjects {
                constructor,
                prototype,
                call,
                add_callback,
            })
        })();
        if result.is_err() {
            self.objects.truncate(object_count);
            self.properties = property_count;
            self.string_units = string_units;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::super::Limits;
    use super::*;

    fn setup() -> Vm {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm
    }

    fn own_ledger(vm: &Vm, id: ObjectId) -> Vec<(String, u8)> {
        vm.objects[id.0]
            .properties
            .keys()
            .map(|key| (String::from_utf16(key).unwrap(), vm.own_flags(id, key)))
            .collect()
    }

    #[test]
    fn fresh_external_interface_matches_native_own_ledger() {
        let mut vm = setup();
        let function_parent = vm.function_prototype;
        let object_parent = vm.object_prototype;
        let global_keys = vm.objects[vm.global.0].insertion_keys.clone();
        let root_keys = vm.objects[vm.root.0].insertion_keys.clone();
        let graph = vm.build_external_interface(73).unwrap();

        assert_eq!(
            own_ledger(&vm, graph.constructor),
            [
                ("__proto__".into(), 3),
                ("addCallback".into(), 7),
                ("available".into(), 0),
                ("call".into(), 7),
                ("prototype".into(), 3),
            ]
        );
        assert_eq!(
            own_ledger(&vm, graph.prototype),
            [("__proto__".into(), 3), ("constructor".into(), 3)]
        );
        for id in [graph.call, graph.add_callback] {
            assert_eq!(own_ledger(&vm, id), [("__proto__".into(), 3)]);
        }
        for id in [graph.constructor, graph.call, graph.add_callback] {
            assert_eq!(vm.objects[id.0].prototype, function_parent);
        }
        assert_eq!(vm.objects[graph.prototype.0].prototype, object_parent);
        assert!(matches!(
            vm.objects[graph.constructor.0].kind,
            Kind::ExternalInterfaceConstructor
        ));
        assert!(matches!(
            vm.objects[graph.prototype.0].kind,
            Kind::ExternalInterfacePrototype
        ));
        assert!(matches!(vm.objects[graph.call.0].kind, Kind::Native(73)));
        assert!(matches!(
            vm.objects[graph.add_callback.0].kind,
            Kind::UnsupportedNative(_)
        ));
        assert!(matches!(
            vm.objects[graph.constructor.0]
                .properties
                .get(&text("available")),
            Some(Property::NativeMarker(NativeMarker::ExternalAvailable))
        ));
        assert_eq!(
            vm.get(graph.constructor, "prototype").unwrap(),
            Value::Object(graph.prototype)
        );
        assert_eq!(
            vm.get(graph.prototype, "constructor").unwrap(),
            Value::Object(graph.constructor)
        );
        assert_eq!(
            vm.get(graph.constructor, "call").unwrap(),
            Value::Object(graph.call)
        );
        assert_eq!(
            vm.get(graph.constructor, "addCallback").unwrap(),
            Value::Object(graph.add_callback)
        );
        assert_eq!(vm.objects[vm.global.0].insertion_keys, global_keys);
        assert_eq!(vm.objects[vm.root.0].insertion_keys, root_keys);

        let again = vm.build_external_interface(73).unwrap();
        let mut ids = [
            graph.constructor,
            graph.prototype,
            graph.call,
            graph.add_callback,
            again.constructor,
            again.prototype,
            again.call,
            again.add_callback,
        ];
        ids.sort();
        assert!(ids.windows(2).all(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn external_interface_uses_cached_not_public_prototypes() {
        let mut vm = setup();
        let function_parent = vm.function_prototype;
        let object_parent = vm.object_prototype;
        let object = vm.object_constructor.unwrap();
        let decoy = vm.object().unwrap();
        vm.set(object, "prototype", Value::Object(decoy)).unwrap();
        vm.set(vm.global, "Function", Value::Object(decoy)).unwrap();
        vm.set(decoy, "prototype", Value::Object(decoy)).unwrap();

        let graph = vm.build_external_interface(1).unwrap();
        assert_eq!(vm.objects[graph.constructor.0].prototype, function_parent);
        assert_eq!(vm.objects[graph.call.0].prototype, function_parent);
        assert_eq!(vm.objects[graph.add_callback.0].prototype, function_parent);
        assert_eq!(vm.objects[graph.prototype.0].prototype, object_parent);
        assert_eq!(vm.get(object, "prototype").unwrap(), Value::Object(decoy));
        assert_eq!(vm.get(vm.global, "Function").unwrap(), Value::Object(decoy));
    }

    #[test]
    fn external_interface_failure_discards_unpublished_graph() {
        // Fail after different portions of the native setup order. The exact
        // fresh ledger needs four objects, nine properties and 80 name units.
        for (objects_left, properties_left, units_left, expected) in [
            (3, 9, 80, "objects"),
            (4, 8, 80, "properties"),
            (4, 9, 79, "property name units"),
        ] {
            let mut vm = setup();
            let before = (vm.objects.len(), vm.properties, vm.string_units);
            let global_keys = vm.objects[vm.global.0].insertion_keys.clone();
            vm.limits.objects = before.0 + objects_left;
            vm.limits.properties = before.1 + properties_left;
            vm.limits.string_units = before.2 + units_left;
            assert_eq!(
                vm.build_external_interface(1).unwrap_err().kind,
                ErrorKind::Budget(expected)
            );
            assert_eq!((vm.objects.len(), vm.properties, vm.string_units), before);
            assert_eq!(vm.objects[vm.global.0].insertion_keys, global_keys);
            vm.limits = Limits::default();
            assert_eq!(
                vm.build_external_interface(1).unwrap().constructor,
                ObjectId(before.0)
            );
        }
    }

    #[test]
    fn external_interface_requires_both_cached_parents() {
        for missing_function in [false, true] {
            let mut vm = setup();
            if missing_function {
                vm.function_prototype = None;
            } else {
                vm.object_prototype = None;
            }
            let before = (vm.objects.len(), vm.properties, vm.string_units);
            assert!(matches!(
                vm.build_external_interface(1).unwrap_err().kind,
                ErrorKind::Unsupported(_)
            ));
            assert_eq!((vm.objects.len(), vm.properties, vm.string_units), before);
        }
    }
}
