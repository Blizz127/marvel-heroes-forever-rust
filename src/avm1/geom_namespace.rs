//! Canonical, explicitly partial native flash/geom namespace installation.
//!
//! Native bootstrap evidence is in REA_POINT_CONSTRUCTION.md,
//! REA_RECTANGLE_CONSTRUCTION.md and AS2_GLOBAL_BASELINE.md. Omitted namespace
//! entries are unsupported, never fabricated Undefined. Existing nonobjects or
//! namespaces are not repaired.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeomNamespace {
    pub flash: ObjectId,
    pub geom: ObjectId,
}

impl Vm {
    /// Install the proved fresh namespace path and a deferred Point token. This
    /// bounded bootstrap profile rejects a preexisting flash binding; broader
    /// native namespace reuse remains separate from this fresh transaction.
    pub fn install_flash_geom_point(&mut self) -> Result<GeomNamespace> {
        self.install_flash_geom(false)
    }

    /// Install the same fresh path with both proved deferred constructors.
    /// Registration completes before either namespace becomes a partial native
    /// namespace, so the second absent name never uses a runtime unknown read.
    pub fn install_flash_geom_point_rectangle(&mut self) -> Result<GeomNamespace> {
        self.install_flash_geom(true)
    }

    fn install_flash_geom(&mut self, include_rectangle: bool) -> Result<GeomNamespace> {
        if self.depth != 0 || self.saved_active || self.geom_namespace.is_some() {
            return self.fail(ErrorKind::Unsupported(
                "fresh geometry namespace installation",
            ));
        }
        let Some(parent) = self.object_prototype else {
            return self.fail(ErrorKind::Unsupported(
                "geometry requires cached Object prototype",
            ));
        };
        self.deferred_namespace_profile(self.global)?;
        if self.objects[self.global.0].prototype.is_some()
            || self.objects[self.global.0]
                .properties
                .contains_key(&text("flash"))
            || self.builtin_registry.contains_key(&text("Point"))
            || (include_rectangle && self.builtin_registry.contains_key(&text("Rectangle")))
        {
            return self.fail(ErrorKind::Unsupported(
                "geometry namespace/builtin already present",
            ));
        }
        self.check_object(parent)?;
        if !matches!(self.objects[parent.0].kind, Kind::Plain)
            || self.objects[parent.0].prototype.is_some()
            || self.objects[parent.0]
                .properties
                .contains_key(&text("Point"))
            || (include_rectangle
                && self.objects[parent.0]
                    .properties
                    .contains_key(&text("Rectangle")))
            || self.objects[parent.0]
                .properties
                .contains_key(&text("geom"))
        {
            return self.fail(ErrorKind::Unsupported(
                "geometry namespace cached parent profile",
            ));
        }
        // Native registration performs ordinary writes; the admitted parent is
        // data-only with no resolver history. No accessor or host runs here.
        if self.objects[parent.0]
            .properties
            .values()
            .any(|p| !matches!(p, Property::Data(_)))
            || self.objects[parent.0]
                .insertion_keys
                .iter()
                .any(|key| key.as_ref() == text("__resolve").as_ref())
        {
            return self.fail(ErrorKind::Unsupported(
                "geometry namespace cached parent profile",
            ));
        }
        // Two linked namespace objects, two path fields and selected tokens.
        let (properties, units) = if include_rectangle { (6, 41) } else { (5, 32) };
        if self.limits.objects.saturating_sub(self.objects.len()) < 2
            || self.limits.properties.saturating_sub(self.properties) < properties
            || self.limits.string_units.saturating_sub(self.string_units) < units
        {
            return self.fail(ErrorKind::Budget("geometry namespace installation"));
        }
        let before = (self.objects.len(), self.properties, self.string_units);
        let previous_global = self.objects[self.global.0].clone();
        let result = (|| {
            let flash = self.alloc(Kind::Plain, Some(parent))?;
            let geom = self.alloc(Kind::Plain, Some(parent))?;
            self.raw_member(flash, text("geom"), Property::Data(Value::Object(geom)), 0)?;
            self.raw_member(
                self.global,
                text("flash"),
                Property::Data(Value::Object(flash)),
                1,
            )?;
            self.register_deferred_builtin(geom, "Point", BuiltinFactory::Point, 0)?;
            if include_rectangle {
                self.register_deferred_builtin(geom, "Rectangle", BuiltinFactory::Rectangle, 0)?;
            }
            Ok(GeomNamespace { flash, geom })
        })();
        match result {
            Ok(namespace) => {
                // Publishing this guard last keeps the trusted bootstrap's
                // absent-name registration separate from runtime unknown reads.
                self.geom_namespace = Some(namespace);
                Ok(namespace)
            }
            Err(error) => {
                // Selected names were absent on entry. Registration cannot run
                // callbacks or change unrelated entries on this fresh profile.
                self.builtin_registry.remove(&text("Point"));
                if include_rectangle {
                    self.builtin_registry.remove(&text("Rectangle"));
                }
                self.objects[self.global.0] = previous_global;
                self.objects.truncate(before.0);
                self.properties = before.1;
                self.string_units = before.2;
                Err(error)
            }
        }
    }

