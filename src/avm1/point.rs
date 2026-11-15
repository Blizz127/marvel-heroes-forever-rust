//! Bounded canonical Point graph and zero-argument construction.
//!
//! Static native contract: `docs/REA_POINT_CONSTRUCTION.md`, retained private
//! `rea-point-construction/verification.json` SHA-256
//! 091b90e5310925301d80847204cb74a5cd87b23ce74233bdaec2ca839d533100.
//! This original implementation omits the three static and seven prototype
//! methods. Unknown reads, reflection and mutation must remain explicitly gated
//! wherever the VM encounters these kinds, including prototype-chain traversal.
//! Native Integer zero is represented by the existing AVM1 Number category;
//! this does not preserve the native tag or claim allocation-count parity.
//! No namespace registration, deferred resolution or callback runs here.

use super::{ErrorKind, Kind, ObjectId, Property, Result, Text, Value, Vm, text};

/// Canonical identities for the partial native graph, without namespace state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointObjects {
    pub constructor: ObjectId,
    pub prototype: ObjectId,
}

/// Both native roles use category 0x10, but have different getter predicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PointRole {
    Prototype,
    Instance,
}

fn key_is(key: &[u16], name: &str) -> bool {
    key.iter().copied().eq(name.encode_utf16())
}

impl Vm {
    /// Build one unpublished partial graph using the cached native parents.
    /// The deferred registry owns publication/reuse and any outer rollback.
    /// Failure restores this helper's new objects and allocation counters; that
    /// engineering policy is not an emulation of native allocation failure.
    pub fn build_point_graph(&mut self) -> Result<PointObjects> {
        if self.point_objects.is_some() {
            return self.fail(ErrorKind::Unsupported("Point graph already built"));
        }
        self.point_context()?;
        let (function, object) = self.point_parents()?;
        // Two __proto__ keys, prototype, constructor and length: 44 units.
        self.point_allocation_profile(2, 5, 44)?;
        let before = (self.objects.len(), self.properties, self.string_units);
        let result = (|| {
            let constructor = self.alloc(Kind::PointConstructor, Some(function))?;
            let prototype = self.alloc(Kind::PointObject(PointRole::Prototype), Some(object))?;
            // Guarded identities represent the native linked marker metadata;
            // raw descriptor reflection remains outside this partial profile.
            self.raw_member(
                prototype,
                text("constructor"),
                Property::Data(Value::Object(constructor)),
                3,
            )?;
            self.raw_member(
                prototype,
                text("length"),
                Property::Data(Value::Number(0.0)),
                2,
            )?;
            self.raw_member(
                constructor,
                text("prototype"),
                Property::Data(Value::Object(prototype)),
                3,
            )?;
            Ok(PointObjects {
                constructor,
                prototype,
            })
        })();
        match result {
            Ok(graph) => {
                self.point_objects = Some(graph);
                Ok(graph)
            }
            Err(error) => {
                self.objects.truncate(before.0);
                self.properties = before.1;
                self.string_units = before.2;
                Err(error)
            }
        }
    }

    /// Only the shared New/NewMethod zero-argument route may call this helper.
    /// Direct Point(), borrowed callbacks and nonzero arguments stay gated by
    /// central invocation. The returned receiver is the allocated instance;
    /// there is no callback result capable of replacing it on this profile.
    pub(super) fn construct_point_zero(&mut self, constructor: ObjectId) -> Result<ObjectId> {
        self.point_context()?;
        let graph = self.point_graph_profile()?;
        if constructor != graph.constructor {
            return self.fail(ErrorKind::Unsupported(
                "Point requires canonical constructor",
            ));
        }
        // Zero arguments bypass the native current GLOBAL.Object comparison.
        // Cached native parents and the current C.prototype/P.constructor links
        // are validated independently by point_graph_profile.
        self.point_allocation_profile(1, 4, 26)?;
        let before = (self.objects.len(), self.properties, self.string_units);
        let result = (|| {
            let instance = self.alloc(
                Kind::PointObject(PointRole::Instance),
                Some(graph.prototype),
            )?;
            self.raw_member(instance, text("x"), Property::Data(Value::Number(0.0)), 0)?;
            self.raw_member(instance, text("y"), Property::Data(Value::Number(0.0)), 0)?;
            self.raw_member(
                instance,
                text("__constructor__"),
                Property::Data(Value::Object(graph.constructor)),
                3,
            )?;
            Ok(instance)
        })();
        if result.is_err() {
            self.objects.truncate(before.0);
            self.properties = before.1;
            self.string_units = before.2;
        }
        result
    }

