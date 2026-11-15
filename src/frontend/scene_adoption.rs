//! Adopt a complete saved Scene using explicitly supplied live state.
//! No events, initializers, constructors, ExternalInterface or defaults run here.

use super::{
    display_state::{self, DisplayState},
    library::{MovieId, MovieLibrary},
    movie_binding::{self, ConditionalProfile, Validator},
    textfield_host::{self, TextFieldHost},
    textfield_state::{self, AttachedIdentity, Profile, Snapshot, TextFieldState},
};
use crate::{
    avm1::vm::{
        AttachedChild, AttachedClipId, AttachedClipSource, AttachedClipSpec, AttachedEventState,
        AttachedFocusRectState, AttachedMovieBinding, AttachedTextFieldId, AttachedTextFieldSource,
        AttachedTextFieldSpec, DisplayTargetId, LoadedMovieSourceId, Text, TextFieldPrototypeId,
        TextFieldWatchState, Vm, text,
    },
    gfx::{
        CharacterKind, TagData,
        scene::{DefinitionRef, NodeId, NodeKind, Scene},
        text::EditText,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    num::NonZeroU64,
};

#[derive(Clone, Copy, Debug)]
pub struct ClipState {
    pub target: DisplayTargetId,
    pub events: AttachedEventState,
    pub focus: AttachedFocusRectState,
}

#[derive(Clone, Debug)]
pub struct FieldState {
    pub target: DisplayTargetId,
    pub target_flags_6a: u32,
    pub prototype: TextFieldPrototypeId,
    pub watch_state: TextFieldWatchState,
    /// Scene node of the first parent ancestor marked by supplied +6a bit 0x400.
    pub environment_owner: NodeId,
    pub profile: Profile,
    pub snapshot: Snapshot,
}

/// No Default: every runtime condition and value is an explicit caller input.
#[derive(Clone, Debug)]
pub struct SuppliedSceneState {
    pub binding_profile: ConditionalProfile,
    /// All adopted Clip owners use this VM's one builtin context. Parentage alone
    /// does not establish this assertion. TextField owners are not admissible
    /// direct builtin targets in the existing VM API.
    pub one_builtin_context: bool,
    pub domain: NonZeroU64,
    /// Exactly the registered Scene movies, with distinct supplied identities.
    pub sources: BTreeMap<MovieId, LoadedMovieSourceId>,
    pub clips: BTreeMap<NodeId, ClipState>,
    pub fields: BTreeMap<NodeId, FieldState>,
    /// Exactly the named unsupported leaves; unnamed leaves stay in the Scene.
    pub unsupported_targets: BTreeMap<NodeId, DisplayTargetId>,
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub binding: movie_binding::Limits,
    pub display: display_state::Limits,
    pub text_host: textfield_host::Limits,
    pub text_state: textfield_state::Limits,
    /// Additional complete-parent replay work, separate from Validator's work.
    pub replay_tag_visits: usize,
    pub replay_objects: usize,
    pub text_definition_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            binding: movie_binding::Limits::default(),
            display: display_state::Limits::default(),
            text_host: textfield_host::Limits::default(),
            text_state: textfield_state::Limits::default(),
            replay_tag_visits: 2_000_000,
            replay_objects: 100_000,
            text_definition_bytes: 8 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Source,
    SuppliedState,
    Visibility,
    Children,
    Environment,
    CreateClip,
    AdoptRoot,
    BindDisplay,
    BindFocus,
    BuiltinContext,
    CreateTextHost,
    AdoptField,
    AdmitTextState,
    BindTextField,
    FreezeChildren,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdoptionFailure {
    pub node: Option<NodeId>,
    pub stage: Stage,
    /// Any error after mutation starts requires discarding the VM/session.
    /// A false value means this operation has not mutated the supplied VM.
    pub vm_may_be_partial: bool,
    pub message: String,
}
impl fmt::Display for AdoptionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "scene adoption {:?} at {:?}: {}{}",
            self.stage,
            self.node,
            self.message,
            if self.vm_may_be_partial {
                "; discard partial VM/session"
            } else {
                ""
            }
        )
    }
}
impl std::error::Error for AdoptionFailure {}
type Result<T> = std::result::Result<T, AdoptionFailure>;
fn failure(
    node: Option<NodeId>,
    stage: Stage,
    partial: bool,
    message: impl ToString,
) -> AdoptionFailure {
    AdoptionFailure {
        node,
        stage,
        vm_may_be_partial: partial,
        message: message.to_string(),
    }
}
fn pre(node: NodeId, stage: Stage, message: impl ToString) -> AdoptionFailure {
    failure(Some(node), stage, false, message)
}