    pub(super) fn is_geom_namespace(&self, id: ObjectId) -> bool {
        self.geom_namespace
            .is_some_and(|ns| id == ns.flash || id == ns.geom)
    }

    /// Guard the cached context namespace before its first Point factory write.
    /// Program scope/member resolution still runs normally before reaching it.
    pub(super) fn point_namespace_profile(&self, namespace: ObjectId) -> Result<()> {
        self.geometry_namespace_profile(
            namespace,
            "Point requires known cached geom namespace",
            "Point current/cached namespace mismatch",
        )
    }

    pub(super) fn rectangle_namespace_profile(&self, namespace: ObjectId) -> Result<()> {
        self.geometry_namespace_profile(
            namespace,
            "Rectangle requires known cached geom namespace",
            "Rectangle current/cached namespace mismatch",
        )
    }

    fn geometry_namespace_profile(
        &self,
        namespace: ObjectId,
        missing: &'static str,
        mismatch: &'static str,
    ) -> Result<()> {
        let Some(ns) = self.geom_namespace else {
            return self.fail(ErrorKind::Unsupported(missing));
        };
        if namespace != ns.geom
            || !self.geom_link(self.global, "flash", ns.flash, 1)
            || !self.geom_link(ns.flash, "geom", ns.geom, 0)
        {
            return self.fail(ErrorKind::Unsupported(mismatch));
        }
        for id in [ns.flash, ns.geom] {
            self.deferred_namespace_profile(id)?;
            if self.objects[id.0].prototype != self.object_prototype {
                return self.fail(ErrorKind::Unsupported("geometry namespace parent identity"));
            }
        }
        Ok(())
    }

    fn geom_link(&self, owner: ObjectId, key: &str, value: ObjectId, flags: u8) -> bool {
        matches!(self.objects[owner.0].properties.get(&text(key)),
            Some(Property::Data(Value::Object(id))) if *id == value)
            && self.own_flags(owner, &text(key)) == flags
    }

    pub(super) fn geom_namespace_member(&self, id: ObjectId, key: &Text) -> Result<Option<Value>> {
        if !self.is_geom_namespace(id) {
            return Ok(None);
        }
        self.deferred_namespace_profile(id)?;
        match self.objects[id.0].properties.get(key) {
            Some(Property::Data(value)) => Ok(Some(value.clone())),
            _ => self.fail(ErrorKind::Unsupported(
                "unimplemented native namespace member",
            )),
        }
    }

