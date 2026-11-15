//! Saved frontend Kismet preparation. No scheduling, defaults or import loading.
use crate::upk::{
    Package, PackageArchive,
    level::{LevelLinkSlot, LevelObject},
    objects::{self, Property, PropertyList, Span, Value},
    world::{WorldLinkSlot, WorldObject},
};
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    fmt, io,
};

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub exports: usize,
    pub selected_exports: usize,
    pub serialized_bytes: usize,
    pub properties: usize,
    pub array_items: usize,
    pub nesting_depth: usize,
    pub outer_depth: usize,
    pub identity_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            exports: 65_536,
            selected_exports: 4_096,
            serialized_bytes: 8 * 1024 * 1024,
            properties: 65_536,
            array_items: 65_536,
            nesting_depth: 8,
            outer_depth: 128,
            identity_bytes: 4 * 1024 * 1024,
        }
    }
}
#[derive(Debug)]
pub enum Error {
    Budget {
        resource: &'static str,
        limit: usize,
    },
    Invalid {
        export: Option<usize>,
        message: String,
    },
    Unsupported {
        export: usize,
        message: String,
    },
    Read {
        export: usize,
        source: io::Error,
    },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Budget { resource, limit } => {
                write!(f, "startup map {resource} budget exceeded ({limit})")
            }
            Self::Invalid { export, message } => {
                write!(f, "invalid startup map export {export:?}: {message}")
            }
            Self::Unsupported { export, message } => {
                write!(f, "unsupported startup map export {export}: {message}")
            }
            Self::Read { export, source } => {
                write!(f, "cannot read startup map export {export}: {source}")
            }
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            _ => None,
        }
    }
}
type Result<T> = std::result::Result<T, Error>;
fn invalid(export: Option<usize>, message: impl Into<String>) -> Error {
    Error::Invalid {
        export,
        message: message.into(),
    }
}
fn charge(value: &mut usize, amount: usize, limit: usize, resource: &'static str) -> Result<()> {
    *value = value
        .checked_add(amount)
        .filter(|v| *v <= limit)
        .ok_or(Error::Budget { resource, limit })?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceKind {
    Import {
        class_package: String,
        class_name: String,
        outer_reference: i32,
    },
    Export {
        export_index: usize,
        class_reference: i32,
        outer_reference: i32,
        archetype_reference: i32,
    },
}
/// Local package identity; an imported name does not establish a loaded object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reference {
    pub value: i32,
    pub object_path: String,
    pub class_name: String,
    pub kind: ReferenceKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectReference {
    Null,
    Reference(Reference),
}

/// Export-relative spans. A BoolProperty's value is a tag byte, not its payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Origin {
    pub export_index: usize,
    pub serial_offset: usize,
    pub tag: Span,
    pub payload: Span,
    pub encoded_value: Span,
}
impl Origin {
    pub fn logical_value_offset(&self) -> Option<usize> {
        self.serial_offset.checked_add(self.encoded_value.offset)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Saved<T> {
    pub value: T,
    pub origin: Origin,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArrayElement<T> {
    pub value: T,
    pub array_origin: Origin,
    pub encoded_value: Span,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnresolvedResource {
    pub reference: Saved<ObjectReference>,
    pub expected_class: &'static str,
}

/// Full tags preserve every uninterpreted property and its source span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Object {
    pub identity: Reference,
    pub class_identity: Reference,
    pub serial_offset: usize,
    pub serial_size: usize,
    pub net_index: i32,
    pub properties: PropertyList,
    pub parent_sequence: Option<Saved<ObjectReference>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionLink {
    pub properties: PropertyList,
    pub target: Saved<ObjectReference>,
    pub input_index: Saved<usize>,
}
/// None always means no serialized property; it never supplies a native default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Port {
    pub properties: PropertyList,
    pub description: Option<Saved<String>>,
    pub disabled: Option<Saved<bool>>,
    pub disabled_pie: Option<Saved<bool>>,
    pub activation_delay_bits: Option<Saved<u32>>,
    pub has_impulse: Option<Saved<bool>>,
    pub queued_activations: Option<Saved<i32>>,
    pub linked_op: Option<Saved<ObjectReference>>,
    pub links: Option<Saved<Vec<ExecutionLink>>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VariableLink {
    pub properties: PropertyList,
    pub description: Option<Saved<String>>,
    pub expected_type: Option<Saved<ObjectReference>>,
    pub linked_variables: Option<Saved<Vec<ArrayElement<ObjectReference>>>>,
    pub writable: Option<Saved<bool>>,
    pub min_vars: Option<Saved<i32>>,
    pub max_vars: Option<Saved<i32>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sequence {
    pub object: Object,
    pub members: Option<Saved<Vec<ArrayElement<ObjectReference>>>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelLoaded {
    pub object: Object,
    pub outputs: Option<Saved<Vec<Port>>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenMovie {
    pub object: Object,
    pub movie: Option<UnresolvedResource>,
    pub player_class: Option<UnresolvedResource>,
    pub start_paused: Option<Saved<bool>>,
    pub take_focus: Option<Saved<bool>>,
    pub capture_input: Option<Saved<bool>>,
    pub display_with_hud_off: Option<Saved<bool>>,
    pub inputs: Option<Saved<Vec<Port>>>,
    pub outputs: Option<Saved<Vec<Port>>>,
    pub variables: Option<Saved<Vec<VariableLink>>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerVariable {
    pub object: Object,
    pub player_index: Option<Saved<i32>>,
    pub all_players: Option<Saved<bool>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnpreparedTarget {
    pub identity: Reference,
    pub class_identity: Option<Reference>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub selected_exports: usize,
    pub serialized_bytes: usize,
    pub properties: usize,
    pub array_items: usize,
    pub identity_bytes: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartupMap {
    pub export_count: usize,
    pub sequences: Vec<Sequence>,
    pub level_loaded: Vec<LevelLoaded>,
    pub open_movies: Vec<OpenMovie>,
    pub player_variables: Vec<PlayerVariable>,
    /// Exact referenced operations whose bodies are not prepared or executed.
    pub unprepared_targets: Vec<UnpreparedTarget>,
    pub usage: Usage,
}

/// A native field's span is relative to its owning export, without a property tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeOrigin {
    pub export_index: usize,
    pub serial_offset: usize,
    pub encoded_value: Span,
}
impl NativeOrigin {
    pub fn logical_value_offset(&self) -> Option<usize> {
        self.serial_offset.checked_add(self.encoded_value.offset)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeReference {
    pub value: ObjectReference,
    pub origin: NativeOrigin,
}

/// The native GetGameSequence rule uses only the first +0xe8 array element.
/// Later nonnull elements never replace an empty array or its first null entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MainSequence {
    EmptyArray,
    FirstNull {
        link: NativeReference,
    },
    Local {
        link: NativeReference,
        /// Zero-based export index; the encoded reference itself is one-based.
        export_index: usize,
        /// Index within WorldStartupMap::inventory.sequences.
        inventory_index: usize,
    },
}

/// Explicit saved World -> PersistentLevel -> first game-sequence selection.
/// This does not establish the runtime current Level or execute any sequence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldStartupMap {
    pub world_identity: Reference,
    pub world_class_identity: Reference,
    pub world: WorldObject,
    pub persistent_level: NativeReference,
    pub level_identity: Reference,
    pub level_class_identity: Reference,
    pub level: LevelObject,
    pub main_sequence: MainSequence,
    /// The unchanged all-export inventory, including sequences outside this route.
    pub inventory: StartupMap,
    /// Aggregate work, including the two native exports and all inventory work.
    pub usage: Usage,
}

impl WorldStartupMap {
    pub fn selected_sequence(&self) -> Option<&Sequence> {
        match self.main_sequence {
            MainSequence::Local {
                inventory_index, ..
            } => self.inventory.sequences.get(inventory_index),
            _ => None,
        }
    }
}

/// Prepare the saved route for one caller-selected, zero-based World export.
/// Imported links are not loaded. Null/wrong PersistentLevel and wrong local
/// identities are rejected; an absent main sequence remains an explicit result.
pub fn prepare_world(
    archive: &PackageArchive,
    world_export_index: usize,
    limits: Limits,
) -> Result<WorldStartupMap> {
    let package = archive.package();
    if package.exports.len() > limits.exports {
        return Err(Error::Budget {
            resource: "exports",
            limit: limits.exports,
        });
    }
    let mut work = Work {
        package,
        limits,
        usage: Usage::default(),
        identity_bytes: Cell::new(0),
    };
    let (world_identity, world_class_identity) =
        work.native_identity(world_export_index, "Engine.World", 0)?;
    let world_bytes = work.read_native(archive, world_export_index)?;
    let world = WorldObject::parse_export(package, world_export_index, &world_bytes)
        .map_err(|e| native_error(world_export_index, e))?;
    work.native_properties(&world.object.properties)?;
    charge(
        &mut work.usage.array_items,
        world.reference_array_count,
        limits.array_items,
        "array items",
    )?;
    let persistent = world
        .links
        .iter()
        .find(|link| link.slot == WorldLinkSlot::PersistentLevel)
        .ok_or_else(|| {
            invalid(
                Some(world_export_index),
                "World has no PersistentLevel slot",
            )
        })?;
    let persistent_level =
        work.native_reference(world_export_index, persistent.span, persistent.reference)?;
    let level_export_index = local_link(&persistent_level, "PersistentLevel")?;
    let (level_identity, level_class_identity) =
        work.native_identity(level_export_index, "Engine.Level", world_identity.value)?;
    let level_bytes = work.read_native(archive, level_export_index)?;
    let level = LevelObject::parse_export(package, level_export_index, &level_bytes)
        .map_err(|e| native_error(level_export_index, e))?;
    work.native_properties(&level.object.properties)?;
    for count in [
        level.actors.links.len(),
        level.model_components.links.len(),
        level.sequences.links.len(),
        level.url.extra_strings.len(),
    ] {
        charge(
            &mut work.usage.array_items,
            count,
            limits.array_items,
            "array items",
        )?;
    }
    let first_sequence = first_sequence(&mut work, level_export_index, &level)?;
    // Resolve and check the selected role before preparing unrelated inventory.
    let selected_index = match &first_sequence {
        Some(link) if !matches!(link.value, ObjectReference::Null) => {
            let index = local_link(link, "main Sequence")?;
            work.native_identity(index, "Engine.Sequence", level_identity.value)?;
            Some(index)
        }
        _ => None,
    };
    work.usage.identity_bytes = work.identity_bytes.get();
    let mut remaining = limits;
    remaining.selected_exports -= work.usage.selected_exports;
    remaining.serialized_bytes -= work.usage.serialized_bytes;
    remaining.properties -= work.usage.properties;
    remaining.array_items -= work.usage.array_items;
    remaining.identity_bytes -= work.usage.identity_bytes;
    let inventory = prepare(archive, remaining)?;
    let main_sequence = bind_sequence(first_sequence, selected_index, &inventory)?;
    let mut usage = work.usage;
    for (used, amount, limit, name) in [
        (
            &mut usage.selected_exports,
            inventory.usage.selected_exports,
            limits.selected_exports,
            "selected exports",
        ),
        (
            &mut usage.serialized_bytes,
            inventory.usage.serialized_bytes,
            limits.serialized_bytes,
            "serialized bytes",
        ),
        (
            &mut usage.properties,
            inventory.usage.properties,
            limits.properties,
            "properties",
        ),
        (
            &mut usage.array_items,
            inventory.usage.array_items,
            limits.array_items,
            "array items",
        ),
        (
            &mut usage.identity_bytes,
            inventory.usage.identity_bytes,
            limits.identity_bytes,
            "identity bytes",
        ),
    ] {
        charge(used, amount, limit, name)?;
    }
    Ok(WorldStartupMap {
        world_identity,
        world_class_identity,
        world,
        persistent_level,
        level_identity,
        level_class_identity,
        level,
        main_sequence,
        inventory,
        usage,
    })
}

fn native_error(export: usize, error: io::Error) -> Error {
    if error.kind() == io::ErrorKind::Unsupported {
        Error::Unsupported {
            export,
            message: error.to_string(),
        }
    } else {
        invalid(Some(export), error.to_string())
    }
}

fn local_link(link: &NativeReference, role: &str) -> Result<usize> {
    match &link.value {
        ObjectReference::Reference(Reference {
            kind: ReferenceKind::Export { export_index, .. },
            ..
        }) => Ok(*export_index),
        ObjectReference::Null => Err(invalid(
            Some(link.origin.export_index),
            format!("null {role} reference"),
        )),
        _ => Err(Error::Unsupported {
            export: link.origin.export_index,
            message: format!("imported {role} is not loaded by saved startup preparation"),
        }),
    }
}

fn first_sequence(
    work: &mut Work<'_>,
    level_export_index: usize,
    level: &LevelObject,
) -> Result<Option<NativeReference>> {
    let links = level
        .links
        .get(level.sequences.links.clone())
        .ok_or_else(|| {
            invalid(
                Some(level_export_index),
                "Level sequence slice is out of bounds",
            )
        })?;
    let Some(first) = links.first() else {
        return Ok(None);
    };
    if first.slot != LevelLinkSlot::Sequence(0) {
        return Err(invalid(
            Some(level_export_index),
            "first Level sequence slot is not ordinal zero",
        ));
    }
    work.native_reference(level_export_index, first.span, first.reference)
        .map(Some)
}

fn bind_sequence(
    first: Option<NativeReference>,
    selected_index: Option<usize>,
    inventory: &StartupMap,
) -> Result<MainSequence> {
    let Some(link) = first else {
        return Ok(MainSequence::EmptyArray);
    };
    if matches!(link.value, ObjectReference::Null) {
        return Ok(MainSequence::FirstNull { link });
    }
    let export_index = selected_index.ok_or_else(|| {
        invalid(
            Some(link.origin.export_index),
            "selected Sequence was not resolved",
        )
    })?;
    let mut found = inventory.sequences.iter().enumerate().filter(|(_, sequence)| {
        matches!(sequence.object.identity.kind, ReferenceKind::Export { export_index: i, .. } if i == export_index)
    });
    let (inventory_index, sequence) = found.next().ok_or_else(|| {
        invalid(
            Some(export_index),
            "selected Sequence is outside prepared inventory",
        )
    })?;
    if found.next().is_some() {
        return Err(invalid(
            Some(export_index),
            "ambiguous selected Sequence inventory",
        ));
    }
    if link.value != ObjectReference::Reference(sequence.object.identity.clone()) {
        return Err(invalid(
            Some(export_index),
            "selected Sequence identity disagrees with inventory",
        ));
    }
    Ok(MainSequence::Local {
        link,
        export_index,
        inventory_index,
    })
}

#[derive(Clone, Copy)]
enum Kind {
    Sequence,
    LevelLoaded,
    OpenMovie,
    PlayerVariable,
}

/// Parse selected established non-actor UObject layouts in the supplied archive.
/// No filename or Default__ search, import resolution, player or receiver creation.
pub fn prepare(archive: &PackageArchive, limits: Limits) -> Result<StartupMap> {
    let package = archive.package();
    if package.exports.len() > limits.exports {
        return Err(Error::Budget {
            resource: "exports",
            limit: limits.exports,
        });
    }
    let mut work = Work {
        package,
        limits,
        usage: Usage::default(),
        identity_bytes: Cell::new(0),
    };
    let mut result = StartupMap {
        export_count: package.exports.len(),
        sequences: Vec::new(),
        level_loaded: Vec::new(),
        open_movies: Vec::new(),
        player_variables: Vec::new(),
        unprepared_targets: Vec::new(),
        usage: Usage::default(),
    };
    let mut prepared = BTreeSet::new();
    for index in 0..package.exports.len() {
        let Some(kind) = work.kind(index)? else {
            continue;
        };
        charge(
            &mut work.usage.selected_exports,
            1,
            limits.selected_exports,
            "selected exports",
        )?;
        charge(
            &mut work.usage.serialized_bytes,
            package.exports[index].serial_size,
            limits.serialized_bytes,
            "serialized bytes",
        )?;
        let bytes = archive.read_export(index).map_err(|source| Error::Read {
            export: index,
            source,
        })?;
        if bytes.len() < 12 {
            return Err(invalid(
                Some(index),
                "selected UObject lacks net index and None terminator",
            ));
        }
        let props = work.list(index, &bytes, 4, bytes.len(), 0)?;
        if props.end_offset != bytes.len() {
            return Err(Error::Unsupported {
                export: index,
                message: "selected object has trailing native data".into(),
            });
        }
        let object = work.object(index, &bytes, props)?;
        prepared.insert(object.identity.value);
        match kind {
            Kind::Sequence => {
                let members =
                    work.references(index, &bytes, &object.properties, "SequenceObjects")?;
                result.sequences.push(Sequence { object, members });
            }
            Kind::LevelLoaded => {
                let outputs = work.ports(index, &bytes, &object.properties, "OutputLinks")?;
                result.level_loaded.push(LevelLoaded { object, outputs });
            }
            Kind::PlayerVariable => {
                let player_index = work.integer(index, &object.properties, "PlayerIdx")?;
                let all_players = work.boolean(index, &object.properties, "bAllPlayers")?;
                result.player_variables.push(PlayerVariable {
                    object,
                    player_index,
                    all_players,
                });
            }
            Kind::OpenMovie => {
                let p = &object.properties;
                let movie = work.resource(index, p, "Movie", "SwfMovie")?;
                let player_class = work.resource(index, p, "MoviePlayerClass", "Class")?;
                let start_paused = work.boolean(index, p, "bStartPaused")?;
                let take_focus = work.boolean(index, p, "bTakeFocus")?;
                let capture_input = work.boolean(index, p, "bCaptureInput")?;
                let display_with_hud_off = work.boolean(index, p, "bDisplayWithHudOff")?;
                let inputs = work.ports(index, &bytes, p, "InputLinks")?;
                let outputs = work.ports(index, &bytes, p, "OutputLinks")?;
                let variables = work.variables(index, &bytes, p)?;
                result.open_movies.push(OpenMovie {
                    object,
                    movie,
                    player_class,
                    start_paused,
                    take_focus,
                    capture_input,
                    display_with_hud_off,
                    inputs,
                    outputs,
                    variables,
                });
            }
        }
    }
    validate_membership(&result)?;
    let mut targets = BTreeMap::new();
    for port in result
        .level_loaded
        .iter()
        .flat_map(|e| e.outputs.iter().flat_map(|p| &p.value))
        .chain(
            result
                .open_movies
                .iter()
                .flat_map(|e| e.outputs.iter().flat_map(|p| &p.value)),
        )
    {
        for link in port.links.iter().flat_map(|s| &s.value) {
            if let ObjectReference::Reference(reference) = &link.target.value {
                if let Some(movie) = result
                    .open_movies
                    .iter()
                    .find(|m| m.object.identity.value == reference.value)
                {
                    if let Some(inputs) = &movie.inputs {
                        if link.input_index.value >= inputs.value.len() {
                            return Err(invalid(
                                Some(link.input_index.origin.export_index),
                                "OpenMovie input link index is out of bounds",
                            ));
                        }
                    }
                }
                if !prepared.contains(&reference.value) {
                    let class_identity = match reference.kind {
                        ReferenceKind::Export {
                            class_reference: 0, ..
                        }
                        | ReferenceKind::Import { .. } => None,
                        ReferenceKind::Export {
                            class_reference, ..
                        } => Some(work.reference(class_reference)?),
                    };
                    targets.entry(reference.value).or_insert(UnpreparedTarget {
                        identity: reference.clone(),
                        class_identity,
                    });
                }
            }
        }
    }
    result.unprepared_targets = targets.into_values().collect();
    work.usage.identity_bytes = work.identity_bytes.get();
    result.usage = work.usage;
    Ok(result)
}

struct Work<'a> {
    package: &'a Package,
    limits: Limits,
    usage: Usage,
    identity_bytes: Cell<usize>,
}
impl Work<'_> {
    fn native_identity(
        &self,
        index: usize,
        expected_class: &str,
        expected_outer: i32,
    ) -> Result<(Reference, Reference)> {
        let export =
            self.package.exports.get(index).ok_or_else(|| {
                invalid(Some(index), "selected native export index is out of bounds")
            })?;
        let value = index
            .checked_add(1)
            .and_then(|v| i32::try_from(v).ok())
            .ok_or_else(|| invalid(Some(index), "selected export reference overflow"))?;
        let identity = self.reference(value)?;
        let class = self.reference(export.class_index)?;
        if !class.object_path.eq_ignore_ascii_case(expected_class)
            || !matches!(&class.kind, ReferenceKind::Import { class_package, class_name, .. }
                if class_package.eq_ignore_ascii_case("Core") && class_name.eq_ignore_ascii_case("Class"))
        {
            return Err(Error::Unsupported {
                export: index,
                message: format!("saved startup requires exact {expected_class} class import"),
            });
        }
        self.require_class(&class, index)?;
        if export.object_flags & 0x0200_0000_0000_0000 != 0 {
            return Err(Error::Unsupported {
                export: index,
                message: "selected native startup object has an unsupported state-frame preamble"
                    .into(),
            });
        }
        if export.outer_index != expected_outer {
            return Err(invalid(
                Some(index),
                "selected native object's outer disagrees with saved route",
            ));
        }
        Ok((identity, class))
    }

    fn read_native(&mut self, archive: &PackageArchive, index: usize) -> Result<Vec<u8>> {
        let export = self
            .package
            .exports
            .get(index)
            .ok_or_else(|| invalid(Some(index), "native export is out of bounds"))?;
        charge(
            &mut self.usage.selected_exports,
            1,
            self.limits.selected_exports,
            "selected exports",
        )?;
        charge(
            &mut self.usage.serialized_bytes,
            export.serial_size,
            self.limits.serialized_bytes,
            "serialized bytes",
        )?;
        archive.read_export(index).map_err(|source| Error::Read {
            export: index,
            source,
        })
    }

    fn native_properties(&mut self, properties: &PropertyList) -> Result<()> {
        charge(
            &mut self.usage.properties,
            properties.entries.len(),
            self.limits.properties,
            "properties",
        )?;
        for property in &properties.entries {
            if let Value::Array { count, .. } = property.value {
                charge(
                    &mut self.usage.array_items,
                    count,
                    self.limits.array_items,
                    "array items",
                )?;
            }
        }
        Ok(())
    }

    fn native_reference(&self, index: usize, span: Span, value: i32) -> Result<NativeReference> {
        let export = self
            .package
            .exports
            .get(index)
            .ok_or_else(|| invalid(Some(index), "native origin export is out of bounds"))?;
        let end = span
            .offset
            .checked_add(span.length)
            .filter(|end| *end <= export.serial_size)
            .ok_or_else(|| invalid(Some(index), "native reference span is outside export"))?;
        if span.length != 4 || export.serial_offset.checked_add(end).is_none() {
            return Err(invalid(
                Some(index),
                "invalid native reference width or logical offset",
            ));
        }
        Ok(NativeReference {
            value: self.object_reference(value)?,
            origin: NativeOrigin {
                export_index: index,
                serial_offset: export.serial_offset,
                encoded_value: span,
            },
        })
    }

    fn name(&self, n: crate::upk::NameRef) -> Result<String> {
        let name = self
            .package
            .resolve_name(n)
            .ok_or_else(|| invalid(None, "unresolved FName"))?;
        let mut total = self.identity_bytes.get();
        charge(
            &mut total,
            name.len(),
            self.limits.identity_bytes,
            "identity bytes",
        )?;
        self.identity_bytes.set(total);
        Ok(name)
    }
    fn reference(&self, value: i32) -> Result<Reference> {
        if value == 0 {
            return Err(invalid(None, "null reference has no identity"));
        }
        let (class_name, kind) = if value > 0 {
            let index = value as usize - 1;
            let e = self
                .package
                .exports
                .get(index)
                .ok_or_else(|| invalid(None, "export reference out of bounds"))?;
            let class_name = self
                .package
                .export_class_name(index)
                .ok_or_else(|| invalid(Some(index), "unresolved export class"))?;
            (
                class_name,
                ReferenceKind::Export {
                    export_index: index,
                    class_reference: e.class_index,
                    outer_reference: e.outer_index,
                    archetype_reference: e.archetype_index,
                },
            )
        } else {
            let e = self
                .package
                .imports
                .get(value.unsigned_abs() as usize - 1)
                .ok_or_else(|| invalid(None, "import reference out of bounds"))?;
            let class_package = self.name(e.class_package)?;
            let class_name = self.name(e.class_name)?;
            (
                class_name.clone(),
                ReferenceKind::Import {
                    class_package,
                    class_name,
                    outer_reference: e.outer_index,
                },
            )
        };
        let mut current = value;
        let mut seen = BTreeSet::new();
        let mut parts = Vec::new();
        let mut size = 0;
        while current != 0 {
            if parts.len() >= self.limits.outer_depth {
                return Err(Error::Budget {
                    resource: "outer depth",
                    limit: self.limits.outer_depth,
                });
            }
            if !seen.insert(current) {
                return Err(invalid(None, "cyclic object outer chain"));
            }
            let (name, outer) = if current > 0 {
                let e = self
                    .package
                    .exports
                    .get(current as usize - 1)
                    .ok_or_else(|| invalid(None, "outer export reference out of bounds"))?;
                (e.object_name, e.outer_index)
            } else {
                let e = self
                    .package
                    .imports
                    .get(current.unsigned_abs() as usize - 1)
                    .ok_or_else(|| invalid(None, "outer import reference out of bounds"))?;
                (e.object_name, e.outer_index)
            };
            let name = self.name(name)?;
            if name.is_empty() || name.contains('.') {
                return Err(invalid(None, "empty or dotted object path component"));
            }
            charge(
                &mut size,
                name.len() + usize::from(!parts.is_empty()),
                self.limits.identity_bytes,
                "identity bytes",
            )?;
            parts.push(name);
            current = outer;
        }
        parts.reverse();
        Ok(Reference {
            value,
            object_path: parts.join("."),
            class_name,
            kind,
        })
    }
    fn object_reference(&self, value: i32) -> Result<ObjectReference> {
        if value == 0 {
            Ok(ObjectReference::Null)
        } else {
            self.reference(value).map(ObjectReference::Reference)
        }
    }
    fn kind(&self, index: usize) -> Result<Option<Kind>> {
        let name = self
            .package
            .export_class_name(index)
            .ok_or_else(|| invalid(Some(index), "unresolved export class"))?;
        let (kind, path) = match name.to_ascii_lowercase().as_str() {
            "sequence" => (Kind::Sequence, "Engine.Sequence"),
            "seqevent_levelloaded" => (Kind::LevelLoaded, "Engine.SeqEvent_LevelLoaded"),
            "gfxaction_openmovie" => (Kind::OpenMovie, "GFxUI.GFxAction_OpenMovie"),
            "seqvar_player" => (Kind::PlayerVariable, "Engine.SeqVar_Player"),
            _ => return Ok(None),
        };
        let class = self.reference(self.package.exports[index].class_index)?;
        if !class.object_path.eq_ignore_ascii_case(path)
            || !class.class_name.eq_ignore_ascii_case("Class")
            || !matches!(&class.kind,ReferenceKind::Import { class_package,class_name,.. }
                if class_package.eq_ignore_ascii_case("Core") && class_name.eq_ignore_ascii_case("Class"))
        {
            return Err(Error::Unsupported {
                export: index,
                message: format!("unproved class identity for {name}: {}", class.object_path),
            });
        }
        self.require_class(&class, index)?;
        Ok(Some(kind))
    }
    fn require_class(&self, reference: &Reference, index: usize) -> Result<()> {
        match &reference.kind {
            ReferenceKind::Export {
                class_reference: 0, ..
            } => Ok(()),
            ReferenceKind::Import {
                class_package,
                class_name,
                outer_reference,
            } if class_package.eq_ignore_ascii_case("Core")
                && class_name.eq_ignore_ascii_case("Class") =>
            {
                let outer = self.reference(*outer_reference)?;
                if matches!(&outer.kind, ReferenceKind::Import { class_package, class_name, outer_reference: 0 }
                        if class_package.eq_ignore_ascii_case("Core") && class_name.eq_ignore_ascii_case("Package"))
                {
                    Ok(())
                } else {
                    Err(invalid(
                        Some(index),
                        "class import lacks a declared root package",
                    ))
                }
            }
            _ => Err(invalid(
                Some(index),
                "reference is not an established Class identity",
            )),
        }
    }
    fn list(
        &mut self,
        index: usize,
        bytes: &[u8],
        start: usize,
        end: usize,
        depth: usize,
    ) -> Result<PropertyList> {
        if depth > self.limits.nesting_depth {
            return Err(Error::Budget {
                resource: "nesting depth",
                limit: self.limits.nesting_depth,
            });
        }
        let bounded = bytes
            .get(..end)
            .ok_or_else(|| invalid(Some(index), "list range exceeds export"))?;
        let list = objects::parse_tagged_properties(self.package, bounded, start)
            .map_err(|e| invalid(Some(index), e.to_string()))?;
        charge(
            &mut self.usage.properties,
            list.entries.len(),
            self.limits.properties,
            "properties",
        )?;
        let mut names = BTreeSet::new();
        for p in &list.entries {
            let name = self.name(p.name)?;
            if p.name.number != 0 || p.array_index != 0 {
                return Err(Error::Unsupported {
                    export: index,
                    message: "numbered/indexed control property".into(),
                });
            }
            if !names.insert(name.to_ascii_lowercase()) {
                return Err(invalid(Some(index), format!("duplicate {name} property")));
            }
            if let Value::Array { count, .. } = p.value {
                charge(
                    &mut self.usage.array_items,
                    count,
                    self.limits.array_items,
                    "array items",
                )?;
            }
        }
        Ok(list)
    }
    fn origin(&self, index: usize, p: &Property) -> Result<Origin> {
        let serial_offset = self.package.exports[index].serial_offset;
        let encoded_value = if matches!(p.value, Value::Boolean(_)) {
            Span {
                offset: p
                    .payload
                    .offset
                    .checked_sub(1)
                    .ok_or_else(|| invalid(Some(index), "boolean tag underflow"))?,
                length: 1,
            }
        } else {
            p.payload
        };
        serial_offset
            .checked_add(encoded_value.offset)
            .and_then(|v| v.checked_add(encoded_value.length))
            .ok_or_else(|| invalid(Some(index), "logical value span overflow"))?;
        Ok(Origin {
            export_index: index,
            serial_offset,
            tag: p.tag,
            payload: p.payload,
            encoded_value,
        })
    }
    fn saved<T>(&self, index: usize, p: &Property, value: T) -> Result<Saved<T>> {
        Ok(Saved {
            value,
            origin: self.origin(index, p)?,
        })
    }
    fn field<'a>(&self, list: &'a PropertyList, name: &str) -> Option<&'a Property> {
        list.find(self.package, name)
    }
    fn scalar<T>(
        &self,
        index: usize,
        list: &PropertyList,
        name: &str,
        convert: impl FnOnce(&Value) -> Option<T>,
    ) -> Result<Option<Saved<T>>> {
        self.field(list, name)
            .map(|p| {
                let value = convert(&p.value)
                    .ok_or_else(|| invalid(Some(index), format!("wrong scalar type for {name}")))?;
                self.saved(index, p, value)
            })
            .transpose()
    }
    fn boolean(
        &self,
        index: usize,
        list: &PropertyList,
        name: &str,
    ) -> Result<Option<Saved<bool>>> {
        self.scalar(index, list, name, |v| {
            if let Value::Boolean(v) = v {
                Some(*v)
            } else {
                None
            }
        })
    }
    fn integer(&self, index: usize, list: &PropertyList, name: &str) -> Result<Option<Saved<i32>>> {
        self.scalar(index, list, name, |v| {
            if let Value::Integer(v) = v {
                Some(*v)
            } else {
                None
            }
        })
    }
    fn object_field(
        &self,
        index: usize,
        list: &PropertyList,
        name: &str,
    ) -> Result<Option<Saved<ObjectReference>>> {
        self.field(list, name)
            .map(|p| {
                if !self
                    .name(p.property_type)?
                    .eq_ignore_ascii_case("ObjectProperty")
                {
                    return Err(invalid(
                        Some(index),
                        format!("unsupported object tag type for {name}"),
                    ));
                }
                let Value::Object(value) = p.value else {
                    return Err(invalid(
                        Some(index),
                        format!("wrong object type for {name}"),
                    ));
                };
                self.saved(index, p, self.object_reference(value)?)
            })
            .transpose()
    }
    fn resource(
        &self,
        index: usize,
        list: &PropertyList,
        name: &str,
        class: &'static str,
    ) -> Result<Option<UnresolvedResource>> {
        self.object_field(index, list, name)?
            .map(|reference| {
                if let ObjectReference::Reference(r) = &reference.value {
                    if !r.class_name.eq_ignore_ascii_case(class) {
                        return Err(invalid(
                            Some(index),
                            format!("{name} references {} instead of {class}", r.class_name),
                        ));
                    }
                }
                if let ObjectReference::Reference(r) = &reference.value {
                    if class == "Class" {
                        self.require_class(r, index)?;
                    }
                    if class == "SwfMovie" {
                        match &r.kind {
                            ReferenceKind::Import { class_package, .. }
                                if class_package.eq_ignore_ascii_case("GFxUI") => {}
                            ReferenceKind::Export {
                                class_reference, ..
                            } => {
                                let definition = self.reference(*class_reference)?;
                                self.require_class(&definition, index)?;
                                if !definition
                                    .object_path
                                    .eq_ignore_ascii_case("GFxUI.SwfMovie")
                                {
                                    return Err(invalid(
                                        Some(index),
                                        "movie export has an unproved SwfMovie class identity",
                                    ));
                                }
                            }
                            _ => {
                                return Err(invalid(
                                    Some(index),
                                    "movie import has an unproved class package",
                                ));
                            }
                        }
                    }
                }
                Ok(UnresolvedResource {
                    reference,
                    expected_class: class,
                })
            })
            .transpose()
    }
    fn object(&self, index: usize, bytes: &[u8], properties: PropertyList) -> Result<Object> {
        let reference = i32::try_from(index + 1)
            .map_err(|_| invalid(Some(index), "export reference overflow"))?;
        let e = &self.package.exports[index];
        let parent_sequence = self.object_field(index, &properties, "ParentSequence")?;
        Ok(Object {
            identity: self.reference(reference)?,
            class_identity: self.reference(e.class_index)?,
            serial_offset: e.serial_offset,
            serial_size: e.serial_size,
            net_index: i32::from_le_bytes(bytes[..4].try_into().unwrap()),
            properties,
            parent_sequence,
        })
    }
    fn references(
        &self,
        index: usize,
        bytes: &[u8],
        list: &PropertyList,
        name: &str,
    ) -> Result<Option<Saved<Vec<ArrayElement<ObjectReference>>>>> {
        self.field(list, name)
            .map(|p| {
                let Value::Array { count, elements } = p.value else {
                    return Err(invalid(Some(index), format!("{name} is not an array")));
                };
                if count.checked_mul(4) != Some(elements.length) {
                    return Err(invalid(
                        Some(index),
                        format!("{name} reference array length mismatch"),
                    ));
                }
                let origin = self.origin(index, p)?;
                let data = elements
                    .bytes(bytes)
                    .map_err(|e| invalid(Some(index), e.to_string()))?;
                let mut values = Vec::with_capacity(count);
                for (i, raw) in data.chunks_exact(4).enumerate() {
                    values.push(ArrayElement {
                        value: self
                            .object_reference(i32::from_le_bytes(raw.try_into().unwrap()))?,
                        array_origin: origin.clone(),
                        encoded_value: Span {
                            offset: elements.offset + i * 4,
                            length: 4,
                        },
                    });
                }
                self.saved(index, p, values)
            })
            .transpose()
    }
    fn structs(
        &mut self,
        index: usize,
        bytes: &[u8],
        p: &Property,
        depth: usize,
    ) -> Result<Vec<PropertyList>> {
        let Value::Array { count, elements } = p.value else {
            return Err(invalid(Some(index), "expected tagged-struct array"));
        };
        let end = elements
            .offset
            .checked_add(elements.length)
            .ok_or_else(|| invalid(Some(index), "struct array range overflow"))?;
        if count > elements.length / 8 {
            return Err(invalid(
                Some(index),
                "struct array count exceeds its byte range",
            ));
        }
        let mut at = elements.offset;
        let mut result = Vec::with_capacity(count);
        for _ in 0..count {
            let list = self.list(index, bytes, at, end, depth)?;
            at = list.end_offset;
            result.push(list);
        }
        if at != end {
            return Err(invalid(
                Some(index),
                "struct array has trailing bytes or count mismatch",
            ));
        }
        Ok(result)
    }
    fn ports(
        &mut self,
        index: usize,
        bytes: &[u8],
        list: &PropertyList,
        name: &str,
    ) -> Result<Option<Saved<Vec<Port>>>> {
        let Some(p) = self.field(list, name) else {
            return Ok(None);
        };
        let mut ports = Vec::new();
        for properties in self.structs(index, bytes, p, 1)? {
            let description = self.scalar(index, &properties, "LinkDesc", |v| {
                if let Value::String(v) = v {
                    Some(v.clone())
                } else {
                    None
                }
            })?;
            let disabled = self.boolean(index, &properties, "bDisabled")?;
            let disabled_pie = self.boolean(index, &properties, "bDisabledPIE")?;
            let has_impulse = self.boolean(index, &properties, "bHasImpulse")?;
            let queued_activations = self.integer(index, &properties, "QueuedActivations")?;
            let linked_op = self.object_field(index, &properties, "LinkedOp")?;
            let activation_delay_bits = self.scalar(index, &properties, "ActivateDelay", |v| {
                if let Value::FloatBits(v) = v {
                    Some(*v)
                } else {
                    None
                }
            })?;
            let links = if let Some(prop) = self.field(&properties, "Links") {
                let mut links = Vec::new();
                for fields in self.structs(index, bytes, prop, 2)? {
                    let target = self
                        .object_field(index, &fields, "LinkedOp")?
                        .ok_or_else(|| invalid(Some(index), "execution link lacks LinkedOp"))?;
                    if target.value == ObjectReference::Null {
                        return Err(invalid(Some(index), "execution link has null LinkedOp"));
                    }
                    let input = self
                        .integer(index, &fields, "InputLinkIdx")?
                        .ok_or_else(|| invalid(Some(index), "execution link lacks InputLinkIdx"))?;
                    let value = usize::try_from(input.value)
                        .map_err(|_| invalid(Some(index), "negative input link index"))?;
                    links.push(ExecutionLink {
                        properties: fields,
                        target,
                        input_index: Saved {
                            value,
                            origin: input.origin,
                        },
                    });
                }
                Some(self.saved(index, prop, links)?)
            } else {
                None
            };
            ports.push(Port {
                properties,
                description,
                disabled,
                disabled_pie,
                activation_delay_bits,
                has_impulse,
                queued_activations,
                linked_op,
                links,
            });
        }
        Ok(Some(self.saved(index, p, ports)?))
    }
    fn variables(
        &mut self,
        index: usize,
        bytes: &[u8],
        list: &PropertyList,
    ) -> Result<Option<Saved<Vec<VariableLink>>>> {
        let Some(p) = self.field(list, "VariableLinks") else {
            return Ok(None);
        };
        let mut result = Vec::new();
        for properties in self.structs(index, bytes, p, 1)? {
            let description = self.scalar(index, &properties, "LinkDesc", |v| {
                if let Value::String(v) = v {
                    Some(v.clone())
                } else {
                    None
                }
            })?;
            let expected_type = self.object_field(index, &properties, "ExpectedType")?;
            if let Some(Saved {
                value: ObjectReference::Reference(r),
                ..
            }) = &expected_type
            {
                self.require_class(r, index)?;
            }
            let linked_variables = self.references(index, bytes, &properties, "LinkedVariables")?;
            let writable = self.boolean(index, &properties, "bWriteable")?;
            let min_vars = self.integer(index, &properties, "MinVars")?;
            let max_vars = self.integer(index, &properties, "MaxVars")?;
            result.push(VariableLink {
                properties,
                description,
                expected_type,
                linked_variables,
                writable,
                min_vars,
                max_vars,
            });
        }
        Ok(Some(self.saved(index, p, result)?))
    }
}

fn validate_membership(map: &StartupMap) -> Result<()> {
    let mut memberships = BTreeMap::new();
    for sequence in &map.sequences {
        if let Some(members) = &sequence.members {
            let mut seen = BTreeSet::new();
            for member in &members.value {
                let ObjectReference::Reference(r) = &member.value else {
                    return Err(invalid(None, "null sequence member"));
                };
                let ReferenceKind::Export {
                    outer_reference, ..
                } = r.kind
                else {
                    return Err(invalid(None, "external sequence membership unsupported"));
                };
                if !seen.insert(r.value) || outer_reference != sequence.object.identity.value {
                    return Err(invalid(
                        None,
                        "duplicate sequence member or mismatched outer",
                    ));
                }
                if memberships
                    .insert(r.value, sequence.object.identity.value)
                    .is_some()
                {
                    return Err(invalid(None, "ambiguous sequence membership"));
                }
            }
        }
    }
    for object in map
        .level_loaded
        .iter()
        .map(|v| &v.object)
        .chain(map.open_movies.iter().map(|v| &v.object))
        .chain(map.player_variables.iter().map(|v| &v.object))
        .chain(map.sequences.iter().map(|v| &v.object))
    {
        if let Some(parent) = &object.parent_sequence {
            if let ObjectReference::Reference(r) = &parent.value {
                let sequence = map
                    .sequences
                    .iter()
                    .find(|s| s.object.identity.value == r.value)
                    .ok_or_else(|| {
                        invalid(
                            Some(parent.origin.export_index),
                            "ParentSequence is outside prepared local Sequence profile",
                        )
                    })?;
                let ReferenceKind::Export {
                    outer_reference, ..
                } = object.identity.kind
                else {
                    unreachable!()
                };
                if outer_reference != r.value {
                    return Err(invalid(
                        Some(parent.origin.export_index),
                        "ParentSequence disagrees with object outer",
                    ));
                }
                if sequence.members.is_some()
                    && memberships.get(&object.identity.value) != Some(&r.value)
                {
                    return Err(invalid(
                        Some(parent.origin.export_index),
                        "object absent from its saved parent SequenceObjects",
                    ));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upk::{Export, Import, NameEntry, NameRef, Summary, TableLocation};

    const NAMES: &[&str] = &[
        "None",
        "Flag",
        "BoolProperty",
        "Links",
        "ArrayProperty",
        "LinkedVariables",
        "LinkDesc",
        "StrProperty",
        "Core",
        "Package",
        "Class",
        "Engine",
        "Sequence",
        "BadNamespace",
        "SwfMovie",
        "Movie",
        "ObjectProperty",
        "MoviePlayerClass",
        "LinkedOp",
        "InputLinkIdx",
        "IntProperty",
        "bStartPaused",
        "GFxUI",
    ];
    fn n(index: usize) -> NameRef {
        NameRef { index, number: 0 }
    }
    fn package() -> Package {
        let table = |count| TableLocation { count, offset: 0 };
        Package {
            summary: Summary {
                file_version: 868,
                licensee_version: 3,
                header_size: 0,
                group: String::new(),
                package_flags: 0,
                names: table(NAMES.len()),
                exports: table(1),
                imports: table(3),
                depends_offset: 0,
                import_export_guids_offset: 0,
                import_guids_count: 0,
                export_guids_count: 0,
                thumbnail_table_offset: 0,
                guid: [0; 16],
                generations: Vec::new(),
                engine_version: 0,
                cooker_version: 0,
                compression_flags: 0,
                chunks: Vec::new(),
                package_source: 0,
                parsed_bytes: 0,
            },
            names: NAMES
                .iter()
                .map(|s| NameEntry {
                    value: (*s).into(),
                    flags: 0,
                })
                .collect(),
            imports: vec![
                Import {
                    class_package: n(8),
                    class_name: n(9),
                    outer_index: 0,
                    object_name: n(11),
                },
                Import {
                    class_package: n(8),
                    class_name: n(10),
                    outer_index: -1,
                    object_name: n(12),
                },
                Import {
                    class_package: n(22),
                    class_name: n(14),
                    outer_index: -1,
                    object_name: n(15),
                },
            ],
            exports: vec![Export {
                class_index: -2,
                super_index: 0,
                outer_index: 0,
                object_name: n(1),
                archetype_index: 0,
                object_flags: 0,
                serial_size: 0,
                serial_offset: 1000,
                export_flags: 0,
                generation_net_object_counts: Vec::new(),
                guid: [0; 16],
                package_flags: 0,
            }],
        }
    }
    fn work(p: &Package) -> Work<'_> {
        Work {
            package: p,
            limits: Limits::default(),
            usage: Usage::default(),
            identity_bytes: Cell::new(0),
        }
    }
    fn word(b: &mut Vec<u8>, v: i32) {
        b.extend(v.to_le_bytes());
    }
    fn name(b: &mut Vec<u8>, v: i32) {
        word(b, v);
        word(b, 0);
    }
    fn tag(b: &mut Vec<u8>, key: i32, kind: i32, size: i32) {
        name(b, key);
        name(b, kind);
        word(b, size);
        word(b, 0);
    }
    fn end(b: &mut Vec<u8>) {
        name(b, 0);
    }

    fn route_package() -> Package {
        let mut p = package();
        let mut add_name = |value: &str| {
            let index = p.names.len();
            p.names.push(NameEntry {
                value: value.into(),
                flags: 0,
            });
            n(index)
        };
        let world_class = add_name("World");
        let level_class = add_name("Level");
        let world_name = add_name("TheWorld");
        let level_name = add_name("PersistentLevel");
        let sequence_name = add_name("MainSequence");
        p.imports.truncate(2);
        for object_name in [world_class, level_class] {
            p.imports.push(Import {
                class_package: n(8),
                class_name: n(10),
                outer_index: -1,
                object_name,
            });
        }
        let template = p.exports[0].clone();
        p.exports = [
            (world_name, -3, 0),
            (level_name, -4, 1),
            (sequence_name, -2, 2),
        ]
        .into_iter()
        .map(|(object_name, class_index, outer_index)| Export {
            object_name,
            class_index,
            outer_index,
            serial_size: 128,
            ..template.clone()
        })
        .collect();
        p
    }

    fn route_level(p: &mut Package, references: &[i32]) -> LevelObject {
        let mut bytes = Vec::new();
        // NetIndex, None, self, empty actors, four empty URL strings, empty
        // URL extra strings, port/word, null model, empty model components.
        for v in [0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 7777, 0, 0, 0] {
            word(&mut bytes, v);
        }
        word(&mut bytes, references.len() as i32);
        for value in references {
            word(&mut bytes, *value);
        }
        p.exports[1].serial_size = bytes.len();
        LevelObject::parse_export(p, 1, &bytes).unwrap()
    }

    fn route_inventory(p: &Package) -> StartupMap {
        let w = work(p);
        StartupMap {
            export_count: p.exports.len(),
            sequences: vec![Sequence {
                object: Object {
                    identity: w.reference(3).unwrap(),
                    class_identity: w.reference(-2).unwrap(),
                    serial_offset: p.exports[2].serial_offset,
                    serial_size: 12,
                    net_index: 0,
                    properties: PropertyList {
                        entries: Vec::new(),
                        start_offset: 4,
                        end_offset: 12,
                        terminator: Span {
                            offset: 4,
                            length: 8,
                        },
                    },
                    parent_sequence: None,
                },
                members: None,
            }],
            level_loaded: Vec::new(),
            open_movies: Vec::new(),
            player_variables: Vec::new(),
            unprepared_targets: Vec::new(),
            usage: Usage::default(),
        }
    }

    #[test]
    fn native_route_requires_engine_class_outer_and_supported_prefix() {
        let mut p = route_package();
        for (i, class, outer) in [
            (0, "Engine.World", 0),
            (1, "Engine.Level", 1),
            (2, "Engine.Sequence", 2),
        ] {
            assert!(work(&p).native_identity(i, class, outer).is_ok());
        }
        assert!(work(&p).native_identity(9, "Engine.World", 0).is_err());
        assert!(work(&p).native_identity(0, "Engine.Level", 0).is_err());
        assert!(work(&p).native_identity(1, "Engine.Level", 0).is_err());
        p.imports[3].class_package = n(13);
        assert!(work(&p).native_identity(1, "Engine.Level", 1).is_err());
        p.imports[3].class_package = n(8);
        p.exports[2].object_flags |= 0x0200_0000_0000_0000;
        assert!(work(&p).native_identity(2, "Engine.Sequence", 2).is_err());
        p.exports[0].outer_index = 2;
        assert!(work(&p).native_identity(1, "Engine.Level", 1).is_err());
    }

    #[test]
    fn sequence_selection_preserves_empty_first_null_and_ordinal_zero() {
        let mut p = route_package();
        let empty = route_level(&mut p, &[]);
        assert!(first_sequence(&mut work(&p), 1, &empty).unwrap().is_none());
        let later = route_level(&mut p, &[0, 3]);
        let first = first_sequence(&mut work(&p), 1, &later).unwrap().unwrap();
        assert_eq!(first.value, ObjectReference::Null);
        assert_eq!(
            first.origin.encoded_value,
            later.links[later.sequences.links.start].span
        );
        assert!(matches!(
            bind_sequence(Some(first), None, &route_inventory(&p)).unwrap(),
            MainSequence::FirstNull { .. }
        ));
        let mut selected = route_level(&mut p, &[3, 0]);
        let first = first_sequence(&mut work(&p), 1, &selected)
            .unwrap()
            .unwrap();
        assert_eq!(local_link(&first, "main Sequence").unwrap(), 2);
        assert!(matches!(
            bind_sequence(Some(first), Some(2), &route_inventory(&p)).unwrap(),
            MainSequence::Local {
                export_index: 2,
                inventory_index: 0,
                ..
            }
        ));
        selected.links[selected.sequences.links.start].slot = LevelLinkSlot::Sequence(1);
        assert!(first_sequence(&mut work(&p), 1, &selected).is_err());
    }

    #[test]
    fn selected_native_reference_and_inventory_cannot_change_identity() {
        let p = route_package();
        let span = Span {
            offset: 12,
            length: 4,
        };
        let w = work(&p);
        assert!(local_link(&w.native_reference(0, span, 0).unwrap(), "PersistentLevel").is_err());
        assert!(local_link(&w.native_reference(0, span, -4).unwrap(), "PersistentLevel").is_err());
        assert!(w.native_reference(0, span, 4).is_err());
        assert!(w.native_reference(0, span, i32::MIN).is_err());
        assert!(
            w.native_reference(
                0,
                Span {
                    offset: usize::MAX,
                    length: 4
                },
                1
            )
            .is_err()
        );
        assert!(
            w.native_reference(
                0,
                Span {
                    offset: 12,
                    length: 8
                },
                1
            )
            .is_err()
        );
        let first = w.native_reference(1, span, 3).unwrap();
        assert_eq!(first.origin.logical_value_offset(), Some(1012));
        let mut inventory = route_inventory(&p);
        inventory.sequences[0].object.identity.object_path = "OtherWorld.Sequence".into();
        assert!(bind_sequence(Some(first.clone()), Some(2), &inventory).is_err());
        inventory = route_inventory(&p);
        inventory.sequences.push(inventory.sequences[0].clone());
        assert!(bind_sequence(Some(first.clone()), Some(2), &inventory).is_err());
        inventory.sequences.clear();
        assert!(bind_sequence(Some(first), Some(2), &inventory).is_err());
        let mut w = work(&p);
        w.limits.identity_bytes = 0;
        assert!(matches!(
            w.native_identity(0, "Engine.World", 0),
            Err(Error::Budget {
                resource: "identity bytes",
                ..
            })
        ));
    }

    #[test]
    fn absent_false_null_and_bool_tag_origin_stay_distinct() {
        let p = package();
        let mut w = work(&p);
        let mut bytes = vec![0; 4];
        tag(&mut bytes, 21, 2, 0);
        bytes.push(0);
        tag(&mut bytes, 17, 16, 4);
        word(&mut bytes, 0);
        end(&mut bytes);
        let fields = w.list(0, &bytes, 4, bytes.len(), 0).unwrap();
        let flag = w.boolean(0, &fields, "bStartPaused").unwrap().unwrap();
        assert!(!flag.value);
        assert_eq!(
            flag.origin.encoded_value,
            Span {
                offset: 28,
                length: 1
            }
        );
        assert_eq!(
            flag.origin.payload,
            Span {
                offset: 29,
                length: 0
            }
        );
        assert_eq!(flag.origin.logical_value_offset(), Some(1028));
        assert!(w.boolean(0, &fields, "bMissing").unwrap().is_none());
        assert_eq!(
            w.resource(0, &fields, "MoviePlayerClass", "Class")
                .unwrap()
                .unwrap()
                .reference
                .value,
            ObjectReference::Null
        );
        assert!(w.list(0, &bytes[..28], 4, 28, 0).is_err());
    }

    #[test]
    fn nested_counts_and_boundaries_cannot_consume_the_next_property() {
        let p = package();
        let mut w = work(&p);
        let mut bytes = vec![0; 4];
        tag(&mut bytes, 3, 4, 12);
        word(&mut bytes, 1);
        end(&mut bytes);
        end(&mut bytes);
        let fields = w.list(0, &bytes, 4, bytes.len(), 0).unwrap();
        let array = &fields.entries[0];
        assert_eq!(w.structs(0, &bytes, array, 1).unwrap().len(), 1);
        let mut wrong = array.clone();
        wrong.value = Value::Array {
            count: 2,
            elements: Span {
                offset: 32,
                length: 8,
            },
        };
        assert!(w.structs(0, &bytes, &wrong, 1).is_err());
        wrong.value = Value::Array {
            count: 0,
            elements: Span {
                offset: 32,
                length: 8,
            },
        };
        assert!(w.structs(0, &bytes, &wrong, 1).is_err());
        wrong.value = Value::Array {
            count: 1,
            elements: Span {
                offset: 32,
                length: 7,
            },
        };
        assert!(w.structs(0, &bytes, &wrong, 1).is_err());
        w.limits.nesting_depth = 0;
        assert!(matches!(
            w.structs(0, &bytes, array, 1),
            Err(Error::Budget {
                resource: "nesting depth",
                ..
            })
        ));
    }

    #[test]
    fn duplicate_fields_wrong_resource_classes_and_array_lengths_reject() {
        let p = package();
        let mut w = work(&p);
        let mut bytes = vec![0; 4];
        for _ in 0..2 {
            tag(&mut bytes, 21, 2, 0);
            bytes.push(1);
        }
        end(&mut bytes);
        assert!(w.list(0, &bytes, 4, bytes.len(), 0).is_err());
        bytes = vec![0; 4];
        tag(&mut bytes, 17, 16, 4);
        word(&mut bytes, -3);
        end(&mut bytes);
        let fields = w.list(0, &bytes, 4, bytes.len(), 0).unwrap();
        assert!(w.resource(0, &fields, "MoviePlayerClass", "Class").is_err());
        bytes = vec![0; 4];
        tag(&mut bytes, 5, 4, 8);
        word(&mut bytes, 2);
        word(&mut bytes, -1);
        end(&mut bytes);
        let fields = w.list(0, &bytes, 4, bytes.len(), 0).unwrap();
        assert!(w.references(0, &bytes, &fields, "LinkedVariables").is_err());
    }

    #[test]
    fn full_class_identity_outer_cycles_and_work_budgets_are_checked() {
        let mut p = package();
        assert!(work(&p).kind(0).unwrap().is_some());
        p.imports[1].class_package = n(13);
        assert!(work(&p).kind(0).is_err());
        p.imports[1].class_package = n(8);
        p.imports[0].outer_index = -1;
        assert!(work(&p).kind(0).is_err());
        p.imports[0].outer_index = 0;
        let mut w = work(&p);
        w.limits.identity_bytes = 0;
        assert!(matches!(
            w.reference(-2),
            Err(Error::Budget {
                resource: "identity bytes",
                ..
            })
        ));
        assert!(work(&p).reference(i32::MIN).is_err());
        assert!(work(&p).reference(2).is_err());
        let mut w = work(&p);
        w.limits.properties = 0;
        let mut bytes = vec![0; 4];
        tag(&mut bytes, 21, 2, 0);
        bytes.push(0);
        end(&mut bytes);
        assert!(matches!(
            w.list(0, &bytes, 4, bytes.len(), 0),
            Err(Error::Budget {
                resource: "properties",
                ..
            })
        ));
    }
}