pub struct AdoptedScene {
    pub root: AttachedClipId,
    pub clips: BTreeMap<NodeId, AttachedClipId>,
    pub fields: BTreeMap<NodeId, AttachedTextFieldId>,
    pub display: DisplayState,
    pub text_fields: TextFieldHost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Category {
    Clip,
    Field,
    Unsupported,
}
#[derive(Clone, Copy)]
struct Source {
    movie: MovieId,
    character: u16,
    version: u8,
}
struct Checked {
    display: DisplayState,
    clips: BTreeMap<NodeId, Source>,
    fields: BTreeMap<NodeId, Source>,
    children: BTreeMap<NodeId, Vec<(Text, NodeId, Category)>>,
}

fn charge(total: &mut usize, amount: usize, limit: usize, node: NodeId, name: &str) -> Result<()> {
    *total = total
        .checked_add(amount)
        .filter(|n| *n <= limit)
        .ok_or_else(|| pre(node, Stage::Source, format!("{name} budget exceeded")))?;
    Ok(())
}

fn local_definition<'a>(
    library: &'a MovieLibrary,
    scene: &Scene<'_>,
    node: NodeId,
    parent: Source,
) -> Result<(&'a crate::gfx::Movie, DefinitionRef)> {
    let n = &scene.nodes[node];
    let NodeKind::Definition(d) = n.kind else {
        return Err(pre(
            node,
            Stage::Source,
            "non-Clip leaf must be an exact local definition",
        ));
    };
    let raw = n
        .raw
        .as_ref()
        .ok_or_else(|| pre(node, Stage::Source, "leaf lacks placement"))?;
    if d.movie_id != parent.movie
        || n.placement_movie != parent.movie
        || raw.character_id != Some(d.character_id)
        || raw.class_name.is_some()
    {
        return Err(pre(
            node,
            Stage::Source,
            "leaf differs from local parent source/placement",
        ));
    }
    let movie = &library
        .entry(d.movie_id)
        .ok_or_else(|| pre(node, Stage::Source, "leaf movie absent"))?
        .asset
        .movie;
    let dictionary = movie
        .dictionary
        .get(&d.character_id)
        .ok_or_else(|| pre(node, Stage::Source, "leaf dictionary entry absent"))?;
    let tag = movie
        .tags
        .get(d.tag_index)
        .ok_or_else(|| pre(node, Stage::Source, "leaf tag absent"))?;
    if dictionary.id != d.character_id
        || dictionary.kind != d.kind
        || dictionary.tag_index != d.tag_index
        || dictionary.bounds != d.bounds
        || !matches!(tag.data, TagData::Definition { id, bounds } if id == d.character_id && bounds == d.bounds)
    {
        return Err(pre(
            node,
            Stage::Source,
            "leaf definition/dictionary/tag identity mismatch",
        ));
    }
    Ok((movie, d))
}