    /// None means the receiver is not a Point kind. Every unknown Point member
    /// fails rather than exposing an omitted native descriptor as Undefined.
    /// Consult this at each generic lookup hop, before descriptor dispatch.
    pub(super) fn point_read(&self, id: ObjectId, key: &Text) -> Result<Option<Value>> {
        self.check_object(id)?;
        if !matches!(
            self.objects[id.0].kind,
            Kind::PointConstructor | Kind::PointObject(_)
        ) {
            return Ok(None);
        }
        let graph = self.point_graph_profile()?;
        let value = if id == graph.constructor {
            if key_is(key, "__proto__") {
                Value::Object(self.function_prototype.unwrap())
            } else if key_is(key, "prototype") {
                Value::Object(graph.prototype)
            } else {
                return self.fail(ErrorKind::Unsupported(
                    "unimplemented Point callable member",
                ));
            }
        } else {
            let role = self.point_object_profile(id, graph)?;
            if key_is(key, "__proto__") {
                Value::Object(match role {
                    PointRole::Prototype => self.object_prototype.unwrap(),
                    PointRole::Instance => graph.prototype,
                })
            } else if key_is(key, "constructor")
                || (role == PointRole::Instance && key_is(key, "__constructor__"))
            {
                Value::Object(graph.constructor)
            } else if role == PointRole::Prototype && key_is(key, "length") {
                Value::Number(0.0)
            } else if role == PointRole::Instance && (key_is(key, "x") || key_is(key, "y")) {
                // The validated own data shadows ancestors. No coercion,
                // deferred materialization or accessor invocation occurs here.
                let Some(Property::Data(value)) = self.objects[id.0].properties.get(key) else {
                    unreachable!("Point instance profile requires own x/y data")
                };
                value.clone()
            } else {
                return self.fail(ErrorKind::Unsupported("unimplemented Point object member"));
            }
        };
        Ok(Some(value))
    }

    /// The initial construction profile admits no Point mutation. Future x/y
    /// storage support must keep its watch/descriptor and value-copy contract
    /// explicit; unknown properties must not fall through to a generic setter.
    pub(super) fn point_write(&mut self, id: ObjectId, _key: &Text, _value: Value) -> Result<bool> {
        self.check_object(id)?;
        if matches!(
            self.objects[id.0].kind,
            Kind::PointConstructor | Kind::PointObject(_)
        ) {
            return self.fail(ErrorKind::Unsupported("Point mutation profile"));
        }
        Ok(false)
    }

    fn point_context(&self) -> Result<()> {
        let Some(target) = self.current_target else {
            return self.fail(ErrorKind::Unsupported("Point requires current target"));
        };
        if !self.is_bound_builtin_context_target(target)? {
            return self.fail(ErrorKind::Unsupported(
                "Point requires bound builtin context",
            ));
        }
        Ok(())
    }

    fn point_allocation_profile(
        &self,
        objects: usize,
        properties: usize,
        units: usize,
    ) -> Result<()> {
        if objects > self.limits.objects.saturating_sub(self.objects.len()) {
            return self.fail(ErrorKind::Budget("objects"));
        }
        if properties > self.limits.properties.saturating_sub(self.properties) {
            return self.fail(ErrorKind::Budget("properties"));
        }
        if units > self.limits.string_units.saturating_sub(self.string_units) {
            return self.fail(ErrorKind::Budget("allocated string units"));
        }
        Ok(())
    }

    fn point_parents(&self) -> Result<(ObjectId, ObjectId)> {
        let (Some(function), Some(object)) = (self.function_prototype, self.object_prototype)
        else {
            return self.fail(ErrorKind::Unsupported(
                "Point requires cached native parents",
            ));
        };
        self.check_object(function)?;
        self.check_object(object)?;
        if function == object
            || !matches!(self.objects[function.0].kind, Kind::FunctionPrototype)
            || !matches!(self.objects[object.0].kind, Kind::Plain)
            || self.objects[object.0].prototype.is_some()
            || !self.point_link_matches(function, object)
            || [function, object].iter().any(|id| {
                let parent = &self.objects[id.0];
                parent
                    .properties
                    .values()
                    .any(|value| matches!(value, Property::Accessor { .. }))
                    || parent
                        .insertion_keys
                        .iter()
                        .any(|key| key_is(key, "__resolve"))
            })
        {
            return self.fail(ErrorKind::Unsupported("Point cached parent profile"));
        }
        Ok((function, object))
    }

