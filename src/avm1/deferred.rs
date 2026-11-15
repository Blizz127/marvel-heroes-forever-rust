//! Name tokens and a bounded registry for one explicitly associated AS2 context.
//!
//! Native contracts: `docs/DEFERRED_BUILTIN_VALUES.md` and
//! `docs/EXTERNAL_INTERFACE_BINDING.md`. The token contains only a name; scalar
//! operations and language equality belong to the executor, not this registry.
//! Factory 0x1401f2280 and resolver 0x140194670 are retained in private
//! `rea-external-interface/final.bundle.json`. This module admits only an explicit
//! bootstrap namespace and supported ordinary publication. It does not emulate
//! native missing/null factories, reentrant factories, watch state or resolvers.

use super::{ErrorKind, Kind, ObjectId, Property, Result, Text, Value, Vm, text};

/// A retained name, without a factory, namespace or originating Environment.
///
/// Rust equality compares names for registry/profile checks. It is not AVM1
/// equality: native equality of two retained tokens ignores their names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredBuiltinName(Text);

impl DeferredBuiltinName {
    pub fn name(&self) -> &[u16] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinFactory {
    ExternalInterface {
        call_binding: u32,
    },
    /// Bounded canonical native String graph and primitive-call boxing profile.
    String,
    /// Preserve the actual token; reject only when materialization is required.
    Unsupported,
}

#[derive(Debug, Clone)]
pub(super) struct BuiltinEntry {
    // Explicit context namespace supplied by the bootstrap host. This is not
    // information carried by a token, nor another context's creation origin.
    namespace: ObjectId,
    factory: BuiltinFactory,
    cached: Option<ObjectId>,
}

impl Vm {
    /// Assert that this live target uses this VM's one builtin context.
    ///
    /// The host supplies the association; root identity or parentage is never
    /// inferred. Plain targets are for explicitly controlled execution. Actual
    /// attached owners retain their typed identity; storage and unbound objects
    /// cannot stand in for an attached target. Retirement is not supported.
    pub fn bind_builtin_context_target(&mut self, target: ObjectId) -> Result<()> {
        self.check_object(target)?;
        match self.objects[target.0].kind {
            Kind::Plain => {}
            Kind::AttachedOwner(clip) if self.attached_owner(clip)? == target => {}
            _ => {
                return self.fail(ErrorKind::Unsupported(
                    "builtin context target requires plain target or attached owner",
                ));
            }
        }
        if !self.builtin_targets.contains_key(&target)
            && self.builtin_targets.len() >= self.limits.objects
        {
            return self.fail(ErrorKind::Budget("builtin context targets"));
        }
        self.builtin_targets.insert(target, ());
        Ok(())
    }

    /// Register one bootstrap token, without running its factory.
    ///
    /// The host asserts that `namespace` is the corresponding namespace in this
    /// VM's context. The bounded profile accepts ASCII identifier names of at
    /// most 128 bytes and flags 0/1. New entries require no own or inherited
    /// property of that name. These are engineering admission limits, not native
    /// string/namespace limits. A repeated registered name returns its token and
    /// leaves both the original entry and every namespace unchanged.
    pub fn register_deferred_builtin(
        &mut self,
        namespace: ObjectId,
        name: &str,
        factory: BuiltinFactory,
        flags: u8,
    ) -> Result<Value> {
        let mut bytes = name.bytes();
        let valid_first = bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$'));
        if name.len() > 128
            || !valid_first
            || !bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
            || matches!(name, "__proto__" | "__resolve")
        {
            return self.fail(ErrorKind::Unsupported("deferred builtin name profile"));
        }
        let key = text(name);
        if let Some((registered, _)) = self.builtin_registry.get_key_value(&key) {
            return Ok(Value::DeferredBuiltin(DeferredBuiltinName(
                registered.clone(),
            )));
        }
        if flags > 1 {
            return self.fail(ErrorKind::Unsupported("deferred builtin bootstrap flags"));
        }
        if matches!(factory, BuiltinFactory::ExternalInterface { .. })
            && (name != "ExternalInterface" || flags != 0)
        {
            return self.fail(ErrorKind::Unsupported(
                "ExternalInterface factory requires exact name and flags zero",
            ));
        }
        if matches!(factory, BuiltinFactory::String)
            && (name != "String" || namespace != self.global || flags != 1)
        {
            return self.fail(ErrorKind::Unsupported(
                "String factory requires exact flags-one global token",
            ));
        }
        self.deferred_namespace_profile(namespace)?;
        if self.descriptor(namespace, &key)?.is_some() {
            return self.fail(ErrorKind::Unsupported(
                "deferred registration would replace a namespace member",
            ));
        }
        if self.builtin_registry.len() >= self.limits.properties {
            return self.fail(ErrorKind::Budget("deferred builtin registry"));
        }
        let token = Value::DeferredBuiltin(DeferredBuiltinName(key.clone()));
        // The supported ordinary setter validates the value and inherited
        // descriptor. For this absent key it either fails before changing the
        // object, or installs one data property. No script callback is invoked.
        self.set_key(namespace, key.clone(), token.clone())?;
        self.objects[namespace.0].flags.insert(key.clone(), flags);
        self.builtin_registry.insert(
            key,
            BuiltinEntry {
                namespace,
                factory,
                cached: None,
            },
        );
        Ok(token)
    }

