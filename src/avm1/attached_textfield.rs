//! Explicit attached TextField identity and three native member requests.
//!
//! Native paths and remaining lifecycle requirements are recorded in
//! `docs/TEXTFIELD_RUNTIME.md`. Adoption requires a host-established live case-4
//! instance and current selector-10 prototype identity. No constructor, binding
//! hook, text document or renderer is fabricated here. Storage and public display
//! values remain distinct; the host owns all actual text and format state.

use super::{
    AttachedClipId, AttachedTarget, CodeOrigin, DisplayTargetId, ErrorKind, Host, Kind,
    LoadedMovieSourceId, ObjectId, Result, Text, Value, Vm, text,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AttachedTextFieldId(pub usize);
/// Diagnostic internal storage identity; not an ordinary public Object value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedTextFieldStorageId(pub usize);
/// Nonzero identity supplied by the host for the actual current prototype.
/// This is neither a VM ObjectId nor proof of native prototype resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextFieldPrototypeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextFieldWatchState {
    Absent,
    Present,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedTextFieldSource {
    pub movie: LoadedMovieSourceId,
    pub edit_text_id: u16,
    pub header_version: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedTextFieldSpec {
    pub target: DisplayTargetId,
    pub source: AttachedTextFieldSource,
    pub parent: AttachedClipId,
    pub environment_owner: AttachedClipId,
    pub target_flags_6a: u32,
    pub prototype: TextFieldPrototypeId,
    pub watch_state: TextFieldWatchState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedTextFieldInfo {
    pub target: DisplayTargetId,
    pub source: AttachedTextFieldSource,
    pub parent: AttachedClipId,
    pub environment_owner: AttachedClipId,
    pub target_flags_6a: u32,
    pub prototype: TextFieldPrototypeId,
    pub watch_state: TextFieldWatchState,
    pub owner: ObjectId,
    pub storage: Option<AttachedTextFieldStorageId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextFieldMember {
    Text,
    Html,
    HtmlText,
}

#[derive(Debug, Clone)]
pub struct TextFieldMemberRequest {
    pub origin: Option<CodeOrigin>,
    pub source_offset: Option<usize>,
    pub context: Option<ObjectId>,
    pub environment_owner: ObjectId,
    pub field: AttachedTextFieldId,
    pub owner: ObjectId,
    pub target: DisplayTargetId,
    pub source: AttachedTextFieldSource,
    pub member: TextFieldMember,
}

#[derive(Debug, Clone, Copy)]
struct TextFieldStorage {
    object: ObjectId,
    prototype: TextFieldPrototypeId,
}

#[derive(Debug, Clone)]
pub(super) struct AttachedTextField {
    pub(super) spec: AttachedTextFieldSpec,
    pub(super) owner: ObjectId,
    storage: Option<TextFieldStorage>,
}

impl Vm {
    pub(super) fn check_attached_text_field(&self, id: AttachedTextFieldId) -> Result<()> {
        if id.0 < self.text_fields.len() {
            Ok(())
        } else {
            self.fail(ErrorKind::InvalidAttachedTextField(id))
        }
    }

    /// Adopt an established fixed live association, before its parent's complete
    /// child snapshot is frozen. Source IDs do not discover or create an instance.
    /// Prototype mutation, watch-state changes, reparenting and retirement are
    /// outside this profile; observations must remain valid for later accesses.
    pub fn adopt_attached_text_field(
        &mut self,
        spec: AttachedTextFieldSpec,
    ) -> Result<AttachedTextFieldId> {
        if spec.source.header_version != self.movie_version
            || self.attached_info(spec.parent)?.environment_header_version != self.movie_version
            || self
                .attached_info(spec.environment_owner)?
                .environment_header_version
                != self.movie_version
        {
            return self.fail(ErrorKind::Unsupported(
                "TextField loaded movie version profile",
            ));
        }
        self.adopt_text_field_snapshot(spec)
    }

    /// Explicit local EditText definition of the parent's bound MovieDef. The
    /// host validates that resource association; this method verifies its source
    /// identity/version against the established parent binding. The field's
    /// first marked Environment ancestor may independently belong to another
    /// admitted 7+ movie. No script-origin version is substituted for either.
    pub fn adopt_attached_text_field_from_parent_movie(
        &mut self,
        spec: AttachedTextFieldSpec,
    ) -> Result<AttachedTextFieldId> {
        let parent = self.attached_info(spec.parent)?;
        if !(7..=19).contains(&spec.source.header_version)
            || spec.source.movie != parent.environment_movie
            || spec.source.header_version != parent.environment_header_version
        {
            return self.fail(ErrorKind::Unsupported(
                "TextField local parent MovieDef binding",
            ));
        }
        self.adopt_text_field_snapshot(spec)
    }

    fn adopt_text_field_snapshot(
        &mut self,
        spec: AttachedTextFieldSpec,
    ) -> Result<AttachedTextFieldId> {
        if spec.target_flags_6a >> 24 != 0x52 {
            return self.fail(ErrorKind::Unsupported("TextField case4 owner-offset field"));
        }
        if spec.prototype.0 == 0 {
            return self.fail(ErrorKind::Unsupported(
                "TextField current prototype identity",
            ));
        }
        if self.attached_targets.contains_key(&spec.target) {
            return self.fail(ErrorKind::Unsupported("duplicate attached target identity"));
        }
        if self.attached_info(spec.parent)?.child_snapshot_installed {
            return self.fail(ErrorKind::Unsupported(
                "TextField adoption after frozen child snapshot",
            ));
        }
        self.text_field_environment(&spec)?;
        if self.text_fields.len() >= self.limits.objects {
            return self.fail(ErrorKind::Budget("attached TextFields"));
        }
        let id = AttachedTextFieldId(self.text_fields.len());
        let owner = self.alloc(Kind::AttachedTextFieldOwner(id), None)?;
        self.text_fields.push(AttachedTextField {
            spec,
            owner,
            storage: None,
        });
        self.attached_targets
            .insert(spec.target, AttachedTarget::TextField(id));
        Ok(id)
    }

    pub fn attached_text_field_info(
        &self,
        id: AttachedTextFieldId,
    ) -> Result<AttachedTextFieldInfo> {
        self.check_attached_text_field(id)?;
        let field = &self.text_fields[id.0];
        Ok(AttachedTextFieldInfo {
            target: field.spec.target,
            source: field.spec.source,
            parent: field.spec.parent,
            environment_owner: field.spec.environment_owner,
            target_flags_6a: field.spec.target_flags_6a,
            prototype: field.spec.prototype,
            watch_state: field.spec.watch_state,
            owner: field.owner,
            storage: field
                .storage
                .map(|storage| AttachedTextFieldStorageId(storage.object.0)),
        })
    }

    pub fn attached_text_field_owner(&self, id: AttachedTextFieldId) -> Result<ObjectId> {
        self.check_attached_text_field(id)?;
        Ok(self.text_fields[id.0].owner)
    }

    pub fn attached_text_field_value(&self, id: AttachedTextFieldId) -> Result<Value> {
        self.check_attached_text_field(id)?;
        Ok(Value::DisplayTextField(id))
    }

    fn text_field_environment(&self, spec: &AttachedTextFieldSpec) -> Result<ObjectId> {
        self.check_attached(spec.environment_owner)?;
        let mut current = Some(spec.parent);
        let mut depth = 0;
        while let Some(id) = current {
            if depth >= self.limits.prototype_depth {
                return self.fail(ErrorKind::Budget("TextField Environment parent depth"));
            }
            let info = self.attached_info(id)?;
            if info.event_state.target_flags & 0x400 != 0 {
                if id != spec.environment_owner {
                    return self.fail(ErrorKind::Unsupported(
                        "TextField first Environment ancestor mismatch",
                    ));
                }
                return Ok(info.owner);
            }
            current = info.parent;
            depth += 1;
        }
        self.fail(ErrorKind::Unsupported(
            "TextField has no marked Environment ancestor",
        ))
    }

    fn text_field_request(
        &self,
        id: AttachedTextFieldId,
        key: &Text,
    ) -> Result<TextFieldMemberRequest> {
        self.check_attached_text_field(id)?;
        let member = if key.as_ref() == text("text").as_ref() {
            TextFieldMember::Text
        } else if key.as_ref() == text("html").as_ref() {
            TextFieldMember::Html
        } else if key.as_ref() == text("htmlText").as_ref() {
            TextFieldMember::HtmlText
        } else {
            return self.fail(ErrorKind::Unsupported("TextField native member profile"));
        };
        let Some(context) = self.current_target else {
            return self.fail(ErrorKind::Unsupported(
                "TextField requires active script context",
            ));
        };
        self.check_object(context)?;
        if !matches!(
            self.objects[context.0].kind,
            Kind::Plain | Kind::AttachedOwner(_)
        ) {
            return self.fail(ErrorKind::Unsupported(
                "TextField script context target profile",
            ));
        }
        let field = &self.text_fields[id.0];
        // Marker bits can change through the Clip state API. Recheck the exact
        // first ancestor, rather than using a stale adoption-time association.
        let environment_owner = self.text_field_environment(&field.spec)?;
        Ok(TextFieldMemberRequest {
            origin: self.origin(),
            source_offset: self.current.map(|current| current.0),
            context: Some(context),
            environment_owner,
            field: id,
            owner: field.owner,
            target: field.spec.target,
            source: field.spec.source,
            member,
        })
    }

    fn validate_text_field_value(&self, member: TextFieldMember, value: &Value) -> Result<()> {
        match (member, value) {
            (TextFieldMember::Html, Value::Bool(_)) => Ok(()),
            (TextFieldMember::Text | TextFieldMember::HtmlText, Value::String(units)) => {
                // Bound validation work without charging an existing input a
                // second time. Returned strings are charged separately below.
                if units.len() > self.limits.string_units {
                    return self.fail(ErrorKind::Budget("TextField string units"));
                }
                if units.contains(&0)
                    || char::decode_utf16(units.iter().copied()).any(|unit| unit.is_err())
                {
                    return self.fail(ErrorKind::Unsupported(
                        "TextField valid NUL-free UTF-16 profile",
                    ));
                }
                Ok(())
            }
            _ => self.fail(ErrorKind::Unsupported(
                "TextField exact Boolean/String member value",
            )),
        }
    }

    fn ensure_text_field_storage(&mut self, id: AttachedTextFieldId) -> Result<ObjectId> {
        if let Some(storage) = self.text_fields[id.0].storage {
            self.check_object(storage.object)?;
            if !matches!(self.objects[storage.object.0].kind, Kind::BoundTextFieldStorage(owner) if owner == id)
                || storage.prototype != self.text_fields[id.0].spec.prototype
            {
                return self.fail(ErrorKind::Unsupported(
                    "TextField storage association mismatch",
                ));
            }
            return Ok(storage.object);
        }
        let object = self.alloc(Kind::BoundTextFieldStorage(id), None)?;
        self.text_fields[id.0].storage = Some(TextFieldStorage {
            object,
            prototype: self.text_fields[id.0].spec.prototype,
        });
        Ok(object)
    }

    pub(super) fn attached_text_field_read(
        &mut self,
        id: AttachedTextFieldId,
        key: &Text,
        host: &mut impl Host,
    ) -> Result<Value> {
        let request = self.text_field_request(id, key)?;
        let member = request.member;
        self.host_budget()?;
        let value = host
            .get_text_field_member(request)
            .map_err(|error| self.error(ErrorKind::Host(error)))?;
        self.validate_text_field_value(member, &value)?;
        if let Value::String(units) = &value {
            if units.len() > self.limits.string_units.saturating_sub(self.string_units) {
                return self.fail(ErrorKind::Budget("allocated string units"));
            }
            // Immutable host text is adopted without changing its code units.
            self.string_units += units.len();
        }
        Ok(value)
    }

    pub(super) fn attached_text_field_write(
        &mut self,
        id: AttachedTextFieldId,
        key: Text,
        value: Value,
        host: &mut impl Host,
    ) -> Result<()> {
        let request = self.text_field_request(id, &key)?;
        self.validate_text_field_value(request.member, &value)?;
        if self.text_fields[id.0].spec.watch_state != TextFieldWatchState::Absent {
            return self.fail(ErrorKind::Unsupported(
                "TextField applicable watch is not known absent",
            ));
        }
        // Engineering admission happens before allocation; no native side-effect
        // claim is made for rejected input, unknown watch or exhausted budget.
        if self.host_calls >= self.limits.host_calls {
            return self.fail(ErrorKind::Budget("host calls"));
        }
        self.ensure_text_field_storage(id)?;
        self.host_budget()?;
        // Native storage precedes the admitted setter body. A backend error
        // keeps that storage and the consumed call budget; no rollback is implied.
        host.set_text_field_member(request, value)
            .map_err(|error| self.error(ErrorKind::Host(error)))
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        AttachedClipSource, AttachedClipSpec, AttachedEventState, HostCall, HostCommand, Limits,
    };
    use super::*;

    #[derive(Default)]
    struct Backend {
        gets: Vec<TextFieldMemberRequest>,
        sets: Vec<(TextFieldMemberRequest, Value)>,
        read: Option<Value>,
        fail_write: bool,
    }
    impl Host for Backend {
        fn call(&mut self, _: HostCall<'_>) -> std::result::Result<Value, String> {
            Err("unexpected call".into())
        }
        fn command(&mut self, _: ObjectId, _: HostCommand) -> std::result::Result<(), String> {
            Err("unexpected command".into())
        }
        fn get_text_field_member(
            &mut self,
            request: TextFieldMemberRequest,
        ) -> std::result::Result<Value, String> {
            self.gets.push(request);
            self.read.clone().ok_or_else(|| "no read result".into())
        }
        fn set_text_field_member(
            &mut self,
            request: TextFieldMemberRequest,
            value: Value,
        ) -> std::result::Result<(), String> {
            self.sets.push((request, value));
            if self.fail_write {
                Err("explicit backend profile rejection".into())
            } else {
                Ok(())
            }
        }
    }

    fn clip(
        vm: &mut Vm,
        target: u64,
        parent: Option<AttachedClipId>,
        marked: bool,
    ) -> AttachedClipId {
        vm.create_attached_movie_clip(AttachedClipSpec {
            target: DisplayTargetId(target),
            source: AttachedClipSource {
                movie: LoadedMovieSourceId(10),
                sprite_id: 1,
                header_version: 15,
            },
            parent,
            event_state: AttachedEventState {
                target_flags: 0x5200_0000 | if marked { 0x400 } else { 0 },
                load_flags: 0,
                source_flags: 0,
            },
        })
        .unwrap()
    }
    fn fixture() -> (Vm, AttachedClipId, AttachedClipId, AttachedTextFieldSpec) {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        let root = clip(&mut vm, 100, None, true);
        let parent = clip(&mut vm, 101, Some(root), false);
        let spec = AttachedTextFieldSpec {
            target: DisplayTargetId(200),
            source: AttachedTextFieldSource {
                movie: LoadedMovieSourceId(20),
                edit_text_id: 1034,
                header_version: 15,
            },
            parent,
            environment_owner: root,
            target_flags_6a: 0x5200_0180,
            prototype: TextFieldPrototypeId(0x1000),
            watch_state: TextFieldWatchState::Absent,
        };
        vm.current_target = Some(vm.attached_owner(parent).unwrap());
        (vm, root, parent, spec)
    }

    #[test]
    fn text_field_identity_is_distinct_and_storage_keeps_opaque_prototype() {
        let (mut vm, root, _, spec) = fixture();
        let id = vm.adopt_attached_text_field(spec).unwrap();
        let second = vm
            .adopt_attached_text_field(AttachedTextFieldSpec {
                target: DisplayTargetId(201),
                ..spec
            })
            .unwrap();
        let info = vm.attached_text_field_info(id).unwrap();
        assert_eq!(info.prototype, TextFieldPrototypeId(0x1000));
        assert_eq!(info.source, spec.source);
        assert!(info.storage.is_none());
        assert_eq!(
            vm.attached_text_field_value(id).unwrap(),
            Value::DisplayTextField(id)
        );
        assert_ne!(info.owner, vm.attached_text_field_owner(second).unwrap());
        assert!(vm.objects[info.owner.0].properties.is_empty());
        assert!(vm.objects[info.owner.0].prototype.is_none());
        let mut host = Backend::default();
        let unicode = Value::string("é😀");
        vm.attached_text_field_write(id, text("text"), unicode.clone(), &mut host)
            .unwrap();
        let (request, value) = &host.sets[0];
        assert_eq!(*value, unicode);
        assert_eq!(request.field, id);
        assert_eq!(request.owner, info.owner);
        assert_eq!(request.target, spec.target);
        assert_eq!(request.environment_owner, vm.attached_owner(root).unwrap());
        assert_ne!(request.context, Some(info.owner));
        assert_ne!(request.context, Some(request.environment_owner));
        let storage = vm.text_fields[id.0].storage.unwrap();
        assert_eq!(storage.prototype, spec.prototype);
        assert_ne!(storage.object, info.owner);
        assert!(vm.objects[storage.object.0].properties.is_empty());
        assert!(vm.objects[storage.object.0].prototype.is_none());
        assert!(
            vm.attached_text_field_info(second)
                .unwrap()
                .storage
                .is_none()
        );
    }

    #[test]
    fn first_environment_ancestor_is_checked_at_adoption_and_each_access() {
        let (mut vm, root, parent, spec) = fixture();
        let mut state = vm.attached_event_state(parent).unwrap();
        state.target_flags |= 0x400;
        vm.update_attached_event_state(parent, state).unwrap();
        assert!(vm.adopt_attached_text_field(spec).is_err());
        let id = vm
            .adopt_attached_text_field(AttachedTextFieldSpec {
                environment_owner: parent,
                ..spec
            })
            .unwrap();
        state.target_flags &= !0x400;
        vm.update_attached_event_state(parent, state).unwrap();
        let mut host = Backend {
            read: Some(Value::Bool(false)),
            ..Backend::default()
        };
        assert!(
            vm.attached_text_field_read(id, &text("html"), &mut host)
                .is_err()
        );
        assert!(host.gets.is_empty());
        assert_eq!(vm.host_calls, 0);
        assert_eq!(
            vm.attached_text_field_info(id).unwrap().environment_owner,
            parent
        );
        assert_ne!(parent, root);
    }

    #[test]
    fn local_definition_version_stays_separate_from_marked_environment() {
        use super::super::AttachedMovieBinding;
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        let root = clip(&mut vm, 100, None, true);
        let parent = vm
            .create_attached_movie_clip_with_binding(
                AttachedClipSpec {
                    target: DisplayTargetId(101),
                    source: AttachedClipSource {
                        movie: LoadedMovieSourceId(11),
                        sprite_id: 1039,
                        header_version: 11,
                    },
                    parent: Some(root),
                    event_state: AttachedEventState {
                        target_flags: 0x5200_0180,
                        load_flags: 0,
                        source_flags: 0,
                    },
                },
                AttachedMovieBinding::LocalSourceOrImportedLocalExport,
            )
            .unwrap();
        let spec = AttachedTextFieldSpec {
            target: DisplayTargetId(200),
            source: AttachedTextFieldSource {
                movie: LoadedMovieSourceId(11),
                edit_text_id: 1034,
                header_version: 11,
            },
            parent,
            environment_owner: root,
            target_flags_6a: 0x5200_0180,
            prototype: TextFieldPrototypeId(1),
            watch_state: TextFieldWatchState::Absent,
        };
        assert!(vm.adopt_attached_text_field(spec).is_err());
        assert!(
            vm.adopt_attached_text_field(AttachedTextFieldSpec {
                source: AttachedTextFieldSource {
                    header_version: 15,
                    ..spec.source
                },
                ..spec
            })
            .is_err()
        );
        for source in [
            AttachedTextFieldSource {
                header_version: 15,
                ..spec.source
            },
            AttachedTextFieldSource {
                movie: LoadedMovieSourceId(12),
                ..spec.source
            },
        ] {
            assert!(
                vm.adopt_attached_text_field_from_parent_movie(AttachedTextFieldSpec {
                    source,
                    ..spec
                })
                .is_err()
            );
        }
        let field = vm
            .adopt_attached_text_field_from_parent_movie(spec)
            .unwrap();
        let owner = vm.attached_text_field_owner(field).unwrap();
        assert!(vm.bind_loaded_movie_target(owner, 15).is_err());
        vm.current_target = Some(vm.attached_owner(parent).unwrap());
        let mut host = Backend::default();
        vm.attached_text_field_write(field, text("html"), Value::Bool(true), &mut host)
            .unwrap();
        let request = &host.sets[0].0;
        assert_eq!(request.source.header_version, 11);
        assert_eq!(request.source.movie, LoadedMovieSourceId(11));
        assert_eq!(request.environment_owner, vm.attached_owner(root).unwrap());
        assert_eq!(request.context, Some(vm.attached_owner(parent).unwrap()));
        assert_eq!(
            vm.attached_info(root).unwrap().environment_header_version,
            15
        );
        assert_eq!(
            vm.attached_info(parent).unwrap().environment_header_version,
            11
        );
        let mut state = vm.attached_event_state(parent).unwrap();
        state.target_flags |= 0x400;
        vm.update_attached_event_state(parent, state).unwrap();
        assert!(
            vm.attached_text_field_write(field, text("html"), Value::Bool(false), &mut host)
                .is_err()
        );
        let second = vm
            .adopt_attached_text_field_from_parent_movie(AttachedTextFieldSpec {
                target: DisplayTargetId(201),
                environment_owner: parent,
                ..spec
            })
            .unwrap();
        vm.current_target = Some(vm.attached_owner(root).unwrap());
        vm.attached_text_field_write(second, text("html"), Value::Bool(false), &mut host)
            .unwrap();
        assert_eq!(
            host.sets[1].0.environment_owner,
            vm.attached_owner(parent).unwrap()
        );
        assert_eq!(
            host.sets[1].0.context,
            Some(vm.attached_owner(root).unwrap())
        );
    }

    #[test]
    fn adoption_rejects_invalid_identity_and_frozen_parent_without_publication() {
        let (mut vm, _, parent, spec) = fixture();
        let count = vm.objects.len();
        for changed in [
            AttachedTextFieldSpec {
                target: DisplayTargetId(100),
                ..spec
            },
            AttachedTextFieldSpec {
                prototype: TextFieldPrototypeId(0),
                ..spec
            },
            AttachedTextFieldSpec {
                target_flags_6a: 0x5100_0180,
                ..spec
            },
            AttachedTextFieldSpec {
                source: AttachedTextFieldSource {
                    header_version: 10,
                    ..spec.source
                },
                ..spec
            },
            AttachedTextFieldSpec {
                environment_owner: parent,
                ..spec
            },
        ] {
            assert!(vm.adopt_attached_text_field(changed).is_err());
        }
        vm.limits.prototype_depth = 1;
        assert!(vm.adopt_attached_text_field(spec).is_err());
        vm.limits.prototype_depth = 256;
        vm.set_attached_children(parent, vec![]).unwrap();
        assert!(vm.adopt_attached_text_field(spec).is_err());
        assert_eq!(vm.objects.len(), count);
        assert!(vm.text_fields.is_empty());
        assert!(!vm.attached_targets.contains_key(&spec.target));
    }

    #[test]
    fn setter_admission_and_backend_failure_have_distinct_storage_effects() {
        let (mut vm, _, _, spec) = fixture();
        let id = vm.adopt_attached_text_field(spec).unwrap();
        let mut host = Backend {
            fail_write: true,
            ..Backend::default()
        };
        for (key, value) in [
            ("html", Value::Number(1.0)),
            ("text", Value::Bool(true)),
            ("text", Value::String(vec![0xd800].into())),
            ("htmlText", Value::String(vec![0].into())),
            ("Text", Value::string("x")),
        ] {
            assert!(
                vm.attached_text_field_write(id, text(key), value, &mut host)
                    .is_err()
            );
        }
        assert!(vm.attached_text_field_info(id).unwrap().storage.is_none());
        vm.limits.host_calls = 0;
        assert!(
            vm.attached_text_field_write(id, text("html"), Value::Bool(true), &mut host)
                .is_err()
        );
        assert!(vm.attached_text_field_info(id).unwrap().storage.is_none());
        vm.limits.host_calls = 100;
        vm.limits.objects = vm.objects.len();
        assert!(
            vm.attached_text_field_write(id, text("html"), Value::Bool(true), &mut host)
                .is_err()
        );
        vm.limits.objects += 1;
        assert!(host.sets.is_empty());
        assert_eq!(vm.host_calls, 0);
        assert!(matches!(
            vm.attached_text_field_write(id, text("html"), Value::Bool(true), &mut host)
                .unwrap_err()
                .kind,
            ErrorKind::Host(_)
        ));
        let storage = vm.attached_text_field_info(id).unwrap().storage.unwrap();
        assert_eq!(vm.host_calls, 1);
        host.fail_write = false;
        vm.attached_text_field_write(id, text("html"), Value::Bool(false), &mut host)
            .unwrap();
        assert_eq!(
            vm.attached_text_field_info(id).unwrap().storage,
            Some(storage)
        );
        assert_eq!(host.sets.len(), 2);
    }

    #[test]
    fn watches_and_missing_context_reject_without_dispatch_or_storage() {
        let (mut vm, _, _, spec) = fixture();
        let mut host = Backend::default();
        for (offset, watch_state) in [TextFieldWatchState::Present, TextFieldWatchState::Unknown]
            .into_iter()
            .enumerate()
        {
            let id = vm
                .adopt_attached_text_field(AttachedTextFieldSpec {
                    target: DisplayTargetId(200 + offset as u64),
                    watch_state,
                    ..spec
                })
                .unwrap();
            assert!(
                vm.attached_text_field_write(id, text("html"), Value::Bool(true), &mut host)
                    .is_err()
            );
            assert!(vm.attached_text_field_info(id).unwrap().storage.is_none());
        }
        let id = vm
            .adopt_attached_text_field(AttachedTextFieldSpec {
                target: DisplayTargetId(202),
                ..spec
            })
            .unwrap();
        vm.current_target = None;
        assert!(
            vm.attached_text_field_read(id, &text("html"), &mut host)
                .is_err()
        );
        assert!(
            vm.attached_text_field_write(id, text("html"), Value::Bool(true), &mut host)
                .is_err()
        );
        assert!(host.sets.is_empty());
        assert!(host.gets.is_empty());
        assert_eq!(vm.host_calls, 0);
    }

    #[test]
    fn getter_validates_return_and_charges_exact_utf16_without_creating_storage() {
        let (mut vm, _, _, spec) = fixture();
        let id = vm.adopt_attached_text_field(spec).unwrap();
        let mut host = Backend {
            read: Some(Value::string("wrong")),
            ..Backend::default()
        };
        let before = vm.string_units;
        assert!(
            vm.attached_text_field_read(id, &text("html"), &mut host)
                .is_err()
        );
        host.read = Some(Value::Bool(true));
        assert!(
            vm.attached_text_field_read(id, &text("text"), &mut host)
                .is_err()
        );
        host.read = Some(Value::String(vec![0xdc00].into()));
        assert!(
            vm.attached_text_field_read(id, &text("text"), &mut host)
                .is_err()
        );
        host.read = Some(Value::string("é😀"));
        vm.limits.string_units = before + 2;
        assert!(
            vm.attached_text_field_read(id, &text("text"), &mut host)
                .is_err()
        );
        assert_eq!(vm.string_units, before);
        assert_eq!(vm.host_calls, 4);
        vm.limits.string_units = before + 3;
        assert_eq!(
            vm.attached_text_field_read(id, &text("text"), &mut host)
                .unwrap(),
            Value::string("é😀")
        );
        assert_eq!(vm.string_units, before + 3);
        assert!(vm.attached_text_field_info(id).unwrap().storage.is_none());
        assert_eq!(vm.host_calls, 5);
        vm.limits.host_calls = 5;
        assert!(
            vm.attached_text_field_read(id, &text("html"), &mut host)
                .is_err()
        );
        assert_eq!(host.gets.len(), 5);
    }
}
