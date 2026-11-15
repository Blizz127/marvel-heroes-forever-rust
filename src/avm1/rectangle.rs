//! Partial native Rectangle graph and the zero-argument New route.
//!
//! Contract: `docs/REA_RECTANGLE_CONSTRUCTION.md`, private retained verification
//! `rea-rectangle-construction/verification.json`, SHA-256
//! 85102e75375831dc691e1a78aec6c5c79b4f7ca3996846d0f68479408c89d35e.
//! Rectangle has its own category, getter and setter dispatch. Its seven stored
//! prototype placeholders must never stand in for computed instance getters.
//! Fifteen real prototype methods are omitted: unknown reads and reflection
//! remain gated at every lookup hop. All Rectangle writes are unsupported here.
//! Existing Value::Number represents native Integer zero without retaining that
//! native tag. Rust allocation accounting is not native allocation-count parity.

use super::{ErrorKind, Kind, ObjectId, Property, Result, Text, Value, Vm, text};

const PROTOTYPE_FIELDS: [&str; 7] = [
    "left",
    "top",
    "right",
    "bottom",
    "topLeft",
    "bottomRight",
    "size",
];
const INSTANCE_FIELDS: [&str; 4] = ["x", "y", "width", "height"];
const CONSTRUCTOR_LEDGER: [(&str, u8); 2] = [("__proto__", 3), ("prototype", 3)];
const PROTOTYPE_LEDGER: [(&str, u8); 9] = [
    ("__proto__", 3),
    ("constructor", 3),
    ("left", 2),
    ("top", 2),
    ("right", 2),
    ("bottom", 2),
    ("topLeft", 2),
    ("bottomRight", 2),
    ("size", 2),
];
const INSTANCE_LEDGER: [(&str, u8); 6] = [
    ("__proto__", 3),
    ("x", 0),
    ("y", 0),
    ("width", 0),
    ("height", 0),
    ("__constructor__", 3),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectangleObjects {
    pub constructor: ObjectId,
    pub prototype: ObjectId,
}

/// Both roles have native category 0x11; their prototype predicates differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RectangleRole {
    Prototype,
    Instance,
}

fn key_is(key: &[u16], name: &str) -> bool {
    key.iter().copied().eq(name.encode_utf16())
}

