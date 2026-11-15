//! Explicit live source MovieClip owners. No Scene IDs or implicit attachment.
//! Native evidence and profile limits are recorded in ATTACHED_MOVIECLIP.md.
use super::{
    AttachedEventState, ErrorKind, Kind, ObjectId, Property, Result, Text, Value, Vm, text,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AttachedClipId(pub usize);
/// Host identity independent of a Scene node index or VM object identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayTargetId(pub u64);
/// Host identity for a successfully loaded source movie, not a native pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LoadedMovieSourceId(pub u64);
/// Diagnostic only. Bound storage must not be exposed as an ordinary Value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedStorageId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedClipSource {
    pub movie: LoadedMovieSourceId,
    pub sprite_id: u16,
    pub header_version: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedClipSpec {
    pub target: DisplayTargetId,
    pub source: AttachedClipSource,
    /// A complete live case-3 parent chain. Reparenting/retirement are unsupported.
    pub parent: Option<AttachedClipId>,
    /// Actual caller-owned state at this checkpoint; never guessed from defaults.
    pub event_state: AttachedEventState,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachedChild {
    Clip {
        name: Text,
        clip: AttachedClipId,
    },
    /// Retains a known named child whose receiver category is not implemented.
    /// A matching name rejects rather than falling through to the prototype.
    Unsupported {
        name: Text,
        target: DisplayTargetId,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum AttachedMember {
    Missing,
    Present(Value),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachedWrite {
    Created,
    Updated,
    ReadOnly,
}
#[derive(Debug, Clone)]
pub struct AttachedInfo {
    pub target: DisplayTargetId,
    pub source: AttachedClipSource,
    pub parent: Option<AttachedClipId>,
    pub owner: ObjectId,
    pub storage: Option<AttachedStorageId>,
    pub prototype: Option<ObjectId>,
    pub child_snapshot_installed: bool,
    pub mouse_event_marker: bool,
    pub event_state: AttachedEventState,
}
#[derive(Debug, Clone)]
enum ChildTarget {
    Clip(AttachedClipId),
    Unsupported(DisplayTargetId),
}
enum PrototypeMember {
    Missing,
    Data(Value),
    Accessor,
}
#[derive(Debug, Clone)]
pub(super) struct AttachedClip {
    spec: AttachedClipSpec,
    owner: ObjectId,
    storage: Option<ObjectId>,
    prototype: Option<ObjectId>,
    children: Option<BTreeMap<Text, ChildTarget>>,
    mouse_event_marker: bool,
}

impl Vm {
    pub(super) fn check_attached(&self, id: AttachedClipId) -> Result<()> {
        if id.0 < self.attached.len() {
            Ok(())
        } else {
            self.fail(ErrorKind::InvalidAttachedClip(id))
        }
    }
    /// The host must establish the live source case-3 construction route and
    /// loaded movie identity. This never upgrades a preexisting VM object.
    pub fn create_attached_movie_clip(&mut self, spec: AttachedClipSpec) -> Result<AttachedClipId> {
        // DWORD+6a overlaps byte+6d: this concrete case-3 owner lives at+148,
        // so its encoded quarter-byte owner offset is52, not an event default.
        if spec.event_state.target_flags >> 24 != 0x52 {
            return self.fail(ErrorKind::Unsupported("attached case3 owner-offset field"));
        }
        if spec.source.header_version != self.movie_version {
            return self.fail(ErrorKind::Unsupported(
                "attached loaded movie version profile",
            ));
        }
        if self.attached_targets.contains_key(&spec.target) {
            return self.fail(ErrorKind::Unsupported("duplicate attached target identity"));
        }
        let mut parent = spec.parent;
        if let Some(parent) = parent {
            self.check_attached(parent)?;
            if self.attached[parent.0].children.is_some() {
                return self.fail(ErrorKind::Unsupported(
                    "attached child creation after frozen snapshot",
                ));
            }
        }
        let mut depth = 0;
        while let Some(id) = parent {
            self.check_attached(id)?;
            if depth >= self.limits.prototype_depth {
                return self.fail(ErrorKind::Budget("attached parent depth"));
            }
            parent = self.attached[id.0].spec.parent;
            depth += 1;
        }
        let (Some(constructor), Some(prototype)) =
            (self.movie_clip_constructor, self.movie_clip_prototype)
        else {
            return self.fail(ErrorKind::Unsupported(
                "attached source requires native MovieClip",
            ));
        };
        for (owner, key, expected) in [
            (self.global, "MovieClip", constructor),
            (constructor, "prototype", prototype),
        ] {
            if !matches!(self.objects[owner.0].properties.get(&text(key)),
                Some(Property::Data(Value::Object(actual))) if *actual == expected)
            {
                return self.fail(ErrorKind::Unsupported(
                    "attached modified MovieClip binding/prototype",
                ));
            }
        }
        let id = AttachedClipId(self.attached.len());
        let owner = self.alloc(Kind::AttachedOwner(id), None)?;
        self.attached.push(AttachedClip {
            spec,
            owner,
            storage: None,
            prototype: Some(prototype),
            children: None,
            mouse_event_marker: false,
        });
        self.attached_targets.insert(spec.target, Some(id));
        self.loaded_movie_targets
            .insert(owner, spec.source.header_version);
        Ok(id)
    }
    /// Adopt only before registering any script or constructing a host tied to
    /// the old root. Existing independent Plain-object diagnostics are unaffected.
    pub fn adopt_attached_root(&mut self, id: AttachedClipId) -> Result<()> {
        self.check_attached(id)?;
        let previous = &self.objects[self.root.0];
        if !self.code.is_empty()
            || self.attached[id.0].spec.parent.is_some()
            || !matches!(previous.kind, Kind::Plain)
            || previous.prototype.is_some()
            || previous.properties.len() != 1
            || !matches!(previous.properties.get(&text("_root")),
                Some(Property::Data(Value::Object(root))) if *root == self.root)
            || self
                .objects
                .iter()
                .any(|object| matches!(object.kind, Kind::Script(_)))
        {
            return self.fail(ErrorKind::Unsupported(
                "attached root adoption after initialization",
            ));
        }
        self.root = self.attached[id.0].owner;
        Ok(())
    }
    pub fn attached_owner(&self, id: AttachedClipId) -> Result<ObjectId> {
        self.check_attached(id)?;
        Ok(self.attached[id.0].owner)
    }
    pub fn attached_value(&self, id: AttachedClipId) -> Result<Value> {
        self.check_attached(id)?;
        Ok(Value::DisplayClip(id))
    }
    pub fn attached_info(&self, id: AttachedClipId) -> Result<AttachedInfo> {
        self.check_attached(id)?;
        let clip = &self.attached[id.0];
        Ok(AttachedInfo {
            target: clip.spec.target,
            source: clip.spec.source,
            parent: clip.spec.parent,
            owner: clip.owner,
            storage: clip.storage.map(|id| AttachedStorageId(id.0)),
            prototype: clip.prototype,
            child_snapshot_installed: clip.children.is_some(),
            mouse_event_marker: clip.mouse_event_marker,
            event_state: clip.spec.event_state,
        })
    }
    pub fn attached_event_state(&self, id: AttachedClipId) -> Result<AttachedEventState> {
        self.check_attached(id)?;
        Ok(self.attached[id.0].spec.event_state)
    }
    pub fn update_attached_event_state(
        &mut self,
        id: AttachedClipId,
        state: AttachedEventState,
    ) -> Result<()> {
        self.check_attached(id)?;
        let old = self.attached[id.0].spec.event_state;
        if state.target_flags >> 24 != 0x52 {
            return self.fail(ErrorKind::Unsupported("attached case3 owner-offset field"));
        }
        // The native child cache can bypass the list eligibility recheck.
        // Without cache invalidation/lifecycle evidence these bits stay fixed.
        if (state.target_flags ^ old.target_flags) & 0x180 != 0 {
            return self.fail(ErrorKind::Unsupported(
                "attached child eligibility/cache transition",
            ));
        }
        self.attached[id.0].spec.event_state = state;
        Ok(())
    }
    /// Install one complete, unique named-child snapshot. Unnamed entries cannot
    /// match an admitted nonempty name. Rename/removal/cache changes are not modeled.
    pub fn set_attached_children(
        &mut self,
        id: AttachedClipId,
        children: Vec<AttachedChild>,
    ) -> Result<()> {
        self.check_attached(id)?;
        if self.attached[id.0].children.is_some() {
            return self.fail(ErrorKind::Unsupported(
                "attached child snapshot replacement",
            ));
        }
        if children.len()
            > self
                .limits
                .properties
                .saturating_sub(self.attached_child_entries)
        {
            return self.fail(ErrorKind::Budget("attached child entries"));
        }
        let mut names = BTreeMap::new();
        let mut targets = BTreeSet::new();
        let mut unsupported_targets = BTreeSet::new();
        let mut units = 0usize;
        for entry in children {
            let (name, target, value) = match entry {
                AttachedChild::Clip { name, clip } => {
                    self.check_attached(clip)?;
                    if self.attached[clip.0].spec.parent != Some(id) {
                        return self.fail(ErrorKind::Unsupported("attached child parent mismatch"));
                    }
                    (
                        name,
                        self.attached[clip.0].spec.target,
                        ChildTarget::Clip(clip),
                    )
                }
                AttachedChild::Unsupported { name, target } => {
                    if self.attached_targets.contains_key(&target) {
                        return self.fail(ErrorKind::Unsupported(
                            "attached child target identity already bound",
                        ));
                    }
                    unsupported_targets.insert(target);
                    (name, target, ChildTarget::Unsupported(target))
                }
            };
            units = units.saturating_add(name.len());
            if units > self.limits.string_units.saturating_sub(self.string_units) {
                return self.fail(ErrorKind::Budget("attached child name units"));
            }
            if name.is_empty()
                || name.contains(&0)
                || char::decode_utf16(name.iter().copied()).any(|unit| unit.is_err())
                || names.contains_key(&name)
                || !targets.insert(target)
            {
                return self.fail(ErrorKind::Unsupported(
                    "attached unique live child-name profile",
                ));
            }
            names.insert(name, value);
        }
        self.string_units += units;
        self.attached_child_entries += names.len();
        for target in unsupported_targets {
            self.attached_targets.insert(target, None);
        }
        self.attached[id.0].children = Some(names);
        Ok(())
    }
    fn ensure_attached_storage(&mut self, id: AttachedClipId) -> Result<ObjectId> {
        self.check_attached(id)?;
        if let Some(storage) = self.attached[id.0].storage {
            if !matches!(self.objects[storage.0].kind, Kind::BoundStorage(owner) if owner == id) {
                return self.fail(ErrorKind::Unsupported("attached storage owner mismatch"));
            }
            return Ok(storage);
        }
        let prototype = self.attached[id.0].prototype;
        let marker = match prototype {
            Some(prototype) => self.movie_clip_prototype_event_marker(prototype)?,
            None => false,
        };
        let storage = self.alloc(Kind::BoundStorage(id), prototype)?;
        self.attached[id.0].storage = Some(storage);
        self.attached[id.0].mouse_event_marker |= marker;
        Ok(storage)
    }
    /// The native owner creates storage first, forwards the parent/hook there,
    /// then changes its display prototype. It never retags the owner or storage.
    pub fn set_attached_prototype(
        &mut self,
        id: AttachedClipId,
        prototype: Option<ObjectId>,
    ) -> Result<()> {
        self.check_attached(id)?;
        let marker = if let Some(prototype) = prototype {
            self.check_object(prototype)?;
            self.movie_clip_prototype_event_marker(prototype)?
        } else {
            false
        };
        let storage = self.ensure_attached_storage(id)?;
        if self.objects[storage.0]
            .properties
            .contains_key(&text("__proto__"))
            && self.own_flags(storage, &text("__proto__")) != 3
        {
            return self.fail(ErrorKind::Unsupported(
                "attached storage prototype metadata flags",
            ));
        }
        self.raw_member(
            storage,
            text("__proto__"),
            Property::Data(prototype.map_or(Value::Null, Value::Object)),
            3,
        )?;
        self.objects[storage.0].prototype = prototype;
        self.attached[id.0].mouse_event_marker |= marker;
        self.attached[id.0].prototype = prototype;
        Ok(())
    }
    pub(super) fn attached_member_name(&self, key: &Text) -> Result<()> {
        // Exact selected names are absent from the original standard-member
        // map; both possible native flag29 routes fall through ordinary handling.
        // This is not an underscore heuristic or an arbitrary host-classifier bit.
        if [
            "gfx",
            "Function",
            "Object",
            "MarvelGlobals",
            "flash",
            "String",
            "disabled",
            "enableInitCallback",
            "localTextLabel",
            "button",
            "localizedText_tf",
        ]
        .iter()
        .any(|name| key.as_ref() == text(name).as_ref())
        {
            Ok(())
        } else {
            self.fail(ErrorKind::Unsupported(
                "attached native member-name profile",
            ))
        }
    }
    fn attached_resolver_guard(&self, id: ObjectId) -> Result<()> {
        if self.objects[id.0]
            .insertion_keys
            .iter()
            .any(|name| name.as_ref() == text("__resolve").as_ref())
        {
            return self.fail(ErrorKind::Unsupported("attached resolver mutation history"));
        }
        Ok(())
    }
    fn attached_prototype_member(
        &self,
        mut cursor: Option<ObjectId>,
        key: &Text,
    ) -> Result<PrototypeMember> {
        let mut depth = 0;
        let mut native_clip_seen = false;
        while let Some(id) = cursor {
            self.check_object(id)?;
            if depth >= self.limits.prototype_depth {
                return self.fail(ErrorKind::Budget("attached prototype lookup depth"));
            }
            self.attached_resolver_guard(id)?;
            let object = &self.objects[id.0];
            match object.kind {
                Kind::Plain => {}
                Kind::MovieClip(super::MovieClipRole::UnboundPrototype)
                    if Some(id) == self.movie_clip_prototype =>
                {
                    native_clip_seen = true;
                    let (Some(parent), Some(constructor), Some(object_constructor)) = (
                        self.object_prototype,
                        self.movie_clip_constructor,
                        self.object_constructor,
                    ) else {
                        return self
                            .fail(ErrorKind::Unsupported("attached builtin prototype profile"));
                    };
                    if object.prototype != Some(parent)
                        || self.objects[parent.0].prototype.is_some()
                        || !matches!(object.properties.get(&text("constructor")), Some(Property::Data(Value::Object(value))) if *value == constructor)
                        || !matches!(self.objects[parent.0].properties.get(&text("constructor")), Some(Property::Data(Value::Object(value))) if *value == object_constructor)
                    {
                        return self.fail(ErrorKind::Unsupported(
                            "attached modified builtin prototype chain",
                        ));
                    }
                }
                _ => return self.fail(ErrorKind::Unsupported("attached prototype object kind")),
            }
            if let Some(property) = object.properties.get(key) {
                return Ok(match property {
                    Property::Data(value) => PrototypeMember::Data(value.clone()),
                    Property::Accessor { .. } => PrototypeMember::Accessor,
                });
            }
            cursor = object.prototype;
            depth += 1;
        }
        if native_clip_seen
            && ![
                "Function",
                "gfx",
                "onLoad",
                "Object",
                "MarvelGlobals",
                "flash",
                "String",
            ]
            .iter()
            .any(|name| key.as_ref() == text(name).as_ref())
        {
            return self.fail(ErrorKind::Unsupported(
                "attached missing builtin prototype member",
            ));
        }
        Ok(PrototypeMember::Missing)
    }
    /// The caller supplies actual saved load-event presence from its owned event
    /// map. Raw property presence includes noncallable/undefined and accessors,
    /// never checks children, and never invokes a getter.
    pub fn attached_load_present(
        &self,
        id: AttachedClipId,
        saved_load_event: bool,
    ) -> Result<bool> {
        self.check_attached(id)?;
        if saved_load_event {
            return Ok(true);
        }
        let clip = &self.attached[id.0];
        let key = text("onLoad");
        if let Some(storage) = clip.storage {
            self.attached_resolver_guard(storage)?;
            if self.objects[storage.0].properties.contains_key(&key) {
                return Ok(true);
            }
        }
        Ok(!matches!(
            self.attached_prototype_member(clip.prototype, &key)?,
            PrototypeMember::Missing
        ))
    }
    pub fn read_attached_member(&self, id: AttachedClipId, key: &str) -> Result<AttachedMember> {
        self.attached_lookup(id, &text(key))
    }
    pub(super) fn attached_lookup(&self, id: AttachedClipId, key: &Text) -> Result<AttachedMember> {
        self.check_attached(id)?;
        self.attached_member_name(key)?;
        let clip = &self.attached[id.0];
        if let Some(storage) = clip.storage {
            self.attached_resolver_guard(storage)?;
            if let Some(member) = self.objects[storage.0].properties.get(key) {
                return match member {
                    Property::Data(value) => Ok(AttachedMember::Present(value.clone())),
                    Property::Accessor { .. } => {
                        self.fail(ErrorKind::Unsupported("attached own accessor read"))
                    }
                };
            }
        }
        let Some(children) = &clip.children else {
            return self.fail(ErrorKind::Unsupported(
                "attached complete child snapshot required",
            ));
        };
        match children.get(key) {
            Some(ChildTarget::Clip(child)) => {
                let flags = self.attached[child.0].spec.event_state.target_flags;
                if flags & 0x180 == 0x180 {
                    return Ok(AttachedMember::Present(Value::DisplayClip(*child)));
                }
            }
            Some(ChildTarget::Unsupported(_target)) => {
                return self.fail(ErrorKind::Unsupported(
                    "attached named child receiver category",
                ));
            }
            None => {}
        }
        match self.attached_prototype_member(clip.prototype, key)? {
            PrototypeMember::Missing => Ok(AttachedMember::Missing),
            PrototypeMember::Data(value) => Ok(AttachedMember::Present(value)),
            PrototypeMember::Accessor => {
                self.fail(ErrorKind::Unsupported("attached inherited accessor read"))
            }
        }
    }
    pub fn write_attached_member(
        &mut self,
        id: AttachedClipId,
        key: &str,
        value: Value,
    ) -> Result<AttachedWrite> {
        self.attached_write(id, text(key), value)
    }
    pub(super) fn attached_write(
        &mut self,
        id: AttachedClipId,
        key: Text,
        value: Value,
    ) -> Result<AttachedWrite> {
        self.check_attached(id)?;
        self.attached_member_name(&key)?;
        self.check_value(&value)?;
        let storage = self.ensure_attached_storage(id)?;
        self.attached_resolver_guard(storage)?;
        let own = self.objects[storage.0].properties.get(&key);
        if matches!(own, Some(Property::Accessor { .. })) {
            return self.fail(ErrorKind::Unsupported("attached own accessor assignment"));
        }
        let existed = own.is_some();
        if existed && self.own_flags(storage, &key) & 4 != 0 {
            return Ok(AttachedWrite::ReadOnly);
        }
        if !existed {
            let mut parent = self.objects[storage.0].prototype;
            let mut depth = 0;
            while let Some(id) = parent {
                if depth >= self.limits.prototype_depth {
                    return self.fail(ErrorKind::Budget("attached setter prototype depth"));
                }
                self.check_member_target(id, &key)?;
                self.attached_resolver_guard(id)?;
                if matches!(
                    self.objects[id.0].properties.get(&key),
                    Some(Property::Accessor { .. })
                ) {
                    return self.fail(ErrorKind::Unsupported(
                        "attached inherited accessor assignment",
                    ));
                }
                // Native normal setter skips inherited data, even read-only data,
                // while searching for a deeper inherited accessor.
                parent = self.objects[id.0].prototype;
                depth += 1;
            }
        }
        let flags = if existed {
            self.own_flags(storage, &key)
        } else {
            0
        };
        self.raw_member(storage, key, Property::Data(value), flags)?;
        Ok(if existed {
            AttachedWrite::Updated
        } else {
            AttachedWrite::Created
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::Program;
    use super::super::{Limits, RejectHost};
    use super::*;
    use std::sync::Arc;

    fn vm() -> Vm {
        let mut vm = Vm::new(15, Limits::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        vm
    }
    fn spec(target: u64, parent: Option<AttachedClipId>) -> AttachedClipSpec {
        // Explicit controlled state fixture, not a native post-callback default.
        AttachedClipSpec {
            target: DisplayTargetId(target),
            source: AttachedClipSource {
                movie: LoadedMovieSourceId(9),
                sprite_id: 27,
                header_version: 15,
            },
            parent,
            event_state: AttachedEventState {
                target_flags: 0x5200_0180,
                load_flags: 0,
                source_flags: 0,
            },
        }
    }
    fn read_variable(vm: &mut Vm, name: &str) -> Value {
        let mut bytes = vec![0x96];
        bytes.extend_from_slice(&((name.len() + 2) as u16).to_le_bytes());
        bytes.push(0);
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend([0, 0x1c, 0x3e]);
        let code = vm
            .register(Arc::new(Program::parse(&bytes).unwrap()))
            .unwrap();
        vm.execute(code, vm.root(), &mut RejectHost).unwrap().value
    }
    #[test]
    fn identity_root_adoption_and_storage_are_explicit() {
        let mut vm = vm();
        let previous = vm.root();
        let mut invalid = spec(99, None);
        invalid.event_state.target_flags &= 0x00ff_ffff;
        assert!(vm.create_attached_movie_clip(invalid).is_err());
        let clip = vm.create_attached_movie_clip(spec(100, None)).unwrap();
        let owner = vm.attached_owner(clip).unwrap();
        assert_ne!(owner, previous);
        assert_eq!(vm.attached_value(clip).unwrap(), Value::DisplayClip(clip));
        assert!(vm.attached_info(clip).unwrap().storage.is_none());
        assert!(vm.object_id(Value::DisplayClip(clip)).is_err());
        assert!(vm.check_value(&Value::Object(owner)).is_err());
        assert!(vm.create_attached_movie_clip(spec(100, None)).is_err());
        vm.adopt_attached_root(clip).unwrap();
        assert_eq!(read_variable(&mut vm, "this"), Value::DisplayClip(clip));
        assert_eq!(read_variable(&mut vm, "_root"), Value::DisplayClip(clip));
        assert!(vm.adopt_attached_root(clip).is_err());
        let prototype = vm.object().unwrap();
        vm.set(prototype, "onPress", Value::Bool(false)).unwrap();
        vm.set_attached_prototype(clip, Some(prototype)).unwrap();
        let info = vm.attached_info(clip).unwrap();
        let storage = ObjectId(info.storage.unwrap().0);
        assert_ne!(owner, storage);
        assert_eq!(vm.own_flags(storage, &text("__proto__")), 3);
        assert!(
            !vm.objects[storage.0]
                .properties
                .contains_key(&text("__constructor__"))
        );
        assert!(vm.check_value(&Value::Object(storage)).is_err());
        assert!(info.mouse_event_marker);
        vm.set_attached_prototype(clip, None).unwrap();
        assert!(vm.attached_info(clip).unwrap().mouse_event_marker);
    }
    #[test]
    fn own_undefined_then_children_then_prototype_and_data_shadowing() {
        let mut vm = vm();
        let parent = vm.create_attached_movie_clip(spec(1, None)).unwrap();
        let child = vm
            .create_attached_movie_clip(spec(2, Some(parent)))
            .unwrap();
        let prototype = vm.object().unwrap();
        vm.raw_member(
            prototype,
            text("button"),
            Property::Data(Value::Number(7.0)),
            4,
        )
        .unwrap();
        vm.raw_member(
            prototype,
            text("localTextLabel"),
            Property::Data(Value::string("inherited")),
            4,
        )
        .unwrap();
        vm.set_attached_prototype(parent, Some(prototype)).unwrap();
        assert!(vm.read_attached_member(parent, "button").is_err()); // no complete child snapshot
        vm.set_attached_children(
            parent,
            vec![AttachedChild::Clip {
                name: text("button"),
                clip: child,
            }],
        )
        .unwrap();
        assert_eq!(
            vm.read_attached_member(parent, "button").unwrap(),
            AttachedMember::Present(Value::DisplayClip(child))
        );
        let mut state = vm.attached_event_state(child).unwrap();
        state.target_flags &= !0x80;
        assert!(vm.update_attached_event_state(child, state).is_err());
        let inactive_parent = vm.create_attached_movie_clip(spec(3, None)).unwrap();
        let mut inactive_spec = spec(4, Some(inactive_parent));
        inactive_spec.event_state.target_flags &= !0x80;
        let inactive = vm.create_attached_movie_clip(inactive_spec).unwrap();
        vm.set_attached_prototype(inactive_parent, Some(prototype))
            .unwrap();
        vm.set_attached_children(
            inactive_parent,
            vec![AttachedChild::Clip {
                name: text("button"),
                clip: inactive,
            }],
        )
        .unwrap();
        assert_eq!(
            vm.read_attached_member(inactive_parent, "button").unwrap(),
            AttachedMember::Present(Value::Number(7.0))
        );
        assert_eq!(
            vm.write_attached_member(parent, "button", Value::Undefined)
                .unwrap(),
            AttachedWrite::Created
        );
        assert_eq!(
            vm.read_attached_member(parent, "button").unwrap(),
            AttachedMember::Present(Value::Undefined)
        );
        assert!(
            vm.has_key(vm.attached_owner(parent).unwrap(), &text("button"))
                .unwrap()
        );
        assert_eq!(
            vm.write_attached_member(parent, "localTextLabel", Value::string("own"))
                .unwrap(),
            AttachedWrite::Created
        );
        assert_eq!(
            vm.get(prototype, "localTextLabel").unwrap(),
            Value::string("inherited")
        );
        let storage = ObjectId(vm.attached_info(parent).unwrap().storage.unwrap().0);
        assert_eq!(vm.own_flags(storage, &text("localTextLabel")), 0);
        vm.raw_member(
            storage,
            text("button"),
            Property::Data(Value::Number(9.0)),
            4,
        )
        .unwrap();
        assert_eq!(
            vm.write_attached_member(parent, "button", Value::Number(10.0))
                .unwrap(),
            AttachedWrite::ReadOnly
        );
        assert_eq!(
            vm.read_attached_member(parent, "button").unwrap(),
            AttachedMember::Present(Value::Number(9.0))
        );
        assert!(vm.set_attached_children(parent, vec![]).is_err());
    }
    #[test]
    fn unsupported_children_and_deeper_accessors_never_disappear() {
        let mut vm = vm();
        let parent = vm.create_attached_movie_clip(spec(1, None)).unwrap();
        let prototype = vm.object().unwrap();
        vm.set(prototype, "localizedText_tf", Value::Number(3.0))
            .unwrap();
        vm.set_attached_prototype(parent, Some(prototype)).unwrap();
        vm.set_attached_children(
            parent,
            vec![AttachedChild::Unsupported {
                name: text("localizedText_tf"),
                target: DisplayTargetId(2),
            }],
        )
        .unwrap();
        assert!(matches!(
            vm.read_attached_member(parent, "localizedText_tf")
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported("attached named child receiver category")
        ));
        let deeper = vm.object().unwrap();
        let callback = vm.native(999).unwrap();
        vm.raw_member(
            deeper,
            text("button"),
            Property::Accessor {
                getter: Value::Object(callback),
                setter: Some(Value::Object(callback)),
            },
            0,
        )
        .unwrap();
        vm.set(prototype, "__proto__", Value::Object(deeper))
            .unwrap();
        vm.raw_member(
            prototype,
            text("button"),
            Property::Data(Value::Number(1.0)),
            4,
        )
        .unwrap();
        assert!(matches!(
            vm.write_attached_member(parent, "button", Value::Bool(true))
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported("attached inherited accessor assignment")
        ));
        assert_eq!(vm.host_calls, 0);
        for name in [
            "enabled",
            "tabEnabled",
            "scale9Grid",
            "rendererString",
            "onPress",
            "_x",
        ] {
            assert!(vm.read_attached_member(parent, name).is_err());
            assert!(vm.write_attached_member(parent, name, Value::Null).is_err());
        }
    }
    #[test]
    fn attached_missing_and_found_undefined_preserve_global_lookup() {
        let mut vm = vm();
        let function = vm.install_function_conversion().unwrap();
        let clip = vm.create_attached_movie_clip(spec(1, None)).unwrap();
        vm.set_attached_children(clip, vec![]).unwrap();
        assert_eq!(
            vm.read_attached_member(clip, "Function").unwrap(),
            AttachedMember::Missing
        );
        vm.adopt_attached_root(clip).unwrap();
        assert_eq!(read_variable(&mut vm, "Function"), Value::Object(function));
        vm.write_attached_member(clip, "Function", Value::Undefined)
            .unwrap();
        assert_eq!(read_variable(&mut vm, "Function"), Value::Undefined);
        assert!(!vm.attached_load_present(clip, false).unwrap());
        assert!(vm.attached_load_present(clip, true).unwrap());
        let prototype = vm.object().unwrap();
        vm.set(prototype, "onLoad", Value::Undefined).unwrap();
        vm.set_attached_prototype(clip, Some(prototype)).unwrap();
        assert!(vm.attached_load_present(clip, false).unwrap());
        vm.set(prototype, "__resolve", Value::Null).unwrap();
        assert!(vm.read_attached_member(clip, "gfx").is_err());
        vm.objects[prototype.0]
            .properties
            .remove(&text("__resolve"));
        assert!(vm.read_attached_member(clip, "gfx").is_err()); // mutation history stays gated
    }
    #[test]
    fn child_snapshots_validate_parent_identity_duplicates_and_budgets() {
        let mut vm = vm();
        let parent = vm.create_attached_movie_clip(spec(1, None)).unwrap();
        let other = vm.create_attached_movie_clip(spec(2, None)).unwrap();
        assert!(
            vm.set_attached_children(
                parent,
                vec![AttachedChild::Clip {
                    name: text("button"),
                    clip: other
                }]
            )
            .is_err()
        );
        let child = vm
            .create_attached_movie_clip(spec(3, Some(parent)))
            .unwrap();
        let entry = AttachedChild::Clip {
            name: text("button"),
            clip: child,
        };
        assert!(
            vm.set_attached_children(parent, vec![entry.clone(), entry.clone()])
                .is_err()
        );
        assert!(!vm.attached_info(parent).unwrap().child_snapshot_installed);
        vm.limits.string_units = vm.string_units;
        assert!(matches!(
            vm.set_attached_children(parent, vec![entry])
                .unwrap_err()
                .kind,
            ErrorKind::Budget(_)
        ));
        assert!(!vm.attached_info(parent).unwrap().child_snapshot_installed);
        let constructor = vm.movie_clip_constructor.unwrap();
        vm.set(constructor, "prototype", Value::Null).unwrap();
        assert!(vm.create_attached_movie_clip(spec(4, None)).is_err());
    }
}
