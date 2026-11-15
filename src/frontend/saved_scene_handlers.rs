//! Install an explicitly selected saved Scene handler subset, without delivery.
//! Original loading/installer contracts: SAVED_CLIP_ACTION_LOADING.md and
//! SAVED_HANDLER_TABLE.md. Supplied live association is never inferred from files.

use super::{
    library::{MovieId, MovieLibrary},
    movie_binding::{self, ConditionalProfile, Validator},
    saved_handlers::{self, SavedHandlers},
    scene_adoption::AdoptedScene,
};
use crate::{
    avm1::{
        self, Program,
        vm::{AttachedClipId, AttachedMovieBinding, SavedActionId, SavedActionSource, Vm},
    },
    gfx::{
        CharacterKind, ClipAction, TagData,
        scene::{ActionKind, NodeId, NodeKind, Scene, TagRef},
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::Arc,
};

/// Explicit supported live route; no Default and no native initialization claim.
#[derive(Clone, Copy, Debug)]
pub struct Profile {
    pub binding: ConditionalProfile,
    /// AdoptedScene and all its IDs still belong to this exact VM/session.
    pub same_adopted_vm: bool,
    /// The instance's retained AS2 PO2/PO3 records are the saved placement list.
    /// Later timeline replacement/removal or resource substitution is excluded.
    pub retained_placement_records: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub binding: movie_binding::Limits,
    pub parser: avm1::Limits,
    pub table: saved_handlers::Limits,
    pub scene_bindings: usize,
    pub unbound_tags: usize,
    pub selected_records: usize,
    pub total_source_bytes: usize,
    pub placement_tag_visits: usize,
    pub clip_record_visits: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            binding: movie_binding::Limits::default(),
            parser: avm1::Limits::default(),
            table: saved_handlers::Limits::default(),
            scene_bindings: 100_000,
            unbound_tags: 100_000,
            selected_records: 4096,
            total_source_bytes: 16 * 1024 * 1024,
            placement_tag_visits: 1_000_000,
            clip_record_visits: 100_000,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Profile,
    Selection,
    Owner,
    Source,
    Program,
    Register,
    Install,
    Publish,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub scene_action: Option<usize>,
    pub node: Option<NodeId>,
    pub stage: Stage,
    /// Any error after VM registration begins requires session discard.
    pub vm_may_be_partial: bool,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "saved Scene handlers {:?} at action {:?}, node {:?}: {}{}",
            self.stage,
            self.scene_action,
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
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn error(
    action: Option<usize>,
    node: Option<NodeId>,
    stage: Stage,
    message: impl ToString,
) -> Error {
    Error {
        scene_action: action,
        node,
        stage,
        vm_may_be_partial: false,
        message: message.to_string(),
    }
}
fn charge(total: &mut usize, amount: usize, limit: usize, name: &'static str) -> Result<()> {
    *total = total
        .checked_add(amount)
        .filter(|n| *n <= limit)
        .ok_or_else(|| error(None, None, Stage::Source, name))?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordSource {
    pub scene_action: usize,
    pub owner_node: NodeId,
    pub owner: AttachedClipId,
    /// Placing movie; it need not be the receiver's resolved definition movie.
    pub movie: MovieId,
    pub header_version: u8,
    pub placement_tag: TagRef,
    pub record_index: usize,
    pub record_offset: usize,
    pub span: avm1::Span,
    /// Original per-record mask, before splitting into event keys.
    pub original_mask: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Installation {
    pub source: RecordSource,
    pub action: SavedActionId,
}
pub struct InstalledSceneHandlers {
    pub tables: BTreeMap<NodeId, SavedHandlers<SavedActionId>>,
    /// Canonical Scene order, which preserves each owner's serialized record order.
    pub installations: Vec<Installation>,
    /// All remaining Scene bindings, including non-clip/unsupported kinds.
    /// Being unselected makes no claim about support or lifecycle eligibility.
    pub unselected_bindings: Vec<usize>,
    /// Available action tags with no binding in this saved snapshot, unchanged.
    pub unbound_action_tags: Vec<TagRef>,
}
struct Prepared {
    source: RecordSource,
    program: Arc<Program>,
}
struct OwnerRecords {
    tag: TagRef,
    records: Vec<ClipAction>,
    action_indices: Vec<usize>,
}

/// Selection must be strictly increasing Scene action indices. Duplicate indices
/// reject; equal source bytes at distinct record/instance indices stay distinct.
/// No existing table is extended, and no cross-call deduplication is inferred.
/// All returned tables and IDs must stay associated with this exact VM/session.
/// Preflight leaves VM untouched. Every later failure requires discarding it;
/// tables are returned only after the entire batch succeeds.
pub fn install_selected(
    vm: &mut Vm,
    library: &MovieLibrary,
    scene: &Scene<'_>,
    adopted: &AdoptedScene,
    selected: &[usize],
    profile: Profile,
    limits: Limits,
) -> Result<InstalledSceneHandlers> {
    let prepared = preflight(vm, library, scene, adopted, selected, profile, limits)?;
    let mut tables = BTreeMap::new();
    let mut installations = Vec::new();
    for item in prepared {
        let mutation_error = |stage, message: String| {
            let mut e = error(
                Some(item.source.scene_action),
                Some(item.source.owner_node),
                stage,
                message,
            );
            e.vm_may_be_partial = true;
            e
        };
        let code = vm
            .register(item.program)
            .map_err(|e| mutation_error(Stage::Register, e.to_string()))?;
        let action = vm
            .install_saved_action(
                item.source.owner,
                SavedActionSource {
                    code,
                    span: item.source.span,
                },
                item.source.original_mask,
            )
            .map_err(|e| mutation_error(Stage::Install, e.to_string()))?;
        tables
            .entry(item.source.owner_node)
            .or_insert_with(|| SavedHandlers::new(item.source.owner, limits.table))
            .install_from_vm(vm, action)
            .map_err(|e| mutation_error(Stage::Publish, e.to_string()))?;
        installations.push(Installation {
            source: item.source,
            action,
        });
    }
    let chosen: BTreeSet<_> = selected.iter().copied().collect();
    Ok(InstalledSceneHandlers {
        tables,
        installations,
        unselected_bindings: (0..scene.actions.len())
            .filter(|i| !chosen.contains(i))
            .collect(),
        unbound_action_tags: scene.unbound_action_tags.clone(),
    })
}

fn preflight(
    vm: &Vm,
    library: &MovieLibrary,
    scene: &Scene<'_>,
    adopted: &AdoptedScene,
    selected: &[usize],
    profile: Profile,
    limits: Limits,
) -> Result<Vec<Prepared>> {
    if !profile.same_adopted_vm || !profile.retained_placement_records {
        return Err(error(
            None,
            None,
            Stage::Profile,
            "same VM and retained AS2 placement association must be supplied",
        ));
    }
    if scene.actions.len() > limits.scene_bindings
        || scene.unbound_action_tags.len() > limits.unbound_tags
        || selected.len() > limits.selected_records
    {
        return Err(error(
            None,
            None,
            Stage::Selection,
            "Scene/selection budget exceeded",
        ));
    }
    for (position, &index) in selected.iter().enumerate() {
        if index >= scene.actions.len() || position > 0 && selected[position - 1] >= index {
            return Err(error(
                Some(index),
                None,
                Stage::Selection,
                "selection must be in bounds, unique and strictly increasing",
            ));
        }
    }
    let mut validator = Validator::new(library, scene, profile.binding, limits.binding)
        .map_err(|e| error(None, None, Stage::Source, e))?;
    let clip_nodes: BTreeSet<_> = scene
        .nodes
        .iter()
        .filter(|n| {
            matches!(n.kind, NodeKind::Root)
                || n.kind
                    .definition()
                    .is_some_and(|d| d.kind == CharacterKind::Sprite)
        })
        .map(|n| n.id)
        .collect();
    if !adopted.clips.keys().eq(clip_nodes.iter())
        || adopted.display.root() != scene.root
        || adopted.display.nodes().len() != scene.nodes.len()
        || adopted.clips.get(&scene.root) != Some(&adopted.root)
        || vm.attached_owner(adopted.root).ok() != Some(vm.root())
    {
        return Err(error(
            None,
            None,
            Stage::Owner,
            "adopted root/Clip/Scene association mismatch",
        ));
    }
    let mut clip_ids = BTreeSet::new();
    for (&node, &clip) in &adopted.clips {
        let fail = |message: String| error(None, Some(node), Stage::Owner, message);
        let facts = validator.validate(node).map_err(|e| fail(e.to_string()))?;
        let info = vm.attached_info(clip).map_err(|e| fail(e.to_string()))?;
        let source = &scene.nodes[node];
        let saved = &adopted
            .display
            .node(node)
            .map_err(|e| fail(e.to_string()))?
            .source;
        if !clip_ids.insert(clip)
            || adopted.display.node_for_attached(clip) != Some(node)
            || adopted.display.bindings().get(&info.owner) != Some(&node)
            || adopted.display.movie_for_source(info.source.movie) != Some(facts.source.movie_id)
            || info.source.sprite_id != facts.definition.map_or(0, |d| d.character_id)
            || info.source.header_version != facts.source.header.version
            || info.environment_movie != info.source.movie
            || info.environment_header_version != info.source.header_version
            || info.parent != source.parent.and_then(|p| adopted.clips.get(&p).copied())
            || info.binding != Some(AttachedMovieBinding::LocalSourceOrImportedLocalExport)
            || !info.child_snapshot_installed
            || saved.id != source.id
            || saved.parent != source.parent
            || saved.children != source.children
            || saved.segment != source.segment
            || saved.placement_movie != source.placement_movie
            || saved.raw != source.raw
            || saved.kind != source.kind
            || saved.actions != source.actions
        {
            return Err(fail(
                "live owner/source/parent or adopted saved node differs".into(),
            ));
        }
    }
    // Validate every backlink, so the remaining-index report cannot hide an
    // omitted, duplicated or reassigned binding in a caller-modified Scene.
    let mut seen = vec![false; scene.actions.len()];
    for node in &scene.nodes {
        let mut previous = None;
        for &index in &node.actions {
            let Some(action) = scene.actions.get(index) else {
                return Err(error(
                    Some(index),
                    Some(node.id),
                    Stage::Source,
                    "action backlink outside Scene",
                ));
            };
            if action.owner != node.id || seen[index] || previous.is_some_and(|p| p >= index) {
                return Err(error(
                    Some(index),
                    Some(node.id),
                    Stage::Source,
                    "action owner/order/backlink mismatch",
                ));
            }
            seen[index] = true;
            previous = Some(index);
        }
    }
    if seen.contains(&false) {
        return Err(error(
            None,
            None,
            Stage::Source,
            "Scene action lacks its owner backlink",
        ));
    }
    let mut owners = BTreeMap::<NodeId, OwnerRecords>::new();
    let mut tags = 0usize;
    let mut records = 0usize;
    let mut source_bytes = 0usize;
    let mut counts = BTreeMap::<NodeId, (usize, usize)>::new();
    let mut prepared = Vec::new();
    for &index in selected {
        let action = &scene.actions[index];
        let node = action.owner;
        let fail = |stage, message: String| error(Some(index), Some(node), stage, message);
        let ActionKind::Clip {
            event_flags,
            key_code,
        } = action.kind
        else {
            return Err(fail(
                Stage::Selection,
                "selected binding is not a ClipAction".into(),
            ));
        };
        if !matches!(event_flags, 1 | 0x40000 | 0x40001) || key_code.is_some() {
            return Err(fail(
                Stage::Selection,
                "selected mask/key is outside Load/Construct profile".into(),
            ));
        }
        let &owner = adopted
            .clips
            .get(&node)
            .ok_or_else(|| fail(Stage::Owner, "selected owner is not an adopted Clip".into()))?;
        for clip in [owner, adopted.root] {
            if vm
                .attached_event_state(clip)
                .map_err(|e| fail(Stage::Owner, e.to_string()))?
                .target_flags
                & 0x80
                == 0
            {
                return Err(fail(
                    Stage::Owner,
                    "selected/root case3 eligibility bit is absent".into(),
                ));
            }
        }
        if !owners.contains_key(&node) {
            let facts = validator
                .validate(node)
                .map_err(|e| fail(Stage::Source, e.to_string()))?;
            let placement = facts.placement.as_ref().ok_or_else(|| {
                fail(Stage::Source, "selected Clip has no saved placement".into())
            })?;
            let mut operative = None;
            for tag_ref in &placement.contributing_tags {
                charge(
                    &mut tags,
                    1,
                    limits.placement_tag_visits,
                    "placement tag budget exceeded",
                )?;
                let movie = &library
                    .entry(tag_ref.movie_id)
                    .ok_or_else(|| fail(Stage::Source, "placement movie absent".into()))?
                    .asset
                    .movie;
                let tag = movie
                    .tags
                    .get(tag_ref.tag_index)
                    .ok_or_else(|| fail(Stage::Source, "placement tag absent".into()))?;
                if let TagData::Place(p) = &tag.data {
                    if p.has_clip_actions {
                        operative = Some((*tag_ref, p));
                    }
                }
            }
            let (tag_ref, p) = operative.ok_or_else(|| {
                fail(
                    Stage::Source,
                    "no retained placement ClipActions list".into(),
                )
            })?;
            let raw = scene.nodes[node]
                .raw
                .as_ref()
                .ok_or_else(|| fail(Stage::Source, "selected owner lacks raw placement".into()))?;
            if !raw.has_clip_actions || raw.clip_actions != p.clip_actions {
                return Err(fail(
                    Stage::Source,
                    "retained ClipActions differ from last contributing placement list".into(),
                ));
            }
            charge(
                &mut records,
                p.clip_actions.len(),
                limits.clip_record_visits,
                "clip record budget exceeded",
            )?;
            let movie = &library.entry(tag_ref.movie_id).unwrap().asset.movie;
            let tag = &movie.tags[tag_ref.tag_index];
            let body = tag
                .body
                .bytes(movie.bytes())
                .map_err(|e| fail(Stage::Source, e.to_string()))?;
            if !(7..=19).contains(&movie.header.version)
                || !matches!(tag.code, 26 | 70)
                || body.first().is_none_or(|flags| flags & 0x80 == 0)
                || tag.code == 70 && body.get(1).is_none_or(|flags| flags & (0x10 | 0x40) != 0)
            {
                return Err(fail(
                    Stage::Source,
                    "placement is outside recovered AS2 PO2/PO3 loading profile".into(),
                ));
            }
            let action_indices: Vec<_> = scene.nodes[node]
                .actions
                .iter()
                .copied()
                .filter(|&i| matches!(scene.actions[i].kind, ActionKind::Clip { .. }))
                .collect();
            if action_indices.len() != p.clip_actions.len() {
                return Err(fail(
                    Stage::Source,
                    "Scene Clip bindings do not cover retained list".into(),
                ));
            }
            for (&other_index, record) in action_indices.iter().zip(&p.clip_actions) {
                let other = &scene.actions[other_index];
                if other.movie_id != tag_ref.movie_id
                    || other.source_tag.is_some_and(|t| t != tag_ref)
                    || other.bytes != record.actions
                    || other.kind
                        != (ActionKind::Clip {
                            event_flags: record.event_flags,
                            key_code: record.key_code,
                        })
                {
                    return Err(fail(
                        Stage::Source,
                        "Scene Clip binding order/source/record differs".into(),
                    ));
                }
            }
            owners.insert(
                node,
                OwnerRecords {
                    tag: tag_ref,
                    records: p.clip_actions.clone(),
                    action_indices,
                },
            );
        }
        let owner_records = &owners[&node];
        let record_index = owner_records
            .action_indices
            .binary_search(&index)
            .map_err(|_| {
                fail(
                    Stage::Source,
                    "selected record absent from owner list".into(),
                )
            })?;
        let record = &owner_records.records[record_index];
        let movie = &library
            .entry(action.movie_id)
            .ok_or_else(|| fail(Stage::Source, "action source movie absent".into()))?
            .asset
            .movie;
        let tag = &movie.tags[owner_records.tag.tag_index];
        let span = avm1::Span {
            offset: record.actions.offset,
            len: record.actions.len,
        };
        // This profile has 32-bit masks and no KeyPress byte. Check the retained
        // record envelope directly against original bytes; never use AllEventFlags.
        let record_offset = span
            .offset
            .checked_sub(8)
            .ok_or_else(|| fail(Stage::Source, "record envelope underflow".into()))?;
        let end = span
            .end()
            .ok_or_else(|| fail(Stage::Source, "action span overflow".into()))?;
        let tag_end = tag
            .body
            .offset
            .checked_add(tag.body.len)
            .ok_or_else(|| fail(Stage::Source, "tag span overflow".into()))?;
        if record_offset < tag.body.offset || end > tag_end {
            return Err(fail(
                Stage::Source,
                "record lies outside placement tag".into(),
            ));
        }
        let envelope = movie
            .bytes()
            .get(record_offset..span.offset)
            .ok_or_else(|| fail(Stage::Source, "record envelope outside movie".into()))?;
        if u32::from_le_bytes(envelope[..4].try_into().unwrap()) != event_flags
            || u32::from_le_bytes(envelope[4..].try_into().unwrap()) as usize != span.len
        {
            return Err(fail(
                Stage::Source,
                "original record mask/size mismatch".into(),
            ));
        }
        let bytes = span
            .bytes(movie.bytes())
            .map_err(|e| fail(Stage::Source, e.to_string()))?;
        if bytes.first().is_none_or(|b| *b == 0) {
            return Err(fail(
                Stage::Program,
                "selected source fails installer first-byte gate".into(),
            ));
        }
        charge(
            &mut source_bytes,
            bytes.len(),
            limits.total_source_bytes,
            "selected source byte budget exceeded",
        )?;
        let count = counts.entry(node).or_insert((0, 0));
        count.0 = count
            .0
            .checked_add(1)
            .filter(|n| *n <= limits.table.installations)
            .ok_or_else(|| {
                fail(
                    Stage::Selection,
                    "per-owner installation budget exceeded".into(),
                )
            })?;
        count.1 = count
            .1
            .checked_add(event_flags.count_ones() as usize)
            .filter(|n| *n <= limits.table.references)
            .ok_or_else(|| {
                fail(
                    Stage::Selection,
                    "per-owner handler reference budget exceeded".into(),
                )
            })?;
        let program = Program::parse_with_limits(bytes, span.offset, limits.parser)
            .map_err(|e| fail(Stage::Program, e.to_string()))?;
        if program.span != span {
            return Err(fail(
                Stage::Program,
                "parsed full source span mismatch".into(),
            ));
        }
        prepared.push(Prepared {
            source: RecordSource {
                scene_action: index,
                owner_node: node,
                owner,
                movie: action.movie_id,
                header_version: movie.header.version,
                placement_tag: owner_records.tag,
                record_index,
                record_offset,
                span,
                original_mask: event_flags,
            },
            program: Arc::new(program),
        });
    }
    Ok(prepared)
}

#[cfg(test)]
mod tests {
    use super::super::scene_adoption::{self, ClipState, SuppliedSceneState};
    use super::*;
    use crate::{
        avm1::vm::{
            AttachedEventKind, AttachedEventState, AttachedFocusRectState, DisplayTargetId,
            LoadedMovieSourceId, SavedMode,
        },
        gfx::scene::MovieRef,
    };
    use std::num::NonZeroU64;
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
    fn place(out: &mut Vec<u8>, character: u16, depth: u16, name: &str) {
        let mut b = vec![0x22];
        b.extend_from_slice(&depth.to_le_bytes());
        b.extend_from_slice(&character.to_le_bytes());
        b.extend_from_slice(name.as_bytes());
        b.push(0);
        tag(out, 26, &b);
    }
    fn clips(out: &mut Vec<u8>, update: bool, masks: &[u32], code: &[u8]) {
        let mut b = vec![if update { 0x81 } else { 0xa2 }];
        b.extend_from_slice(&1u16.to_le_bytes());
        if !update {
            b.extend_from_slice(&1u16.to_le_bytes());
            b.extend_from_slice(b"child\0");
        }
        b.extend_from_slice(&0u16.to_le_bytes());
        b.extend_from_slice(&masks.iter().fold(0, |a, b| a | b).to_le_bytes());
        for mask in masks {
            b.extend_from_slice(&mask.to_le_bytes());
            b.extend_from_slice(&(code.len() as u32).to_le_bytes());
            b.extend_from_slice(code);
        }
        b.extend_from_slice(&0u32.to_le_bytes());
        tag(out, 26, &b);
    }
    fn library(code: &[u8]) -> MovieLibrary {
        let mut b = b"GFX\x0f\0\0\0\0\x08\0\0\x18\x01\0".to_vec();
        let mut empty = vec![1, 0, 1, 0];
        tag(&mut empty, 1, &[]);
        tag(&mut empty, 0, &[]);
        tag(&mut b, 39, &empty);
        let mut container = vec![2, 0, 1, 0];
        clips(&mut container, false, &[0x40000], &[7, 0]);
        clips(&mut container, true, &[1, 0x40000, 0x40001, 8], code);
        tag(&mut container, 1, &[]);
        tag(&mut container, 0, &[]);
        tag(&mut b, 39, &container);
        place(&mut b, 2, 1, "first");
        place(&mut b, 2, 2, "second");
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
    fn binding() -> ConditionalProfile {
        ConditionalProfile {
            final_ordinary_pair: true,
            no_resource_substitution: true,
            loader_compatibility_succeeded: true,
            selected_resource_non_null_and_eligible: true,
        }
    }
    fn profile() -> Profile {
        Profile {
            binding: binding(),
            same_adopted_vm: true,
            retained_placement_records: true,
        }
    }
    fn adopt(
        lib: &MovieLibrary,
        s: &Scene<'_>,
        vm_limits: crate::avm1::vm::Limits,
    ) -> (Vm, AdoptedScene) {
        let mut vm = Vm::new(15, vm_limits).unwrap();
        vm.install_object_constructor().unwrap();
        vm.install_movie_clip_constructor().unwrap();
        let supplied = SuppliedSceneState {
            binding_profile: binding(),
            one_builtin_context: true,
            domain: NonZeroU64::new(17).unwrap(),
            sources: BTreeMap::from([(0, LoadedMovieSourceId(23))]),
            clips: s
                .nodes
                .iter()
                .map(|n| {
                    (
                        n.id,
                        ClipState {
                            target: DisplayTargetId(100 + n.id as u64),
                            events: AttachedEventState {
                                target_flags: 0x52004180,
                                load_flags: 0,
                                source_flags: 0,
                            },
                            focus: AttachedFocusRectState {
                                target_flags_b8: 0x180,
                            },
                        },
                    )
                })
                .collect(),
            fields: BTreeMap::new(),
            unsupported_targets: BTreeMap::new(),
        };
        let a = scene_adoption::adopt_saved_scene(&mut vm, lib, s, supplied, Default::default())
            .unwrap();
        (vm, a)
    }
    fn rejection(
        lib: &MovieLibrary,
        s: &Scene<'_>,
        mut vm: Vm,
        a: AdoptedScene,
        selected: &[usize],
        profile: Profile,
        limits: Limits,
        stage: Stage,
    ) {
        let root = vm.root();
        let objects = vm.object_count();
        let e = install_selected(&mut vm, lib, s, &a, selected, profile, limits)
            .err()
            .unwrap();
        assert_eq!(e.stage, stage, "{e}");
        assert!(!e.vm_may_be_partial, "{e}");
        assert_eq!(vm.root(), root);
        assert_eq!(vm.object_count(), objects);
        assert_eq!(vm.instruction_count(), 0);
        assert!(vm.saved_action_info(SavedActionId(0)).is_err());
        assert_eq!(
            vm.register(Arc::new(Program::parse(&[0]).unwrap()))
                .unwrap()
                .0,
            0
        );
    }
    #[test]
    fn exact_updated_records_split_masks_and_repeated_instances_preserve_wrappers() {
        let lib = library(&[6, 0, 0xfa, 0xfb]);
        let s = scene(&lib);
        assert_eq!(s.actions.len(), 8);
        let (mut vm, a) = adopt(&lib, &s, Default::default());
        let selected = [0, 1, 2, 4, 5, 6];
        let result = install_selected(
            &mut vm,
            &lib,
            &s,
            &a,
            &selected,
            profile(),
            Limits::default(),
        )
        .unwrap();
        assert_eq!(result.unselected_bindings, [3, 7]);
        assert_eq!(result.unbound_action_tags, s.unbound_action_tags);
        assert_eq!(result.installations.len(), 6);
        assert_eq!(result.tables.len(), 2);
        let updated_tag = lib
            .entry(0)
            .unwrap()
            .asset
            .movie
            .tags
            .iter()
            .enumerate()
            .filter(|(_, t)| matches!(&t.data,TagData::Place(p) if p.has_clip_actions))
            .last()
            .unwrap()
            .0;
        for pair in result.installations.chunks(3) {
            let table = &result.tables[&pair[0].source.owner_node];
            assert_eq!(
                table.handlers(AttachedEventKind::Load).unwrap(),
                [pair[0].action, pair[2].action]
            );
            assert_eq!(
                table.handlers(AttachedEventKind::Construct).unwrap(),
                [pair[1].action, pair[2].action]
            );
            for (i, record) in pair.iter().enumerate() {
                let actual = vm.saved_action_info(record.action).unwrap();
                assert_eq!(actual.owner, record.source.owner);
                assert_eq!(actual.source.span, record.source.span);
                assert_eq!(record.source.span.len, 4);
                assert_eq!(actual.original_mask, [1, 0x40000, 0x40001][i]);
                assert_eq!(
                    actual.mode,
                    if i == 2 {
                        SavedMode::Mode3
                    } else {
                        SavedMode::Mode4
                    }
                );
                assert_eq!(actual.first_pool_offset, None);
                assert_eq!(record.source.placement_tag.tag_index, updated_tag);
                assert_eq!(record.source.record_index, i);
                let program = Program::parse_at(
                    record
                        .source
                        .span
                        .bytes(lib.entry(0).unwrap().asset.movie.bytes())
                        .unwrap(),
                    record.source.span.offset,
                )
                .unwrap();
                assert_eq!(program.trailing.len, 2);
            }
        }
        assert_eq!(
            result.installations[0].source.span,
            result.installations[3].source.span
        );
        assert_ne!(
            result.installations[0].action,
            result.installations[3].action
        );
        assert_ne!(
            result.installations[0].source.owner,
            result.installations[3].source.owner
        );
        assert_eq!(vm.instruction_count(), 0);
    }
    #[test]
    fn selection_profile_mask_and_owner_reject_before_registration() {
        let lib = library(&[6, 0]);
        let s = scene(&lib);
        for list in [&[0, 0][..], &[2, 1], &[8], &[3]] {
            let (vm, a) = adopt(&lib, &s, Default::default());
            rejection(
                &lib,
                &s,
                vm,
                a,
                list,
                profile(),
                Limits::default(),
                Stage::Selection,
            );
        }
        let (vm, a) = adopt(&lib, &s, Default::default());
        let mut p = profile();
        p.same_adopted_vm = false;
        rejection(&lib, &s, vm, a, &[0], p, Limits::default(), Stage::Profile);
        let (vm, mut a) = adopt(&lib, &s, Default::default());
        a.clips.insert(2, a.clips[&1]);
        rejection(
            &lib,
            &s,
            vm,
            a,
            &[0],
            profile(),
            Limits::default(),
            Stage::Owner,
        );
        let (vm, a) = adopt(&lib, &s, Default::default());
        let mut limits = Limits::default();
        limits.table.references = 1;
        rejection(&lib, &s, vm, a, &[2], profile(), limits, Stage::Selection);
    }
    #[test]
    fn altered_binding_and_uninstallable_source_reject_without_publication() {
        let lib = library(&[6, 0]);
        let mut s = scene(&lib);
        let (vm, a) = adopt(&lib, &s, Default::default());
        s.actions[0].bytes.len -= 1;
        rejection(
            &lib,
            &s,
            vm,
            a,
            &[0],
            profile(),
            Limits::default(),
            Stage::Source,
        );
        let s = scene(&lib);
        let (vm, a) = adopt(&lib, &s, Default::default());
        let mut limits = Limits::default();
        limits.total_source_bytes = 1;
        rejection(&lib, &s, vm, a, &[0], profile(), limits, Stage::Source);
        for code in [&[0][..], &[0x96, 4, 0]] {
            let lib = library(code);
            let s = scene(&lib);
            let (vm, a) = adopt(&lib, &s, Default::default());
            rejection(
                &lib,
                &s,
                vm,
                a,
                &[0],
                profile(),
                Limits::default(),
                Stage::Program,
            );
        }
    }
    #[test]
    fn registration_failure_after_first_wrapper_requires_vm_discard() {
        let lib = library(&[6, 0]);
        let s = scene(&lib);
        let mut limits = crate::avm1::vm::Limits::default();
        limits.programs = 1;
        let (mut vm, a) = adopt(&lib, &s, limits);
        let e = install_selected(&mut vm, &lib, &s, &a, &[0, 1], profile(), Limits::default())
            .err()
            .unwrap();
        assert_eq!(
            (e.scene_action, e.stage, e.vm_may_be_partial),
            (Some(1), Stage::Register, true)
        );
        assert!(e.to_string().contains("discard partial VM/session"));
        assert!(vm.saved_action_info(SavedActionId(0)).is_ok());
        assert_eq!(vm.instruction_count(), 0);
    }
}