impl Vm {
    /// Build one partial graph, without namespace or deferred-registry writes.
    /// Only successful completion fills the canonical cache. Outer publication
    /// owns its own rollback; local errors restore every new object and charge.
    /// This failure policy is an engineering bound, not native OOM behavior.
    pub fn build_rectangle_graph(&mut self) -> Result<RectangleObjects> {
        if self.rectangle_objects.is_some() {
            return self.fail(ErrorKind::Unsupported("Rectangle graph already built"));
        }
        self.rectangle_context()?;
        let (function, object) = self.rectangle_parents()?;
        // Four metadata names total38 units; seven prototype fields total40.
        self.rectangle_allocation_profile(2, 11, 78)?;
        let before = (self.objects.len(), self.properties, self.string_units);
        let result = (|| {
            let constructor = self.alloc(Kind::RectangleConstructor, Some(function))?;
            let prototype = self.alloc(
                Kind::RectangleObject(RectangleRole::Prototype),
                Some(object),
            )?;
            // These guarded linked identities represent native marker metadata;
            // they do not authorize generic descriptor inspection or mutation.
            self.raw_member(
                prototype,
                text("constructor"),
                Property::Data(Value::Object(constructor)),
                3,
            )?;
            for name in PROTOTYPE_FIELDS {
                self.raw_member(prototype, text(name), Property::Data(Value::Number(0.0)), 2)?;
            }
            self.raw_member(
                constructor,
                text("prototype"),
                Property::Data(Value::Object(prototype)),
                3,
            )?;
            Ok(RectangleObjects {
                constructor,
                prototype,
            })
        })();
        match result {
            Ok(graph) => {
                self.rectangle_objects = Some(graph);
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

    /// Central New/NewMethod dispatch must establish exactly zero arguments.
    /// Direct calls, argument conversion and borrowed receivers remain gated.
    /// The native zero-argument callback preserves this allocated receiver and
    /// all four initialized values; its temporary return cannot replace it.
    pub(super) fn construct_rectangle_zero(&mut self, constructor: ObjectId) -> Result<ObjectId> {
        self.rectangle_context()?;
        let graph = self.rectangle_graph_profile()?;
        if constructor != graph.constructor {
            return self.fail(ErrorKind::Unsupported(
                "Rectangle requires canonical constructor",
            ));
        }
        // The argc==1 current GLOBAL.Object comparison is absent on New0.
        // The cached parents and live public C/P links are checked separately.
        self.rectangle_allocation_profile(1, 6, 37)?;
        let before = (self.objects.len(), self.properties, self.string_units);
        let result = (|| {
            let instance = self.alloc(
                Kind::RectangleObject(RectangleRole::Instance),
                Some(graph.prototype),
            )?;
            for name in INSTANCE_FIELDS {
                self.raw_member(instance, text(name), Property::Data(Value::Number(0.0)), 0)?;
            }
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

    /// None applies only to a non-Rectangle kind. Unknown Rectangle members
    /// fail explicitly; omitted native methods are never observed as absent.
    /// Generic lookup must call this before dispatch at each prototype hop.
    pub(super) fn rectangle_read(&self, id: ObjectId, key: &Text) -> Result<Option<Value>> {
        self.check_object(id)?;
        if !matches!(
            self.objects[id.0].kind,
            Kind::RectangleConstructor | Kind::RectangleObject(_)
        ) {
            return Ok(None);
        }
        let graph = self.rectangle_graph_profile()?;
        let value = if id == graph.constructor {
            if key_is(key, "__proto__") {
                Value::Object(self.function_prototype.unwrap())
            } else if key_is(key, "prototype") {
                Value::Object(graph.prototype)
            } else {
                return self.fail(ErrorKind::Unsupported(
                    "unimplemented Rectangle callable member",
                ));
            }
        } else {
            let role = self.rectangle_object_profile(id, graph)?;
            if key_is(key, "__proto__") {
                Value::Object(match role {
                    RectangleRole::Prototype => self.object_prototype.unwrap(),
                    RectangleRole::Instance => graph.prototype,
                })
            } else if key_is(key, "constructor")
                || (role == RectangleRole::Instance && key_is(key, "__constructor__"))
            {
                Value::Object(graph.constructor)
            } else if role == RectangleRole::Prototype
                && PROTOTYPE_FIELDS.iter().any(|name| key_is(key, name))
            {
                Value::Number(0.0)
            } else if role == RectangleRole::Instance
                && INSTANCE_FIELDS.iter().any(|name| key_is(key, name))
            {
                let Some(Property::Data(value)) = self.objects[id.0].properties.get(key) else {
                    unreachable!("Rectangle profile requires four own data fields")
                };
                value.clone()
            } else {
                // Includes all seven computed instance fields, regardless of
                // the same spelling's stored descriptor on Rectangle.prototype.
                return self.fail(ErrorKind::Unsupported(
                    "unimplemented Rectangle object member",
                ));
            }
        };
        Ok(Some(value))
    }

    /// Rectangle's normal setter first dispatches seven specialized names.
    /// This initial profile rejects all writes; neither those names nor unknown
    /// members may fall through to a generic or Point setter.
    pub(super) fn rectangle_write(
        &mut self,
        id: ObjectId,
        _key: &Text,
        _value: Value,
    ) -> Result<bool> {
        self.check_object(id)?;
        if matches!(
            self.objects[id.0].kind,
            Kind::RectangleConstructor | Kind::RectangleObject(_)
        ) {
            return self.fail(ErrorKind::Unsupported("Rectangle mutation profile"));
        }
        Ok(false)
    }

    fn rectangle_context(&self) -> Result<()> {
        let Some(target) = self.current_target else {
            return self.fail(ErrorKind::Unsupported("Rectangle requires current target"));
        };
        if !self.is_bound_builtin_context_target(target)? {
            return self.fail(ErrorKind::Unsupported(
                "Rectangle requires bound builtin context",
            ));
        }
        Ok(())
    }

    fn rectangle_allocation_profile(
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

    fn rectangle_parents(&self) -> Result<(ObjectId, ObjectId)> {
        let (Some(function), Some(object)) = (self.function_prototype, self.object_prototype)
        else {
            return self.fail(ErrorKind::Unsupported(
                "Rectangle requires cached native parents",
            ));
        };
        self.check_object(function)?;
        self.check_object(object)?;
        if function == object
            || !matches!(self.objects[function.0].kind, Kind::FunctionPrototype)
            || !matches!(self.objects[object.0].kind, Kind::Plain)
            || self.objects[object.0].prototype.is_some()
            || !self.rectangle_link_matches(function, object)
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
            return self.fail(ErrorKind::Unsupported("Rectangle cached parent profile"));
        }
        Ok((function, object))
    }

    fn rectangle_graph_profile(&self) -> Result<RectangleObjects> {
        let Some(graph) = self.rectangle_objects else {
            return self.fail(ErrorKind::Unsupported("Rectangle graph is not installed"));
        };
        let (function, object) = self.rectangle_parents()?;
        self.check_object(graph.constructor)?;
        self.check_object(graph.prototype)?;
        if graph.constructor == graph.prototype
            || [function, object].contains(&graph.constructor)
            || [function, object].contains(&graph.prototype)
            || !matches!(
                self.objects[graph.constructor.0].kind,
                Kind::RectangleConstructor
            )
            || !matches!(
                self.objects[graph.prototype.0].kind,
                Kind::RectangleObject(RectangleRole::Prototype)
            )
            || !self.rectangle_link_matches(graph.constructor, function)
            || !self.rectangle_link_matches(graph.prototype, object)
            || !self.rectangle_data_link_matches(graph.constructor, "prototype", graph.prototype)
            || !self.rectangle_data_link_matches(graph.prototype, "constructor", graph.constructor)
            || !self.rectangle_ledger_matches(graph.constructor, &CONSTRUCTOR_LEDGER)
            || !self.rectangle_ledger_matches(graph.prototype, &PROTOTYPE_LEDGER)
            || PROTOTYPE_FIELDS.iter().any(|name| {
                !matches!(self.objects[graph.prototype.0].properties.get(&text(name)),
                    Some(Property::Data(Value::Number(value))) if value.to_bits() == 0)
            })
        {
            return self.fail(ErrorKind::Unsupported("Rectangle canonical graph metadata"));
        }
        Ok(graph)
    }

    fn rectangle_object_profile(
        &self,
        id: ObjectId,
        graph: RectangleObjects,
    ) -> Result<RectangleRole> {
        self.check_object(id)?;
        let Kind::RectangleObject(role) = self.objects[id.0].kind else {
            return self.fail(ErrorKind::Unsupported("Rectangle requires typed object"));
        };
        match role {
            RectangleRole::Prototype if id == graph.prototype => {}
            RectangleRole::Instance
                if self.rectangle_link_matches(id, graph.prototype)
                    && self.rectangle_data_link_matches(
                        id,
                        "__constructor__",
                        graph.constructor,
                    )
                    && self.rectangle_ledger_matches(id, &INSTANCE_LEDGER) =>
            {
                for name in INSTANCE_FIELDS {
                    let Some(Property::Data(value)) =
                        self.objects[id.0].properties.get(&text(name))
                    else {
                        return self
                            .fail(ErrorKind::Unsupported("Rectangle own field data profile"));
                    };
                    self.check_value(value)?;
                }
            }
            _ => return self.fail(ErrorKind::Unsupported("Rectangle object metadata profile")),
        }
        Ok(role)
    }

    fn rectangle_link_matches(&self, id: ObjectId, parent: ObjectId) -> bool {
        self.objects[id.0].prototype == Some(parent)
            && self.rectangle_data_link_matches(id, "__proto__", parent)
    }

    fn rectangle_data_link_matches(&self, id: ObjectId, name: &str, expected: ObjectId) -> bool {
        let key = text(name);
        self.own_flags(id, &key) == 3
            && matches!(self.objects[id.0].properties.get(&key), Some(Property::Data(Value::Object(value))) if *value == expected)
    }

    fn rectangle_ledger_matches(&self, id: ObjectId, ledger: &[(&str, u8)]) -> bool {
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

    fn assert_zero(value: Option<Value>) {
        let Some(Value::Number(number)) = value else {
            panic!("represented native Integer0 must be numeric zero")
        };
        assert_eq!(number.to_bits(), 0);
    }

    #[test]
    fn rectangle_graph_and_instances_keep_distinct_roles_and_exact_ledgers() {
        let mut vm = base();
        let global_keys = vm.objects[vm.global.0].insertion_keys.clone();
        let before = counts(&vm);
        let graph = vm.build_rectangle_graph().unwrap();
        assert_eq!(counts(&vm), (before.0 + 2, before.1 + 11, before.2 + 78));
        assert_eq!(vm.rectangle_graph_profile().unwrap(), graph);
        assert_eq!(
            ledger(&vm, graph.constructor),
            vec![("__proto__".into(), 3), ("prototype".into(), 3)]
        );
        assert_eq!(
            ledger(&vm, graph.prototype),
            vec![
                ("__proto__".into(), 3),
                ("constructor".into(), 3),
                ("left".into(), 2),
                ("top".into(), 2),
                ("right".into(), 2),
                ("bottom".into(), 2),
                ("topLeft".into(), 2),
                ("bottomRight".into(), 2),
                ("size".into(), 2),
            ]
        );
        let before = counts(&vm);
        let first = vm.construct_rectangle_zero(graph.constructor).unwrap();
        assert_eq!(counts(&vm), (before.0 + 1, before.1 + 6, before.2 + 37));
        let second = vm.construct_rectangle_zero(graph.constructor).unwrap();
        assert_ne!(first, second);
        assert_ne!(first, graph.prototype);
        assert!(matches!(
            vm.objects[first.0].kind,
            Kind::RectangleObject(RectangleRole::Instance)
        ));
        assert!(matches!(
            vm.objects[graph.prototype.0].kind,
            Kind::RectangleObject(RectangleRole::Prototype)
        ));
        assert_eq!(
            ledger(&vm, first),
            vec![
                ("__proto__".into(), 3),
                ("x".into(), 0),
                ("y".into(), 0),
                ("width".into(), 0),
                ("height".into(), 0),
                ("__constructor__".into(), 3),
            ]
        );
        for id in [first, second] {
            for name in ["x", "y", "width", "height"] {
                assert_zero(vm.rectangle_read(id, &text(name)).unwrap());
            }
            assert_eq!(
                vm.rectangle_read(id, &text("__proto__")).unwrap(),
                Some(Value::Object(graph.prototype))
            );
            for name in ["constructor", "__constructor__"] {
                assert_eq!(
                    vm.rectangle_read(id, &text(name)).unwrap(),
                    Some(Value::Object(graph.constructor))
                );
            }
            assert!(
                !vm.objects[id.0]
                    .properties
                    .contains_key(&text("constructor"))
            );
        }
        assert_eq!(vm.objects[vm.global.0].insertion_keys, global_keys);
        assert_eq!(vm.host_calls, 0);
    }

    #[test]
    fn rectangle_prototype_placeholders_never_replace_computed_instance_getters() {
        let mut vm = base();
        let graph = vm.build_rectangle_graph().unwrap();
        let instance = vm.construct_rectangle_zero(graph.constructor).unwrap();
        for name in [
            "left",
            "top",
            "right",
            "bottom",
            "topLeft",
            "bottomRight",
            "size",
        ] {
            assert_zero(vm.rectangle_read(graph.prototype, &text(name)).unwrap());
            assert!(vm.rectangle_read(instance, &text(name)).is_err());
            assert!(vm.rectangle_read(graph.constructor, &text(name)).is_err());
        }
        for name in ["x", "y", "width", "height", "length", "__constructor__"] {
            assert!(vm.rectangle_read(graph.prototype, &text(name)).is_err());
        }
        for id in [graph.constructor, graph.prototype, instance] {
            for name in ["omittedGeometryMethod", "length", "__resolve"] {
                assert!(vm.rectangle_read(id, &text(name)).is_err());
            }
        }
    }

    #[test]
    fn rectangle_all_writes_reject_without_mutating_instance_or_graph() {
        let mut vm = base();
        let graph = vm.build_rectangle_graph().unwrap();
        let instance = vm.construct_rectangle_zero(graph.constructor).unwrap();
        let before = counts(&vm);
        for id in [graph.constructor, graph.prototype, instance] {
            for name in [
                "x",
                "y",
                "width",
                "height",
                "left",
                "top",
                "right",
                "bottom",
                "topLeft",
                "bottomRight",
                "size",
                "extra",
                "__proto__",
                "constructor",
            ] {
                assert!(
                    vm.rectangle_write(id, &text(name), Value::Number(9.0))
                        .is_err()
                );
            }
        }
        assert_eq!(counts(&vm), before);
        assert_eq!(vm.rectangle_graph_profile().unwrap(), graph);
        for name in ["x", "y", "width", "height"] {
            assert_zero(vm.rectangle_read(instance, &text(name)).unwrap());
        }
        assert_eq!(vm.rectangle_read(vm.root, &text("unknown")).unwrap(), None);
        assert!(
            !vm.rectangle_write(vm.root, &text("x"), Value::Null)
                .unwrap()
        );
    }

    #[test]
    fn rectangle_zero_argument_context_does_not_compare_current_global_object() {
        let mut vm = base();
        let graph = vm.build_rectangle_graph().unwrap();
        vm.set(vm.global, "Object", Value::Null).unwrap();
        let instance = vm.construct_rectangle_zero(graph.constructor).unwrap();
        assert_zero(vm.rectangle_read(instance, &text("width")).unwrap());
        let before = counts(&vm);
        vm.current_target = None;
        assert!(vm.construct_rectangle_zero(graph.constructor).is_err());
        assert_eq!(counts(&vm), before);
        let unbound = vm.object().unwrap();
        vm.current_target = Some(unbound);
        let before = counts(&vm);
        assert!(vm.construct_rectangle_zero(graph.constructor).is_err());
        assert_eq!(counts(&vm), before);
        vm.bind_builtin_context_target(unbound).unwrap();
        assert!(vm.construct_rectangle_zero(graph.constructor).is_ok());
    }

    #[test]
    fn rectangle_resource_preflight_is_atomic_at_graph_and_instance_boundaries() {
        for instance in [false, true] {
            for resource in 0..3 {
                let mut vm = base();
                let graph = instance.then(|| vm.build_rectangle_graph().unwrap());
                let before = counts(&vm);
                let costs = if instance { (1, 6, 37) } else { (2, 11, 78) };
                match resource {
                    0 => vm.limits.objects = before.0 + costs.0 - 1,
                    1 => vm.limits.properties = before.1 + costs.1 - 1,
                    2 => vm.limits.string_units = before.2 + costs.2 - 1,
                    _ => unreachable!(),
                }
                let error = if let Some(graph) = graph {
                    vm.construct_rectangle_zero(graph.constructor).unwrap_err()
                } else {
                    vm.build_rectangle_graph().unwrap_err()
                };
                assert!(matches!(error.kind, ErrorKind::Budget(_)));
                assert_eq!(counts(&vm), before);
                assert_eq!(vm.rectangle_objects, graph);
                vm.limits.objects = before.0 + costs.0;
                vm.limits.properties = before.1 + costs.1;
                vm.limits.string_units = before.2 + costs.2;
                if let Some(graph) = graph {
                    assert_eq!(vm.rectangle_graph_profile().unwrap(), graph);
                    assert!(vm.construct_rectangle_zero(graph.constructor).is_ok());
                } else {
                    assert!(vm.build_rectangle_graph().is_ok());
                }
                assert_eq!(
                    counts(&vm),
                    (before.0 + costs.0, before.1 + costs.1, before.2 + costs.2)
                );
            }
        }
        let mut vm = base();
        let graph = vm.build_rectangle_graph().unwrap();
        let before = counts(&vm);
        assert!(vm.build_rectangle_graph().is_err());
        assert_eq!(counts(&vm), before);
        assert_eq!(vm.rectangle_objects, Some(graph));
    }

    #[test]
    fn rectangle_changed_graph_and_instance_descriptors_fail_closed() {
        for mutation in 0..6 {
            let mut vm = base();
            let graph = vm.build_rectangle_graph().unwrap();
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
                        text("size"),
                        Property::Data(Value::Number(0.0)),
                        0,
                    )
                    .unwrap(),
                4 => vm
                    .raw_member(
                        graph.prototype,
                        text("topLeft"),
                        Property::Data(Value::Number(1.0)),
                        2,
                    )
                    .unwrap(),
                5 => vm
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
                _ => unreachable!(),
            }
            let before = counts(&vm);
            assert!(vm.construct_rectangle_zero(graph.constructor).is_err());
            assert!(vm.rectangle_read(graph.prototype, &text("size")).is_err());
            assert_eq!(counts(&vm), before);
        }
        let mut vm = base();
        let graph = vm.build_rectangle_graph().unwrap();
        let instance = vm.construct_rectangle_zero(graph.constructor).unwrap();
        vm.raw_member(
            instance,
            text("width"),
            Property::Accessor {
                getter: Value::Undefined,
                setter: None,
            },
            0,
        )
        .unwrap();
        assert!(vm.rectangle_read(instance, &text("width")).is_err());
        assert!(vm.rectangle_read(instance, &text("x")).is_err());
    }

    #[test]
    fn rectangle_and_point_roles_and_constructors_cannot_substitute_for_each_other() {
        let mut vm = base();
        let rectangle = vm.build_rectangle_graph().unwrap();
        let point = vm.build_point_graph().unwrap();
        let rect = vm.construct_rectangle_zero(rectangle.constructor).unwrap();
        let pt = vm.construct_point_zero(point.constructor).unwrap();
        let before = counts(&vm);
        assert!(vm.construct_rectangle_zero(point.constructor).is_err());
        assert!(vm.construct_point_zero(rectangle.constructor).is_err());
        assert_eq!(counts(&vm), before);
        assert_eq!(vm.rectangle_read(pt, &text("x")).unwrap(), None);
        assert_eq!(vm.point_read(rect, &text("x")).unwrap(), None);
        assert!(vm.rectangle_read(rect, &text("length")).is_err());
        assert_eq!(vm.rectangle_objects, Some(rectangle));
        assert_eq!(vm.point_objects, Some(point));
    }
}
