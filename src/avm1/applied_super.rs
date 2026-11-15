//! Bounded Super preparation for an ordinary function applied to a live clip.
//!
//! Native paths: `docs/REA_APPLIED_SUPER.md`, private
//! `rea-applied-super/finding-summary.json`. The incoming prototype cursor and
//! the actual display receiver are separate: each Super advances only the former.
//! Exact recorded Extends links, flags and Function2 scopes below are deliberately
//! stricter admission guards than native lookup. They do not model missing-name
//! fallback, getters, deferred constructors, activation capture, or mutation.
//! The central VM owns live attached identity/context and invocation admission.

use super::Operands;
use super::{AttachedClipId, ErrorKind, Kind, ObjectId, Property, Result, Value, Vm, text};

/// Internal invocation state, never an ordinary script object's member table.
#[derive(Debug, Clone, Copy)]
pub(super) struct AttachedSuperCall {
    pub receiver: AttachedClipId,
    pub prototype: Option<ObjectId>,
}

impl Vm {
    /// Prepare one opaque Super reference after the caller validates its attached
    /// context. No class constructor is called and no display prototype changes.
    /// All metadata checks precede the sole VM-object allocation. Unrelated
    /// members, including accessors and resolver history, do not participate in
    /// these proved own-data lookups and are deliberately not inspected.
    pub(super) fn prepare_attached_super(
        &mut self,
        class: ObjectId,
        call: AttachedSuperCall,
    ) -> Result<ObjectId> {
        self.applied_super_function2_profile(class)?;
        let Some(base) = self.class_bases.get(&class).copied() else {
            return self.fail(ErrorKind::Unsupported(
                "attached super requires recorded Extends class",
            ));
        };
        for id in [base.constructor, base.prototype, base.parent_prototype] {
            self.check_object(id)?;
        }
        if call.prototype != Some(base.prototype)
            || class == base.constructor
            || base.prototype == base.parent_prototype
            || !matches!(self.objects[base.prototype.0].kind, Kind::Plain)
        {
            return self.fail(ErrorKind::Unsupported(
                "attached super recorded prototype cursor profile",
            ));
        }
        // Each recorded Extends prototype was freshly allocated for one class.
        // This rejects ambiguous/cyclic metadata without scanning unrelated
        // prototype properties or following arbitrary script lookup behavior.
        self.visit_members(self.class_bases.len())?;
        if self
            .class_bases
            .iter()
            .any(|(owner, other)| *owner != class && other.prototype == base.prototype)
        {
            return self.fail(ErrorKind::Unsupported(
                "attached super duplicate recorded prototype",
            ));
        }
        if !self.applied_super_data_link(class, "prototype", base.prototype, None)
            || !self.applied_super_data_link(
                base.prototype,
                "__constructor__",
                base.constructor,
                Some(3),
            )
            || !self.applied_super_data_link(
                base.prototype,
                "__proto__",
                base.parent_prototype,
                Some(3),
            )
            || self.objects[base.prototype.0].prototype != Some(base.parent_prototype)
            || !self.applied_super_data_link(
                base.constructor,
                "prototype",
                base.parent_prototype,
                None,
            )
        {
            return self.fail(ErrorKind::Unsupported(
                "attached super modified or non-data inheritance metadata",
            ));
        }
        match &self.objects[base.constructor.0].kind {
            Kind::Script(_) => self.applied_super_function2_profile(base.constructor)?,
            Kind::MovieClipConstructor
                if self.movie_clip_constructor == Some(base.constructor)
                    && self.movie_clip_prototype == Some(base.parent_prototype) => {}
            _ => {
                return self.fail(ErrorKind::Unsupported(
                    "attached super requires ordinary Function2 or native MovieClip parent",
                ));
            }
        }
        // ClassBase supplies a nonnull parent. Native Super construction also
        // dereferences that parent; no null fallback is invented here.
        self.alloc(
            Kind::AttachedSuper {
                constructor: base.constructor,
                call: AttachedSuperCall {
                    receiver: call.receiver,
                    prototype: Some(base.parent_prototype),
                },
            },
            None,
        )
    }

    fn applied_super_data_link(
        &self,
        owner: ObjectId,
        key: &str,
        expected: ObjectId,
        flags: Option<u8>,
    ) -> bool {
        let key = text(key);
        matches!(self.objects[owner.0].properties.get(&key),
            Some(Property::Data(Value::Object(value))) if *value == expected)
            && flags.is_none_or(|flags| self.own_flags(owner, &key) == flags)
    }