    pub(super) fn resolve_deferred_builtin(
        &mut self,
        name: &DeferredBuiltinName,
    ) -> Result<ObjectId> {
        let Some(target) = self.current_target else {
            return self.fail(ErrorKind::Unsupported(
                "deferred builtin requires current target",
            ));
        };
        self.check_object(target)?;
        if !self.builtin_targets.contains_key(&target) {
            return self.fail(ErrorKind::Unsupported("unbound builtin context target"));
        }
        let Some(entry) = self.builtin_registry.get(&name.0).cloned() else {
            return self.fail(ErrorKind::UnsupportedDeferredBuiltin(name.0.clone()));
        };
        if let Some(cached) = entry.cached {
            self.check_object(cached)?;
            return Ok(cached);
        }
        let required_flags = match entry.factory {
            BuiltinFactory::ExternalInterface { .. } => 0,
            BuiltinFactory::String => 1,
            BuiltinFactory::Unsupported => {
                return self.fail(ErrorKind::UnsupportedDeferredBuiltin(name.0.clone()));
            }
        };
        self.deferred_namespace_profile(entry.namespace)?;
        if !matches!(
            self.objects[entry.namespace.0].properties.get(&name.0),
            Some(Property::Data(Value::DeferredBuiltin(original))) if original == name
        ) || self.own_flags(entry.namespace, &name.0) != required_flags
        {
            return self.fail(ErrorKind::Unsupported(
                "native factory requires original namespace token and flags",
            ));
        }

        let object_count = self.objects.len();
        let property_count = self.properties;
        let string_units = self.string_units;
        let previous_string_graph = self.string_objects;
        let constructor = match entry.factory {
            BuiltinFactory::ExternalInterface { call_binding } => {
                self.build_external_interface(call_binding)?.constructor
            }
            BuiltinFactory::String => self.build_string_graph()?.constructor,
            BuiltinFactory::Unsupported => unreachable!(),
        };
        if let Err(error) =
            self.set_key(entry.namespace, name.0.clone(), Value::Object(constructor))
        {
            // Under the admitted own-data profile setter errors precede its
            // write. No host/reentrant factory runs between the guard and write.
            self.objects.truncate(object_count);
            self.properties = property_count;
            self.string_units = string_units;
            self.string_objects = previous_string_graph;
            return Err(error);
        }
        // No registry mutation or callback occurs during this supported factory.
        // Reacquire only after publication; a failed build is never cached.
        self.builtin_registry.get_mut(&name.0).unwrap().cached = Some(constructor);
        Ok(constructor)
    }

