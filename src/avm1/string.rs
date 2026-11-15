//! Bounded native String graph, primitive-call boxing and ASCII uppercase.
//!
//! Static contracts: `docs/STRING_NATIVE_METHODS.md`; private combined receipt
//! `rea-string-uppercase/combined-proof-receipt.json`, SHA-256
//! c0a824584a7fc3ec2c6ec04a601b8739144827eb0b1e02c4a3c80c51f028c6b8.
//! This original implementation admits only an unchanged typed graph and
//! NUL-free ASCII. The thirteen other prototype methods are deliberately not
//! represented: unknown reads, mutations and reflection must remain gated by
//! the VM. This is not a complete factory graph or native allocation-count model.
//! Direct String(), arbitrary construction/coercion and borrowed methods remain
//! separate unsupported profiles. No native callback or host handler runs here.

use super::{ErrorKind, Kind, NativeMarker, ObjectId, Property, Result, Text, Value, Vm, text};

/// Canonical identities for this VM's partial native String graph.
///
/// Building the graph does not install a global binding or register a context.
/// The omitted native methods cannot be observed as missing properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringObjects {
    pub constructor: ObjectId,
    pub prototype: ObjectId,
    pub to_upper_case: ObjectId,
    pub from_char_code: ObjectId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StringRole {
    Prototype,
    Wrapper,
}