    pub(super) fn reject_geom_mutation_chain(&self, mut id: ObjectId) -> Result<()> {
        for _ in 0..self.limits.prototype_depth {
            self.check_object(id)?;
            if self.is_geom_namespace(id) {
                return self.fail(ErrorKind::Unsupported("partial native namespace mutation"));
            }
            match self.objects[id.0].prototype {
                Some(parent) => id = parent,
                None => return Ok(()),
            }
        }
        self.fail(ErrorKind::Budget("namespace mutation prototype depth"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> Vm {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm
    }
    fn push(values: &[Value]) -> Vec<u8> {
        let mut data = Vec::new();
        for value in values {
            match value {
                Value::String(value) => {
                    data.push(0);
                    data.extend(String::from_utf16(value).unwrap().as_bytes());
                    data.push(0);
                }
                Value::Number(value) => {
                    data.push(7);
                    data.extend((*value as i32).to_le_bytes());
                }
                _ => unreachable!(),
            }
        }
        let mut bytes = vec![0x96];
        bytes.extend((data.len() as u16).to_le_bytes());
        bytes.extend(data);
        bytes
    }
    fn new_point() -> Vec<u8> {
        new_geometry("Point")
    }
    fn new_geometry(name: &str) -> Vec<u8> {
        let mut bytes = push(&[Value::Number(0.0), Value::string("flash")]);
        bytes.push(0x1c);
        bytes.extend(push(&[Value::string("geom")]));
        bytes.push(0x4e);
        bytes.extend(push(&[Value::string(name)]));
        bytes.push(0x53);
        bytes
    }
    #[test]
    fn point_namespace_stays_deferred_then_newmethod_returns_fresh_instances() {
        let mut vm = setup();
        let ns = vm.install_flash_geom_point().unwrap();
        let retained = vm.get(ns.geom, "Point").unwrap();
        assert!(matches!(retained, Value::DeferredBuiltin(_)));
        assert!(vm.point_objects.is_none());
        assert_eq!(vm.own_flags(vm.global(), &text("flash")), 1);
        assert_eq!(vm.own_flags(ns.flash, &text("geom")), 0);
        assert_eq!(vm.own_flags(ns.geom, &text("Point")), 0);
        assert_eq!(vm.objects[ns.flash.0].prototype, vm.object_prototype);
        assert_eq!(vm.objects[ns.geom.0].prototype, vm.object_prototype);
        let mut bytes = new_point();
        bytes.extend(new_point());
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        vm.bind_builtin_context_target(vm.root()).unwrap();
        let result = vm.execute(code, vm.root(), &mut RejectHost).unwrap();
        let [Value::Object(first), Value::Object(second)] = result.stack.as_slice() else {
            panic!("two points")
        };
        assert_ne!(first, second);
        for id in [first, second] {
            assert_eq!(vm.get(*id, "x").unwrap(), Value::Number(0.0));
            assert_eq!(vm.get(*id, "y").unwrap(), Value::Number(0.0));
        }
        assert_eq!(result.host_calls, 0);
        assert_eq!(
            vm.get(ns.geom, "Point").unwrap(),
            Value::Object(vm.point_objects.unwrap().constructor)
        );
        assert!(matches!(retained, Value::DeferredBuiltin(_)));
        assert!(vm.get(ns.geom, "Rectangle").is_err());
        assert!(vm.own_enumerable_names(ns.geom).is_err());
        assert!(vm.own_property_flags(ns.geom, "Point").is_err());
        assert!(vm.set(ns.geom, "Point", Value::Undefined).is_err());
        assert!(vm.set(*first, "x", Value::Number(3.0)).is_err());
    }
    #[test]
    fn point_factory_current_namespace_context_and_failure_guards_preserve_token() {
        let mut vm = setup();
        let ns = vm.install_flash_geom_point().unwrap();
        let token = vm.get(ns.geom, "Point").unwrap();
        let code = vm
            .register(Arc::new(Program::parse(&new_point()).unwrap()))
            .unwrap();
        let objects = vm.object_count();
        assert!(vm.execute(code, vm.root(), &mut RejectHost).is_err());
        assert_eq!(vm.object_count(), objects);
        assert!(vm.point_objects.is_none());
        assert_eq!(vm.get(ns.geom, "Point").unwrap(), token);
        vm.bind_builtin_context_target(vm.root()).unwrap();
        vm.limits.objects = objects + 1; // The graph requires two objects.
        assert!(vm.execute(code, vm.root(), &mut RejectHost).is_err());
        assert_eq!(vm.object_count(), objects);
        assert!(vm.point_objects.is_none());
        assert_eq!(vm.get(ns.geom, "Point").unwrap(), token);
        vm.limits.objects = Limits::default().objects;
        vm.execute(code, vm.root(), &mut RejectHost).unwrap();
        assert!(vm.point_objects.is_some());
    }
    #[test]
    fn point_namespace_rejects_existing_values_and_budget_failure_without_repair() {
        for value in [Value::Undefined, Value::Number(4.0)] {
            let mut vm = setup();
            vm.set(vm.global(), "flash", value.clone()).unwrap();
            let before = (vm.object_count(), vm.properties, vm.string_units);
            assert!(vm.install_flash_geom_point().is_err());
            assert_eq!(vm.get(vm.global(), "flash").unwrap(), value);
            assert_eq!((vm.object_count(), vm.properties, vm.string_units), before);
            assert!(vm.geom_namespace.is_none());
        }
        let mut vm = setup();
        vm.limits.properties = vm.properties + 4;
        let before = (vm.object_count(), vm.properties, vm.string_units);
        assert!(vm.install_flash_geom_point().is_err());
        assert_eq!((vm.object_count(), vm.properties, vm.string_units), before);
        assert!(
            !vm.objects[vm.global.0]
                .properties
                .contains_key(&text("flash"))
        );
        assert!(!vm.builtin_registry.contains_key(&text("Point")));
    }

    #[test]
    fn combined_namespace_defers_both_graphs_and_newmethod_preserves_types() {
        let mut vm = setup();
        let ns = vm.install_flash_geom_point_rectangle().unwrap();
        let point_token = vm.get(ns.geom, "Point").unwrap();
        let rectangle_token = vm.get(ns.geom, "Rectangle").unwrap();
        assert!(matches!(point_token, Value::DeferredBuiltin(_)));
        assert!(matches!(rectangle_token, Value::DeferredBuiltin(_)));
        assert!(vm.point_objects.is_none());
        assert!(vm.rectangle_objects.is_none());
        assert_eq!(vm.own_flags(vm.global, &text("flash")), 1);
        assert_eq!(vm.own_flags(ns.flash, &text("geom")), 0);
        for name in ["Point", "Rectangle"] {
            assert_eq!(vm.own_flags(ns.geom, &text(name)), 0);
        }
        for id in [ns.flash, ns.geom] {
            assert_eq!(vm.objects[id.0].prototype, vm.object_prototype);
            assert!(
                !vm.objects[id.0]
                    .properties
                    .contains_key(&text("constructor"))
            );
            assert!(
                !vm.objects[id.0]
                    .properties
                    .contains_key(&text("__constructor__"))
            );
        }
        let mut bytes = new_point();
        bytes.extend(new_geometry("Rectangle"));
        bytes.extend(new_geometry("Rectangle"));
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        vm.bind_builtin_context_target(vm.root()).unwrap();
        let result = vm.execute(code, vm.root(), &mut RejectHost).unwrap();
        let [
            Value::Object(point),
            Value::Object(first),
            Value::Object(second),
        ] = result.stack.as_slice()
        else {
            panic!("one Point and two Rectangle results")
        };
        assert_ne!(first, second);
        assert_ne!(point, first);
        assert_ne!(
            vm.point_objects.unwrap().constructor,
            vm.rectangle_objects.unwrap().constructor
        );
        for id in [first, second] {
            for name in ["x", "y", "width", "height"] {
                assert_eq!(vm.get(*id, name).unwrap(), Value::Number(0.0));
            }
            assert!(vm.get(*id, "left").is_err());
        }
        assert!(vm.get(*point, "width").is_err());
        assert_eq!(
            vm.get(ns.geom, "Point").unwrap(),
            Value::Object(vm.point_objects.unwrap().constructor)
        );
        assert_eq!(
            vm.get(ns.geom, "Rectangle").unwrap(),
            Value::Object(vm.rectangle_objects.unwrap().constructor)
        );
        assert!(matches!(point_token, Value::DeferredBuiltin(_)));
        assert!(matches!(rectangle_token, Value::DeferredBuiltin(_)));
        assert_eq!(result.host_calls, 0);
        assert!(vm.get(ns.geom, "Matrix").is_err());
        assert!(vm.own_enumerable_names(ns.geom).is_err());
        assert!(vm.own_property_flags(ns.geom, "Rectangle").is_err());
        assert!(vm.set(ns.geom, "Rectangle", Value::Undefined).is_err());
        let before = (vm.object_count(), vm.properties, vm.string_units);
        assert!(
            vm.register_deferred_builtin(ns.geom, "Matrix", BuiltinFactory::Unsupported, 0)
                .is_err()
        );
        assert_eq!((vm.object_count(), vm.properties, vm.string_units), before);
        assert!(!vm.builtin_registry.contains_key(&text("Matrix")));
    }

    #[test]
    fn combined_namespace_budget_boundaries_are_atomic_and_fresh_only() {
        for resource in 0..3 {
            let mut vm = setup();
            let before = (vm.object_count(), vm.properties, vm.string_units);
            match resource {
                0 => vm.limits.objects = before.0 + 1,
                1 => vm.limits.properties = before.1 + 5,
                2 => vm.limits.string_units = before.2 + 40,
                _ => unreachable!(),
            }
            assert!(vm.install_flash_geom_point_rectangle().is_err());
            assert_eq!((vm.object_count(), vm.properties, vm.string_units), before);
            assert!(vm.geom_namespace.is_none());
            assert!(
                !vm.objects[vm.global.0]
                    .properties
                    .contains_key(&text("flash"))
            );
            assert!(!vm.builtin_registry.contains_key(&text("Point")));
            assert!(!vm.builtin_registry.contains_key(&text("Rectangle")));
            vm.limits.objects = before.0 + 2;
            vm.limits.properties = before.1 + 6;
            vm.limits.string_units = before.2 + 41;
            let ns = vm.install_flash_geom_point_rectangle().unwrap();
            let after = (vm.object_count(), vm.properties, vm.string_units);
            assert_eq!(after, (before.0 + 2, before.1 + 6, before.2 + 41));
            assert!(vm.install_flash_geom_point_rectangle().is_err());
            assert!(vm.install_flash_geom_point().is_err());
            assert_eq!((vm.object_count(), vm.properties, vm.string_units), after);
            assert_eq!(vm.geom_namespace, Some(ns));
        }
        let mut vm = setup();
        let ns = vm.install_flash_geom_point().unwrap();
        let before = (vm.object_count(), vm.properties, vm.string_units);
        assert!(vm.install_flash_geom_point_rectangle().is_err());
        assert_eq!(vm.geom_namespace, Some(ns));
        assert_eq!((vm.object_count(), vm.properties, vm.string_units), before);
        assert!(!vm.builtin_registry.contains_key(&text("Rectangle")));
    }

    #[test]
    fn rectangle_public_prototype_reads_and_inherited_partial_graph_gates() {
        let mut vm = setup();
        vm.install_flash_geom_point_rectangle().unwrap();
        vm.bind_builtin_context_target(vm.root()).unwrap();
        let code = vm
            .register(Arc::new(
                Program::parse(&new_geometry("Rectangle")).unwrap(),
            ))
            .unwrap();
        let execution = vm.execute(code, vm.root(), &mut RejectHost).unwrap();
        let [Value::Object(instance)] = execution.stack.as_slice() else {
            panic!("one Rectangle result")
        };
        let graph = vm.rectangle_objects.unwrap();
        let child = vm.alloc(Kind::Plain, Some(graph.prototype)).unwrap();
        for name in [
            "left",
            "top",
            "right",
            "bottom",
            "topLeft",
            "bottomRight",
            "size",
        ] {
            assert_eq!(vm.get(graph.prototype, name).unwrap(), Value::Number(0.0));
            assert_eq!(vm.get(child, name).unwrap(), Value::Number(0.0));
            assert!(vm.get(*instance, name).is_err());
            assert!(vm.set(child, name, Value::Number(9.0)).is_err());
        }
        for id in [graph.constructor, graph.prototype, *instance, child] {
            assert!(vm.get(id, "omittedGeometryMethod").is_err());
            assert!(vm.set(id, "extra", Value::Null).is_err());
            assert!(vm.delete_member(id, &text("left")).is_err());
            assert!(vm.enumerate_names(id).is_err());
        }
        for id in [graph.constructor, graph.prototype, *instance] {
            assert!(vm.own_enumerable_names(id).is_err());
            assert!(vm.own_property_flags(id, "left").is_err());
        }
    }

    #[test]
    fn point_only_preserves_unrelated_rectangle_while_combined_rejects_collision() {
        for inherited in [false, true] {
            let mut vm = setup();
            let other = vm.object().unwrap();
            let retained = vm
                .register_deferred_builtin(other, "Rectangle", BuiltinFactory::Unsupported, 0)
                .unwrap();
            if inherited {
                vm.raw_member(
                    vm.object_prototype.unwrap(),
                    text("Rectangle"),
                    Property::Data(Value::Number(19.0)),
                    0,
                )
                .unwrap();
            }
            let before = (vm.object_count(), vm.properties, vm.string_units);
            assert!(vm.install_flash_geom_point_rectangle().is_err());
            assert_eq!((vm.object_count(), vm.properties, vm.string_units), before);
            assert!(vm.geom_namespace.is_none());
            assert!(!vm.builtin_registry.contains_key(&text("Point")));
            assert_eq!(vm.get(other, "Rectangle").unwrap(), retained);
            let ns = vm.install_flash_geom_point().unwrap();
            assert_eq!(
                (vm.object_count(), vm.properties, vm.string_units),
                (before.0 + 2, before.1 + 5, before.2 + 32)
            );
            assert_eq!(vm.get(other, "Rectangle").unwrap(), retained);
            assert!(vm.get(ns.geom, "Rectangle").is_err());
            assert!(vm.rectangle_objects.is_none());
        }
        // A parent collision alone is also rejected; it is not repaired into a
        // fresh namespace-owned token when the registry has no Rectangle entry.
        let mut vm = setup();
        let parent = vm.object_prototype.unwrap();
        vm.raw_member(parent, text("Rectangle"), Property::Data(Value::Null), 4)
            .unwrap();
        let before = (vm.object_count(), vm.properties, vm.string_units);
        assert!(vm.install_flash_geom_point_rectangle().is_err());
        assert_eq!((vm.object_count(), vm.properties, vm.string_units), before);
        assert!(vm.builtin_registry.is_empty());
        assert!(vm.geom_namespace.is_none());
    }
}