    fn point_graph_profile(&self) -> Result<PointObjects> {
        let Some(graph) = self.point_objects else {
            return self.fail(ErrorKind::Unsupported("Point graph is not installed"));
        };
        let (function, object) = self.point_parents()?;
        self.check_object(graph.constructor)?;
        self.check_object(graph.prototype)?;
        if graph.constructor == graph.prototype
            || [function, object].contains(&graph.constructor)
            || [function, object].contains(&graph.prototype)
            || !matches!(
                self.objects[graph.constructor.0].kind,
                Kind::PointConstructor
            )
            || !matches!(
                self.objects[graph.prototype.0].kind,
                Kind::PointObject(PointRole::Prototype)
            )
            || !self.point_link_matches(graph.constructor, function)
            || !self.point_link_matches(graph.prototype, object)
            || !self.point_data_link_matches(graph.constructor, "prototype", graph.prototype, 3)
            || !self.point_data_link_matches(graph.prototype, "constructor", graph.constructor, 3)
            || !self.point_ledger_matches(graph.constructor, &[("__proto__", 3), ("prototype", 3)])
            || !self.point_ledger_matches(
                graph.prototype,
                &[("__proto__", 3), ("constructor", 3), ("length", 2)],
            )
            || !matches!(self.objects[graph.prototype.0].properties.get(&text("length")),
                Some(Property::Data(Value::Number(value))) if value.to_bits() == 0)
        {
            return self.fail(ErrorKind::Unsupported("Point canonical graph metadata"));
        }
        Ok(graph)
    }

    fn point_object_profile(&self, id: ObjectId, graph: PointObjects) -> Result<PointRole> {
        self.check_object(id)?;
        let Kind::PointObject(role) = self.objects[id.0].kind else {
            return self.fail(ErrorKind::Unsupported("Point requires typed object"));
        };
        match role {
            PointRole::Prototype if id == graph.prototype => {}
            PointRole::Instance
                if self.point_link_matches(id, graph.prototype)
                    && self.point_data_link_matches(
                        id,
                        "__constructor__",
                        graph.constructor,
                        3,
                    )
                    && self.point_ledger_matches(
                        id,
                        &[("__proto__", 3), ("x", 0), ("y", 0), ("__constructor__", 3)],
                    ) =>
            {
                for key in ["x", "y"] {
                    let Some(Property::Data(value)) = self.objects[id.0].properties.get(&text(key))
                    else {
                        return self.fail(ErrorKind::Unsupported("Point coordinate data profile"));
                    };
                    self.check_value(value)?;
                }
            }
            _ => return self.fail(ErrorKind::Unsupported("Point object metadata profile")),
        }
        Ok(role)
    }

    fn point_link_matches(&self, id: ObjectId, parent: ObjectId) -> bool {
        self.objects[id.0].prototype == Some(parent)
            && self.point_data_link_matches(id, "__proto__", parent, 3)
    }

    fn point_data_link_matches(
        &self,
        id: ObjectId,
        key: &str,
        expected: ObjectId,
        flags: u8,
    ) -> bool {
        let key = text(key);
        self.own_flags(id, &key) == flags
            && matches!(self.objects[id.0].properties.get(&key), Some(Property::Data(Value::Object(value))) if *value == expected)
    }

    fn point_ledger_matches(&self, id: ObjectId, ledger: &[(&str, u8)]) -> bool {
        let object = &self.objects[id.0];
        object.properties.len() == ledger.len()
            && object.flags.len() == ledger.len()
            && object.insertion_keys.len() == ledger.len()
            && object
                .insertion_keys
                .iter()
                .zip(ledger)
                .all(|(key, (name, flags))| {
                    key_is(key, name)
                        && object.properties.contains_key(key)
                        && self.own_flags(id, key) == *flags
                })
    }
}

#[cfg(test)]
mod tests {
    use super::super::Limits;
    use super::*;

    fn base() -> Vm {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.bind_builtin_context_target(vm.root).unwrap();
        vm.current_target = Some(vm.root);
        vm
    }

    fn counts(vm: &Vm) -> (usize, usize, usize) {
        (vm.objects.len(), vm.properties, vm.string_units)
    }