#[derive(Debug, Clone)]
pub(super) struct StringState {
    pub(super) role: StringRole,
    pub(super) payload: Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StringMethod {
    ToUpperCase,
}

fn key_is(key: &[u16], name: &str) -> bool {
    key.iter().copied().eq(name.encode_utf16())
}

impl Vm {
    /// Build and cache one canonical partial graph, without publishing it.
    ///
    /// Existing cached parents are used. Repeated construction is rejected;
    /// the deferred registry owns materialization reuse. Only fresh objects are
    /// written. On failure all new objects and allocation charges are removed.
    /// This rollback is an engineering policy, not native OOM behavior.
    pub fn build_string_graph(&mut self) -> Result<StringObjects> {
        if self.string_objects.is_some() {
            return self.fail(ErrorKind::Unsupported("String graph already built"));
        }
        let (function_parent, object_parent) = self.string_parents()?;
        // Four __proto__ names plus fromCharCode/prototype/constructor/
        // toUpperCase = 79 units; the distinct prototype payload adds 13.
        self.string_allocation_profile(4, 8, 92)?;
        let before = (self.objects.len(), self.properties, self.string_units);
        let result = (|| {
            let constructor = self.alloc(Kind::StringConstructor, Some(function_parent))?;
            let from_char_code = self.alloc(
                Kind::UnsupportedNative("String.fromCharCode"),
                Some(function_parent),
            )?;
            self.raw_member(
                constructor,
                text("fromCharCode"),
                Property::Data(Value::Object(from_char_code)),
                7,
            )?;
            let payload = self.allocate_text("[type Object]".encode_utf16().collect())?;
            let prototype = self.alloc(
                Kind::StringObject(StringState {
                    role: StringRole::Prototype,
                    payload,
                }),
                Some(object_parent),
            )?;
            // As in existing native graphs, linked __proto__ and constructor
            // metadata use guarded data identities internally. Native markers
            // are not ordinary mutable descriptors; raw reflection is gated.
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
            let to_upper_case = self.alloc(
                Kind::StringMethod(StringMethod::ToUpperCase),
                Some(function_parent),
            )?;
            self.raw_member(
                prototype,
                text("toUpperCase"),
                Property::Data(Value::Object(to_upper_case)),
                1,
            )?;
            Ok(StringObjects {
                constructor,
                prototype,
                to_upper_case,
                from_char_code,
            })
        })();
        match result {
            Ok(graph) => {
                self.string_objects = Some(graph);
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

    /// Selected primitive-string named CallMethod boxing path only.
    ///
    /// The admitted plain, parentless, data-only GLOBAL makes normal String
    /// lookup equivalent to its own data read without invoking a getter. The
    /// subsequent Object read remains raw. Deferred resolution still uses the
    /// actual current target and may publish the canonical String factory.
    /// Public metadata is checked after resolution, before wrapper allocation.
    pub(super) fn box_string_for_named_call(&mut self, input: Text) -> Result<ObjectId> {
        self.string_context()?;
        self.string_ascii_profile(&input)?;
        self.check_object(self.global)?;
        let global = &self.objects[self.global.0];
        if !matches!(global.kind, Kind::Plain)
            || global.prototype.is_some()
            || global
                .properties
                .values()
                .any(|p| !matches!(p, Property::Data(_)))
            || global
                .insertion_keys
                .iter()
                .any(|key| key_is(key, "__resolve"))
        {
            return self.fail(ErrorKind::Unsupported("String boxing GLOBAL profile"));
        }
        let selected = match global.properties.get(&text("String")) {
            Some(Property::Data(value @ Value::Object(_))) => value.clone(),
            Some(Property::Data(value @ Value::DeferredBuiltin(name)))
                if key_is(name.name(), "String") =>
            {
                value.clone()
            }
            _ => return self.fail(ErrorKind::Unsupported("current GLOBAL.String binding")),
        };
        let selected = self.materialize_value(selected)?;
        let graph = self.string_graph_profile()?;
        if selected != Value::Object(graph.constructor) {
            return self.fail(ErrorKind::Unsupported("noncanonical current GLOBAL.String"));
        }
        let Some(object_constructor) = self.object_constructor else {
            return self.fail(ErrorKind::Unsupported(
                "String boxing requires cached Object",
            ));
        };
        self.check_object(object_constructor)?;
        if object_constructor == graph.constructor
            || !matches!(
                self.objects[object_constructor.0].kind,
                Kind::ObjectConstructor
            )
            || !matches!(
                self.objects[self.global.0].properties.get(&text("Object")),
                Some(Property::Data(Value::Object(id))) if *id == object_constructor
            )
        {
            return self.fail(ErrorKind::Unsupported(
                "current GLOBAL.Object identity branch",
            ));
        }
        self.string_allocation_profile(1, 3, 30)?;
        let before = (self.objects.len(), self.properties, self.string_units);
        let result = (|| {
            let wrapper = self.alloc(
                Kind::StringObject(StringState {
                    role: StringRole::Wrapper,
                    payload: input,
                }),
                Some(graph.prototype),
            )?;
            self.raw_member(
                wrapper,
                text("length"),
                Property::NativeMarker(NativeMarker::StringLength),
                0,
            )?;
            self.raw_member(
                wrapper,
                text("__constructor__"),
                Property::Data(Value::Object(graph.constructor)),
                3,
            )?;
            Ok(wrapper)
        })();
        if result.is_err() {
            self.objects.truncate(before.0);
            self.properties = before.1;
            self.string_units = before.2;
        }
        // The input Arc is retained, not copied or charged as a new allocation.
        // Native interning need not yield a distinct node. The monotonic VM
        // arena retains this wrapper and charges its object and property costs.
        result
    }

    /// Read only represented canonical native properties.
    ///
    /// This must be consulted before accessor dispatch and when a generic
    /// lookup encounters a String role in its prototype chain. Unknown native
    /// names fail explicitly; they are never inferred to be missing.
    pub(super) fn string_member(&self, id: ObjectId, key: &Text) -> Result<Value> {
        self.check_object(id)?;
        let graph = self.string_graph_profile()?;
        if id == graph.constructor {
            return match key.as_ref() {
                key if key_is(key, "__proto__") => {
                    Ok(Value::Object(self.function_prototype.unwrap()))
                }
                key if key_is(key, "prototype") => Ok(Value::Object(graph.prototype)),
                key if key_is(key, "fromCharCode") => Ok(Value::Object(graph.from_char_code)),
                _ => self.fail(ErrorKind::Unsupported(
                    "unimplemented String callable member",
                )),
            };
        }
        if id == graph.to_upper_case || id == graph.from_char_code {
            return if key_is(key, "__proto__") {
                Ok(Value::Object(self.function_prototype.unwrap()))
            } else {
                self.fail(ErrorKind::Unsupported(
                    "unimplemented String native function member",
                ))
            };
        }
        let state = self.string_object_profile(id, graph)?;
        if key_is(key, "__proto__") {
            return Ok(Value::Object(match state.role {
                StringRole::Prototype => self.object_prototype.unwrap(),
                StringRole::Wrapper => graph.prototype,
            }));
        }
        if key_is(key, "constructor") {
            return Ok(Value::Object(graph.constructor));
        }
        if key_is(key, "__constructor__") && state.role == StringRole::Wrapper {
            return Ok(Value::Object(graph.constructor));
        }
        if key_is(key, "toUpperCase") {
            // The graph and wrapper ledgers above prove the ordinary inherited
            // descriptor lookup, including absence of an own wrapper shadow.
            return Ok(Value::Object(graph.to_upper_case));
        }
        if key_is(key, "length") {
            if state.role == StringRole::Prototype {
                // Unlike a fresh wrapper, the prototype has no own length
                // marker. Generic lookup must miss before virtual computation.
                // Any ordinary inherited value (even Undefined) or accessor is
                // outside this canonical profile and cannot be ignored.
                let parent = self.object_prototype.unwrap();
                if self.objects[parent.0].properties.contains_key(key) {
                    return self.fail(ErrorKind::Unsupported("String prototype length override"));
                }
            }
            return Ok(Value::Number(state.payload.len() as f64));
        }
        self.fail(ErrorKind::Unsupported("unimplemented String object member"))
    }

    /// Invoke the proved method on a canonical temporary wrapper.
    ///
    /// Native argc is ignored, but this initial caller profile admits zero only.
    /// The native category-8 test also accepts String.prototype; invoking that
    /// or a borrowed receiver is intentionally outside this narrower contract.
    pub(super) fn invoke_string_method(
        &mut self,
        method: StringMethod,
        this: Value,
        args: &[Value],
    ) -> Result<Value> {
        self.string_context()?;
        if !args.is_empty() {
            return self.fail(ErrorKind::Unsupported(
                "String method argument-count profile",
            ));
        }
        let Value::Object(id) = this else {
            return self.fail(ErrorKind::Unsupported(
                "String method requires typed wrapper",
            ));
        };
        let graph = self.string_graph_profile()?;
        let state = self.string_object_profile(id, graph)?;
        if state.role != StringRole::Wrapper {
            return self.fail(ErrorKind::Unsupported(
                "String method requires wrapper role",
            ));
        }
        let payload = state.payload.clone();
        self.string_allocation_profile(0, 0, payload.len())?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(payload.len())
            .map_err(|_| self.error(ErrorKind::Budget("String output allocation")))?;
        match method {
            StringMethod::ToUpperCase => {
                output.extend(payload.iter().map(|&unit| {
                    if (u16::from(b'a')..=u16::from(b'z')).contains(&unit) {
                        unit - u16::from(b'a' - b'A')
                    } else {
                        unit
                    }
                }));
            }
        }
        Ok(Value::String(self.allocate_text(output)?))
    }

    fn string_context(&self) -> Result<()> {
        let Some(target) = self.current_target else {
            return self.fail(ErrorKind::Unsupported("String requires current target"));
        };
        self.check_object(target)?;
        if !self.builtin_targets.contains_key(&target) {
            return self.fail(ErrorKind::Unsupported(
                "String requires bound builtin context",
            ));
        }
        Ok(())
    }

    fn string_ascii_profile(&self, payload: &Text) -> Result<()> {
        if payload.len() > self.limits.string_units || payload.len() > u32::MAX as usize {
            return self.fail(ErrorKind::Budget("String input units"));
        }
        if payload.iter().any(|unit| !(1..=127).contains(unit)) {
            return self.fail(ErrorKind::Unsupported(
                "String NUL-free ASCII payload profile",
            ));
        }
        Ok(())
    }

    fn string_allocation_profile(
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

    fn string_parents(&self) -> Result<(ObjectId, ObjectId)> {
        let (Some(function), Some(object)) = (self.function_prototype, self.object_prototype)
        else {
            return self.fail(ErrorKind::Unsupported(
                "String requires cached native parents",
            ));
        };
        self.check_object(function)?;
        self.check_object(object)?;
        if function == object
            || !matches!(self.objects[function.0].kind, Kind::FunctionPrototype)
            || !matches!(self.objects[object.0].kind, Kind::Plain)
            || self.objects[object.0].prototype.is_some()
            || !self.string_link_matches(function, object)
            || [function, object].iter().any(|id| {
                let parent = &self.objects[id.0];
                parent
                    .properties
                    .values()
                    .any(|property| matches!(property, Property::Accessor { .. }))
                    || parent
                        .insertion_keys
                        .iter()
                        .any(|key| key_is(key, "__resolve"))
            })
        {
            return self.fail(ErrorKind::Unsupported("String cached parent profile"));
        }
        Ok((function, object))
    }

    fn string_graph_profile(&self) -> Result<StringObjects> {
        let Some(graph) = self.string_objects else {
            return self.fail(ErrorKind::Unsupported("String graph is not installed"));
        };
        let (function, object) = self.string_parents()?;
        let mut ids = [
            graph.constructor,
            graph.prototype,
            graph.to_upper_case,
            graph.from_char_code,
        ];
        for id in ids {
            self.check_object(id)?;
        }
        ids.sort();
        let prototype_payload = match &self.objects[graph.prototype.0].kind {
            Kind::StringObject(state) if state.role == StringRole::Prototype => {
                key_is(&state.payload, "[type Object]")
            }
            _ => false,
        };
        if ids.windows(2).any(|pair| pair[0] == pair[1])
            || !matches!(
                self.objects[graph.constructor.0].kind,
                Kind::StringConstructor
            )
            || !prototype_payload
            || !matches!(
                self.objects[graph.to_upper_case.0].kind,
                Kind::StringMethod(StringMethod::ToUpperCase)
            )
            || !matches!(
                self.objects[graph.from_char_code.0].kind,
                Kind::UnsupportedNative("String.fromCharCode")
            )
            || !self.string_link_matches(graph.constructor, function)
            || !self.string_link_matches(graph.prototype, object)
            || !self.string_link_matches(graph.to_upper_case, function)
            || !self.string_link_matches(graph.from_char_code, function)
            || !self.string_data_matches(graph.constructor, "prototype", graph.prototype, 3)
            || !self.string_data_matches(graph.constructor, "fromCharCode", graph.from_char_code, 7)
            || !self.string_data_matches(graph.prototype, "constructor", graph.constructor, 3)
            || !self.string_data_matches(graph.prototype, "toUpperCase", graph.to_upper_case, 1)
            || !self.string_ledger_matches(
                graph.constructor,
                &[("__proto__", 3), ("fromCharCode", 7), ("prototype", 3)],
            )
            || !self.string_ledger_matches(
                graph.prototype,
                &[("__proto__", 3), ("constructor", 3), ("toUpperCase", 1)],
            )
            || !self.string_ledger_matches(graph.to_upper_case, &[("__proto__", 3)])
            || !self.string_ledger_matches(graph.from_char_code, &[("__proto__", 3)])
        {
            return self.fail(ErrorKind::Unsupported("String canonical graph metadata"));
        }
        Ok(graph)
    }

    fn string_object_profile(&self, id: ObjectId, graph: StringObjects) -> Result<&StringState> {
        self.check_object(id)?;
        let Kind::StringObject(state) = &self.objects[id.0].kind else {
            return self.fail(ErrorKind::Unsupported("String requires typed object"));
        };
        match state.role {
            StringRole::Prototype if id == graph.prototype => {}
            StringRole::Wrapper
                if self.string_link_matches(id, graph.prototype)
                    && self.string_data_matches(id, "__constructor__", graph.constructor, 3)
                    && self.string_ledger_matches(
                        id,
                        &[("__proto__", 3), ("length", 0), ("__constructor__", 3)],
                    )
                    && matches!(
                        self.objects[id.0].properties.get(&text("length")),
                        Some(Property::NativeMarker(NativeMarker::StringLength))
                    ) => {}
            _ => return self.fail(ErrorKind::Unsupported("String object metadata profile")),
        }
        self.string_ascii_profile(&state.payload)?;
        Ok(state)
    }

    fn string_link_matches(&self, id: ObjectId, parent: ObjectId) -> bool {
        self.objects[id.0].prototype == Some(parent)
            && self.string_data_matches(id, "__proto__", parent, 3)
    }

    fn string_data_matches(&self, id: ObjectId, key: &str, expected: ObjectId, flags: u8) -> bool {
        let key = text(key);
        self.own_flags(id, &key) == flags
            && matches!(self.objects[id.0].properties.get(&key), Some(Property::Data(Value::Object(value))) if *value == expected)
    }

    fn string_ledger_matches(&self, id: ObjectId, ledger: &[(&str, u8)]) -> bool {
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
    use super::super::{BuiltinFactory, Limits};
    use super::*;

    fn base() -> Vm {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.bind_builtin_context_target(vm.root).unwrap();
        vm.current_target = Some(vm.root);
        vm
    }

    fn setup() -> (Vm, StringObjects) {
        let mut vm = base();
        let graph = vm.build_string_graph().unwrap();
        vm.set(vm.global, "String", Value::Object(graph.constructor))
            .unwrap();
        (vm, graph)
    }

    fn counts(vm: &Vm) -> (usize, usize, usize) {
        (vm.objects.len(), vm.properties, vm.string_units)
    }

    #[test]
    fn graph_and_wrapper_have_distinct_native_roles_and_ledgers() {
        let mut vm = base();
        let before = counts(&vm);
        let graph = vm.build_string_graph().unwrap();
        assert_eq!(counts(&vm), (before.0 + 4, before.1 + 8, before.2 + 92));
        assert_eq!(vm.string_graph_profile().unwrap(), graph);
        vm.set(vm.global, "String", Value::Object(graph.constructor))
            .unwrap();
        let before = counts(&vm);
        let input = text("aBc");
        let wrapper = vm.box_string_for_named_call(input.clone()).unwrap();
        assert_eq!(counts(&vm), (before.0 + 1, before.1 + 3, before.2 + 30));
        assert!(
            !vm.objects[wrapper.0]
                .properties
                .contains_key(&text("constructor"))
        );
        assert!(
            !vm.objects[wrapper.0]
                .properties
                .contains_key(&text("toUpperCase"))
        );
        assert!(
            !vm.objects[graph.prototype.0]
                .properties
                .contains_key(&text("length"))
        );
        assert!(
            !vm.objects[graph.prototype.0]
                .properties
                .contains_key(&text("__constructor__"))
        );
        assert_eq!(
            vm.string_member(wrapper, &text("toUpperCase")).unwrap(),
            Value::Object(graph.to_upper_case)
        );
        assert_eq!(
            vm.string_member(wrapper, &text("length")).unwrap(),
            Value::Number(3.0)
        );
        assert_eq!(
            vm.string_member(graph.prototype, &text("length")).unwrap(),
            Value::Number(13.0)
        );
        assert_eq!(
            vm.string_object_profile(wrapper, graph).unwrap().payload,
            input
        );
        assert_eq!(
            vm.string_object_profile(graph.prototype, graph)
                .unwrap()
                .payload,
            text("[type Object]")
        );
        let before = counts(&vm);
        assert!(vm.build_string_graph().is_err());
        assert_eq!(counts(&vm), before);
    }

    #[test]
    fn deferred_current_string_is_materialized_once_and_boxes_are_fresh() {
        let mut vm = base();
        let token = vm
            .register_deferred_builtin(vm.global, "String", BuiltinFactory::String, 1)
            .unwrap();
        assert_eq!(vm.get(vm.global, "String").unwrap(), token);
        assert!(vm.string_objects.is_none());
        let one = vm.box_string_for_named_call(text("a")).unwrap();
        let graph = vm.string_objects.unwrap();
        assert_eq!(
            vm.get(vm.global, "String").unwrap(),
            Value::Object(graph.constructor)
        );
        assert!(matches!(token, Value::DeferredBuiltin(_)));
        let before = counts(&vm);
        let two = vm.box_string_for_named_call(text("b")).unwrap();
        assert_ne!(one, two);
        assert_eq!(vm.string_objects, Some(graph));
        assert_eq!(counts(&vm), (before.0 + 1, before.1 + 3, before.2 + 30));
    }

    #[test]
    fn boxing_requires_current_context_and_current_canonical_globals() {
        for case in 0..7 {
            let (mut vm, graph) = setup();
            match case {
                0 => vm.current_target = None,
                1 => {
                    vm.builtin_targets.clear();
                }
                2 => {
                    vm.objects[vm.global.0].prototype = vm.object_prototype;
                }
                3 => {
                    vm.raw_member(
                        vm.global,
                        text("x"),
                        Property::Accessor {
                            getter: Value::Object(graph.to_upper_case),
                            setter: None,
                        },
                        0,
                    )
                    .unwrap();
                }
                4 => {
                    vm.objects[vm.global.0]
                        .insertion_keys
                        .push(text("__resolve"));
                }
                5 => {
                    vm.raw_member(
                        vm.global,
                        text("String"),
                        Property::Data(Value::Undefined),
                        0,
                    )
                    .unwrap();
                }
                6 => {
                    vm.raw_member(
                        vm.global,
                        text("Object"),
                        Property::Data(Value::Object(graph.constructor)),
                        0,
                    )
                    .unwrap();
                }
                _ => unreachable!(),
            }
            let before = counts(&vm);
            assert!(
                vm.box_string_for_named_call(text("a")).is_err(),
                "case {case}"
            );
            assert_eq!(counts(&vm), before);
        }
    }

    #[test]
    fn graph_changes_and_unknown_native_names_do_not_fake_missing() {
        for case in 0..4 {
            let (mut vm, graph) = setup();
            match case {
                0 => {
                    vm.raw_member(
                        graph.constructor,
                        text("prototype"),
                        Property::Data(Value::Object(vm.object_prototype.unwrap())),
                        3,
                    )
                    .unwrap();
                }
                1 => {
                    vm.raw_member(
                        graph.prototype,
                        text("constructor"),
                        Property::Data(Value::Undefined),
                        3,
                    )
                    .unwrap();
                }
                2 => {
                    vm.objects[graph.to_upper_case.0].prototype = None;
                }
                3 => {
                    vm.objects[graph.prototype.0]
                        .flags
                        .insert(text("toUpperCase"), 0);
                }
                _ => unreachable!(),
            }
            assert!(
                vm.box_string_for_named_call(text("a")).is_err(),
                "case {case}"
            );
        }
        let (mut vm, graph) = setup();
        let wrapper = vm.box_string_for_named_call(text("a")).unwrap();
        for id in [
            wrapper,
            graph.prototype,
            graph.constructor,
            graph.to_upper_case,
            graph.from_char_code,
        ] {
            assert!(matches!(
                vm.string_member(id, &text("notImplemented"))
                    .unwrap_err()
                    .kind,
                ErrorKind::Unsupported(_)
            ));
        }
        vm.raw_member(
            vm.object_prototype.unwrap(),
            text("length"),
            Property::Data(Value::Undefined),
            0,
        )
        .unwrap();
        assert!(vm.string_member(graph.prototype, &text("length")).is_err());
        assert_eq!(
            vm.string_member(wrapper, &text("length")).unwrap(),
            Value::Number(1.0)
        );
    }

    #[test]
    fn uppercase_covers_ascii_and_chunk_edges_without_changing_payload() {
        let (mut vm, graph) = setup();
        let mut cases: Vec<Text> = vec![text(""), (1..=127).collect::<Vec<u16>>().into()];
        for len in [503, 504, 505, 1017] {
            cases.push(vec![u16::from(b'z'); len].into());
        }
        for input in cases {
            let wrapper = vm.box_string_for_named_call(input.clone()).unwrap();
            let result = vm
                .invoke_string_method(StringMethod::ToUpperCase, Value::Object(wrapper), &[])
                .unwrap();
            let expected: Text = input
                .iter()
                .map(|unit| {
                    if (97..=122).contains(unit) {
                        unit - 32
                    } else {
                        *unit
                    }
                })
                .collect::<Vec<_>>()
                .into();
            assert_eq!(result, Value::String(expected));
            assert_eq!(
                vm.string_object_profile(wrapper, graph).unwrap().payload,
                input
            );
        }
        assert!(
            vm.invoke_string_method(
                StringMethod::ToUpperCase,
                Value::Object(graph.prototype),
                &[]
            )
            .is_err()
        );
        assert!(
            vm.invoke_string_method(StringMethod::ToUpperCase, Value::string("a"), &[])
                .is_err()
        );
        let wrapper = vm.box_string_for_named_call(text("a")).unwrap();
        assert!(
            vm.invoke_string_method(
                StringMethod::ToUpperCase,
                Value::Object(wrapper),
                &[Value::Undefined]
            )
            .is_err()
        );
    }

    #[test]
    fn nul_unicode_and_budget_failures_leave_unpublished_allocations_unchanged() {
        for (objects, properties, units) in [(3, 8, 92), (4, 7, 92), (4, 8, 91)] {
            let mut vm = base();
            let before = counts(&vm);
            vm.limits.objects = before.0 + objects;
            vm.limits.properties = before.1 + properties;
            vm.limits.string_units = before.2 + units;
            assert!(vm.build_string_graph().is_err());
            assert_eq!(counts(&vm), before);
            assert!(vm.string_objects.is_none());
        }
        let (mut vm, _) = setup();
        for input in [vec![0], vec![128], vec![0xd800]] {
            let before = counts(&vm);
            assert!(vm.box_string_for_named_call(input.into()).is_err());
            assert_eq!(counts(&vm), before);
        }
        for (objects, properties, units) in [(0, 3, 30), (1, 2, 30), (1, 3, 29)] {
            let (mut vm, _) = setup();
            let before = counts(&vm);
            vm.limits.objects = before.0 + objects;
            vm.limits.properties = before.1 + properties;
            vm.limits.string_units = before.2 + units;
            assert!(vm.box_string_for_named_call(text("a")).is_err());
            assert_eq!(counts(&vm), before);
        }
        let (mut vm, graph) = setup();
        let wrapper = vm.box_string_for_named_call(text("abc")).unwrap();
        let before = counts(&vm);
        vm.limits.string_units = before.2 + 2;
        assert!(
            vm.invoke_string_method(StringMethod::ToUpperCase, Value::Object(wrapper), &[])
                .is_err()
        );
        assert_eq!(counts(&vm), before);
        assert_eq!(
            vm.string_object_profile(wrapper, graph).unwrap().payload,
            text("abc")
        );
    }
}
