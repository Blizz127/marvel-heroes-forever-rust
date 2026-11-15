//! One bounded AS2 class-registration map per explicitly owned VM/root context.
//! No movie namespace, constructor invocation, asset lookup or instance mutation.

use crate::avm1::vm::{
    ClassRegistration, CodeOrigin, Host, HostCall, HostCommand, ObjectId, Text, Value,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_entries: usize,
    pub max_key_units: usize,
    pub max_live_key_units: usize,
    pub max_calls: usize,
    pub max_events: usize,
    pub max_history_key_units: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_entries: 16_384,
            max_key_units: 4096,
            max_live_key_units: 1_000_000,
            max_calls: 100_000,
            max_events: 100_000,
            max_history_key_units: 4_000_000,
        }
    }
}

/// Diagnostic association only: ObjectId is local to its owning VM. Two VMs
/// with the same numeric root ID must still own distinct ClassRegistry values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegistryContext {
    pub root: ObjectId,
    pub movie_version: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredClass {
    /// The original VM constructor object, not an instantiated object or wrapper.
    pub constructor: ObjectId,
    /// Call and definition origins are provenance, never namespace selectors.
    pub origin: Option<CodeOrigin>,
    pub constructor_origin: Option<CodeOrigin>,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Inserted,
    Replaced,
    Removed,
    Absent,
}
#[derive(Clone, Debug)]
pub struct RegistrationEvent {
    pub request: ClassRegistration,
    pub previous: Option<RegisteredClass>,
    pub outcome: Outcome,
    pub returned: bool,
    pub revision: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    /// Includes calls later rejected by key/capacity/history validation.
    pub calls: usize,
    pub live_key_units: usize,
    /// Conservative sum of accepted event-key lengths; shared Arcs count again.
    pub history_key_units: usize,
    /// Successful insert/replacement/present removal; absent removal does not change it.
    pub revision: u64,
}

/// Not Clone: ownership should remain with one VM/root host. The caller must
/// route requests from that context only; ClassRegistration has no VM token.
pub struct ClassRegistry {
    context: RegistryContext,
    entries: BTreeMap<Text, RegisteredClass>,
    events: Vec<RegistrationEvent>,
    limits: Limits,
    stats: Statistics,
}
impl ClassRegistry {
    pub fn new(root: ObjectId, movie_version: u8, limits: Limits) -> Result<Self, String> {
        if !(7..=19).contains(&movie_version) {
            return Err("class registry requires the VM's version 7..19 exact-key profile; older folding is unsupported".into());
        }
        Ok(Self {
            context: RegistryContext {
                root,
                movie_version,
            },
            entries: BTreeMap::new(),
            events: Vec::new(),
            limits,
            stats: Statistics::default(),
        })
    }
    pub fn context(&self) -> RegistryContext {
        self.context
    }
    pub fn stats(&self) -> Statistics {
        self.stats
    }
    /// Sorted UTF-16 inspection order only; this is not native hash/enumeration order.
    pub fn entries(&self) -> &BTreeMap<Text, RegisteredClass> {
        &self.entries
    }
    pub fn events(&self) -> &[RegistrationEvent] {
        &self.events
    }

    fn validate_name(&self, name: &[u16]) -> Result<(), String> {
        if name.is_empty() {
            return Err("class registry empty-name profile is unsupported".into());
        }
        if name.len() > self.limits.max_key_units {
            return Err("class registry key-unit budget exceeded".into());
        }
        Ok(())
    }
    /// Lookup has no side effects and does not select a movie/export or instantiate
    /// the stored constructor. Exact UTF-16 units are compared without conversion.
    pub fn lookup(&self, name: &[u16]) -> Result<Option<&RegisteredClass>, String> {
        self.validate_name(name)?;
        Ok(self.entries.get(name))
    }

