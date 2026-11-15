//! Lazy, bounded parent-chain views of serialized prototype values.
//! Selections are storage-level overrides, not instantiated runtime-class fields.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::io;
use std::mem::size_of;

use crate::calligraphy::{
    BaseType, Blueprint, PrototypeRecord, StructureType,
    prototype::{Field, FieldData, Prototype, Value},
};
use crate::prototype_catalog::PrototypeReader;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    MissingData,
    InvalidChain,
    Cycle,
    Limit,
    Decode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub prototype_id: u64,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "prototype values {:#x}: {}",
            self.prototype_id, self.message
        )
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;

fn fail(kind: ErrorKind, id: u64, message: impl Into<String>) -> Error {
    Error {
        kind,
        prototype_id: id,
        message: message.into(),
    }
}

/// Minimal read-only metadata plus lazy body loading. Implementations must return
/// the record/body for the requested ID without changing the data source.
pub trait PrototypeSource {
    fn record_by_id(&self, id: u64) -> Option<&PrototypeRecord>;
    fn blueprint(&self, id: u64) -> Option<&Blueprint>;
    fn member_uncompressed_size(&self, id: u64) -> Option<usize>;
    fn load(&mut self, id: u64) -> io::Result<Prototype>;
}

impl PrototypeSource for PrototypeReader<'_> {
    fn record_by_id(&self, id: u64) -> Option<&PrototypeRecord> {
        PrototypeReader::record_by_id(self, id)
    }
    fn blueprint(&self, id: u64) -> Option<&Blueprint> {
        PrototypeReader::blueprint(self, id)
    }
    fn member_uncompressed_size(&self, id: u64) -> Option<usize> {
        PrototypeReader::member_uncompressed_size(self, id)
    }
    fn load(&mut self, id: u64) -> io::Result<Prototype> {
        self.read_id(id)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ValueLimits {
    pub max_parent_layers: usize,
    pub max_encoded_bytes: usize,
    /// Accounted decoded vector/box/path storage; excludes allocator/index overhead.
    pub max_storage_bytes: usize,
    pub max_nodes: usize,
    pub max_groups: usize,
    pub max_fields: usize,
    pub max_values: usize,
    pub max_embedded_depth: usize,
    pub max_schema_steps: usize,
}
impl Default for ValueLimits {
    fn default() -> Self {
        Self {
            max_parent_layers: 64,
            max_encoded_bytes: 32 * 1024 * 1024,
            max_storage_bytes: 128 * 1024 * 1024,
            max_nodes: 100_000,
            max_groups: 200_000,
            max_fields: 500_000,
            max_values: 1_000_000,
            max_embedded_depth: 64,
            max_schema_steps: 1_000_000,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ValueStatistics {
    pub loaded_files: usize,
    pub encoded_bytes: usize,
    pub storage_bytes: usize,
    pub nodes: usize,
    pub groups: usize,
    pub fields: usize,
    pub values: usize,
    pub schema_steps: usize,
}

/// Layers are stored default-first, requested-prototype-last.
pub struct PrototypeLayer {
    pub prototype_id: u64,
    pub blueprint_id: u64,
    pub path: String,
    pub parent_id: u64,
    pub is_default: bool,
    prototype: Prototype,
}
impl PrototypeLayer {
    pub fn prototype(&self) -> &Prototype {
        &self.prototype
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FieldKey {
    pub declaring_blueprint_id: u64,
    pub copy_number: u8,
    pub field_id: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FieldSection {
    Simple,
    List,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldOrigin {
    pub layer: usize,
    pub group: usize,
    pub section: FieldSection,
    pub field: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Unresolved {
    /// RHStruct may be a pointer, property ID, property collection or another binding.
    EmbeddedAssignment,
    /// Declaring/schema blueprint binding differs from the containing binding.
    RuntimeAssignmentContext,
    /// Copy numbers participate in runtime mixin identity, not ordinary field identity.
    MixinCopy,
    DuplicateSlotInLayer,
    ConflictingSchema,
    CrossGroupAlias,
    BlueprintLineage,
}

pub struct FieldView {
    key: FieldKey,
    assignments: Vec<FieldOrigin>,
    unresolved: BTreeSet<Unresolved>,
}
impl FieldView {
    pub fn key(&self) -> FieldKey {
        self.key
    }
    /// Every explicitly stored assignment, default-first and in file order.
    pub fn assignments(&self) -> &[FieldOrigin] {
        &self.assignments
    }
    pub fn unresolved(&self) -> &BTreeSet<Unresolved> {
        &self.unresolved
    }
    /// Most recent bytes, even when their runtime merge meaning is unresolved.
    pub fn latest_stored(&self) -> FieldOrigin {
        *self.assignments.last().expect("nonempty field view")
    }
    /// A conservative serialized override selection. This does not certify runtime binding.
    pub fn selected(&self) -> Option<FieldOrigin> {
        self.unresolved.is_empty().then(|| self.latest_stored())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlueprintRelation {
    Same,
    Ancestor,
    Unrelated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParentLink {
    pub child_id: u64,
    pub parent_id: u64,
    pub child_blueprint: u64,
    pub parent_blueprint: u64,
    pub blueprint_relation: BlueprintRelation,
    pub matching_binding_names: bool,
}

/// Encoded fields inherited through explicit prototype-header references.
/// Missing fields remain missing; host-language/class defaults are never invented.
pub struct PrototypeValues {
    target_id: u64,
    layers: Vec<PrototypeLayer>,
    parent_links: Vec<ParentLink>,
    fields: BTreeMap<FieldKey, FieldView>,
    statistics: ValueStatistics,
}

impl PrototypeValues {
    pub fn resolve(source: &mut impl PrototypeSource, id: u64) -> Result<Self> {
        Self::resolve_with_limits(source, id, ValueLimits::default())
    }

    pub fn resolve_with_limits(
        source: &mut impl PrototypeSource,
        id: u64,
        limits: ValueLimits,
    ) -> Result<Self> {
        let mut statistics = ValueStatistics::default();
        let mut layers: Vec<PrototypeLayer> = Vec::new();
        let mut parent_links = Vec::new();
        let mut visited = HashSet::new();
        let mut current = id;
        loop {
            if !visited.insert(current) {
                return Err(fail(
                    ErrorKind::Cycle,
                    current,
                    "cycle in prototype-header parents",
                ));
            }
            charge(
                &mut statistics.loaded_files,
                1,
                limits.max_parent_layers,
                current,
                "parent layer",
            )?;
            let record = source.record_by_id(current).ok_or_else(|| {
                fail(
                    ErrorKind::MissingData,
                    current,
                    "prototype ID absent from directory",
                )
            })?;
            if current == 0 || record.id != current {
                return Err(fail(
                    ErrorKind::InvalidChain,
                    current,
                    "directory ID mismatch or zero ID",
                ));
            }
            let blueprint_id = record.blueprint_id;
            let blueprint = source.blueprint(blueprint_id).ok_or_else(|| {
                fail(
                    ErrorKind::MissingData,
                    current,
                    "directory blueprint is missing",
                )
            })?;
            if blueprint.default_prototype_id != blueprint_id {
                return Err(fail(
                    ErrorKind::InvalidChain,
                    current,
                    "blueprint/default ID mismatch",
                ));
            }
            if record.path.len()
                > limits
                    .max_storage_bytes
                    .saturating_sub(statistics.storage_bytes)
                    .saturating_sub(size_of::<PrototypeLayer>())
            {
                return Err(fail(
                    ErrorKind::Limit,
                    current,
                    "path storage budget exceeded",
                ));
            }
            let path = record.path.clone();
            charge(
                &mut statistics.storage_bytes,
                size_of::<PrototypeLayer>() + path.capacity(),
                limits.max_storage_bytes,
                current,
                "decoded storage",
            )?;
            if let Some(child) = layers.last() {
                let child_blueprint = source.blueprint(child.blueprint_id).ok_or_else(|| {
                    fail(
                        ErrorKind::MissingData,
                        child.prototype_id,
                        "child blueprint disappeared",
                    )
                })?;
                let matching_binding_names =
                    child_blueprint.runtime_binding == blueprint.runtime_binding;
                let relation = blueprint_relation(
                    source,
                    child.blueprint_id,
                    blueprint_id,
                    &mut statistics,
                    limits,
                    current,
                )?;
                parent_links.push(ParentLink {
                    child_id: child.prototype_id,
                    parent_id: current,
                    child_blueprint: child.blueprint_id,
                    parent_blueprint: blueprint_id,
                    blueprint_relation: relation,
                    matching_binding_names,
                });
            }
            let bytes = source.member_uncompressed_size(current).ok_or_else(|| {
                fail(
                    ErrorKind::MissingData,
                    current,
                    "prototype package member is missing",
                )
            })?;
            charge(
                &mut statistics.encoded_bytes,
                bytes,
                limits.max_encoded_bytes,
                current,
                "encoded byte",
            )?;
            let prototype = source
                .load(current)
                .map_err(|error| fail(ErrorKind::Decode, current, error.to_string()))?;
            let Some(parent_id) = prototype.header.reference else {
                return Err(fail(
                    ErrorKind::InvalidChain,
                    current,
                    "top-level reference header absent",
                ));
            };
            if prototype.header.polymorphic
                || (!prototype.header.data_exists && !prototype.groups.is_empty())
            {
                return Err(fail(
                    ErrorKind::InvalidChain,
                    current,
                    "inconsistent top-level header/body",
                ));
            }
            if parent_id == 0 && current != blueprint_id {
                return Err(fail(
                    ErrorKind::InvalidChain,
                    current,
                    "reference-zero terminal is not its blueprint default; no implicit default inserted",
                ));
            }
            account_body(&prototype, &mut statistics, limits, current)?;
            layers.push(PrototypeLayer {
                prototype_id: current,
                blueprint_id,
                path,
                parent_id,
                is_default: current == blueprint_id,
                prototype,
            });
            if parent_id == 0 {
                break;
            }
            current = parent_id;
        }
        layers.reverse();
        parent_links.reverse();
        let lineage_unresolved = parent_links.iter().any(|link| {
            !link.matching_binding_names || link.blueprint_relation == BlueprintRelation::Unrelated
        });
        let mut fields = BTreeMap::<FieldKey, FieldView>::new();
        let mut aliases = BTreeMap::<(u8, u64), BTreeSet<FieldKey>>::new();
        for (layer_index, layer) in layers.iter().enumerate() {
            let containing = source.blueprint(layer.blueprint_id).ok_or_else(|| {
                fail(
                    ErrorKind::MissingData,
                    layer.prototype_id,
                    "containing blueprint disappeared",
                )
            })?;
            for (group_index, group) in layer.prototype.groups.iter().enumerate() {
                let declaring = source.blueprint(group.blueprint_id).ok_or_else(|| {
                    fail(
                        ErrorKind::MissingData,
                        layer.prototype_id,
                        "declaring blueprint missing",
                    )
                })?;
                for (section, entries) in [
                    (FieldSection::Simple, &group.simple_fields),
                    (FieldSection::List, &group.list_fields),
                ] {
                    for (field_index, field) in entries.iter().enumerate() {
                        let schema =
                            source.blueprint(field.schema_blueprint_id).ok_or_else(|| {
                                fail(
                                    ErrorKind::MissingData,
                                    layer.prototype_id,
                                    "matched field schema missing",
                                )
                            })?;
                        let mut member = None;
                        for candidate in &schema.members {
                            charge(
                                &mut statistics.schema_steps,
                                1,
                                limits.max_schema_steps,
                                layer.prototype_id,
                                "field schema lookup",
                            )?;
                            if candidate.id == field.id {
                                member = Some(candidate);
                                break;
                            }
                        }
                        let member = member.ok_or_else(|| {
                            fail(
                                ErrorKind::InvalidChain,
                                layer.prototype_id,
                                "field ID absent from declared schema",
                            )
                        })?;
                        validate_data(
                            &field.data,
                            member.base_type,
                            member.structure,
                            section,
                            layer.prototype_id,
                        )?;
                        let key = FieldKey {
                            declaring_blueprint_id: group.blueprint_id,
                            copy_number: group.copy_number,
                            field_id: field.id,
                        };
                        let origin = FieldOrigin {
                            layer: layer_index,
                            group: group_index,
                            section,
                            field: field_index,
                        };
                        let view = fields.entry(key).or_insert_with(|| FieldView {
                            key,
                            assignments: Vec::new(),
                            unresolved: BTreeSet::new(),
                        });
                        if let Some(previous) = view.assignments.last() {
                            if previous.layer == layer_index {
                                view.unresolved.insert(Unresolved::DuplicateSlotInLayer);
                            }
                            let prior =
                                stored_field(&layers, *previous).expect("stored field origin");
                            if prior.schema_blueprint_id != field.schema_blueprint_id
                                || previous.section != section
                            {
                                view.unresolved.insert(Unresolved::ConflictingSchema);
                            }
                        }
                        view.assignments.push(origin);
                        if lineage_unresolved {
                            view.unresolved.insert(Unresolved::BlueprintLineage);
                        }
                        if group.copy_number != 0 {
                            view.unresolved.insert(Unresolved::MixinCopy);
                        }
                        if containing.runtime_binding.is_empty()
                            || declaring.runtime_binding != containing.runtime_binding
                            || schema.runtime_binding != containing.runtime_binding
                        {
                            view.unresolved.insert(Unresolved::RuntimeAssignmentContext);
                        }
                        if member.base_type == BaseType::EmbeddedPrototype {
                            view.unresolved.insert(Unresolved::EmbeddedAssignment);
                        }
                        aliases
                            .entry((group.copy_number, field.id))
                            .or_default()
                            .insert(key);
                    }
                }
            }
        }
        for keys in aliases.values().filter(|keys| keys.len() > 1) {
            for key in keys {
                fields
                    .get_mut(key)
                    .expect("known alias key")
                    .unresolved
                    .insert(Unresolved::CrossGroupAlias);
            }
        }
        Ok(Self {
            target_id: id,
            layers,
            parent_links,
            fields,
            statistics,
        })
    }

    pub fn target_id(&self) -> u64 {
        self.target_id
    }
    pub fn layers(&self) -> &[PrototypeLayer] {
        &self.layers
    }
    pub fn parent_links(&self) -> &[ParentLink] {
        &self.parent_links
    }
    pub fn fields(&self) -> impl Iterator<Item = &FieldView> {
        self.fields.values()
    }
    pub fn field(&self, key: FieldKey) -> Option<&FieldView> {
        self.fields.get(&key)
    }
    pub fn stored(&self, origin: FieldOrigin) -> Option<&Field> {
        stored_field(&self.layers, origin)
    }
    pub fn origin_layer(&self, origin: FieldOrigin) -> Option<&PrototypeLayer> {
        self.layers.get(origin.layer)
    }
    pub fn statistics(&self) -> ValueStatistics {
        self.statistics
    }
}

fn stored_field(layers: &[PrototypeLayer], origin: FieldOrigin) -> Option<&Field> {
    let group = layers
        .get(origin.layer)?
        .prototype
        .groups
        .get(origin.group)?;
    match origin.section {
        FieldSection::Simple => group.simple_fields.get(origin.field),
        FieldSection::List => group.list_fields.get(origin.field),
    }
}

fn charge(value: &mut usize, count: usize, maximum: usize, id: u64, name: &str) -> Result<()> {
    *value = value
        .checked_add(count)
        .filter(|&next| next <= maximum)
        .ok_or_else(|| fail(ErrorKind::Limit, id, format!("{name} budget exceeded")))?;
    Ok(())
}

fn blueprint_relation(
    source: &impl PrototypeSource,
    child: u64,
    parent: u64,
    stats: &mut ValueStatistics,
    limits: ValueLimits,
    id: u64,
) -> Result<BlueprintRelation> {
    if child == parent {
        return Ok(BlueprintRelation::Same);
    }
    let mut pending = vec![(child, false)];
    let mut active = HashSet::new();
    let mut complete = HashSet::new();
    let mut found = false;
    while let Some((current, leaving)) = pending.pop() {
        if leaving {
            active.remove(&current);
            complete.insert(current);
            continue;
        }
        charge(
            &mut stats.schema_steps,
            1,
            limits.max_schema_steps,
            id,
            "blueprint traversal",
        )?;
        if complete.contains(&current) {
            continue;
        }
        if !active.insert(current) {
            return Err(fail(
                ErrorKind::Cycle,
                id,
                "cycle in blueprint parent metadata",
            ));
        }
        let blueprint = source
            .blueprint(current)
            .ok_or_else(|| fail(ErrorKind::MissingData, id, "missing blueprint ancestor"))?;
        if blueprint.default_prototype_id != current {
            return Err(fail(
                ErrorKind::InvalidChain,
                id,
                "ancestor blueprint/default ID mismatch",
            ));
        }
        found |= current == parent;
        let remaining = limits.max_schema_steps.saturating_sub(stats.schema_steps);
        if blueprint.parents.len() > remaining
            || pending.len() > remaining.saturating_mul(2).saturating_add(1)
        {
            return Err(fail(
                ErrorKind::Limit,
                id,
                "blueprint traversal frontier exceeds budget",
            ));
        }
        pending.push((current, true));
        pending.extend(
            blueprint
                .parents
                .iter()
                .rev()
                .map(|entry| (entry.id, false)),
        );
    }
    Ok(if found {
        BlueprintRelation::Ancestor
    } else {
        BlueprintRelation::Unrelated
    })
}

fn account_body(
    prototype: &Prototype,
    stats: &mut ValueStatistics,
    limits: ValueLimits,
    id: u64,
) -> Result<()> {
    let mut pending = vec![(prototype, 0usize)];
    while let Some((node, depth)) = pending.pop() {
        if depth > limits.max_embedded_depth {
            return Err(fail(ErrorKind::Limit, id, "embedded depth budget exceeded"));
        }
        charge(&mut stats.nodes, 1, limits.max_nodes, id, "decoded node")?;
        charge(
            &mut stats.groups,
            node.groups.len(),
            limits.max_groups,
            id,
            "decoded group",
        )?;
        storage::<crate::calligraphy::prototype::FieldGroup>(
            node.groups.capacity(),
            stats,
            limits,
            id,
        )?;
        for group in &node.groups {
            for fields in [&group.simple_fields, &group.list_fields] {
                charge(
                    &mut stats.fields,
                    fields.len(),
                    limits.max_fields,
                    id,
                    "decoded field",
                )?;
                storage::<Field>(fields.capacity(), stats, limits, id)?;
                for field in fields {
                    let values = match &field.data {
                        FieldData::Simple(value) => std::slice::from_ref(value),
                        FieldData::List(values) => {
                            storage::<Value>(values.capacity(), stats, limits, id)?;
                            values.as_slice()
                        }
                    };
                    charge(
                        &mut stats.values,
                        values.len(),
                        limits.max_values,
                        id,
                        "decoded value",
                    )?;
                    for value in values {
                        if let Value::EmbeddedPrototype(child) = value {
                            storage::<Prototype>(1, stats, limits, id)?;
                            if pending.len() >= limits.max_nodes.saturating_sub(stats.nodes) {
                                return Err(fail(
                                    ErrorKind::Limit,
                                    id,
                                    "decoded node frontier exceeds budget",
                                ));
                            }
                            pending.push((
                                child,
                                depth.checked_add(1).ok_or_else(|| {
                                    fail(ErrorKind::Limit, id, "embedded depth overflow")
                                })?,
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn storage<T>(
    count: usize,
    stats: &mut ValueStatistics,
    limits: ValueLimits,
    id: u64,
) -> Result<()> {
    let bytes = count
        .checked_mul(size_of::<T>())
        .ok_or_else(|| fail(ErrorKind::Limit, id, "decoded storage overflow"))?;
    charge(
        &mut stats.storage_bytes,
        bytes,
        limits.max_storage_bytes,
        id,
        "decoded storage",
    )
}

fn validate_data(
    data: &FieldData,
    base: BaseType,
    structure: StructureType,
    section: FieldSection,
    id: u64,
) -> Result<()> {
    let values = match (data, structure, section) {
        (FieldData::Simple(value), StructureType::Simple, FieldSection::Simple) => {
            std::slice::from_ref(value)
        }
        (FieldData::List(values), StructureType::List, FieldSection::List) => values,
        _ => {
            return Err(fail(
                ErrorKind::InvalidChain,
                id,
                "field structure disagrees with schema",
            ));
        }
    };
    for value in values {
        let matches = matches!(
            (value, base),
            (Value::Asset(_), BaseType::Asset)
                | (Value::Boolean(_), BaseType::Boolean)
                | (Value::Curve(_), BaseType::Curve)
                | (Value::Double(_), BaseType::Double)
                | (Value::Long(_), BaseType::Long)
                | (Value::Prototype(_), BaseType::Prototype)
                | (Value::EmbeddedPrototype(_), BaseType::EmbeddedPrototype)
                | (Value::String(_), BaseType::String)
                | (Value::Type(_), BaseType::Type)
        );
        if !matches {
            return Err(fail(
                ErrorKind::InvalidChain,
                id,
                "field value type disagrees with schema",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calligraphy::{
        BlueprintMember, BlueprintReference, PrototypeDataHeader, prototype::FieldGroup,
    };

    struct Source {
        records: BTreeMap<u64, PrototypeRecord>,
        schemas: BTreeMap<u64, Blueprint>,
        bodies: BTreeMap<u64, Prototype>,
        loads: Vec<u64>,
    }
    impl PrototypeSource for Source {
        fn record_by_id(&self, id: u64) -> Option<&PrototypeRecord> {
            self.records.get(&id)
        }
        fn blueprint(&self, id: u64) -> Option<&Blueprint> {
            self.schemas.get(&id)
        }
        fn member_uncompressed_size(&self, id: u64) -> Option<usize> {
            self.bodies.contains_key(&id).then_some(100)
        }
        fn load(&mut self, id: u64) -> io::Result<Prototype> {
            self.loads.push(id);
            self.bodies
                .get(&id)
                .cloned()
                .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
        }
    }
    fn fixture() -> Source {
        let members = [
            (101, BaseType::Long, StructureType::Simple),
            (102, BaseType::Prototype, StructureType::List),
            (103, BaseType::EmbeddedPrototype, StructureType::List),
            (104, BaseType::EmbeddedPrototype, StructureType::Simple),
        ]
        .into_iter()
        .map(|(id, base_type, structure)| BlueprintMember {
            id,
            name: format!("Field{id}"),
            base_type,
            structure,
            subtype: None,
        })
        .collect();
        let blueprint = Blueprint {
            runtime_binding: "Synthetic".into(),
            default_prototype_id: 1,
            parents: vec![],
            contributors: vec![],
            members,
        };
        let records = [1, 10, 20]
            .into_iter()
            .map(|id| {
                (
                    id,
                    PrototypeRecord {
                        id,
                        guid: id + 100,
                        blueprint_id: 1,
                        flags: 0,
                        path: format!(
                            "Synthetic{id}.{}",
                            if id == 1 { "defaults" } else { "prototype" }
                        ),
                    },
                )
            })
            .collect();
        Source {
            records,
            schemas: BTreeMap::from([(1, blueprint)]),
            bodies: BTreeMap::from([
                (
                    1,
                    body(
                        0,
                        vec![field(101, FieldData::Simple(Value::Long(7)))],
                        vec![],
                    ),
                ),
                (
                    10,
                    body(
                        1,
                        vec![field(101, FieldData::Simple(Value::Long(9)))],
                        vec![],
                    ),
                ),
                (20, body(1, vec![], vec![])),
            ]),
            loads: vec![],
        }
    }
    fn field(id: u64, data: FieldData) -> Field {
        Field {
            id,
            schema_blueprint_id: 1,
            data,
        }
    }
    fn body(parent: u64, simple: Vec<Field>, list: Vec<Field>) -> Prototype {
        Prototype {
            header: PrototypeDataHeader {
                reference: Some(parent),
                data_exists: true,
                polymorphic: false,
            },
            groups: vec![FieldGroup {
                blueprint_id: 1,
                copy_number: 0,
                simple_fields: simple,
                list_fields: list,
            }],
        }
    }
    fn key(id: u64) -> FieldKey {
        FieldKey {
            declaring_blueprint_id: 1,
            copy_number: 0,
            field_id: id,
        }
    }

    #[test]
    fn lazy_parent_override_preserves_both_origins() {
        let mut source = fixture();
        let view = PrototypeValues::resolve(&mut source, 10).unwrap();
        assert_eq!(source.loads, vec![10, 1]);
        assert_eq!(
            view.layers()
                .iter()
                .map(|l| l.prototype_id)
                .collect::<Vec<_>>(),
            vec![1, 10]
        );
        assert!(view.layers()[0].is_default);
        let field = view.field(key(101)).unwrap();
        assert_eq!(field.assignments().len(), 2);
        assert_eq!(
            view.stored(field.selected().unwrap()).unwrap().data,
            FieldData::Simple(Value::Long(9))
        );
        assert_eq!(
            view.stored(field.assignments()[0]).unwrap().data,
            FieldData::Simple(Value::Long(7))
        );
    }

    #[test]
    fn reference_only_child_inherits_and_absent_fields_stay_absent() {
        let mut source = fixture();
        source.bodies.insert(
            10,
            Prototype {
                header: PrototypeDataHeader {
                    reference: Some(1),
                    data_exists: false,
                    polymorphic: false,
                },
                groups: vec![],
            },
        );
        let view = PrototypeValues::resolve(&mut source, 10).unwrap();
        let origin = view.field(key(101)).unwrap().selected().unwrap();
        assert_eq!(view.origin_layer(origin).unwrap().prototype_id, 1);
        assert!(view.field(key(999)).is_none());
        assert_eq!(source.loads.len(), 2);
    }

    #[test]
    fn plain_empty_list_replaces_but_embedded_empty_list_stays_unresolved() {
        let mut source = fixture();
        source.bodies.insert(
            1,
            body(
                0,
                vec![],
                vec![
                    field(102, FieldData::List(vec![Value::Prototype(20)])),
                    field(103, FieldData::List(vec![])),
                ],
            ),
        );
        source.bodies.insert(
            10,
            body(
                1,
                vec![],
                vec![
                    field(102, FieldData::List(vec![])),
                    field(103, FieldData::List(vec![])),
                ],
            ),
        );
        let view = PrototypeValues::resolve(&mut source, 10).unwrap();
        let selected = view.field(key(102)).unwrap().selected().unwrap();
        assert_eq!(view.origin_layer(selected).unwrap().prototype_id, 10);
        assert_eq!(view.stored(selected).unwrap().data, FieldData::List(vec![]));
        let embedded = view.field(key(103)).unwrap();
        assert!(embedded.selected().is_none());
        assert!(
            embedded
                .unresolved()
                .contains(&Unresolved::EmbeddedAssignment)
        );
        assert_eq!(embedded.assignments().len(), 2);
    }

    #[test]
    fn parent_cycles_missing_ids_and_nondefault_terminal_fail() {
        let mut source = fixture();
        source.bodies.get_mut(&1).unwrap().header.reference = Some(10);
        assert_eq!(
            PrototypeValues::resolve(&mut source, 10)
                .err()
                .unwrap()
                .kind,
            ErrorKind::Cycle
        );
        let mut source = fixture();
        source.bodies.get_mut(&10).unwrap().header.reference = Some(999);
        assert_eq!(
            PrototypeValues::resolve(&mut source, 10)
                .err()
                .unwrap()
                .kind,
            ErrorKind::MissingData
        );
        let mut source = fixture();
        source.bodies.get_mut(&10).unwrap().header.reference = Some(0);
        assert_eq!(
            PrototypeValues::resolve(&mut source, 10)
                .err()
                .unwrap()
                .kind,
            ErrorKind::InvalidChain
        );
    }

    #[test]
    fn limits_bound_loading_and_retained_values() {
        let defaults = ValueLimits::default();
        for limits in [
            ValueLimits {
                max_parent_layers: 1,
                ..defaults
            },
            ValueLimits {
                max_encoded_bytes: 100,
                ..defaults
            },
        ] {
            let mut source = fixture();
            assert_eq!(
                PrototypeValues::resolve_with_limits(&mut source, 10, limits)
                    .err()
                    .unwrap()
                    .kind,
                ErrorKind::Limit
            );
            assert_eq!(source.loads, vec![10]);
        }
        for limits in [
            ValueLimits {
                max_storage_bytes: 0,
                ..defaults
            },
            ValueLimits {
                max_nodes: 0,
                ..defaults
            },
            ValueLimits {
                max_groups: 0,
                ..defaults
            },
            ValueLimits {
                max_fields: 0,
                ..defaults
            },
            ValueLimits {
                max_values: 0,
                ..defaults
            },
            ValueLimits {
                max_schema_steps: 0,
                ..defaults
            },
        ] {
            let mut source = fixture();
            assert_eq!(
                PrototypeValues::resolve_with_limits(&mut source, 10, limits)
                    .err()
                    .unwrap()
                    .kind,
                ErrorKind::Limit
            );
        }
        let mut source = fixture();
        source.bodies.insert(
            10,
            body(
                1,
                vec![field(
                    104,
                    FieldData::Simple(Value::EmbeddedPrototype(Box::new(body(1, vec![], vec![])))),
                )],
                vec![],
            ),
        );
        assert_eq!(
            PrototypeValues::resolve_with_limits(
                &mut source,
                10,
                ValueLimits {
                    max_embedded_depth: 0,
                    ..defaults
                }
            )
            .err()
            .unwrap()
            .kind,
            ErrorKind::Limit
        );
    }

    #[test]
    fn conflicting_contexts_and_duplicate_slots_are_explicit() {
        let mut source = fixture();
        let mut group = source.bodies[&10].groups[0].clone();
        group.copy_number = 1;
        source.bodies.get_mut(&10).unwrap().groups.push(group);
        let repeated = source.bodies[&10].groups[0].simple_fields[0].clone();
        source.bodies.get_mut(&10).unwrap().groups[0]
            .simple_fields
            .push(repeated);
        let view = PrototypeValues::resolve(&mut source, 10).unwrap();
        assert!(
            view.field(key(101))
                .unwrap()
                .unresolved()
                .contains(&Unresolved::DuplicateSlotInLayer)
        );
        let copy = FieldKey {
            copy_number: 1,
            ..key(101)
        };
        assert!(
            view.field(copy)
                .unwrap()
                .unresolved()
                .contains(&Unresolved::MixinCopy)
        );
    }

    #[test]
    fn metadata_lineage_is_checked_without_claiming_runtime_types() {
        let mut source = fixture();
        let mut child = source.schemas[&1].clone();
        child.default_prototype_id = 2;
        child.runtime_binding = "DifferentBinding".into();
        source.schemas.insert(2, child);
        source.records.get_mut(&10).unwrap().blueprint_id = 2;
        let view = PrototypeValues::resolve(&mut source, 10).unwrap();
        assert_eq!(
            view.parent_links()[0].blueprint_relation,
            BlueprintRelation::Unrelated
        );
        assert!(!view.parent_links()[0].matching_binding_names);
        assert!(view.field(key(101)).unwrap().selected().is_none());
        assert!(
            view.field(key(101))
                .unwrap()
                .unresolved()
                .contains(&Unresolved::BlueprintLineage)
        );
    }

    #[test]
    fn blueprint_cycles_and_cross_group_aliases_do_not_silently_merge() {
        let mut source = fixture();
        let mut child = source.schemas[&1].clone();
        child.default_prototype_id = 2;
        child.parents.push(BlueprintReference { id: 2, copies: 1 });
        source.schemas.insert(2, child);
        source.records.get_mut(&10).unwrap().blueprint_id = 2;
        assert_eq!(
            PrototypeValues::resolve(&mut source, 10)
                .err()
                .unwrap()
                .kind,
            ErrorKind::Cycle
        );
        let mut source = fixture();
        let mut other = source.schemas[&1].clone();
        other.default_prototype_id = 2;
        source.schemas.insert(2, other);
        let mut group = source.bodies[&10].groups[0].clone();
        group.blueprint_id = 2;
        source.bodies.get_mut(&10).unwrap().groups.push(group);
        let view = PrototypeValues::resolve(&mut source, 10).unwrap();
        assert!(
            view.fields()
                .all(|field| field.unresolved().contains(&Unresolved::CrossGroupAlias))
        );
    }
}