fn preflight(
    library: &MovieLibrary,
    scene: &Scene<'_>,
    input: &SuppliedSceneState,
    limits: Limits,
) -> Result<Checked> {
    if !input.one_builtin_context {
        return Err(failure(
            None,
            Stage::SuppliedState,
            false,
            "one builtin context must be explicitly asserted",
        ));
    }
    let display = DisplayState::from_scene(scene, limits.display)
        .map_err(|e| failure(None, Stage::Source, false, e))?;
    let mut validator = Validator::new(library, scene, input.binding_profile, limits.binding)
        .map_err(|e| failure(None, Stage::Source, false, e))?;
    if !input.sources.keys().eq(scene.movies.keys())
        || input.sources.values().collect::<BTreeSet<_>>().len() != input.sources.len()
    {
        return Err(failure(
            None,
            Stage::SuppliedState,
            false,
            "source map must cover exact movies with distinct identities",
        ));
    }
    let mut categories = BTreeMap::new();
    let mut clips = BTreeMap::new();
    let mut fields = BTreeMap::new();
    for node in &scene.nodes {
        let kind = if matches!(node.kind, NodeKind::Root)
            || node
                .kind
                .definition()
                .is_some_and(|d| d.kind == CharacterKind::Sprite)
        {
            let facts = validator
                .validate(node.id)
                .map_err(|e| pre(node.id, Stage::Source, e))?;
            clips.insert(
                node.id,
                Source {
                    movie: facts.source.movie_id,
                    character: facts.definition.map_or(0, |d| d.character_id),
                    version: facts.source.header.version,
                },
            );
            Category::Clip
        } else {
            let parent = node
                .parent
                .and_then(|id| clips.get(&id))
                .copied()
                .ok_or_else(|| {
                    pre(
                        node.id,
                        Stage::Source,
                        "leaf parent must be an admitted Clip",
                    )
                })?;
            let (movie, d) = local_definition(library, scene, node.id, parent)?;
            if movie.tags[d.tag_index].code == 37 {
                if d.kind != CharacterKind::Text || !node.unsupported.is_empty() {
                    return Err(pre(
                        node.id,
                        Stage::Source,
                        "EditText has wrong kind or unsupported structural state",
                    ));
                }
                fields.insert(
                    node.id,
                    Source {
                        movie: d.movie_id,
                        character: d.character_id,
                        version: movie.header.version,
                    },
                );
                Category::Field
            } else {
                if !node.children.is_empty() {
                    return Err(pre(
                        node.id,
                        Stage::Source,
                        "unsupported receiver must remain a leaf",
                    ));
                }
                Category::Unsupported
            }
        };
        categories.insert(node.id, kind);
    }
    if !input.clips.keys().eq(clips.keys()) || !input.fields.keys().eq(fields.keys()) {
        return Err(failure(
            None,
            Stage::SuppliedState,
            false,
            "supplied Clip/field keys must cover exact source categories",
        ));
    }
    let expected_unsupported: BTreeSet<_> = scene
        .nodes
        .iter()
        .filter(|n| {
            categories[&n.id] == Category::Unsupported
                && n.segment.as_ref().and_then(|p| p.name.as_ref()).is_some()
        })
        .map(|n| n.id)
        .collect();
    if !input
        .unsupported_targets
        .keys()
        .eq(expected_unsupported.iter())
    {
        return Err(failure(
            None,
            Stage::SuppliedState,
            false,
            "unsupported target keys must cover exact named leaves",
        ));
    }
    let mut identities = BTreeSet::new();
    for (id, target) in input
        .clips
        .iter()
        .map(|(id, s)| (*id, s.target))
        .chain(input.fields.iter().map(|(id, s)| (*id, s.target)))
        .chain(
            input
                .unsupported_targets
                .iter()
                .map(|(id, target)| (*id, *target)),
        )
    {
        if !identities.insert(target) {
            return Err(pre(
                id,
                Stage::SuppliedState,
                "duplicate supplied display target",
            ));
        }
    }
    for (&id, supplied) in &input.clips {
        if supplied.events.target_flags >> 24 != 0x52 {
            return Err(pre(
                id,
                Stage::SuppliedState,
                "Clip owner-offset high byte must be 0x52",
            ));
        }
        if (supplied.events.target_flags & 0x4000 != 0)
            != display
                .node(id)
                .map_err(|e| pre(id, Stage::Visibility, e))?
                .local_visible
        {
            return Err(pre(
                id,
                Stage::Visibility,
                "supplied bit14 differs from saved local visibility",
            ));
        }
    }
    let mut retained = 0usize;
    let mut definition_bytes = 0usize;
    for (&id, supplied) in &input.fields {
        if supplied.target_flags_6a >> 24 != 0x52
            || supplied.prototype.0 == 0
            || supplied.watch_state != TextFieldWatchState::Absent
        {
            return Err(pre(
                id,
                Stage::SuppliedState,
                "field owner/prototype/absent-watch profile is not supplied",
            ));
        }
        let mut at = scene.nodes[id].parent;
        let mut first = None;
        while let Some(parent) = at {
            let state = input.clips.get(&parent).ok_or_else(|| {
                pre(
                    id,
                    Stage::Environment,
                    "Environment chain contains a non-Clip",
                )
            })?;
            if state.events.target_flags & 0x400 != 0 {
                first = Some(parent);
                break;
            }
            at = scene.nodes[parent].parent;
        }
        if first != Some(supplied.environment_owner) {
            return Err(pre(
                id,
                Stage::Environment,
                "supplied Environment owner is not first marked ancestor",
            ));
        }
        let d = scene.nodes[id]
            .kind
            .definition()
            .ok_or_else(|| pre(id, Stage::Source, "field definition absent"))?;
        let movie = &library
            .entry(d.movie_id)
            .ok_or_else(|| pre(id, Stage::Source, "field movie absent"))?
            .asset
            .movie;
        let tag = &movie.tags[d.tag_index];
        charge(
            &mut definition_bytes,
            tag.body.len,
            limits.text_definition_bytes,
            id,
            "EditText definition bytes",
        )?;
        let bytes = tag
            .body
            .bytes(movie.bytes())
            .map_err(|e| pre(id, Stage::Source, e))?;
        let edit =
            EditText::parse(bytes, tag.body.offset).map_err(|e| pre(id, Stage::Source, e))?;
        if edit.id != d.character_id || Some(edit.bounds) != d.bounds {
            return Err(pre(
                id,
                Stage::Source,
                "original EditText bytes differ from definition identity/bounds",
            ));
        }
        for bytes in [
            supplied.snapshot.last_input.len(),
            supplied.snapshot.document.text.len(),
        ] {
            charge(
                &mut retained,
                bytes,
                limits.text_host.max_retained_text_bytes,
                id,
                "supplied retained text bytes",
            )?;
        }
    }
    if fields.len() > limits.text_host.max_fields || clips.len() > limits.text_host.max_contexts {
        return Err(failure(
            None,
            Stage::SuppliedState,
            false,
            "text host field/context budget exceeded",
        ));
    }
    let mut children = BTreeMap::new();
    let mut replay_tags = 0usize;
    let mut replay_objects = 0usize;
    for (&id, source) in &clips {
        let movie = &library
            .entry(source.movie)
            .ok_or_else(|| pre(id, Stage::Source, "parent movie absent"))?
            .asset
            .movie;
        let timeline = movie
            .timelines
            .get(&source.character)
            .ok_or_else(|| pre(id, Stage::Source, "parent timeline absent"))?;
        charge(
            &mut replay_tags,
            timeline.tags.len(),
            limits.replay_tag_visits,
            id,
            "parent replay tags",
        )?;
        if timeline.tags.iter().any(|&tag| tag >= movie.tags.len()) {
            return Err(pre(id, Stage::Source, "parent timeline tag outside movie"));
        }
        let frame = movie
            .static_frame(source.character, 0)
            .map_err(|e| pre(id, Stage::Source, e))?;
        charge(
            &mut replay_objects,
            frame.objects.len(),
            limits.replay_objects,
            id,
            "parent replay objects",
        )?;
        let source_children = &scene.nodes[id].children;
        if frame.objects.len() != source_children.len() {
            return Err(pre(
                id,
                Stage::Children,
                "Scene omits or adds saved children",
            ));
        }
        let mut names = BTreeSet::new();
        let mut named = Vec::new();
        for ((depth, raw), &child) in frame.objects.iter().zip(source_children) {
            let node = &scene.nodes[child];
            let part = node
                .segment
                .as_ref()
                .ok_or_else(|| pre(child, Stage::Children, "child placement segment absent"))?;
            if part.depth != *depth
                || part.name != raw.name
                || node.raw.as_ref() != Some(raw)
                || node.placement_movie != source.movie
                || node.parent != Some(id)
            {
                return Err(pre(
                    child,
                    Stage::Children,
                    "saved complete child order/placement/name differs",
                ));
            }
            if let Some(name) = &part.name {
                if name.is_empty() || name.contains('\0') || !names.insert(name.as_str()) {
                    return Err(pre(
                        child,
                        Stage::Children,
                        "saved child name is empty, NUL-bearing or duplicate",
                    ));
                }
                named.push((text(name), child, categories[&child]));
            }
        }
        children.insert(id, named);
    }
    Ok(Checked {
        display,
        clips,
        fields,
        children,
    })
}