    /// The VM supplies its validated function-or-null request. No origin is
    /// required, inferred or used as a key. All mutation budgets are checked first.
    pub fn register(&mut self, request: ClassRegistration) -> Result<bool, String> {
        self.stats.calls = bounded_add(self.stats.calls, 1, self.limits.max_calls, "call")?;
        self.validate_name(&request.name)?;
        if self.events.len() >= self.limits.max_events {
            return Err("class registry event budget exceeded".into());
        }
        let history_units = bounded_add(
            self.stats.history_key_units,
            request.name.len(),
            self.limits.max_history_key_units,
            "history key-unit",
        )?;
        let previous = self.entries.get(request.name.as_ref()).cloned();
        let outcome = match (request.constructor, previous.is_some()) {
            (Some(_), false) => Outcome::Inserted,
            (Some(_), true) => Outcome::Replaced,
            (None, true) => Outcome::Removed,
            (None, false) => Outcome::Absent,
        };
        let live_units = match outcome {
            Outcome::Inserted => {
                if self.entries.len() >= self.limits.max_entries {
                    return Err("class registry entry budget exceeded".into());
                }
                bounded_add(
                    self.stats.live_key_units,
                    request.name.len(),
                    self.limits.max_live_key_units,
                    "live key-unit",
                )?
            }
            Outcome::Removed => self
                .stats
                .live_key_units
                .checked_sub(request.name.len())
                .ok_or("class registry live key-unit accounting underflow")?,
            _ => self.stats.live_key_units,
        };
        let changed = outcome != Outcome::Absent;
        let revision = if changed {
            self.stats
                .revision
                .checked_add(1)
                .ok_or("class registry revision overflow")?
        } else {
            self.stats.revision
        };
        if let Some(constructor) = request.constructor {
            self.entries.insert(
                request.name.clone(),
                RegisteredClass {
                    constructor,
                    origin: request.origin,
                    constructor_origin: request.constructor_origin,
                    revision,
                },
            );
        } else {
            self.entries.remove(request.name.as_ref());
        }
        self.stats.live_key_units = live_units;
        self.stats.history_key_units = history_units;
        self.stats.revision = revision;
        self.events.push(RegistrationEvent {
            request,
            previous,
            outcome,
            returned: changed,
            revision,
        });
        Ok(changed)
    }
}
impl Host for ClassRegistry {
    fn call(&mut self, _: HostCall<'_>) -> Result<Value, String> {
        Err("class registry has no native function bindings".into())
    }
    fn command(&mut self, _: ObjectId, _: HostCommand) -> Result<(), String> {
        Err("class registry has no timeline or other command bindings".into())
    }
    fn register_class(&mut self, request: ClassRegistration) -> Result<bool, String> {
        self.register(request)
    }
}
fn bounded_add(value: usize, amount: usize, limit: usize, kind: &str) -> Result<usize, String> {
    let next = value
        .checked_add(amount)
        .ok_or_else(|| format!("class registry {kind} counter overflow"))?;
    if next > limit {
        return Err(format!("class registry {kind} budget exceeded"));
    }
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm1::vm::{self, CodeId, Vm, text};
    fn request(
        name: Text,
        constructor: Option<usize>,
        call: usize,
        definition: usize,
    ) -> ClassRegistration {
        ClassRegistration {
            name,
            constructor: constructor.map(ObjectId),
            origin: Some(CodeOrigin {
                code: CodeId(call),
                source_offset: 100 + call,
            }),
            constructor_origin: constructor.map(|_| CodeOrigin {
                code: CodeId(definition),
                source_offset: 200 + definition,
            }),
        }
    }
    fn registry() -> ClassRegistry {
        ClassRegistry::new(ObjectId(1), 15, Limits::default()).unwrap()
    }

    #[test]
    fn origins_do_not_partition_root_context_and_replacement_keeps_exact_identity() {
        let mut registry = registry();
        let key = text("Cursor");
        assert!(
            registry
                .register(request(key.clone(), Some(10), 1, 5))
                .unwrap()
        );
        assert!(
            registry
                .register(request(key.clone(), Some(20), 7, 11))
                .unwrap()
        );
        assert_eq!(registry.entries().len(), 1);
        let value = registry.lookup(&key).unwrap().unwrap();
        assert_eq!(value.constructor, ObjectId(20));
        assert_eq!(value.origin.unwrap().code, CodeId(7));
        assert_eq!(value.constructor_origin.unwrap().code, CodeId(11));
        assert_eq!(
            registry.events()[1].previous.as_ref().unwrap().constructor,
            ObjectId(10)
        );
        assert_eq!(registry.events()[1].outcome, Outcome::Replaced);
        assert!(
            registry
                .register(request(key.clone(), None, 13, 0))
                .unwrap()
        );
        assert!(registry.lookup(&key).unwrap().is_none());
        assert!(!registry.register(request(key, None, 14, 0)).unwrap());
        assert_eq!(registry.stats().revision, 3);
        assert_eq!(registry.stats().live_key_units, 0);
        assert_eq!(registry.events()[3].outcome, Outcome::Absent);
    }

    #[test]
    fn contexts_and_exact_utf16_keys_are_isolated() {
        let mut first = registry();
        let mut second = registry();
        assert_eq!(first.context(), second.context()); // Same numeric VM-local root IDs.
        for (key, id) in [
            (text("Cursor"), 10),
            (text("cursor"), 11),
            (Text::from(vec![0xd800, 0]), 12),
            (Text::from(vec![0xfffd, 0]), 13),
        ] {
            first
                .register(request(key.clone(), Some(id), 0, 0))
                .unwrap();
            assert!(second.lookup(&key).unwrap().is_none());
            assert_eq!(
                first.lookup(&key).unwrap().unwrap().constructor,
                ObjectId(id)
            );
        }
        second
            .register(request(text("Cursor"), Some(30), 0, 0))
            .unwrap();
        assert_eq!(
            first.lookup(&text("Cursor")).unwrap().unwrap().constructor,
            ObjectId(10)
        );
        assert_eq!(
            second.lookup(&text("Cursor")).unwrap().unwrap().constructor,
            ObjectId(30)
        );
        assert_eq!(first.entries().len(), 4);
        assert!(ClassRegistry::new(ObjectId(1), 6, Limits::default()).is_err());
    }

    #[test]
    fn all_admission_failures_preserve_membership_and_replacement_works_at_capacity() {
        let key = text("A");
        for limits in [
            Limits {
                max_entries: 0,
                ..Limits::default()
            },
            Limits {
                max_key_units: 0,
                ..Limits::default()
            },
            Limits {
                max_live_key_units: 0,
                ..Limits::default()
            },
            Limits {
                max_calls: 0,
                ..Limits::default()
            },
            Limits {
                max_events: 0,
                ..Limits::default()
            },
            Limits {
                max_history_key_units: 0,
                ..Limits::default()
            },
        ] {
            let mut registry = ClassRegistry::new(ObjectId(1), 15, limits).unwrap();
            assert!(
                registry
                    .register(request(key.clone(), Some(2), 0, 0))
                    .is_err()
            );
            assert!(registry.entries().is_empty());
            assert!(registry.events().is_empty());
            assert_eq!(registry.stats().revision, 0);
        }
        let mut registry = ClassRegistry::new(
            ObjectId(1),
            15,
            Limits {
                max_entries: 1,
                max_live_key_units: 1,
                max_history_key_units: 2,
                ..Limits::default()
            },
        )
        .unwrap();
        registry
            .register(request(key.clone(), Some(2), 0, 0))
            .unwrap();
        registry
            .register(request(key.clone(), Some(3), 1, 1))
            .unwrap();
        assert!(registry.register(request(key.clone(), None, 2, 0)).is_err());
        assert_eq!(
            registry.lookup(&key).unwrap().unwrap().constructor,
            ObjectId(3)
        );
        assert_eq!(registry.stats().live_key_units, 1);
        assert_eq!(registry.stats().revision, 2);
    }

    #[test]
    fn vm_builtin_routes_to_registry_without_calling_constructor() {
        let mut vm = Vm::new(15, vm::Limits::default()).unwrap();
        let object = vm.install_object_constructor().unwrap();
        let function = vm.get(object, "registerClass").unwrap();
        let constructor = vm.native(9999).unwrap();
        let mut registry = ClassRegistry::new(vm.root(), 15, Limits::default()).unwrap();
        let call = vm
            .call(
                function.clone(),
                Value::Object(object),
                &[Value::string("Bound"), Value::Object(constructor)],
                &mut registry,
            )
            .unwrap();
        assert_eq!(call.value, Value::Bool(true));
        assert_eq!(call.host_calls, 1);
        let found = registry.lookup(&text("Bound")).unwrap().unwrap();
        assert_eq!(found.constructor, constructor);
        assert_eq!(found.origin, None);
        assert_eq!(found.constructor_origin, None);
        let result = vm
            .call(
                function.clone(),
                Value::Object(object),
                &[Value::string("Bound"), Value::Null],
                &mut registry,
            )
            .unwrap();
        assert_eq!(result.value, Value::Bool(true));
        let absent = vm
            .call(
                function,
                Value::Object(object),
                &[Value::string("Bound"), Value::Null],
                &mut registry,
            )
            .unwrap();
        assert_eq!(absent.value, Value::Bool(false));
    }
}
