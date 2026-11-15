//! Explicit live source MovieClip owners. No Scene IDs or implicit attachment.
//! Native evidence and profile limits are recorded in ATTACHED_MOVIECLIP.md.
use super::{
    AttachedEventState, AttachedFocusRectState, AttachedNameClass, AttachedTextFieldId,
    BaselineMembership, BaselineOwners, CodeOrigin, ErrorKind, Kind, NativeNameProfile, ObjectId,
    Property, Result, Text, Value, Vm, text,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

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
/// Host-established native resource binding route. The host must validate the
/// actual local resource or ordinary imported-local-export pair; source filename,
/// bytecode origin and the placing parent's version are not this witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachedMovieBinding {
    LocalSourceOrImportedLocalExport,
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
    TextField {
        name: Text,
        field: AttachedTextFieldId,
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
    /// An admitted native target field changed; no ordinary member was written.
    NativeUpdated,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedFocusRectInfo {
    pub state: AttachedFocusRectState,
    /// Last executed setter's provenance; None for a snapshot or host write
    /// through the public member API. This is bounded diagnostic metadata.
    pub last_write: Option<CodeOrigin>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachedTabEnabledInfo {
    /// A copy of the same supplied target+b8 word used by focus rectangle state.
    pub target_flags_b8: u32,
    /// Last admitted tabEnabled setter; independent of focus-write provenance.
    pub last_write: Option<CodeOrigin>,
}
/// Caller-established current contents of an existing coherent display handle.
/// This is not a placement/filename-derived default or a request to rename it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachedNameSnapshot {
    pub target: DisplayTargetId,
    pub name: Text,
}
#[derive(Debug, Clone)]
pub struct AttachedInfo {
    pub target: DisplayTargetId,
    pub source: AttachedClipSource,
    /// MovieDef selected by native CreateInfo binding, separate from the
    /// definition's source role even when the admitted witness makes them equal.
    pub environment_movie: LoadedMovieSourceId,
    pub environment_header_version: u8,
    pub binding: Option<AttachedMovieBinding>,
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
    TextField(AttachedTextFieldId),
    Unsupported(DisplayTargetId),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AttachedTarget {
    Clip(AttachedClipId),
    TextField(AttachedTextFieldId),
    Unsupported,
}
enum PrototypeMember {
    Missing,
    Data(Value),
    Accessor,
}
#[derive(Debug, Clone)]
pub(super) struct AttachedClip {
    spec: AttachedClipSpec,
    environment_movie: LoadedMovieSourceId,
    environment_header_version: u8,
    binding: Option<AttachedMovieBinding>,
    focus_rect: Option<AttachedFocusRectInfo>,
    tab_enabled_last_write: Option<CodeOrigin>,
    name_snapshot: Option<Text>,
    owner: ObjectId,
    storage: Option<ObjectId>,
    prototype: Option<ObjectId>,
    children: Option<BTreeMap<Text, ChildTarget>>,
    mouse_event_marker: bool,
}

impl Vm {
    /// Install an authenticated owner-private original native inventory once,
    /// before compiling scripts. This supplies membership facts only; all live
    /// instance, state, lookup and native-callback guards remain independent.
    pub fn install_attached_name_profile(&mut self, profile: Arc<NativeNameProfile>) -> Result<()> {
        if self.attached_name_profile.is_some()
            || !self.code.is_empty()
            || self.current_target.is_some()
        {
            return self.fail(ErrorKind::Unsupported(
                "attached name profile installation phase",
            ));
        }
        self.attached_name_profile = Some(profile);
        Ok(())
    }
    pub(super) fn check_attached(&self, id: AttachedClipId) -> Result<()> {
        if id.0 < self.attached.len() {
            Ok(())
        } else {
            self.fail(ErrorKind::InvalidAttachedClip(id))
        }
    }
    /// Exact ASSetPropFlags own-storage path; this is not a normal member
    /// lookup or an Object coercion. Missing storage would invoke native lazy
    /// construction and stays outside this existing-storage profile.
    pub(super) fn attached_property_flags_storage(
        &self,
        id: AttachedClipId,
        key: &Text,
    ) -> Result<ObjectId> {
        self.check_attached(id)?;
        // These modeled data descriptors stand in for special native metadata;
        // the ordinary stored-value proof does not cover rewriting their flags.
        if ["__proto__", "__constructor__", "constructor"]
            .iter()
            .any(|name| key.as_ref() == text(name).as_ref())
        {
            return self.fail(ErrorKind::Unsupported(
                "attached property flags special metadata",
            ));
        }
        let Some(context) = self.current_target else {
            return self.fail(ErrorKind::Unsupported(
                "attached property flags require active Environment",
            ));
        };
        self.attached_invocation_context(id, context)?;
        if !self
            .loaded_movie_targets
            .get(&context)
            .is_some_and(|v| (7..=19).contains(v))
        {
            return self.fail(ErrorKind::Unsupported(
                "attached property flags require known loaded target version",
            ));
        }
        let Some(storage) = self.attached[id.0].storage else {
            return self.fail(ErrorKind::Unsupported(
                "attached property flags without existing storage",
            ));
        };
        if !matches!(self.objects[storage.0].kind, Kind::BoundStorage(clip) if clip == id)
            || !matches!(
                self.objects[storage.0].properties.get(key),
                Some(Property::Data(_))
            )
        {
            return self.fail(ErrorKind::Unsupported(
                "attached property flags require existing own data member",
            ));
        }
        Ok(storage)
    }
    /// The host must establish the live source case-3 construction route and
    /// loaded movie identity. This never upgrades a preexisting VM object.
    pub fn create_attached_movie_clip(&mut self, spec: AttachedClipSpec) -> Result<AttachedClipId> {
        if spec.source.header_version != self.movie_version
            || spec
                .parent
                .map(|id| self.attached_info(id))
                .transpose()?
                .is_some_and(|parent| parent.environment_header_version != self.movie_version)
        {
            return self.fail(ErrorKind::Unsupported(
                "attached loaded movie version profile",
            ));
        }
        self.create_attached_in_binding(spec, None)
    }
    /// Explicit bounded local/imported-export adoption. The FINAL selected
    /// resource pair must remain the eligible, successfully resolved direct
    /// local/exported pair: later native resource-list substitutions are outside
    /// this witness. Every native target
    /// keeps the MovieDef/version of its selected binding, not the placing
    /// parent or script-function origin. Alias/substitution bindings are absent.
    pub fn create_attached_movie_clip_with_binding(
        &mut self,
        spec: AttachedClipSpec,
        binding: AttachedMovieBinding,
    ) -> Result<AttachedClipId> {
        if !(7..=19).contains(&spec.source.header_version) {
            return self.fail(ErrorKind::Unsupported(
                "attached binding execution version outside 7..19",
            ));
        }
        self.create_attached_in_binding(spec, Some(binding))
    }
    fn create_attached_in_binding(
        &mut self,
        spec: AttachedClipSpec,
        binding: Option<AttachedMovieBinding>,
    ) -> Result<AttachedClipId> {
        // DWORD+6a overlaps byte+6d: this concrete case-3 owner lives at+148,
        // so its encoded quarter-byte owner offset is52, not an event default.
        if spec.event_state.target_flags >> 24 != 0x52 {
            return self.fail(ErrorKind::Unsupported("attached case3 owner-offset field"));
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
            environment_movie: spec.source.movie,
            environment_header_version: spec.source.header_version,
            binding,
            focus_rect: None,
            tab_enabled_last_write: None,
            name_snapshot: None,
            owner,
            storage: None,
            prototype: Some(prototype),
            children: None,
            mouse_event_marker: false,
        });
        self.attached_targets
            .insert(spec.target, AttachedTarget::Clip(id));
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
            environment_movie: clip.environment_movie,
            environment_header_version: clip.environment_header_version,
            binding: clip.binding,
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
    /// Bind a known current name for this exact live target and its existing
    /// public handle. The caller must establish that association independently.
    /// Authentic empty names are valid. An unknown/missing handle is not empty.
    pub fn bind_attached_name_snapshot(
        &mut self,
        id: AttachedClipId,
        snapshot: AttachedNameSnapshot,
    ) -> Result<()> {
        self.check_attached(id)?;
        if snapshot.target != self.attached[id.0].spec.target {
            return self.fail(ErrorKind::Unsupported("attached name snapshot target mismatch"));
        }
        if self.attached[id.0].name_snapshot.is_some() {
            return self.fail(ErrorKind::Unsupported("attached name snapshot already bound"));
        }
        if snapshot.name.len() > self.limits.string_units.saturating_sub(self.string_units) {
            return self.fail(ErrorKind::Budget("attached name snapshot string units"));
        }
        if snapshot.name.contains(&0)
            || char::decode_utf16(snapshot.name.iter().copied()).any(|unit| unit.is_err())
        {
            return self.fail(ErrorKind::Unsupported("attached name snapshot UTF-16 profile"));
        }
        self.string_units += snapshot.name.len();
        self.attached[id.0].name_snapshot = Some(snapshot.name);
        Ok(())
    }
    pub fn attached_name_snapshot(&self, id: AttachedClipId) -> Result<Option<AttachedNameSnapshot>> {
        self.check_attached(id)?;
        Ok(self.attached[id.0].name_snapshot.as_ref().map(|name| AttachedNameSnapshot {
            target: self.attached[id.0].spec.target,
            name: Arc::clone(name),
        }))
    }
    /// Discard a witness after externally reported name/handle mutation. This
    /// performs no native rename, child-map update or lifecycle transition.
    /// String accounting is cumulative and is not refunded by invalidation.
    pub fn invalidate_attached_name_snapshot(&mut self, id: AttachedClipId) -> Result<()> {
        self.check_attached(id)?;
        self.attached[id.0].name_snapshot = None;
        Ok(())
    }
    /// Bind an independently supplied raw target+b8 snapshot once. No native
    /// default is inferred from event flags, source bytes or root identity.
    pub fn bind_attached_focus_rect(
        &mut self,
        id: AttachedClipId,
        state: AttachedFocusRectState,
    ) -> Result<()> {
        self.check_attached(id)?;
        if self.attached[id.0].focus_rect.is_some() {
            return self.fail(ErrorKind::Unsupported(
                "attached focus rectangle already bound",
            ));
        }
        self.attached[id.0].focus_rect = Some(AttachedFocusRectInfo {
            state,
            last_write: None,
        });
        Ok(())
    }
    pub fn attached_focus_rect(&self, id: AttachedClipId) -> Result<Option<AttachedFocusRectInfo>> {
        self.check_attached(id)?;
        Ok(self.attached[id.0].focus_rect)
    }
    /// Supply the shared native target+b8 word once. The legacy focus-specific
    /// binding API and this API address exactly the same backing state.
    pub fn bind_attached_target_status(&mut self, id: AttachedClipId, word: u32) -> Result<()> {
        self.bind_attached_focus_rect(
            id,
            AttachedFocusRectState {
                target_flags_b8: word,
            },
        )
    }
    pub fn attached_target_status(&self, id: AttachedClipId) -> Result<Option<u32>> {
        self.check_attached(id)?;
        Ok(self.attached[id.0]
            .focus_rect
            .map(|info| info.state.target_flags_b8))
    }
    pub fn attached_tab_enabled(
        &self,
        id: AttachedClipId,
    ) -> Result<Option<AttachedTabEnabledInfo>> {
        self.check_attached(id)?;
        Ok(self.attached[id.0]
            .focus_rect
            .map(|info| AttachedTabEnabledInfo {
                target_flags_b8: info.state.target_flags_b8,
                last_write: self.attached[id.0].tab_enabled_last_write,
            }))
    }
    fn attached_tab_enabled_name(&self, key: &Text) -> Result<bool> {
        if key.as_ref() != text("tabEnabled").as_ref() {
            return Ok(false);
        }
        if self.attached_name_profile.as_ref().is_some_and(|profile| {
            profile.classify_attached_name(key) != AttachedNameClass::Standard(29)
        }) {
            return self.fail(ErrorKind::Unsupported(
                "attached tabEnabled native ID profile",
            ));
        }
        Ok(true)
    }
    fn attached_focus_rect_name(&self, key: &Text) -> Result<bool> {
        if key.as_ref() != text("_focusrect").as_ref() {
            return Ok(false);
        }
        if self.attached_name_profile.as_ref().is_some_and(|profile| {
            profile.classify_attached_name(key) != AttachedNameClass::Standard(17)
        }) {
            return self.fail(ErrorKind::Unsupported(
                "attached focus rectangle native ID profile",
            ));
        }
        Ok(true)
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
                AttachedChild::TextField { name, field } => {
                    let info = self.attached_text_field_info(field)?;
                    if info.parent != id {
                        return self.fail(ErrorKind::Unsupported(
                            "attached TextField child parent mismatch",
                        ));
                    }
                    (name, info.target, ChildTarget::TextField(field))
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
            self.attached_targets
                .insert(target, AttachedTarget::Unsupported);
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
        if key.as_ref() == text("autoSize").as_ref() {
            // Authenticated standard ID48 is NOT implemented by this case-3
            // native dispatcher. Exact false-return branches continue into
            // ordinary handling; this does not admit other standard members or
            // the distinct case-4 TextField's autosize behavior.
            return if self.attached_name_profile.as_ref().is_some_and(|profile| {
                profile.classify_attached_name(key) == AttachedNameClass::Standard(48)
            }) {
                Ok(())
            } else {
                self.fail(ErrorKind::Unsupported(
                    "attached autoSize native ID profile",
                ))
            };
        }
        if let Some(profile) = &self.attached_name_profile {
            return match profile.classify_attached_name(key) {
                AttachedNameClass::Ordinary => Ok(()),
                AttachedNameClass::Standard(_) => self.fail(ErrorKind::Unsupported(
                    "attached standard native member requires its native implementation",
                )),
                AttachedNameClass::UnsupportedName(_) => self.fail(ErrorKind::Unsupported(
                    "attached native member-name profile",
                )),
            };
        }
        // Exact selected names are absent from the original standard-member
        // map; both possible native flag29 routes fall through ordinary handling.
        // This is not an underscore heuristic or an arbitrary host-classifier bit.
        if [
            "gfx",
            "Function",
            "Object",
            "MovieClip",
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
                    Property::NativeMarker(_) => {
                        return self
                            .fail(ErrorKind::Unsupported("attached native descriptor lookup"));
                    }
                });
            }
            // The partial Rust MovieClip prototype does not contain all native
            // functions. A known omitted own native member must stop HERE,
            // before an Object.prototype script addition could shadow it.
            if Some(id) == self.movie_clip_prototype {
                if let Some(profile) = &self.attached_name_profile {
                    if matches!(profile.native_baseline_membership(key),
                        BaselineMembership::Stored(owners) if owners.contains(BaselineOwners::MOVIECLIP))
                    {
                        return self.fail(ErrorKind::Unsupported(
                            "attached omitted native MovieClip member",
                        ));
                    }
                }
            }
            cursor = object.prototype;
            depth += 1;
        }
        if native_clip_seen {
            if let Some(profile) = &self.attached_name_profile {
                return match profile.native_baseline_membership(key) {
                    BaselineMembership::Absent => Ok(PrototypeMember::Missing),
                    _ => self.fail(ErrorKind::Unsupported(
                        "attached missing builtin prototype member",
                    )),
                };
            }
            if ![
                "Function",
                "gfx",
                "onLoad",
                "Object",
                "MovieClip",
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
        if key.as_ref() == text("_name").as_ref() {
            if self.attached_name_profile.as_ref().is_some_and(|profile| {
                profile.classify_attached_name(key) != AttachedNameClass::Standard(13)
            }) {
                return self.fail(ErrorKind::Unsupported("attached name native ID profile"));
            }
            let Some(name) = &self.attached[id.0].name_snapshot else {
                return self.fail(ErrorKind::Unsupported("attached current name is unbound"));
            };
            // Existing handle+10 is a native handled String result. It precedes
            // own storage, children, prototypes and resolver/accessor fallback.
            return Ok(AttachedMember::Present(Value::String(Arc::clone(name))));
        }
        if key.as_ref() == text("_visible").as_ref() {
            if self.attached_name_profile.as_ref().is_some_and(|profile| {
                profile.classify_attached_name(key) != AttachedNameClass::Standard(7)
            }) {
                return self.fail(ErrorKind::Unsupported(
                    "attached visibility native ID profile",
                ));
            }
            // Native ID 7 getter reads this target's +0x6a WORD bit 14.
            // Parent/effective visibility and render-node state are separate.
            // The setter has additional effects and remains unsupported here.
            return Ok(AttachedMember::Present(Value::Bool(
                self.attached[id.0].spec.event_state.target_flags & 0x4000 != 0,
            )));
        }
        if self.attached_focus_rect_name(key)? {
            let Some(info) = self.attached[id.0].focus_rect else {
                return self.fail(ErrorKind::Unsupported(
                    "attached focus rectangle state is unbound",
                ));
            };
            return Ok(AttachedMember::Present(
                info.state.read().map_or(Value::Null, Value::Bool),
            ));
        }
        if self.attached_tab_enabled_name(key)? {
            let Some(info) = self.attached[id.0].focus_rect else {
                return self.fail(ErrorKind::Unsupported("attached target status is unbound"));
            };
            return Ok(AttachedMember::Present(
                info.state
                    .read_tab_enabled()
                    .map_or(Value::Undefined, Value::Bool),
            ));
        }
        self.attached_member_name(key)?;
        let clip = &self.attached[id.0];
        if let Some(storage) = clip.storage {
            self.attached_resolver_guard(storage)?;
            if let Some(member) = self.objects[storage.0].properties.get(key) {
                return match member {
                    Property::Data(value) => Ok(AttachedMember::Present(value.clone())),
                    Property::NativeMarker(_) => {
                        self.fail(ErrorKind::Unsupported("attached native descriptor lookup"))
                    }
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
            Some(ChildTarget::TextField(field)) => {
                if self.attached_text_field_info(*field)?.target_flags_6a & 0x180 == 0x180 {
                    return Ok(AttachedMember::Present(Value::DisplayTextField(*field)));
                }
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
        self.attached_write_at(id, text(key), value, None)
    }
    pub(super) fn attached_write(
        &mut self,
        id: AttachedClipId,
        key: Text,
        value: Value,
    ) -> Result<AttachedWrite> {
        self.attached_write_at(id, key, value, self.origin())
    }
    fn attached_write_at(
        &mut self,
        id: AttachedClipId,
        key: Text,
        value: Value,
        origin: Option<CodeOrigin>,
    ) -> Result<AttachedWrite> {
        self.check_attached(id)?;
        if self.attached_focus_rect_name(&key)? {
            if value != Value::Bool(false) {
                return self.fail(ErrorKind::Unsupported(
                    "attached focus rectangle exact false profile",
                ));
            }
            let Some(mut info) = self.attached[id.0].focus_rect else {
                return self.fail(ErrorKind::Unsupported(
                    "attached focus rectangle state is unbound",
                ));
            };
            info.state.clear();
            info.last_write = origin;
            self.attached[id.0].focus_rect = Some(info);
            return Ok(AttachedWrite::NativeUpdated);
        }
        if self.attached_tab_enabled_name(&key)? {
            if value != Value::Bool(false) {
                return self.fail(ErrorKind::Unsupported(
                    "attached tabEnabled exact false profile",
                ));
            }
            // Native derived dispatch can run storage watches before the base
            // setter. This model admits only its freshly constructed storage:
            // native +58 starts null; supported raw/data/prototype operations
            // preserve it, and every watch installer remains unsupported.
            // There is no API for adopting preexisting or untracked storage.
            if let Some(storage) = self.attached[id.0].storage {
                self.check_object(storage)?;
                if !matches!(self.objects[storage.0].kind, Kind::BoundStorage(owner) if owner == id)
                {
                    return self.fail(ErrorKind::Unsupported(
                        "attached tabEnabled no-watch storage profile",
                    ));
                }
            }
            let Some(mut info) = self.attached[id.0].focus_rect else {
                return self.fail(ErrorKind::Unsupported("attached target status is unbound"));
            };
            info.state.disable_tab();
            // Focus provenance stays unchanged: only the shared word and this
            // member's independent diagnostic provenance are updated.
            self.attached[id.0].focus_rect = Some(info);
            self.attached[id.0].tab_enabled_last_write = origin;
            return Ok(AttachedWrite::NativeUpdated);
        }
        self.attached_member_name(&key)?;
        // General ordinary names are not proof that the native pre-write hook
        // is inert. These routes were absent from the earlier exact-name slice.
        if key.starts_with(&[b'o' as u16, b'n' as u16])
            || ["rendererString", "rendererFloat", "disableBatching"]
                .iter()
                .any(|name| key.as_ref() == text(name).as_ref())
        {
            return self.fail(ErrorKind::Unsupported("attached native pre-write hook"));
        }
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

    fn synthetic_profile() -> Arc<NativeNameProfile> {
        // Synthetic inventory only; real owner-private names never enter tests.
        let mut data = format!(
            "MHF-ATTACHED-NAMES\t1\nbuild\t1.52.0.1700\nsource-sha256\t{}\ncounts\t117\t44\t9\n",
            "ab".repeat(32)
        );
        for id in 0..117 {
            let alias = id <= 21 || (110..=115).contains(&id);
            let name = if id == 48 {
                "autoSize".to_owned()
            } else {
                format!("{}native{id:03}", if alias { "_" } else { "" })
            };
            data.push_str(&format!("standard\t{id}\t{}\t{name}\n", u8::from(alias),));
        }
        let mut mc = vec!["constructor".to_owned(), "__proto__".to_owned()];
        mc.extend((0..42).map(|id| format!("mc{id:03}")));
        let mut object = vec!["constructor".to_owned()];
        object.extend((0..8).map(|id| format!("obj{id:03}")));
        for (role, mut names) in [("movieclip", mc), ("object", object)] {
            names.sort();
            for name in names {
                data.push_str(&format!("{role}\t{name}\n"));
            }
        }
        Arc::new(
            NativeNameProfile::parse_verified(
                data.as_bytes(),
                &super::super::ExpectedProfile {
                    build: "1.52.0.1700",
                    source_sha256: [0xab; 32],
                    inventory_sha256: [0x51; 32],
                },
                |_| Ok([0x51; 32]),
            )
            .unwrap(),
        )
    }

    #[test]
    fn verified_name_profile_keeps_live_shadowing_and_native_boundaries() {
        let mut vm = vm();
        let root = vm.create_attached_movie_clip(spec(1, None)).unwrap();
        vm.set_attached_children(
            root,
            vec![AttachedChild::Unsupported {
                name: text("unsupportedChild"),
                target: DisplayTargetId(2),
            }],
        )
        .unwrap();
        assert!(vm.read_attached_member(root, "ordinaryName").is_err());
        vm.install_attached_name_profile(synthetic_profile())
            .unwrap();
        assert!(
            vm.install_attached_name_profile(synthetic_profile())
                .is_err()
        );
        assert_eq!(
            vm.read_attached_member(root, "ordinaryName").unwrap(),
            AttachedMember::Missing
        );
        assert!(vm.read_attached_member(root, "unsupportedChild").is_err());
        vm.write_attached_member(root, "unsupportedChild", Value::Undefined)
            .unwrap();
        assert_eq!(
            vm.read_attached_member(root, "unsupportedChild").unwrap(),
            AttachedMember::Present(Value::Undefined)
        );
        for key in ["native022", "_unmodeled", "é"] {
            assert!(vm.read_attached_member(root, key).is_err());
            assert!(
                vm.write_attached_member(root, key, Value::Undefined)
                    .is_err()
            );
        }
        for key in [
            "onPress",
            "onUnrecognized",
            "rendererString",
            "rendererFloat",
            "disableBatching",
        ] {
            assert!(
                vm.write_attached_member(root, key, Value::Undefined)
                    .is_err()
            );
        }
        assert_eq!(
            vm.read_attached_member(root, "onUnrecognized").unwrap(),
            AttachedMember::Missing
        );
    }

    #[test]
    fn verified_name_profile_does_not_skip_omitted_native_own_members() {
        let mut vm = vm();
        vm.install_attached_name_profile(synthetic_profile())
            .unwrap();
        let root = vm.create_attached_movie_clip(spec(1, None)).unwrap();
        vm.set_attached_children(root, vec![]).unwrap();
        let object_prototype = vm.object_prototype.unwrap();
        vm.set(
            object_prototype,
            "mc000",
            Value::string("must not shadow omitted native own key"),
        )
        .unwrap();
        assert!(matches!(
            vm.read_attached_member(root, "mc000").unwrap_err().kind,
            ErrorKind::Unsupported("attached omitted native MovieClip member")
        ));
        vm.write_attached_member(root, "mc000", Value::Undefined)
            .unwrap();
        assert_eq!(
            vm.read_attached_member(root, "mc000").unwrap(),
            AttachedMember::Present(Value::Undefined)
        );
        vm.raw_member(
            object_prototype,
            text("ordinaryGetter"),
            Property::Accessor {
                getter: Value::Undefined,
                setter: None,
            },
            0,
        )
        .unwrap();
        assert!(vm.read_attached_member(root, "ordinaryGetter").is_err());
        vm.raw_member(
            object_prototype,
            text("__resolve"),
            Property::Data(Value::Undefined),
            0,
        )
        .unwrap();
        vm.delete_member(object_prototype, &text("__resolve"))
            .unwrap();
        assert!(vm.read_attached_member(root, "ordinaryMissing").is_err());
    }

    #[test]
    fn current_name_snapshot_distinguishes_unknown_empty_and_receiver_and_never_renames() {
        let mut vm = vm();
        let root = vm.create_attached_movie_clip(spec(1, None)).unwrap();
        let child = vm.create_attached_movie_clip(spec(2, Some(root))).unwrap();
        assert!(vm.read_attached_member(root, "_name").is_err());
        assert_eq!(vm.attached_name_snapshot(root).unwrap(), None);
        let before = (vm.object_count(), vm.properties, vm.string_units);
        vm.bind_attached_name_snapshot(root, AttachedNameSnapshot {
            target: DisplayTargetId(1), name: text(""),
        }).unwrap();
        let supplied = text("CaseSensitive 🦀");
        vm.bind_attached_name_snapshot(child, AttachedNameSnapshot {
            target: DisplayTargetId(2), name: Arc::clone(&supplied),
        }).unwrap();
        assert_eq!(vm.read_attached_member(root, "_name").unwrap(),
            AttachedMember::Present(Value::string("")));
        assert_eq!(vm.read_attached_member(child, "_name").unwrap(),
            AttachedMember::Present(Value::String(Arc::clone(&supplied))));
        assert_eq!(vm.object_count(), before.0);
        assert_eq!(vm.properties, before.1);
        assert_eq!(vm.string_units, before.2 + supplied.len());
        assert!(vm.attached_info(child).unwrap().storage.is_none());
        assert!(!vm.attached_info(child).unwrap().child_snapshot_installed);
        assert!(vm.write_attached_member(child, "_name", Value::string("rename")).is_err());
        assert_eq!(vm.attached_name_snapshot(child).unwrap().unwrap().name, supplied);
        let charged = vm.string_units;
        vm.invalidate_attached_name_snapshot(child).unwrap();
        assert!(vm.read_attached_member(child, "_name").is_err());
        assert_eq!(vm.string_units, charged);
        vm.bind_attached_name_snapshot(child, AttachedNameSnapshot {
            target: DisplayTargetId(2), name: text("new externally established snapshot"),
        }).unwrap();
        assert_eq!(vm.read_attached_member(root, "_name").unwrap(),
            AttachedMember::Present(Value::string("")));
    }

    #[test]
    fn current_name_snapshot_rejects_invalid_admission_without_publication() {
        let mut vm = vm();
        let root = vm.create_attached_movie_clip(spec(1, None)).unwrap();
        let charged = vm.string_units;
        for snapshot in [
            AttachedNameSnapshot {target: DisplayTargetId(2), name: text("wrong target")},
            AttachedNameSnapshot {target: DisplayTargetId(1), name: text("nul\0name")},
            AttachedNameSnapshot {target: DisplayTargetId(1), name: vec![0xd800].into()},
        ] {
            assert!(vm.bind_attached_name_snapshot(root, snapshot).is_err());
            assert_eq!(vm.attached_name_snapshot(root).unwrap(), None);
            assert_eq!(vm.string_units, charged);
        }
        vm.limits.string_units = charged + 2;
        assert!(vm.bind_attached_name_snapshot(root, AttachedNameSnapshot {
            target: DisplayTargetId(1), name: text("three"),
        }).is_err());
        vm.bind_attached_name_snapshot(root, AttachedNameSnapshot {
            target: DisplayTargetId(1), name: text("ok"),
        }).unwrap();
        assert!(vm.bind_attached_name_snapshot(root, AttachedNameSnapshot {
            target: DisplayTargetId(1), name: text(""),
        }).is_err());
        assert_eq!(vm.read_attached_member(root, "_name").unwrap(),
            AttachedMember::Present(Value::string("ok")));
        assert!(vm.attached_name_snapshot(AttachedClipId(999)).is_err());
        assert!(vm.invalidate_attached_name_snapshot(AttachedClipId(999)).is_err());
        vm.install_attached_name_profile(synthetic_profile()).unwrap();
        // This synthetic profile intentionally maps another spelling to ID13.
        assert!(vm.read_attached_member(root, "_name").is_err());
        assert_eq!(vm.attached_name_snapshot(root).unwrap().unwrap().name, text("ok"));
    }

    #[test]
    fn verified_name_profile_installation_rejects_after_code_registration() {
        let mut vm = vm();
        vm.register(Arc::new(Program::parse(&[0]).unwrap()))
            .unwrap();
        assert!(
            vm.install_attached_name_profile(synthetic_profile())
                .is_err()
        );
        assert!(vm.attached_name_profile.is_none());
    }

    #[test]
    fn explicit_local_import_binding_keeps_target_version_and_old_gate() {
        let mut vm = vm();
        let root = vm.create_attached_movie_clip(spec(1, None)).unwrap();
        let mut imported = spec(2, Some(root));
        imported.source.movie = LoadedMovieSourceId(11);
        imported.source.header_version = 11;
        assert!(vm.create_attached_movie_clip(imported).is_err());
        let witness = AttachedMovieBinding::LocalSourceOrImportedLocalExport;
        let child = vm
            .create_attached_movie_clip_with_binding(imported, witness)
            .unwrap();
        let info = vm.attached_info(child).unwrap();
        assert_eq!(info.source, imported.source);
        assert_eq!(info.environment_movie, LoadedMovieSourceId(11));
        assert_eq!(info.environment_header_version, 11);
        assert_eq!(info.binding, Some(witness));
        assert_eq!(
            vm.attached_info(root).unwrap().environment_header_version,
            15
        );
        assert_eq!(vm.loaded_movie_targets.get(&info.owner), Some(&11));
        assert!(vm.create_attached_movie_clip(spec(3, Some(child))).is_err());
        assert!(vm.bind_loaded_movie_target(info.owner, 15).is_err());
        vm.bind_loaded_movie_target(info.owner, 11).unwrap();
        assert_eq!(vm.loaded_movie_targets.get(&info.owner), Some(&11));
        for version in [5, 6, 20] {
            imported.target = DisplayTargetId(100 + u64::from(version));
            imported.source.header_version = version;
            assert!(
                vm.create_attached_movie_clip_with_binding(imported, witness)
                    .is_err()
            );
        }
        let ordinary = vm.object().unwrap();
        assert!(vm.bind_loaded_movie_target(ordinary, 11).is_err());
        vm.bind_loaded_movie_target(ordinary, 15).unwrap();
    }

    #[test]
    fn visibility_getter_reads_local_native_bit_without_storage_or_parent_effects() {
        let mut vm = vm();
        let parent = vm.create_attached_movie_clip(spec(1, None)).unwrap();
        let mut child_spec = spec(2, Some(parent));
        child_spec.event_state.target_flags |= 0x4000;
        let child = vm.create_attached_movie_clip(child_spec).unwrap();
        let object_count = vm.objects.len();
        let properties = vm.properties;
        let parent_state = vm.attached_event_state(parent).unwrap();
        let child_state = vm.attached_event_state(child).unwrap();
        // No complete child snapshot or lazy storage is needed for ID 7.
        assert_eq!(
            vm.read_attached_member(parent, "_visible").unwrap(),
            AttachedMember::Present(Value::Bool(false))
        );
        assert_eq!(
            vm.read_attached_member(child, "_visible").unwrap(),
            AttachedMember::Present(Value::Bool(true))
        );
        assert_eq!(vm.objects.len(), object_count);
        assert_eq!(vm.properties, properties);
        assert!(vm.attached_info(parent).unwrap().storage.is_none());
        assert!(vm.attached_info(child).unwrap().storage.is_none());
        assert_eq!(vm.attached_event_state(parent).unwrap(), parent_state);
        assert_eq!(vm.attached_event_state(child).unwrap(), child_state);
        assert!(
            vm.write_attached_member(child, "_visible", Value::Bool(false))
                .is_err()
        );
        assert_eq!(vm.attached_event_state(child).unwrap(), child_state);

        vm.write_attached_member(child, "disabled", Value::Bool(false))
            .unwrap();
        let storage = ObjectId(vm.attached_info(child).unwrap().storage.unwrap().0);
        vm.raw_member(
            storage,
            text("_visible"),
            Property::Accessor {
                getter: Value::Undefined,
                setter: None,
            },
            7,
        )
        .unwrap();
        assert_eq!(
            vm.read_attached_member(child, "_visible").unwrap(),
            AttachedMember::Present(Value::Bool(true))
        );
        assert!(matches!(
            vm.objects[storage.0].properties.get(&text("_visible")),
            Some(Property::Accessor { .. })
        ));
        assert_eq!(vm.own_flags(storage, &text("_visible")), 7);

        // Supplied current native state changes independently of the parent.
        let mut parent_after = parent_state;
        parent_after.target_flags |= 0x4000;
        let mut child_after = child_state;
        child_after.target_flags &= !0x4000;
        vm.update_attached_event_state(parent, parent_after)
            .unwrap();
        vm.update_attached_event_state(child, child_after).unwrap();
        assert_eq!(
            vm.read_attached_member(parent, "_visible").unwrap(),
            AttachedMember::Present(Value::Bool(true))
        );
        assert_eq!(
            vm.read_attached_member(child, "_visible").unwrap(),
            AttachedMember::Present(Value::Bool(false))
        );
        assert_eq!(vm.attached_event_state(child).unwrap(), child_after);
        assert!(
            vm.read_attached_member(AttachedClipId(999), "_visible")
                .is_err()
        );
        // This synthetic inventory deliberately does not identify _visible as
        // ID 7. An inconsistent installed profile must not be ignored.
        vm.install_attached_name_profile(synthetic_profile())
            .unwrap();
        assert!(vm.read_attached_member(child, "_visible").is_err());
    }

    #[test]
    fn auto_size_case3_false_dispatch_keeps_ordinary_shadowing_and_write_guards() {
        let mut vm = vm();
        let root = vm.create_attached_movie_clip(spec(1, None)).unwrap();
        let child = vm.create_attached_movie_clip(spec(2, Some(root))).unwrap();
        assert!(vm.read_attached_member(root, "autoSize").is_err());
        assert!(
            vm.write_attached_member(root, "autoSize", Value::string("none"))
                .is_err()
        );
        assert!(vm.attached_info(root).unwrap().storage.is_none());
        vm.install_attached_name_profile(synthetic_profile())
            .unwrap();
        assert_eq!(
            vm.attached_name_profile
                .as_ref()
                .unwrap()
                .classify_attached_name(&text("autoSize")),
            AttachedNameClass::Standard(48)
        );
        vm.set_attached_children(
            root,
            vec![AttachedChild::Clip {
                name: text("autoSize"),
                clip: child,
            }],
        )
        .unwrap();
        assert_eq!(
            vm.read_attached_member(root, "autoSize").unwrap(),
            AttachedMember::Present(Value::DisplayClip(child))
        );
        assert!(vm.attached_info(root).unwrap().storage.is_none());
        assert_eq!(
            vm.write_attached_member(root, "autoSize", Value::string("none"))
                .unwrap(),
            AttachedWrite::Created
        );
        let storage = ObjectId(vm.attached_info(root).unwrap().storage.unwrap().0);
        assert_eq!(
            vm.read_attached_member(root, "autoSize").unwrap(),
            AttachedMember::Present(Value::string("none"))
        );
        vm.raw_member(
            storage,
            text("autoSize"),
            Property::Data(Value::Undefined),
            4,
        )
        .unwrap();
        assert_eq!(
            vm.read_attached_member(root, "autoSize").unwrap(),
            AttachedMember::Present(Value::Undefined)
        );
        assert_eq!(
            vm.write_attached_member(root, "autoSize", Value::string("later"))
                .unwrap(),
            AttachedWrite::ReadOnly
        );
        assert_eq!(vm.own_flags(storage, &text("autoSize")), 4);
        vm.raw_member(
            storage,
            text("autoSize"),
            Property::Accessor {
                getter: Value::Undefined,
                setter: None,
            },
            0,
        )
        .unwrap();
        assert!(vm.read_attached_member(root, "autoSize").is_err());
        assert!(
            vm.write_attached_member(root, "autoSize", Value::string("later"))
                .is_err()
        );
        assert!(vm.read_attached_member(root, "native047").is_err());
        assert!(
            vm.write_attached_member(root, "native047", Value::string("later"))
                .is_err()
        );
        assert!(vm.attached_info(child).unwrap().storage.is_none());
        let prototype = vm.object().unwrap();
        vm.raw_member(
            prototype,
            text("autoSize"),
            Property::Accessor {
                getter: Value::Undefined,
                setter: None,
            },
            0,
        )
        .unwrap();
        vm.set_attached_prototype(child, Some(prototype)).unwrap();
        assert!(
            vm.write_attached_member(child, "autoSize", Value::string("none"))
                .is_err()
        );
    }

    #[test]
    fn tab_enabled_uses_shared_supplied_status_without_storage_or_normal_descriptors() {
        let mut vm = vm();
        let parent = vm.create_attached_movie_clip(spec(1, None)).unwrap();
        let child = vm
            .create_attached_movie_clip(spec(2, Some(parent)))
            .unwrap();
        assert!(vm.read_attached_member(child, "tabEnabled").is_err());
        assert!(
            vm.write_attached_member(child, "tabEnabled", Value::Bool(false))
                .is_err()
        );
        assert!(vm.attached_info(child).unwrap().storage.is_none());
        vm.bind_attached_target_status(parent, 0x1234_01e8).unwrap();
        vm.bind_attached_target_status(child, 0x5678_0188).unwrap();
        assert!(
            vm.bind_attached_focus_rect(child, AttachedFocusRectState { target_flags_b8: 0 })
                .is_err()
        );
        assert_eq!(
            vm.read_attached_member(parent, "tabEnabled").unwrap(),
            AttachedMember::Present(Value::Bool(true))
        );
        assert_eq!(
            vm.read_attached_member(child, "tabEnabled").unwrap(),
            AttachedMember::Present(Value::Undefined)
        );
        let events = vm.attached_event_state(child).unwrap();
        let count = vm.object_count();
        assert_eq!(
            vm.write_attached_member(child, "tabEnabled", Value::Bool(false))
                .unwrap(),
            AttachedWrite::NativeUpdated
        );
        assert_eq!(vm.object_count(), count);
        assert!(vm.attached_info(child).unwrap().storage.is_none());
        assert_eq!(vm.attached_target_status(child).unwrap(), Some(0x5678_01c8));
        assert_eq!(
            vm.attached_target_status(parent).unwrap(),
            Some(0x1234_01e8)
        );
        assert_eq!(
            vm.read_attached_member(child, "_focusrect").unwrap(),
            AttachedMember::Present(Value::Bool(true))
        );
        vm.write_attached_member(child, "_focusrect", Value::Bool(false))
            .unwrap();
        assert_eq!(vm.attached_target_status(child).unwrap(), Some(0x5678_0148));
        assert_eq!(
            vm.read_attached_member(child, "tabEnabled").unwrap(),
            AttachedMember::Present(Value::Bool(false))
        );
        vm.write_attached_member(child, "disabled", Value::Bool(false))
            .unwrap();
        let storage = ObjectId(vm.attached_info(child).unwrap().storage.unwrap().0);
        vm.raw_member(
            storage,
            text("tabEnabled"),
            Property::Accessor {
                getter: Value::Undefined,
                setter: None,
            },
            7,
        )
        .unwrap();
        let count = vm.object_count();
        assert_eq!(
            vm.write_attached_member(child, "tabEnabled", Value::Bool(false))
                .unwrap(),
            AttachedWrite::NativeUpdated
        );
        assert_eq!(
            vm.read_attached_member(child, "tabEnabled").unwrap(),
            AttachedMember::Present(Value::Bool(false))
        );
        assert!(matches!(
            vm.objects[storage.0].properties.get(&text("tabEnabled")),
            Some(Property::Accessor { .. })
        ));
        assert_eq!(vm.own_flags(storage, &text("tabEnabled")), 7);
        assert_eq!(vm.object_count(), count);
        assert_eq!(vm.attached_event_state(child).unwrap(), events);
        for value in [
            Value::Bool(true),
            Value::Number(0.0),
            Value::Null,
            Value::Undefined,
        ] {
            assert!(
                vm.write_attached_member(child, "tabEnabled", value)
                    .is_err()
            );
            assert_eq!(vm.attached_target_status(child).unwrap(), Some(0x5678_0148));
        }
        assert!(
            vm.read_attached_member(AttachedClipId(999), "tabEnabled")
                .is_err()
        );
        // Deliberately inconsistent synthetic standard-name table is rejected.
        vm.install_attached_name_profile(synthetic_profile())
            .unwrap();
        assert!(vm.read_attached_member(child, "tabEnabled").is_err());
        assert!(
            vm.write_attached_member(child, "tabEnabled", Value::Bool(false))
                .is_err()
        );
    }

    #[test]
    fn focus_rect_native_state_is_explicit_local_and_bypasses_ordinary_members() {
        let mut vm = vm();
        let root = vm.create_attached_movie_clip(spec(1, None)).unwrap();
        let child = vm.create_attached_movie_clip(spec(2, Some(root))).unwrap();
        assert_eq!(vm.attached_focus_rect(root).unwrap(), None);
        assert!(vm.read_attached_member(root, "_focusrect").is_err());
        assert!(
            vm.write_attached_member(root, "_focusrect", Value::Bool(false))
                .is_err()
        );
        assert!(vm.attached_info(root).unwrap().storage.is_none());
        let root_state = AttachedFocusRectState {
            target_flags_b8: 0x9abc_0180,
        };
        let child_state = AttachedFocusRectState {
            target_flags_b8: 0x7654_0000,
        };
        vm.bind_attached_focus_rect(root, root_state).unwrap();
        vm.bind_attached_focus_rect(child, child_state).unwrap();
        assert!(vm.bind_attached_focus_rect(root, child_state).is_err());
        assert_eq!(
            vm.read_attached_member(root, "_focusrect").unwrap(),
            AttachedMember::Present(Value::Bool(true))
        );
        assert_eq!(
            vm.read_attached_member(child, "_focusrect").unwrap(),
            AttachedMember::Present(Value::Null)
        );
        for value in [
            Value::Bool(true),
            Value::Number(0.0),
            Value::Undefined,
            Value::Null,
        ] {
            assert!(vm.write_attached_member(root, "_focusrect", value).is_err());
            assert_eq!(
                vm.attached_focus_rect(root).unwrap().unwrap().state,
                root_state
            );
        }
        let events = vm.attached_event_state(root).unwrap();
        assert_eq!(
            vm.write_attached_member(root, "_focusrect", Value::Bool(false))
                .unwrap(),
            AttachedWrite::NativeUpdated
        );
        assert!(vm.attached_info(root).unwrap().storage.is_none());
        assert_eq!(
            vm.attached_focus_rect(root)
                .unwrap()
                .unwrap()
                .state
                .target_flags_b8,
            0x9abc_0100
        );
        assert_eq!(
            vm.attached_focus_rect(child).unwrap().unwrap().state,
            child_state
        );
        assert_eq!(vm.attached_event_state(root).unwrap(), events);
        vm.write_attached_member(root, "disabled", Value::Bool(true))
            .unwrap();
        let storage = ObjectId(vm.attached_info(root).unwrap().storage.unwrap().0);
        vm.raw_member(
            storage,
            text("_focusrect"),
            Property::Accessor {
                getter: Value::Undefined,
                setter: Some(Value::Undefined),
            },
            4,
        )
        .unwrap();
        assert_eq!(
            vm.write_attached_member(root, "_focusrect", Value::Bool(false))
                .unwrap(),
            AttachedWrite::NativeUpdated
        );
        assert_eq!(
            vm.read_attached_member(root, "_focusrect").unwrap(),
            AttachedMember::Present(Value::Bool(false))
        );
        assert!(matches!(
            vm.objects[storage.0].properties.get(&text("_focusrect")),
            Some(Property::Accessor { .. })
        ));
        assert_eq!(vm.own_flags(storage, &text("_focusrect")), 4);
    }

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
