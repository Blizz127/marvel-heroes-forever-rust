//! Bounded fresh native Mouse graph and zero-argument hide dispatch.
//!
//! Contract: docs/REA_MOUSE_HIDE.md; retained rea-mouse-hide/verification.json
//! SHA-256 26238131e33099d7d6f3083100470ec7b3ca07db40a625c7446df1b3d57a6b8c.
//! Exact getter joins also use retained callable-member-review.json (SHA-256
//! 16f5cd9a93a9a8d785781e2d8b3b9749d4245472eedc1acd66fb1c5a2ce6bb86) and
//! prototype-member-review.json (SHA-256
//! 660ccfd4e980692a1e018d2513108cd85017ebe0390e7d815ba7f32470d9eac6).
//! The graph retains a genuine empty Array and a separate internal callback.
//! Native method identities are distinct even when their callbacks coincide.
//! Only actual C.hide and the four explicit prototype Undefined callbacks are
//! invoked here. No listener behavior, cursor effect, absent native member,
//! native allocation-count parity or native allocation-failure policy is implied.

use super::{
    ErrorKind, Host, Kind, MouseHideRequest, ObjectId, Property, Result, Text, Value, Vm, text,
};

const CONSTRUCTOR_LEDGER: [(&str, u8); 8] = [
    ("__proto__", 3),
    ("addListener", 1),
    ("broadcastMessage", 1),
    ("removeListener", 1),
    ("_listeners", 1),
    ("show", 7),
    ("hide", 7),
    ("prototype", 3),
];
const PROTOTYPE_LEDGER: [(&str, u8); 6] = [
    ("__proto__", 3),
    ("constructor", 3),
    ("addListener", 1),
    ("removeListener", 1),
    ("show", 1),
    ("hide", 1),
];
const PARENT_LEDGER: [(&str, u8); 1] = [("__proto__", 3)];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseObjects {
    pub constructor: ObjectId,
    pub prototype: ObjectId,
    pub listeners: ObjectId,
    /// Same actual Array retained internally by the native callable at +0x78.
    pub retained_listeners: ObjectId,
    pub add_listener: ObjectId,
    pub broadcast_message: ObjectId,
    pub remove_listener: ObjectId,
    pub show: ObjectId,
    pub hide: ObjectId,
    pub prototype_add_listener: ObjectId,
    pub prototype_remove_listener: ObjectId,
    pub prototype_show: ObjectId,
    pub prototype_hide: ObjectId,
    /// Separate function retained at callable +0x90, with no named own member.
    pub internal_callback: ObjectId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MouseMethod {
    AddListener,
    BroadcastMessage,
    RemoveListener,
    Show,
    Hide,
    PrototypeAddListener,
    PrototypeRemoveListener,
    PrototypeShow,
    PrototypeHide,
    InternalCallback,
}

impl MouseObjects {
    fn methods(self) -> [(ObjectId, MouseMethod); 10] {
        [
            (self.add_listener, MouseMethod::AddListener),
            (self.broadcast_message, MouseMethod::BroadcastMessage),
            (self.remove_listener, MouseMethod::RemoveListener),
            (self.show, MouseMethod::Show),
            (self.hide, MouseMethod::Hide),
            (
                self.prototype_add_listener,
                MouseMethod::PrototypeAddListener,
            ),
            (
                self.prototype_remove_listener,
                MouseMethod::PrototypeRemoveListener,
            ),
            (self.prototype_show, MouseMethod::PrototypeShow),
            (self.prototype_hide, MouseMethod::PrototypeHide),
            (self.internal_callback, MouseMethod::InternalCallback),
        ]
    }
}

fn key_is(key: &[u16], name: &str) -> bool {
    key.iter().copied().eq(name.encode_utf16())
}

impl Vm {
    /// Build the successful fresh graph without publishing a global or token.
    /// The cached native parents are used directly, independently of mutable
    /// public constructor globals. Local failures restore unpublished VM state.
    pub fn build_mouse_graph(&mut self) -> Result<MouseObjects> {
        if self.mouse_objects.is_some() {
            return self.fail(ErrorKind::Unsupported("Mouse graph already built"));
        }
        self.mouse_context()?;
        let (function, object, array) = self.mouse_parents()?;
        // Original Rust representation: C/P/L plus ten distinct function objects.
        // Metadata is guarded internal linkage, not generic mutable descriptors.
        self.mouse_allocation_profile(13, 25, 229)?;
        let before = (self.objects.len(), self.properties, self.string_units);
        let result = (|| {
            let constructor = self.alloc(Kind::MouseConstructor, Some(function))?;
            let add_listener = self.mouse_add_method(
                constructor,
                "addListener",
                MouseMethod::AddListener,
                1,
                function,
            )?;
            let broadcast_message = self.mouse_add_method(
                constructor,
                "broadcastMessage",
                MouseMethod::BroadcastMessage,
                1,
                function,
            )?;
            let remove_listener = self.mouse_add_method(
                constructor,
                "removeListener",
                MouseMethod::RemoveListener,
                1,
                function,
            )?;
            let listeners = self.alloc(Kind::Array(0), Some(array))?;
            self.raw_member(
                constructor,
                text("_listeners"),
                Property::Data(Value::Object(listeners)),
                1,
            )?;
            let show =
                self.mouse_add_method(constructor, "show", MouseMethod::Show, 7, function)?;
            let hide =
                self.mouse_add_method(constructor, "hide", MouseMethod::Hide, 7, function)?;
            let internal_callback = self.alloc(
                Kind::MouseMethod(MouseMethod::InternalCallback),
                Some(function),
            )?;
            let prototype = self.alloc(Kind::MousePrototype, Some(object))?;
            // The P constructor performs the shared metadata join before its
            // four prototype methods are installed by the factory.
            self.raw_member(
                constructor,
                text("prototype"),
                Property::Data(Value::Object(prototype)),
                3,
            )?;
            self.raw_member(
                prototype,
                text("constructor"),
                Property::Data(Value::Object(constructor)),
                3,
            )?;
            let prototype_add_listener = self.mouse_add_method(
                prototype,
                "addListener",
                MouseMethod::PrototypeAddListener,
                1,
                function,
            )?;
            let prototype_remove_listener = self.mouse_add_method(
                prototype,
                "removeListener",
                MouseMethod::PrototypeRemoveListener,
                1,
                function,
            )?;
            let prototype_show =
                self.mouse_add_method(prototype, "show", MouseMethod::PrototypeShow, 1, function)?;
            let prototype_hide =
                self.mouse_add_method(prototype, "hide", MouseMethod::PrototypeHide, 1, function)?;
            Ok(MouseObjects {
                constructor,
                prototype,
                listeners,
                retained_listeners: listeners,
                add_listener,
                broadcast_message,
                remove_listener,
                show,
                hide,
                prototype_add_listener,
                prototype_remove_listener,
                prototype_show,
                prototype_hide,
                internal_callback,
            })
        })();
        match result {
            Ok(graph) => {
                self.mouse_objects = Some(graph);
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

    fn mouse_add_method(
        &mut self,
        owner: ObjectId,
        name: &str,
        method: MouseMethod,
        flags: u8,
        function: ObjectId,
    ) -> Result<ObjectId> {
        let id = self.alloc(Kind::MouseMethod(method), Some(function))?;
        self.raw_member(owner, text(name), Property::Data(Value::Object(id)), flags)?;
        Ok(id)
    }

    /// Category predicate for central lookup/mutation/reflection guards. It does
    /// not establish a canonical graph or grant permission to invoke a method.
    pub(super) fn is_mouse_native(&self, id: ObjectId) -> bool {
        self.objects.get(id.0).is_some_and(|object| {
            matches!(
                object.kind,
                Kind::MouseConstructor | Kind::MousePrototype | Kind::MouseMethod(_)
            )
        })
    }

    /// Identity predicate protects the genuine Array through every public alias.
    pub(super) fn is_mouse_listener(&self, id: ObjectId) -> bool {
        self.mouse_objects
            .is_some_and(|graph| graph.listeners == id || graph.retained_listeners == id)
    }

    /// Only the exact proved fresh own-name ledgers are admitted. Internal
    /// parent links and saved constructor identity supply marker results; these
    /// are not arbitrary mutable data descriptors. Unknown native/Array reads
    /// never become fabricated absence. Call at every prototype lookup hop.
    pub(super) fn mouse_read(&self, id: ObjectId, key: &Text) -> Result<Option<Value>> {
        self.check_object(id)?;
        if !self.is_mouse_native(id) && !self.is_mouse_listener(id) {
            return Ok(None);
        }
        self.mouse_context()?;
        let graph = self.mouse_graph_profile()?;
        let value = if id == graph.constructor {
            [
                (self.function_prototype.unwrap(), "__proto__"),
                (graph.add_listener, "addListener"),
                (graph.broadcast_message, "broadcastMessage"),
                (graph.remove_listener, "removeListener"),
                (graph.listeners, "_listeners"),
                (graph.show, "show"),
                (graph.hide, "hide"),
                (graph.prototype, "prototype"),
            ]
            .into_iter()
            .find(|(_, name)| key_is(key, name))
            .map(|(id, _)| id)
        } else if id == graph.prototype {
            [
                (self.object_prototype.unwrap(), "__proto__"),
                (graph.constructor, "constructor"),
                (graph.prototype_add_listener, "addListener"),
                (graph.prototype_remove_listener, "removeListener"),
                (graph.prototype_show, "show"),
                (graph.prototype_hide, "hide"),
            ]
            .into_iter()
            .find(|(_, name)| key_is(key, name))
            .map(|(id, _)| id)
        } else {
            None
        };
        if let Some(value) = value {
            return Ok(Some(Value::Object(value)));
        }
        self.fail(ErrorKind::Unsupported("unimplemented Mouse member read"))
    }

    /// `this` is deliberately absent: actual hide does not inspect it. The
    /// concrete function identity, active Environment and supplied host matter.
    /// A successful ABI request alone is not evidence of a game/window effect.
    pub(super) fn invoke_mouse_method(
        &mut self,
        id: ObjectId,
        method: MouseMethod,
        args: &[Value],
        host: &mut impl Host,
    ) -> Result<Value> {
        self.check_object(id)?;
        let graph = self.mouse_graph_profile()?;
        if !graph.methods().contains(&(id, method)) {
            return self.fail(ErrorKind::Unsupported("Mouse method identity mismatch"));
        }
        match method {
            MouseMethod::Hide => {
                if !args.is_empty() {
                    return self.fail(ErrorKind::Unsupported("Mouse.hide requires zero arguments"));
                }
                let context = self.mouse_context()?;
                let request = MouseHideRequest {
                    origin: self.origin(),
                    context,
                    index: 0,
                };
                self.host_budget()?;
                host.mouse_hide(request)
                    .map_err(|message| self.error(ErrorKind::Host(message)))?;
                Ok(Value::Undefined)
            }
            MouseMethod::PrototypeAddListener
            | MouseMethod::PrototypeRemoveListener
            | MouseMethod::PrototypeShow
            | MouseMethod::PrototypeHide => {
                // Complete native 18d3a0 ignores receiver, argc and Environment.
                Ok(Value::Undefined)
            }
            _ => self.fail(ErrorKind::Unsupported(
                "unimplemented Mouse method invocation",
            )),
        }
    }

    fn mouse_context(&self) -> Result<ObjectId> {
        let Some(target) = self.current_target else {
            return self.fail(ErrorKind::Unsupported("Mouse requires current target"));
        };
        if !self.is_bound_builtin_context_target(target)? {
            return self.fail(ErrorKind::Unsupported(
                "Mouse requires bound builtin context",
            ));
        }
        Ok(target)
    }

    fn mouse_allocation_profile(
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

    fn mouse_parents(&self) -> Result<(ObjectId, ObjectId, ObjectId)> {
        let (Some(function), Some(object), Some(array)) = (
            self.function_prototype,
            self.object_prototype,
            self.array_prototype,
        ) else {
            return self.fail(ErrorKind::Unsupported(
                "Mouse requires cached native parents",
            ));
        };
        for id in [function, object, array] {
            self.check_object(id)?;
        }
        if function == object
            || function == array
            || object == array
            || !matches!(self.objects[function.0].kind, Kind::FunctionPrototype)
            || !matches!(self.objects[object.0].kind, Kind::Plain)
            || !matches!(self.objects[array.0].kind, Kind::Array(0))
            || self.objects[object.0].prototype.is_some()
            || !self.mouse_link_matches(function, object)
            || !self.mouse_link_matches(array, object)
            || [function, object, array].iter().any(|id| {
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
            return self.fail(ErrorKind::Unsupported("Mouse cached parent profile"));
        }
        Ok((function, object, array))
    }

    fn mouse_graph_profile(&self) -> Result<MouseObjects> {
        let Some(graph) = self.mouse_objects else {
            return self.fail(ErrorKind::Unsupported("Mouse graph is not installed"));
        };
        let (function, object, array) = self.mouse_parents()?;
        let mut ids = vec![graph.constructor, graph.prototype, graph.listeners];
        ids.extend(graph.methods().iter().map(|(id, _)| *id));
        for (index, id) in ids.iter().enumerate() {
            self.check_object(*id)?;
            if ids[..index].contains(id) || [function, object, array].contains(id) {
                return self.fail(ErrorKind::Unsupported("Mouse graph identity profile"));
            }
        }
        if !matches!(
            self.objects[graph.constructor.0].kind,
            Kind::MouseConstructor
        ) || !matches!(self.objects[graph.prototype.0].kind, Kind::MousePrototype)
            || !matches!(self.objects[graph.listeners.0].kind, Kind::Array(0))
            || graph.retained_listeners != graph.listeners
            || !self.mouse_link_matches(graph.constructor, function)
            || !self.mouse_link_matches(graph.prototype, object)
            || !self.mouse_link_matches(graph.listeners, array)
            || !self.mouse_ledger_matches(graph.constructor, &CONSTRUCTOR_LEDGER)
            || !self.mouse_ledger_matches(graph.prototype, &PROTOTYPE_LEDGER)
            || !self.mouse_ledger_matches(graph.listeners, &PARENT_LEDGER)
        {
            return self.fail(ErrorKind::Unsupported("Mouse canonical graph metadata"));
        }
        for (owner, name, expected) in [
            (graph.constructor, "addListener", graph.add_listener),
            (
                graph.constructor,
                "broadcastMessage",
                graph.broadcast_message,
            ),
            (graph.constructor, "removeListener", graph.remove_listener),
            (graph.constructor, "show", graph.show),
            (graph.constructor, "hide", graph.hide),
            (graph.constructor, "_listeners", graph.listeners),
            (graph.constructor, "prototype", graph.prototype),
            (graph.prototype, "addListener", graph.prototype_add_listener),
            (
                graph.prototype,
                "removeListener",
                graph.prototype_remove_listener,
            ),
            (graph.prototype, "show", graph.prototype_show),
            (graph.prototype, "hide", graph.prototype_hide),
            (graph.prototype, "constructor", graph.constructor),
        ] {
            if !self.mouse_data_matches(owner, name, expected) {
                return self.fail(ErrorKind::Unsupported("Mouse canonical member identity"));
            }
        }
        for (id, method) in graph.methods() {
            if !matches!(self.objects[id.0].kind, Kind::MouseMethod(actual) if actual == method)
                || !self.mouse_link_matches(id, function)
                || !self.mouse_ledger_matches(id, &PARENT_LEDGER)
            {
                return self.fail(ErrorKind::Unsupported("Mouse canonical method profile"));
            }
        }
        Ok(graph)
    }

    fn mouse_data_matches(&self, id: ObjectId, name: &str, expected: ObjectId) -> bool {
        matches!(self.objects[id.0].properties.get(&text(name)),
            Some(Property::Data(Value::Object(value))) if *value == expected)
    }

    fn mouse_link_matches(&self, id: ObjectId, parent: ObjectId) -> bool {
        self.objects[id.0].prototype == Some(parent)
            && self.own_flags(id, &text("__proto__")) == 3
            && self.mouse_data_matches(id, "__proto__", parent)
    }

    fn mouse_ledger_matches(&self, id: ObjectId, ledger: &[(&str, u8)]) -> bool {
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
    use super::super::{CodeId, HostCall, HostCommand, Limits};
    use super::*;

    /// ABI-test-only recorder. It does not implement a native handler/window
    /// effect and must never stand in for the host during original replay.
    #[derive(Default)]
    struct AbiRecorder {
        requests: Vec<MouseHideRequest>,
        error: Option<String>,
    }
    impl Host for AbiRecorder {
        fn call(&mut self, _call: HostCall<'_>) -> std::result::Result<Value, String> {
            Err("unexpected generic host call in Mouse ABI test".into())
        }
        fn command(
            &mut self,
            _target: ObjectId,
            _command: HostCommand,
        ) -> std::result::Result<(), String> {
            Err("unexpected command in Mouse ABI test".into())
        }
        fn mouse_hide(&mut self, request: MouseHideRequest) -> std::result::Result<(), String> {
            self.requests.push(request);
            match &self.error {
                Some(message) => Err(message.clone()),
                None => Ok(()),
            }
        }
    }
    fn base() -> Vm {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.install_array_constructor().unwrap();
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
    fn mouse_fresh_graph_retains_real_array_and_distinct_function_identities() {
        let mut vm = base();
        let before = counts(&vm);
        let global_keys = vm.objects[vm.global.0].insertion_keys.clone();
        let graph = vm.build_mouse_graph().unwrap();
        assert_eq!(counts(&vm), (before.0 + 13, before.1 + 25, before.2 + 229));
        assert_eq!(vm.mouse_graph_profile().unwrap(), graph);
        assert_eq!(
            ledger(&vm, graph.constructor),
            CONSTRUCTOR_LEDGER.map(|(name, flags)| (name.into(), flags))
        );
        assert_eq!(
            ledger(&vm, graph.prototype),
            PROTOTYPE_LEDGER.map(|(name, flags)| (name.into(), flags))
        );
        assert_eq!(graph.listeners, graph.retained_listeners);
        assert!(matches!(vm.objects[graph.listeners.0].kind, Kind::Array(0)));
        assert_eq!(ledger(&vm, graph.listeners), vec![("__proto__".into(), 3)]);
        assert_ne!(graph.hide, graph.prototype_hide);
        assert!(vm.is_mouse_native(graph.internal_callback));
        assert!(vm.is_mouse_listener(graph.listeners));
        assert!(!vm.objects[graph.constructor.0].properties.values().any(|value|
            matches!(value,Property::Data(Value::Object(id)) if *id==graph.internal_callback)));
        assert_eq!(global_keys, vm.objects[vm.global.0].insertion_keys);
        let after = counts(&vm);
        assert!(vm.build_mouse_graph().is_err());
        assert_eq!(counts(&vm), after);
    }

    #[test]
    fn mouse_uses_cached_parents_after_public_constructor_globals_change() {
        let mut vm = base();
        for name in ["Object", "Function", "Array"] {
            vm.set(vm.global, name, Value::Undefined).unwrap();
        }
        let graph = vm.build_mouse_graph().unwrap();
        assert_eq!(
            vm.objects[graph.constructor.0].prototype,
            vm.function_prototype
        );
        assert_eq!(vm.objects[graph.prototype.0].prototype, vm.object_prototype);
        assert_eq!(vm.objects[graph.listeners.0].prototype, vm.array_prototype);
    }

    #[test]
    fn mouse_build_budget_and_context_failures_do_not_publish_or_charge() {
        for budget in 0..3 {
            let mut vm = base();
            let before = counts(&vm);
            match budget {
                0 => vm.limits.objects = before.0 + 12,
                1 => vm.limits.properties = before.1 + 24,
                _ => vm.limits.string_units = before.2 + 228,
            }
            assert!(vm.build_mouse_graph().is_err());
            assert_eq!(counts(&vm), before);
            assert!(vm.mouse_objects.is_none());
        }
        let mut vm = base();
        vm.current_target = None;
        let before = counts(&vm);
        assert!(vm.build_mouse_graph().is_err());
        assert_eq!(counts(&vm), before);
        vm.current_target = Some(vm.global);
        assert!(vm.build_mouse_graph().is_err());
        vm.current_target = Some(vm.root);
        vm.array_prototype = None;
        assert!(vm.build_mouse_graph().is_err());
        assert_eq!(counts(&vm), before);
    }

    #[test]
    fn mouse_getter_is_exact_and_unknown_native_reads_never_become_absence() {
        let mut vm = base();
        let graph = vm.build_mouse_graph().unwrap();
        assert_eq!(
            vm.mouse_read(graph.constructor, &text("hide")).unwrap(),
            Some(Value::Object(graph.hide))
        );
        assert_eq!(
            vm.mouse_read(graph.prototype, &text("hide")).unwrap(),
            Some(Value::Object(graph.prototype_hide))
        );
        assert_eq!(
            vm.mouse_read(graph.constructor, &text("__proto__"))
                .unwrap(),
            vm.function_prototype.map(Value::Object)
        );
        assert_eq!(
            vm.mouse_read(graph.prototype, &text("constructor"))
                .unwrap(),
            Some(Value::Object(graph.constructor))
        );
        assert_eq!(
            vm.mouse_read(graph.constructor, &text("_listeners"))
                .unwrap(),
            Some(Value::Object(graph.listeners))
        );
        for (id, key) in [
            (graph.constructor, "Hide"),
            (graph.constructor, "unknown"),
            (graph.prototype, "broadcastMessage"),
            (graph.listeners, "length"),
            (graph.listeners, "0"),
            (graph.hide, "call"),
            (graph.internal_callback, "__proto__"),
        ] {
            assert!(vm.mouse_read(id, &text(key)).is_err());
        }
        assert_eq!(vm.mouse_read(vm.global, &text("anything")).unwrap(), None);
    }

    #[test]
    fn mouse_hide_abi_uses_active_target_and_origin_and_returns_undefined() {
        let mut vm = base();
        let graph = vm.build_mouse_graph().unwrap();
        let context = vm.object().unwrap();
        vm.bind_builtin_context_target(context).unwrap();
        vm.current_target = Some(context);
        vm.current_code = Some(CodeId(0));
        vm.current = Some((6391, 0x52));
        let before = counts(&vm);
        let mut host = AbiRecorder::default();
        assert_eq!(
            vm.invoke_mouse_method(graph.hide, MouseMethod::Hide, &[], &mut host)
                .unwrap(),
            Value::Undefined
        );
        assert_eq!(
            host.requests,
            vec![MouseHideRequest {
                origin: vm.origin(),
                context,
                index: 0
            }]
        );
        assert_eq!(vm.host_calls, 1);
        assert_eq!(counts(&vm), before);
        assert_eq!(vm.mouse_graph_profile().unwrap(), graph);
    }

    #[test]
    fn mouse_central_invoke_ignores_this_but_public_call_does_not_infer_environment() {
        let mut vm = base();
        let graph = vm.build_mouse_graph().unwrap();
        let mut host = AbiRecorder::default();
        for receiver in [Value::Null, Value::Number(7.0), Value::Object(vm.global)] {
            assert_eq!(
                vm.invoke(Value::Object(graph.hide), receiver, &[], &mut host)
                    .unwrap(),
                Value::Undefined
            );
        }
        assert_eq!(host.requests.len(), 3);
        assert!(
            host.requests
                .iter()
                .all(|request| request.context == vm.root && request.index == 0)
        );
        // Public call clears the interpreter context. Even a Mouse receiver
        // cannot provide the native Environment-to-root/handler association.
        assert!(
            vm.call(
                Value::Object(graph.hide),
                Value::Object(graph.constructor),
                &[],
                &mut host
            )
            .is_err()
        );
        assert_eq!(host.requests.len(), 3);
        assert_eq!(vm.host_calls, 0);
    }

    #[test]
    fn mouse_hide_rejects_before_dispatch_and_propagates_host_failure() {
        let mut vm = base();
        let graph = vm.build_mouse_graph().unwrap();
        let mut host = AbiRecorder::default();
        assert!(
            vm.invoke_mouse_method(
                graph.hide,
                MouseMethod::Hide,
                &[Value::Number(0.0)],
                &mut host
            )
            .is_err()
        );
        assert!(
            vm.invoke_mouse_method(graph.prototype_hide, MouseMethod::Hide, &[], &mut host)
                .is_err()
        );
        vm.current_target = None;
        assert!(
            vm.invoke_mouse_method(graph.hide, MouseMethod::Hide, &[], &mut host)
                .is_err()
        );
        vm.current_target = Some(vm.global);
        assert!(
            vm.invoke_mouse_method(graph.hide, MouseMethod::Hide, &[], &mut host)
                .is_err()
        );
        vm.current_target = Some(vm.root);
        vm.limits.host_calls = 0;
        assert!(
            vm.invoke_mouse_method(graph.hide, MouseMethod::Hide, &[], &mut host)
                .is_err()
        );
        assert_eq!(vm.host_calls, 0);
        assert!(host.requests.is_empty());
        vm.limits.host_calls = 1;
        host.error = Some("ABI-test handler unavailable".into());
        assert!(
            matches!(vm.invoke_mouse_method(graph.hide,MouseMethod::Hide,&[],&mut host).unwrap_err().kind,ErrorKind::Host(message) if message=="ABI-test handler unavailable")
        );
        assert_eq!(vm.host_calls, 1);
        assert_eq!(host.requests.len(), 1);
    }

    #[test]
    fn mouse_prototype_callbacks_are_undefined_without_actual_hide_effect() {
        let mut vm = base();
        let graph = vm.build_mouse_graph().unwrap();
        let mut host = AbiRecorder::default();
        vm.current_target = None;
        for (id, method) in graph.methods() {
            if matches!(
                method,
                MouseMethod::PrototypeAddListener
                    | MouseMethod::PrototypeRemoveListener
                    | MouseMethod::PrototypeShow
                    | MouseMethod::PrototypeHide
            ) {
                assert_eq!(
                    vm.invoke_mouse_method(id, method, &[Value::Null], &mut host)
                        .unwrap(),
                    Value::Undefined
                );
            } else {
                assert!(vm.invoke_mouse_method(id, method, &[], &mut host).is_err());
            }
        }
        assert!(host.requests.is_empty());
        assert_eq!(vm.host_calls, 0);
    }

    #[test]
    fn mouse_changed_graph_or_listener_alias_fails_before_host_dispatch() {
        for mutation in 0..4 {
            let mut vm = base();
            let graph = vm.build_mouse_graph().unwrap();
            match mutation {
                0 => {
                    vm.objects[graph.constructor.0].properties.insert(
                        text("hide"),
                        Property::Data(Value::Object(graph.prototype_hide)),
                    );
                }
                1 => {
                    vm.objects[graph.listeners.0].kind = Kind::Array(1);
                }
                2 => {
                    vm.mouse_objects.as_mut().unwrap().retained_listeners =
                        vm.array_prototype.unwrap();
                }
                _ => {
                    vm.objects[graph.hide.0].flags.insert(text("__proto__"), 0);
                }
            }
            let mut host = AbiRecorder::default();
            assert!(
                vm.invoke_mouse_method(graph.hide, MouseMethod::Hide, &[], &mut host)
                    .is_err()
            );
            assert!(host.requests.is_empty());
            assert_eq!(vm.host_calls, 0);
        }
    }
}