/// The caller installs native Object/MovieClip/name identities beforehand and
/// establishes every supplied condition. This function registers no script.
/// Preflight failures leave VM untouched. Once mutation starts, ANY error requires
/// discarding the VM/session; no rollback or safe retry of partial state is claimed.
pub fn adopt_saved_scene(
    vm: &mut Vm,
    library: &MovieLibrary,
    scene: &Scene<'_>,
    input: SuppliedSceneState,
    limits: Limits,
) -> Result<AdoptedScene> {
    let checked = preflight(library, scene, &input, limits)?;
    adopt_checked(vm, scene, input, limits, checked)
}

fn adopt_checked(
    vm: &mut Vm,
    scene: &Scene<'_>,
    mut input: SuppliedSceneState,
    limits: Limits,
    mut checked: Checked,
) -> Result<AdoptedScene> {
    let mut clips = BTreeMap::new();
    let mut fields = BTreeMap::new();
    let err = |node, stage, e: String| failure(Some(node), stage, true, e);
    for (&movie, &source) in &input.sources {
        checked
            .display
            .bind_movie_source(source, movie)
            .map_err(|e| err(scene.root, Stage::BindDisplay, e.to_string()))?;
    }
    for (&id, source) in &checked.clips {
        let supplied = input.clips[&id];
        let parent = scene.nodes[id].parent.map(|p| clips[&p]);
        let clip = vm
            .create_attached_movie_clip_with_binding(
                AttachedClipSpec {
                    target: supplied.target,
                    source: AttachedClipSource {
                        movie: input.sources[&source.movie],
                        sprite_id: source.character,
                        header_version: source.version,
                    },
                    parent,
                    event_state: supplied.events,
                },
                AttachedMovieBinding::LocalSourceOrImportedLocalExport,
            )
            .map_err(|e| err(id, Stage::CreateClip, e.to_string()))?;
        if id == scene.root {
            // Preserve the VM's authoritative fresh-root phase guard unchanged.
            vm.adopt_attached_root(clip)
                .map_err(|e| err(id, Stage::AdoptRoot, e.to_string()))?;
        }
        checked
            .display
            .bind_attached(vm, clip, id)
            .map_err(|e| err(id, Stage::BindDisplay, e.to_string()))?;
        vm.bind_attached_focus_rect(clip, supplied.focus)
            .map_err(|e| err(id, Stage::BindFocus, e.to_string()))?;
        vm.bind_builtin_context_target(
            vm.attached_owner(clip)
                .map_err(|e| err(id, Stage::BuiltinContext, e.to_string()))?,
        )
        .map_err(|e| err(id, Stage::BuiltinContext, e.to_string()))?;
        clips.insert(id, clip);
    }
    let root = clips[&scene.root];
    let mut text_fields = TextFieldHost::new(vm.root(), input.domain, limits.text_host)
        .map_err(|e| err(scene.root, Stage::CreateTextHost, e))?;
    for (&id, &clip) in &clips {
        text_fields
            .bind_context(vm, clip)
            .map_err(|e| err(id, Stage::CreateTextHost, e))?;
    }
    for (&id, source) in &checked.fields {
        let supplied = input
            .fields
            .remove(&id)
            .ok_or_else(|| err(id, Stage::AdoptField, "missing preflight field".into()))?;
        let parent = scene.nodes[id]
            .parent
            .ok_or_else(|| err(id, Stage::AdoptField, "field parent absent".into()))?;
        let field = vm
            .adopt_attached_text_field_from_parent_movie(AttachedTextFieldSpec {
                target: supplied.target,
                source: AttachedTextFieldSource {
                    movie: input.sources[&source.movie],
                    edit_text_id: source.character,
                    header_version: source.version,
                },
                parent: clips[&parent],
                environment_owner: clips[&supplied.environment_owner],
                target_flags_6a: supplied.target_flags_6a,
                prototype: supplied.prototype,
                watch_state: supplied.watch_state,
            })
            .map_err(|e| err(id, Stage::AdoptField, e.to_string()))?;
        let owner = vm
            .attached_text_field_owner(field)
            .map_err(|e| err(id, Stage::AdoptField, e.to_string()))?;
        let state = TextFieldState::from_snapshot(
            AttachedIdentity {
                domain: input.domain,
                scene_node: id,
                avm_owner: owner.0,
            },
            supplied.profile,
            supplied.snapshot,
            limits.text_state,
        )
        .map_err(|e| err(id, Stage::AdmitTextState, e.to_string()))?;
        text_fields
            .bind(vm, &checked.display, field, state)
            .map_err(|e| err(id, Stage::BindTextField, e))?;
        fields.insert(id, field);
    }
    for (&id, names) in &checked.children {
        let children = names
            .iter()
            .map(|(name, node, category)| match category {
                Category::Clip => AttachedChild::Clip {
                    name: name.clone(),
                    clip: clips[node],
                },
                Category::Field => AttachedChild::TextField {
                    name: name.clone(),
                    field: fields[node],
                },
                Category::Unsupported => AttachedChild::Unsupported {
                    name: name.clone(),
                    target: input.unsupported_targets[node],
                },
            })
            .collect();
        vm.set_attached_children(clips[&id], children)
            .map_err(|e| err(id, Stage::FreezeChildren, e.to_string()))?;
    }
    Ok(AdoptedScene {
        root,
        clips,
        fields,
        display: checked.display,
        text_fields,
    })
}