    fn deferred_namespace_profile(&self, namespace: ObjectId) -> Result<()> {
        self.check_object(namespace)?;
        if self.object_prototype == Some(namespace) {
            return self.fail(ErrorKind::Unsupported(
                "builtin prototype is not a namespace",
            ));
        }
        let mut current = Some(namespace);
        let mut depth = 0;
        while let Some(id) = current {
            if depth >= self.limits.prototype_depth {
                return self.fail(ErrorKind::Budget("deferred namespace prototype depth"));
            }
            self.check_object(id)?;
            let object = &self.objects[id.0];
            if !matches!(object.kind, Kind::Plain)
                || object
                    .properties
                    .values()
                    .any(|value| !matches!(value, Property::Data(_)))
                || (depth != 0 && object.prototype.is_some())
            {
                return self.fail(ErrorKind::Unsupported(
                    "deferred ordinary namespace profile",
                ));
            }
            // Insertion history survives deletion. Removing __resolve must not
            // turn an unproved resolver-mutated object into a fresh namespace.
            if object
                .insertion_keys
                .iter()
                .any(|key| key.as_ref() == text("__resolve").as_ref())
            {
                return self.fail(ErrorKind::Unsupported(
                    "deferred namespace resolver history",
                ));
            }
            if object.prototype.is_some() && object.prototype != self.object_prototype {
                return self.fail(ErrorKind::Unsupported("deferred namespace parent profile"));
            }
            current = object.prototype;
            depth += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::Limits;
    use super::*;

    fn setup() -> (Vm, ObjectId) {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        let namespace = vm.object().unwrap();
        (vm, namespace)
    }
    fn token(value: Value) -> DeferredBuiltinName {
        let Value::DeferredBuiltin(name) = value else {
            panic!("expected retained token")
        };
        name
    }
    fn register_ei(vm: &mut Vm, namespace: ObjectId) -> DeferredBuiltinName {
        token(
            vm.register_deferred_builtin(
                namespace,
                "ExternalInterface",
                BuiltinFactory::ExternalInterface { call_binding: 91 },
                0,
            )
            .unwrap(),
        )
    }
    fn activate(vm: &mut Vm) {
        vm.bind_builtin_context_target(vm.root).unwrap();
        vm.current_target = Some(vm.root);
    }

    #[test]
    fn registration_is_name_only_and_does_not_materialize() {
        let (mut vm, namespace) = setup();
        let before = vm.objects.len();
        let first = token(
            vm.register_deferred_builtin(namespace, "Stage", BuiltinFactory::Unsupported, 1)
                .unwrap(),
        );
        assert_eq!(first.name(), text("Stage").as_ref());
        assert_eq!(vm.objects.len(), before);
        assert_eq!(vm.own_flags(namespace, &text("Stage")), 1);
        // Repeated registration must not replace a factory or touch even an
        // invalid proposed namespace; the context map's name entry wins first.
        let repeated = token(
            vm.register_deferred_builtin(
                ObjectId(usize::MAX),
                "Stage",
                BuiltinFactory::ExternalInterface { call_binding: 2 },
                7,
            )
            .unwrap(),
        );
        assert_eq!(first, repeated);
        assert!(matches!(
            vm.builtin_registry[&text("Stage")].factory,
            BuiltinFactory::Unsupported
        ));
        assert!(vm.builtin_registry[&text("Stage")].cached.is_none());
        assert_eq!(vm.objects.len(), before);
    }

    #[test]
    fn materialization_requires_context_and_retained_token_uses_cached_identity() {
        let (mut vm, namespace) = setup();
        let retained = register_ei(&mut vm, namespace);
        let before = vm.objects.len();
        assert!(vm.resolve_deferred_builtin(&retained).is_err());
        vm.current_target = Some(vm.root);
        assert!(vm.resolve_deferred_builtin(&retained).is_err());
        assert_eq!(vm.objects.len(), before);
        activate(&mut vm);
        let resolved = vm.resolve_deferred_builtin(&retained).unwrap();
        assert_eq!(vm.objects.len(), before + 4);
        assert_eq!(
            vm.get(namespace, "ExternalInterface").unwrap(),
            Value::Object(resolved)
        );
        assert_eq!(retained.name(), text("ExternalInterface").as_ref());
        vm.set(namespace, "ExternalInterface", Value::Number(3.0))
            .unwrap();
        assert_eq!(vm.resolve_deferred_builtin(&retained).unwrap(), resolved);
        assert_eq!(
            vm.get(namespace, "ExternalInterface").unwrap(),
            Value::Number(3.0)
        );
        let other = vm.object().unwrap();
        vm.current_target = Some(other);
        assert!(vm.resolve_deferred_builtin(&retained).is_err());
        vm.bind_builtin_context_target(other).unwrap();
        assert_eq!(vm.resolve_deferred_builtin(&retained).unwrap(), resolved);
    }

    #[test]
    fn unsupported_and_unregistered_tokens_do_not_create_objects_or_fake_null() {
        let (mut vm, namespace) = setup();
        let unsupported = token(
            vm.register_deferred_builtin(namespace, "Stage", BuiltinFactory::Unsupported, 1)
                .unwrap(),
        );
        activate(&mut vm);
        let before = vm.objects.len();
        for name in [unsupported.clone(), DeferredBuiltinName(text("Array"))] {
            assert_eq!(
                vm.resolve_deferred_builtin(&name).unwrap_err().kind,
                ErrorKind::UnsupportedDeferredBuiltin(name.0.clone())
            );
        }
        assert_eq!(vm.objects.len(), before);
        assert_eq!(
            vm.get(namespace, "Stage").unwrap(),
            Value::DeferredBuiltin(unsupported)
        );
    }

    #[test]
    fn changed_token_flags_and_resolver_history_reject_before_factory() {
        for mutation in 0..3 {
            let (mut vm, namespace) = setup();
            let name = register_ei(&mut vm, namespace);
            activate(&mut vm);
            match mutation {
                0 => {
                    vm.objects[namespace.0]
                        .flags
                        .insert(text("ExternalInterface"), 1);
                }
                1 => vm.set(namespace, "ExternalInterface", Value::Null).unwrap(),
                _ => {
                    vm.set(namespace, "__resolve", Value::Null).unwrap();
                    vm.delete_member(namespace, &text("__resolve")).unwrap();
                }
            }
            let before = (vm.objects.len(), vm.properties, vm.string_units);
            assert!(vm.resolve_deferred_builtin(&name).is_err());
            assert_eq!((vm.objects.len(), vm.properties, vm.string_units), before);
            assert!(vm.builtin_registry[&name.0].cached.is_none());
        }
    }

    #[test]
    fn registration_rejects_inherited_descriptors_and_accessor_namespaces() {
        let (mut vm, _) = setup();
        let parent = vm.object_prototype.unwrap();
        let namespace = vm.alloc(Kind::Plain, Some(parent)).unwrap();
        vm.raw_member(parent, text("Stage"), Property::Data(Value::Number(4.0)), 4)
            .unwrap();
        assert!(
            vm.register_deferred_builtin(namespace, "Stage", BuiltinFactory::Unsupported, 0)
                .is_err()
        );
        assert!(
            !vm.objects[namespace.0]
                .properties
                .contains_key(&text("Stage"))
        );
        vm.raw_member(
            parent,
            text("unrelated"),
            Property::Accessor {
                getter: Value::Null,
                setter: None,
            },
            0,
        )
        .unwrap();
        assert!(
            vm.register_deferred_builtin(namespace, "Array", BuiltinFactory::Unsupported, 0)
                .is_err()
        );
        assert!(vm.builtin_registry.is_empty());
    }

    #[test]
    fn failed_factory_is_not_published_or_cached_and_can_retry() {
        let (mut vm, namespace) = setup();
        let name = register_ei(&mut vm, namespace);
        activate(&mut vm);
        let before = (vm.objects.len(), vm.properties, vm.string_units);
        vm.limits.properties = before.1 + 8;
        assert_eq!(
            vm.resolve_deferred_builtin(&name).unwrap_err().kind,
            ErrorKind::Budget("properties")
        );
        assert_eq!((vm.objects.len(), vm.properties, vm.string_units), before);
        assert_eq!(
            vm.get(namespace, "ExternalInterface").unwrap(),
            Value::DeferredBuiltin(name.clone())
        );
        assert!(vm.builtin_registry[&name.0].cached.is_none());
        vm.limits = Limits::default();
        let result = vm.resolve_deferred_builtin(&name).unwrap();
        assert_eq!(result, ObjectId(before.0));
    }

    #[test]
    fn admission_rejects_invalid_names_flags_targets_and_partial_registration() {
        let (mut vm, namespace) = setup();
        for name in ["", "a.b", "\u{e9}", "__proto__", "__resolve"] {
            assert!(
                vm.register_deferred_builtin(namespace, name, BuiltinFactory::Unsupported, 0)
                    .is_err()
            );
        }
        assert!(
            vm.register_deferred_builtin(
                namespace,
                &"a".repeat(129),
                BuiltinFactory::Unsupported,
                0
            )
            .is_err()
        );
        assert!(
            vm.register_deferred_builtin(namespace, "Stage", BuiltinFactory::Unsupported, 2)
                .is_err()
        );
        assert!(
            vm.register_deferred_builtin(
                namespace,
                "ExternalInterface",
                BuiltinFactory::ExternalInterface { call_binding: 1 },
                1,
            )
            .is_err()
        );
        let native = vm.native(1).unwrap();
        assert!(vm.bind_builtin_context_target(native).is_err());
        assert!(
            vm.bind_builtin_context_target(ObjectId(usize::MAX))
                .is_err()
        );
        let before = (vm.properties, vm.string_units);
        vm.limits.string_units = vm.string_units;
        assert!(
            vm.register_deferred_builtin(namespace, "Stage", BuiltinFactory::Unsupported, 0)
                .is_err()
        );
        assert_eq!((vm.properties, vm.string_units), before);
        assert!(vm.builtin_registry.is_empty());
        assert!(vm.objects[namespace.0].properties.is_empty());
    }
}