    fn applied_super_function2_profile(&self, class: ObjectId) -> Result<()> {
        self.check_object(class)?;
        let Kind::Script(function) = &self.objects[class.0].kind else {
            return self.fail(ErrorKind::Unsupported(
                "attached super requires ordinary ScriptFunction2 class",
            ));
        };
        if function.scopes.as_slice() != [function.target, self.global] {
            return self.fail(ErrorKind::Unsupported(
                "attached super captured activation/With profile",
            ));
        }
        let Some(program) = self.code.get(function.code.0) else {
            return self.fail(ErrorKind::InvalidCode(function.code));
        };
        let Some(instruction) = program.instruction_at(function.offset) else {
            return self.fail(ErrorKind::InvalidCode(function.code));
        };
        if !matches!(&instruction.operands,
            Operands::Function(definition)
                if instruction.opcode == 0x8e
                    && definition.flags.is_some()
                    && definition.register_count.is_some())
        {
            return self.fail(ErrorKind::Unsupported(
                "attached super requires ordinary ScriptFunction2 class",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::Program;
    use super::super::{
        AttachedClipSource, AttachedClipSpec, AttachedEventState, DisplayTargetId, Limits,
        LoadedMovieSourceId, RejectHost,
    };
    use super::*;
    use std::sync::Arc;

    fn function(vm: &mut Vm) -> ObjectId {
        // Synthetic anonymous Function2, no arguments or body. The helper tests
        // metadata preparation only; central tests own attached invocation.
        let bytes = [0x8e, 8, 0, 0, 0, 0, 3, 0x2a, 0, 0, 0];
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        let value = vm
            .execute(code, vm.root(), &mut RejectHost)
            .unwrap()
            .stack
            .pop()
            .unwrap();
        let Value::Object(id) = value else {
            panic!("Function2 object expected")
        };
        id
    }
    fn extend(vm: &mut Vm, class: ObjectId, parent: ObjectId) {
        vm.set(vm.root(), "testChild", Value::Object(class))
            .unwrap();
        vm.set(vm.root(), "testParent", Value::Object(parent))
            .unwrap();
        let mut bytes = Vec::new();
        for name in ["testChild", "testParent"] {
            bytes.push(0x96);
            bytes.extend(((name.len() + 2) as u16).to_le_bytes());
            bytes.push(0);
            bytes.extend(name.as_bytes());
            bytes.extend([0, 0x1c]);
        }
        bytes.push(0x69);
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        vm.execute(code, vm.root(), &mut RejectHost).unwrap();
    }
    fn fixture() -> (Vm, ObjectId, ObjectId, AttachedSuperCall) {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        let native = vm.install_movie_clip_constructor().unwrap();
        let parent = function(&mut vm);
        extend(&mut vm, parent, native);
        let class = function(&mut vm);
        extend(&mut vm, class, parent);
        let receiver = vm
            .create_attached_movie_clip(AttachedClipSpec {
                target: DisplayTargetId(100),
                source: AttachedClipSource {
                    movie: LoadedMovieSourceId(1),
                    sprite_id: 2,
                    header_version: 15,
                },
                parent: None,
                event_state: AttachedEventState {
                    target_flags: 0x5200_0180,
                    load_flags: 0,
                    source_flags: 0,
                },
            })
            .unwrap();
        let call = AttachedSuperCall {
            receiver,
            prototype: Some(vm.class_bases[&class].prototype),
        };
        (vm, class, parent, call)
    }
    fn counts(vm: &Vm) -> (usize, usize, usize) {
        (vm.objects.len(), vm.properties, vm.string_units)
    }
    fn rejected_without_allocation(vm: &mut Vm, class: ObjectId, call: AttachedSuperCall) {
        let before = counts(vm);
        assert!(vm.prepare_attached_super(class, call).is_err());
        assert_eq!(counts(vm), before);
        assert_eq!(vm.host_calls, 0);
    }
    #[test]
    fn attached_super_advances_cursor_keeps_receiver_and_ignores_unrelated_accessors() {
        let (mut vm, class, parent, call) = fixture();
        let base = vm.class_bases[&class];
        let getter = vm.native(900).unwrap();
        for key in ["label", "__resolve"] {
            vm.raw_member(
                base.prototype,
                text(key),
                Property::Accessor {
                    getter: Value::Object(getter),
                    setter: None,
                },
                0,
            )
            .unwrap();
        }
        let initial_prototype = vm.attached_info(call.receiver).unwrap().prototype;
        let before = counts(&vm);
        let first = vm.prepare_attached_super(class, call).unwrap();
        let Kind::AttachedSuper {
            constructor,
            call: next,
        } = vm.objects[first.0].kind.clone()
        else {
            panic!()
        };
        assert_eq!(constructor, parent);
        assert_eq!(next.receiver, call.receiver);
        assert_eq!(next.prototype, Some(base.parent_prototype));
        assert_eq!(counts(&vm), (before.0 + 1, before.1, before.2));
        assert!(vm.objects[first.0].properties.is_empty());
        assert!(vm.objects[first.0].prototype.is_none());
        let second = vm.prepare_attached_super(parent, next).unwrap();
        let Kind::AttachedSuper {
            constructor,
            call: final_call,
        } = vm.objects[second.0].kind.clone()
        else {
            panic!()
        };
        assert_eq!(Some(constructor), vm.movie_clip_constructor);
        assert_eq!(final_call.receiver, call.receiver);
        assert_eq!(final_call.prototype, vm.movie_clip_prototype);
        let repeat = vm.prepare_attached_super(class, call).unwrap();
        assert_ne!(repeat, first);
        assert_eq!(
            vm.attached_info(call.receiver).unwrap().prototype,
            initial_prototype
        );
        assert!(vm.attached_info(call.receiver).unwrap().storage.is_none());
        assert_eq!(vm.host_calls, 0);
    }
    #[test]
    fn attached_super_rejects_each_metadata_shadow_or_changed_link_before_allocation() {
        for mutation in 0..9 {
            let (mut vm, class, parent, mut call) = fixture();
            let base = vm.class_bases[&class];
            match mutation {
                0 => call.prototype = None,
                1 => vm
                    .raw_member(
                        base.prototype,
                        text("__constructor__"),
                        Property::Data(Value::Undefined),
                        3,
                    )
                    .unwrap(),
                2 => vm
                    .raw_member(
                        base.prototype,
                        text("__constructor__"),
                        Property::Accessor {
                            getter: Value::Object(parent),
                            setter: None,
                        },
                        3,
                    )
                    .unwrap(),
                3 => {
                    vm.objects[base.prototype.0]
                        .flags
                        .insert(text("__constructor__"), 0);
                }
                4 => vm
                    .raw_member(
                        base.prototype,
                        text("__proto__"),
                        Property::Data(Value::Object(base.prototype)),
                        3,
                    )
                    .unwrap(),
                5 => {
                    vm.objects[base.prototype.0].prototype = None;
                }
                6 => vm
                    .raw_member(
                        class,
                        text("prototype"),
                        Property::Accessor {
                            getter: Value::Object(parent),
                            setter: None,
                        },
                        0,
                    )
                    .unwrap(),
                7 => vm
                    .raw_member(
                        parent,
                        text("prototype"),
                        Property::Data(Value::Object(base.prototype)),
                        0,
                    )
                    .unwrap(),
                8 => {
                    let duplicate = function(&mut vm);
                    vm.class_bases.insert(duplicate, base);
                }
                _ => unreachable!(),
            }
            rejected_without_allocation(&mut vm, class, call);
        }
    }
    #[test]
    fn attached_super_rejects_captured_scopes_parent_kind_and_object_budget() {
        let (mut vm, class, parent, call) = fixture();
        let Kind::Script(function) = &mut vm.objects[parent.0].kind else {
            panic!()
        };
        function.scopes.push(function.target);
        rejected_without_allocation(&mut vm, class, call);
        let (mut vm, class, parent, call) = fixture();
        vm.objects[parent.0].kind = Kind::Native(1);
        rejected_without_allocation(&mut vm, class, call);
        let (mut vm, class, _, call) = fixture();
        vm.limits.objects = vm.objects.len();
        let before = counts(&vm);
        assert_eq!(
            vm.prepare_attached_super(class, call).unwrap_err().kind,
            ErrorKind::Budget("objects")
        );
        assert_eq!(counts(&vm), before);
    }
}