    fn ledger(vm: &Vm, id: ObjectId) -> Vec<(String, u8)> {
        vm.objects[id.0]
            .insertion_keys
            .iter()
            .map(|key| (String::from_utf16(key).unwrap(), vm.own_flags(id, key)))
            .collect()
    }

    #[test]
    fn point_partial_graph_and_fresh_instances_keep_roles_order_and_flags() {
        let mut vm = base();
        let global_keys = vm.objects[vm.global.0].insertion_keys.clone();
        let before = counts(&vm);
        let graph = vm.build_point_graph().unwrap();
        assert_eq!(counts(&vm), (before.0 + 2, before.1 + 5, before.2 + 44));
        assert_eq!(vm.point_graph_profile().unwrap(), graph);
        assert_eq!(
            ledger(&vm, graph.constructor),
            vec![("__proto__".into(), 3), ("prototype".into(), 3)]
        );
        assert_eq!(
            ledger(&vm, graph.prototype),
            vec![
                ("__proto__".into(), 3),
                ("constructor".into(), 3),
                ("length".into(), 2)
            ]
        );
        assert_eq!(
            vm.point_read(graph.prototype, &text("length")).unwrap(),
            Some(Value::Number(0.0))
        );
        let before = counts(&vm);
        let first = vm.construct_point_zero(graph.constructor).unwrap();
        assert_eq!(counts(&vm), (before.0 + 1, before.1 + 4, before.2 + 26));
        let second = vm.construct_point_zero(graph.constructor).unwrap();
        assert_ne!(first, second);
        assert_ne!(first, graph.prototype);
        assert!(matches!(
            vm.objects[first.0].kind,
            Kind::PointObject(PointRole::Instance)
        ));
        assert!(matches!(
            vm.objects[graph.prototype.0].kind,
            Kind::PointObject(PointRole::Prototype)
        ));
        assert_eq!(
            ledger(&vm, first),
            vec![
                ("__proto__".into(), 3),
                ("x".into(), 0),
                ("y".into(), 0),
                ("__constructor__".into(), 3)
            ]
        );
        for id in [first, second] {
            assert_eq!(
                vm.point_read(id, &text("__proto__")).unwrap(),
                Some(Value::Object(graph.prototype))
            );
            for key in ["constructor", "__constructor__"] {
                assert_eq!(
                    vm.point_read(id, &text(key)).unwrap(),
                    Some(Value::Object(graph.constructor))
                );
            }
            assert!(
                !vm.objects[id.0]
                    .properties
                    .contains_key(&text("constructor"))
            );
            for key in ["x", "y"] {
                let Some(Value::Number(value)) = vm.point_read(id, &text(key)).unwrap() else {
                    panic!("fresh coordinate must be represented numeric zero")
                };
                assert_eq!(value.to_bits(), 0);
            }
        }
        assert_eq!(vm.objects[vm.global.0].insertion_keys, global_keys);
        assert_eq!(vm.host_calls, 0);
    }

    #[test]
    fn point_unknown_reads_and_all_point_mutations_stay_explicit() {
        let mut vm = base();
        let graph = vm.build_point_graph().unwrap();
        let instance = vm.construct_point_zero(graph.constructor).unwrap();
        let before = counts(&vm);
        for id in [graph.constructor, graph.prototype, instance] {
            for key in ["omittedNativeMember", "toString", "__resolve"] {
                assert!(vm.point_read(id, &text(key)).is_err());
            }
            for key in ["x", "y", "length", "extra", "__proto__", "constructor"] {
                assert!(vm.point_write(id, &text(key), Value::Number(9.0)).is_err());
            }
        }
        assert!(vm.point_read(instance, &text("length")).is_err());
        assert!(
            vm.point_read(graph.prototype, &text("__constructor__"))
                .is_err()
        );
        assert!(vm.point_read(graph.prototype, &text("x")).is_err());
        assert_eq!(vm.point_read(vm.root, &text("unknown")).unwrap(), None);
        assert!(!vm.point_write(vm.root, &text("x"), Value::Null).unwrap());
        assert_eq!(
            vm.point_read(instance, &text("x")).unwrap(),
            Some(Value::Number(0.0))
        );
        assert_eq!(counts(&vm), before);
    }

