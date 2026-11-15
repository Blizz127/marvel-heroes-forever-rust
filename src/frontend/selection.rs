//! Explicit Selection listener state for the frontend AVM1 host.
//!
//! Registering a listener changes this bounded list. Focus acquisition, event
//! dispatch and native input are separate responsibilities; they are not stubbed.

use crate::avm1::vm::{Host, HostCall, HostCommand, ObjectId, Value, Vm};
use std::{error::Error, io};

const ADD_LISTENER: u32 = 0x4d48_0001;
const REMOVE_LISTENER: u32 = 0x4d48_0002;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_listeners: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_listeners: 4096,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub add_calls: u64,
    pub remove_calls: u64,
}

pub struct SelectionHost {
    object: ObjectId,
    listeners: Vec<ObjectId>,
    max_listeners: usize,
    stats: Stats,
}

impl SelectionHost {
    /// Install only Selection.addListener/removeListener on a fresh explicit
    /// global binding. All other host operations keep rejecting defaults.
    pub fn install(vm: &mut Vm, limits: Limits) -> Result<Self, Box<dyn Error>> {
        if limits.max_listeners == 0 || limits.max_listeners > 100_000 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid Selection listener budget",
            )
            .into());
        }
        if !matches!(vm.get(vm.global(), "Selection")?, Value::Undefined) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Selection global already exists",
            )
            .into());
        }
        let object = vm.object()?;
        let add = vm.native(ADD_LISTENER)?;
        let remove = vm.native(REMOVE_LISTENER)?;
        vm.set(object, "addListener", Value::Object(add))?;
        vm.set(object, "removeListener", Value::Object(remove))?;
        vm.set(vm.global(), "Selection", Value::Object(object))?;
        Ok(Self {
            object,
            listeners: Vec::new(),
            max_listeners: limits.max_listeners,
            stats: Stats::default(),
        })
    }

    pub fn object(&self) -> ObjectId {
        self.object
    }
    /// Stored association order. This accessor does not assert native event
    /// delivery order or supply a dispatch snapshot/reentrancy policy.
    pub fn listeners(&self) -> &[ObjectId] {
        &self.listeners
    }
    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Composable routing entry point: callers may route other bindings to a
    /// different host. Unknown bindings return None without changing state.
    pub fn handle_call(&mut self, call: &HostCall<'_>) -> Option<Result<Value, String>> {
        if !matches!(call.binding, ADD_LISTENER | REMOVE_LISTENER) {
            return None;
        }
        Some(self.listener_call(call))
    }

    fn listener_call(&mut self, call: &HostCall<'_>) -> Result<Value, String> {
        if call.this != Value::Object(self.object) {
            return Err("unsupported Selection method receiver".into());
        }
        let [Value::Object(listener)] = call.args else {
            return Err("Selection listener profile requires exactly one object".into());
        };
        match call.binding {
            ADD_LISTENER => {
                if !self.listeners.contains(listener) {
                    if self.listeners.len() >= self.max_listeners {
                        return Err("Selection listener budget exceeded".into());
                    }
                    self.listeners.push(*listener);
                }
                self.stats.add_calls = self.stats.add_calls.saturating_add(1);
                // Selection.addListener is Void. AsBroadcaster.addListener has
                // a different documented return contract and is not reused.
                Ok(Value::Undefined)
            }
            REMOVE_LISTENER => {
                let index = self
                    .listeners
                    .iter()
                    .position(|candidate| candidate == listener);
                if let Some(index) = index {
                    self.listeners.remove(index);
                }
                self.stats.remove_calls = self.stats.remove_calls.saturating_add(1);
                Ok(Value::Bool(index.is_some()))
            }
            _ => unreachable!("binding checked before routing"),
        }
    }
}

impl Host for SelectionHost {
    fn call(&mut self, call: HostCall<'_>) -> Result<Value, String> {
        self.handle_call(&call)
            .unwrap_or_else(|| Err(format!("unhandled native binding {}", call.binding)))
    }
    fn command(&mut self, _: ObjectId, command: HostCommand) -> Result<(), String> {
        Err(format!("unhandled host command {command:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avm1::vm;

    #[test]
    fn native_calls_change_membership_and_preserve_void_boolean_contracts() {
        let mut vm = Vm::new(15, vm::Limits::default()).unwrap();
        let mut host = SelectionHost::install(&mut vm, Limits { max_listeners: 1 }).unwrap();
        let listener = vm.object().unwrap();
        let other = vm.object().unwrap();
        let selection = Value::Object(host.object());
        let add = vm.get(host.object(), "addListener").unwrap();
        let remove = vm.get(host.object(), "removeListener").unwrap();
        for _ in 0..2 {
            let result = vm
                .call(
                    add.clone(),
                    selection.clone(),
                    &[Value::Object(listener)],
                    &mut host,
                )
                .unwrap();
            assert_eq!(result.value, Value::Undefined);
            assert_eq!(host.listeners(), &[listener]);
        }
        assert!(
            vm.call(
                add.clone(),
                selection.clone(),
                &[Value::Object(other)],
                &mut host
            )
            .is_err()
        );
        assert!(
            vm.call(add.clone(), selection.clone(), &[Value::Null], &mut host)
                .is_err()
        );
        assert!(
            vm.call(
                add,
                Value::Object(other),
                &[Value::Object(listener)],
                &mut host
            )
            .is_err()
        );
        assert_eq!(host.listeners(), &[listener]);
        let absent = vm
            .call(
                remove.clone(),
                selection.clone(),
                &[Value::Object(other)],
                &mut host,
            )
            .unwrap();
        assert_eq!(absent.value, Value::Bool(false));
        let removed = vm
            .call(
                remove.clone(),
                selection.clone(),
                &[Value::Object(listener)],
                &mut host,
            )
            .unwrap();
        assert_eq!(removed.value, Value::Bool(true));
        let absent = vm
            .call(remove, selection, &[Value::Object(listener)], &mut host)
            .unwrap();
        assert_eq!(absent.value, Value::Bool(false));
        assert!(host.listeners().is_empty());
        assert_eq!(
            host.stats(),
            Stats {
                add_calls: 2,
                remove_calls: 3
            }
        );
        assert!(SelectionHost::install(&mut vm, Limits::default()).is_err());
    }
}