#[cfg(test)]
mod tests {
    use super::textfield_state::{
        DefinitionHtmlProfile, Document, FormatProfile, Invalidation, ParagraphFormat, SidePath,
        TextFormat,
    };
    use super::*;
    use crate::{avm1::vm::Value, gfx::scene::MovieRef};

    mod fixture {
        // Fabricated test package only: all names/metadata are invented, movie bytes supplied.
        // PackageArchive owns its Vec, and MovieLibrary owns the parsed Movie: no file guard.
        use crate::{
            frontend::{
                AssetSet,
                library::{Limits, MovieLibrary},
            },
            upk::PackageArchive,
        };
        use std::io;

        pub fn synthetic_movie_library(gfx: &[u8]) -> io::Result<MovieLibrary> {
            fn words(out: &mut Vec<u8>, values: &[u32]) {
                for value in values {
                    out.extend_from_slice(&value.to_le_bytes());
                }
            }
            fn string(out: &mut Vec<u8>, value: &str) {
                words(out, &[value.len() as u32 + 1]);
                out.extend_from_slice(value.as_bytes());
                out.push(0);
            }
            fn patch(out: &mut [u8], at: usize, value: usize) {
                out[at..at + 4].copy_from_slice(&(value as u32).to_le_bytes());
            }
            let length = u32::try_from(gfx.len())
                .ok()
                .filter(|n| *n <= 1_048_576)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "synthetic movie exceeds 1 MiB test budget",
                    )
                })?;
            // UObject NetIndex, RawData ArrayProperty tag, array count/data, None sentinel.
            let mut object = Vec::new();
            words(&mut object, &[0, 4, 0, 5, 0, length + 4, 0, length]);
            object.extend_from_slice(gfx);
            words(&mut object, &[6, 0]);

            let mut package = Vec::new();
            words(&mut package, &[0x9e2a83c1, 868 | (3 << 16), 0]);
            string(&mut package, "None");
            words(&mut package, &[0, 7]); // package flags and name count
            let names_slot = package.len();
            words(&mut package, &[0, 1]); // name offset and export count
            let exports_slot = package.len();
            words(&mut package, &[0, 1]); // export offset and import count
            let imports_slot = package.len();
            // Import offset; dependency/GUID/thumbnail table fields, all absent.
            words(&mut package, &[0; 6]);
            package.extend_from_slice(&[0; 16]); // package GUID
            // Generations, engine/cooker, compression flags/chunks, source: all absent/zero.
            words(&mut package, &[0; 6]);

            let names = package.len();
            for name in [
                "Core",
                "Class",
                "SwfMovie",
                "SyntheticMovie",
                "RawData",
                "ArrayProperty",
                "None",
            ] {
                string(&mut package, name);
                words(&mut package, &[0, 0]); // name flags
            }
            let imports = package.len();
            // Core.Class named SwfMovie, no outer.
            words(&mut package, &[0, 0, 1, 0, 0, 2, 0]);
            let exports = package.len();
            // Imported class -1, no super/outer, SyntheticMovie name, no archetype/flags.
            words(
                &mut package,
                &[u32::MAX, 0, 0, 3, 0, 0, 0, 0, object.len() as u32],
            );
            let serial_slot = package.len();
            words(&mut package, &[0, 0, 0]); // serial offset, flags, generation count
            package.extend_from_slice(&[0; 16]); // export GUID
            words(&mut package, &[0]); // export package flags
            let header_end = package.len();
            for (slot, offset) in [
                (8, header_end),
                (names_slot, names),
                (exports_slot, exports),
                (imports_slot, imports),
                (serial_slot, header_end),
            ] {
                patch(&mut package, slot, offset);
            }
            package.extend_from_slice(&object);
            let assets = AssetSet::new(vec![PackageArchive::parse(package)?])?;
            MovieLibrary::load(&assets, &[0], &[], Limits::default())
        }
    }
    fn tag(out: &mut Vec<u8>, code: u16, body: &[u8]) {
        if body.len() < 63 {
            out.extend_from_slice(&((code << 6) | body.len() as u16).to_le_bytes());
        } else {
            out.extend_from_slice(&((code << 6) | 63).to_le_bytes());
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        }
        out.extend_from_slice(body);
    }
    fn place(out: &mut Vec<u8>, id: u16, depth: u16, name: &str, hidden: bool) {
        let mut b = vec![0x26];
        if hidden {
            b.push(0x20);
        }
        b.extend_from_slice(&depth.to_le_bytes());
        b.extend_from_slice(&id.to_le_bytes());
        b.push(0);
        b.extend_from_slice(name.as_bytes());
        b.push(0);
        if hidden {
            b.push(0);
        }
        tag(out, if hidden { 70 } else { 26 }, &b);
    }
    fn library(hidden: bool, duplicate: bool) -> MovieLibrary {
        let mut b = b"GFX\x0f\0\0\0\0\x08\0\0\x18\x01\0".to_vec();
        let mut edit = vec![3, 0, 8, 0, 0x80, 0];
        edit.extend_from_slice(b"\0Fixture text\0");
        tag(&mut b, 37, &edit);
        tag(&mut b, 2, &[4, 0, 8, 0]);
        let mut inner = vec![5, 0, 1, 0];
        place(&mut inner, 3, 1, "label", false);
        tag(&mut inner, 1, &[]);
        tag(&mut inner, 0, &[]);
        tag(&mut b, 39, &inner);
        let mut panel = vec![2, 0, 1, 0];
        place(&mut panel, 5, 1, "inner", false);
        tag(&mut panel, 1, &[]);
        tag(&mut panel, 0, &[]);
        tag(&mut b, 39, &panel);
        place(&mut b, 2, 1, "panel", hidden);
        place(
            &mut b,
            4,
            2,
            if duplicate { "panel" } else { "shape" },
            false,
        );
        tag(&mut b, 1, &[]);
        tag(&mut b, 0, &[]);
        let n = b.len() as u32;
        b[4..8].copy_from_slice(&n.to_le_bytes());
        fixture::synthetic_movie_library(&b).unwrap()
    }
    fn scene(lib: &MovieLibrary) -> Scene<'_> {
        Scene::first_frame(
            MovieRef::new(0, &lib.entry(0).unwrap().asset.movie),
            lib,
            Default::default(),
        )
        .unwrap()
    }
    fn supplied(scene: &Scene<'_>) -> SuppliedSceneState {
        let format = FormatProfile {
            text: TextFormat(NonZeroU64::new(11).unwrap()),
            paragraph: ParagraphFormat(NonZeroU64::new(12).unwrap()),
        };
        SuppliedSceneState {
            binding_profile: ConditionalProfile {
                final_ordinary_pair: true,
                no_resource_substitution: true,
                loader_compatibility_succeeded: true,
                selected_resource_non_null_and_eligible: true,
            },
            one_builtin_context: true,
            domain: NonZeroU64::new(100).unwrap(),
            sources: BTreeMap::from([(0, LoadedMovieSourceId(200))]),
            clips: (0..3)
                .map(|id| {
                    (
                        id,
                        ClipState {
                            target: DisplayTargetId(300 + id as u64),
                            events: AttachedEventState {
                                target_flags: 0x52000180
                                    | if id == 0 { 0x400 } else { 0 }
                                    | if scene.nodes[id].raw.as_ref().is_none_or(|r| r.visible) {
                                        0x4000
                                    } else {
                                        0
                                    },
                                load_flags: 0x123,
                                source_flags: 0x456,
                            },
                            focus: AttachedFocusRectState {
                                target_flags_b8: 0x180,
                            },
                        },
                    )
                })
                .collect(),
            fields: BTreeMap::from([(
                3,
                FieldState {
                    target: DisplayTargetId(303),
                    target_flags_6a: 0x52000180,
                    prototype: TextFieldPrototypeId(400),
                    watch_state: TextFieldWatchState::Absent,
                    environment_owner: 0,
                    profile: Profile {
                        watches: SidePath::Absent,
                        translation_service: SidePath::Absent,
                        stylesheet: SidePath::Absent,
                        editor: SidePath::Absent,
                        variable_binding: SidePath::Absent,
                        image_callbacks: SidePath::Absent,
                    },
                    snapshot: Snapshot {
                        raw_target_flags: 0,
                        raw_view_flags_149: 0,
                        last_input: "last supplied".into(),
                        document: Document {
                            text: "document supplied".into(),
                            run_format: Some(format),
                        },
                        current_defaults: format,
                        definition_html: DefinitionHtmlProfile {
                            run_format: format,
                            defaults_after_nonempty: format,
                        },
                        pending_invalidation: Invalidation {
                            view_mask: 0x80,
                            render_dirty: true,
                        },
                    },
                },
            )]),
            unsupported_targets: BTreeMap::from([(4, DisplayTargetId(304))]),
        }
    }
    fn vm() -> Vm {
        let mut vm = Vm::new(15, Default::default()).unwrap();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        vm
    }
    fn rejected(
        lib: &MovieLibrary,
        s: &Scene<'_>,
        input: SuppliedSceneState,
        limits: Limits,
        stage: Stage,
    ) {
        let mut vm = vm();
        let old_root = vm.root();
        let old_count = vm.object_count();
        let e = adopt_saved_scene(&mut vm, lib, s, input, limits)
            .err()
            .unwrap();
        assert_eq!(e.stage, stage, "{e}");
        assert!(!e.vm_may_be_partial, "{e}");
        assert_eq!(vm.root(), old_root);
        assert_eq!(vm.object_count(), old_count);
        assert_eq!(vm.instruction_count(), 0);
    }
    #[test]
    fn complete_adoption_preserves_supplied_state_and_local_visibility() {
        let lib = library(true, false);
        let s = scene(&lib);
        let input = supplied(&s);
        let expected = input.clone();
        let mut vm = vm();
        assert!(!s.nodes[2].effective_visible);
        assert!(s.nodes[2].raw.as_ref().unwrap().visible);
        let adopted = adopt_saved_scene(&mut vm, &lib, &s, input, Limits::default()).unwrap();
        assert_eq!(
            (
                adopted.display.nodes().len(),
                adopted.clips.len(),
                adopted.fields.len()
            ),
            (5, 3, 1)
        );
        assert_eq!(vm.root(), vm.attached_owner(adopted.root).unwrap());
        assert!(adopted.display.node(2).unwrap().local_visible);
        assert!(!adopted.display.node(2).unwrap().effective_visible);
        for (&id, &clip) in &adopted.clips {
            let info = vm.attached_info(clip).unwrap();
            assert_eq!(info.event_state, expected.clips[&id].events);
            assert!(info.child_snapshot_installed);
            assert_eq!(
                vm.attached_focus_rect(clip).unwrap().unwrap().state,
                expected.clips[&id].focus
            );
            assert!(vm.set_attached_children(clip, vec![]).is_err());
        }
        let field = adopted.fields[&3];
        let info = vm.attached_text_field_info(field).unwrap();
        assert_eq!(info.environment_owner, adopted.clips[&0]);
        assert_eq!(info.parent, adopted.clips[&2]);
        assert_eq!(
            adopted.text_fields.state(field).unwrap().snapshot(),
            &expected.fields[&3].snapshot
        );
        assert_eq!(
            adopted.display.movie_for_source(LoadedMovieSourceId(200)),
            Some(0)
        );
        assert_eq!(vm.instruction_count(), 0);
        assert_eq!(adopted.text_fields.statistics().calls, 0);
        assert_eq!(adopted.display.stats().property_calls, 0);
    }
    #[test]
    fn supplied_coverage_identity_context_and_ancestor_fail_before_mutation() {
        let lib = library(false, false);
        let s = scene(&lib);
        let base = supplied(&s);
        let mut a = base.clone();
        a.one_builtin_context = false;
        rejected(&lib, &s, a, Limits::default(), Stage::SuppliedState);
        let mut a = base.clone();
        a.sources.clear();
        rejected(&lib, &s, a, Limits::default(), Stage::SuppliedState);
        let mut a = base.clone();
        a.clips.remove(&2);
        rejected(&lib, &s, a, Limits::default(), Stage::SuppliedState);
        let mut a = base.clone();
        a.unsupported_targets.clear();
        rejected(&lib, &s, a, Limits::default(), Stage::SuppliedState);
        let mut a = base.clone();
        a.fields.get_mut(&3).unwrap().target = a.clips[&0].target;
        rejected(&lib, &s, a, Limits::default(), Stage::SuppliedState);
        let mut a = base.clone();
        a.fields.get_mut(&3).unwrap().prototype = TextFieldPrototypeId(0);
        rejected(&lib, &s, a, Limits::default(), Stage::SuppliedState);
        let mut a = base.clone();
        a.fields.get_mut(&3).unwrap().environment_owner = 2;
        rejected(&lib, &s, a, Limits::default(), Stage::Environment);
        let mut a = base.clone();
        a.clips.get_mut(&1).unwrap().events.target_flags |= 0x400;
        rejected(&lib, &s, a, Limits::default(), Stage::Environment);
        let mut a = base;
        a.clips.get_mut(&2).unwrap().events.target_flags &= !0x4000;
        rejected(&lib, &s, a, Limits::default(), Stage::Visibility);
    }
    #[test]
    fn altered_saved_source_duplicate_names_and_replay_budget_reject() {
        let lib = library(false, false);
        let mut s = scene(&lib);
        let a = supplied(&s);
        s.nodes[3].raw.as_mut().unwrap().character_id = Some(4);
        rejected(&lib, &s, a, Limits::default(), Stage::Source);
        let s = scene(&lib);
        let a = supplied(&s);
        let mut limits = Limits::default();
        limits.replay_tag_visits = 0;
        rejected(&lib, &s, a, limits, Stage::Source);
        let lib = library(false, true);
        let s = scene(&lib);
        let a = supplied(&s);
        rejected(&lib, &s, a, Limits::default(), Stage::Children);
    }
    #[test]
    fn fresh_root_guard_and_field_state_failure_require_discard() {
        let lib = library(false, false);
        let s = scene(&lib);
        let a = supplied(&s);
        let mut v = vm();
        let old = v.root();
        let count = v.object_count();
        v.set(old, "already_initialized", Value::Bool(true))
            .unwrap();
        let e = adopt_saved_scene(&mut v, &lib, &s, a, Limits::default())
            .err()
            .unwrap();
        assert_eq!(
            (e.node, e.stage, e.vm_may_be_partial),
            (Some(0), Stage::AdoptRoot, true)
        );
        assert_eq!(v.root(), old);
        assert!(v.object_count() > count);
        let mut a = supplied(&s);
        a.fields.get_mut(&3).unwrap().snapshot.document.text = "<unadmitted>".into();
        let mut v = vm();
        let e = adopt_saved_scene(&mut v, &lib, &s, a, Limits::default())
            .err()
            .unwrap();
        assert_eq!(
            (e.node, e.stage, e.vm_may_be_partial),
            (Some(3), Stage::AdmitTextState, true)
        );
        assert!(e.to_string().contains("discard partial VM/session"));
        assert_eq!(v.instruction_count(), 0);
    }
}