    #[test]
    fn point_zero_argument_context_does_not_compare_current_global_object() {
        let mut vm = base();
        let graph = vm.build_point_graph().unwrap();
        vm.set(vm.global, "Object", Value::Null).unwrap();
        let instance = vm.construct_point_zero(graph.constructor).unwrap();
        assert_eq!(
            vm.point_read(instance, &text("x")).unwrap(),
            Some(Value::Number(0.0))
        );
        let before = counts(&vm);
        vm.current_target = None;
        assert!(vm.construct_point_zero(graph.constructor).is_err());
        assert_eq!(counts(&vm), before);
        let unbound = vm.object().unwrap();
        vm.current_target = Some(unbound);
        let before = counts(&vm);
        assert!(vm.construct_point_zero(graph.constructor).is_err());
        assert_eq!(counts(&vm), before);
        vm.bind_builtin_context_target(unbound).unwrap();
        assert!(vm.construct_point_zero(graph.constructor).is_ok());
    }

    #[test]
    fn point_graph_budget_rejection_has_no_partial_cache_or_allocations() {
        for resource in 0..3 {
            let mut vm = base();
            let before = counts(&vm);
            match resource {
                0 => vm.limits.objects = before.0 + 1,
                1 => vm.limits.properties = before.1 + 4,
                2 => vm.limits.string_units = before.2 + 43,
                _ => unreachable!(),
            }
            assert!(matches!(
                vm.build_point_graph().unwrap_err().kind,
                ErrorKind::Budget(_)
            ));
            assert_eq!(counts(&vm), before);
            assert_eq!(vm.point_objects, None);
            vm.limits = Limits::default();
            assert!(vm.build_point_graph().is_ok());
        }
        let mut vm = base();
        let before = counts(&vm);
        vm.limits.objects = before.0 + 2;
        vm.limits.properties = before.1 + 5;
        vm.limits.string_units = before.2 + 44;
        let graph = vm.build_point_graph().unwrap();
        let after = counts(&vm);
        assert!(vm.build_point_graph().is_err());
        assert_eq!(counts(&vm), after);
        assert_eq!(vm.point_objects, Some(graph));
    }

    #[test]
    fn point_instance_budget_rejection_preserves_completed_graph() {
        for resource in 0..3 {
            let mut vm = base();
            let graph = vm.build_point_graph().unwrap();
            let before = counts(&vm);
            match resource {
                0 => vm.limits.objects = before.0,
                1 => vm.limits.properties = before.1 + 3,
                2 => vm.limits.string_units = before.2 + 25,
                _ => unreachable!(),
            }
            assert!(matches!(
                vm.construct_point_zero(graph.constructor).unwrap_err().kind,
                ErrorKind::Budget(_)
            ));
            assert_eq!(counts(&vm), before);
            assert_eq!(vm.point_objects, Some(graph));
            assert_eq!(vm.point_graph_profile().unwrap(), graph);
            vm.limits = Limits::default();
            assert!(vm.construct_point_zero(graph.constructor).is_ok());
        }
    }

    #[test]
    fn point_changed_graph_links_descriptors_or_parents_reject_before_allocation() {
        for mutation in 0..6 {
            let mut vm = base();
            let graph = vm.build_point_graph().unwrap();
            match mutation {
                0 => vm
                    .raw_member(
                        graph.constructor,
                        text("prototype"),
                        Property::Data(Value::Null),
                        3,
                    )
                    .unwrap(),
                1 => vm
                    .raw_member(
                        graph.prototype,
                        text("constructor"),
                        Property::Data(Value::Undefined),
                        3,
                    )
                    .unwrap(),
                2 => vm.objects[graph.prototype.0].prototype = None,
                3 => vm
                    .raw_member(
                        graph.prototype,
                        text("length"),
                        Property::Data(Value::Number(0.0)),
                        0,
                    )
                    .unwrap(),
                4 => vm
                    .raw_member(
                        graph.constructor,
                        text("prototype"),
                        Property::Accessor {
                            getter: Value::Undefined,
                            setter: None,
                        },
                        3,
                    )
                    .unwrap(),
                5 => vm
                    .raw_member(
                        graph.prototype,
                        text("extra"),
                        Property::Data(Value::Null),
                        0,
                    )
                    .unwrap(),
                _ => unreachable!(),
            }
            let before = counts(&vm);
            assert!(vm.construct_point_zero(graph.constructor).is_err());
            assert!(vm.point_read(graph.prototype, &text("length")).is_err());
            assert_eq!(counts(&vm), before);
        }
    }
}
