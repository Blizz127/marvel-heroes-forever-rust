//! Bounded configured localization labels for the evidenced ASCII profile.
//! Source identity and native operations: docs/LOCALIZED_LABEL_LOOKUP.md.

use super::localized_prototype::{self, PreparedPrototype};
use crate::{
    calligraphy::{
        AssetType, AssetValue, BaseType, Blueprint, BlueprintMember, Directory, DirectoryKind,
        DirectoryRecord, GenericRecord, PrototypeRecord, StructureType,
        prototype::{FieldData, Prototype, PrototypeLimits, SchemaSet, Value},
    },
    prototype_values::{FieldKey, FieldOrigin, PrototypeSource, PrototypeValues, ValueLimits},
    sip::{SipArchive, SipLimits},
};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufReader, Read, Seek},
    path::Path,
    sync::Arc,
};

/// This exact name is selected by the recovered native setup function.
pub const GLOBALS_MEMBER: &str = "Calligraphy/Globals/Globals.defaults";

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub sip: SipLimits,
    /// Sum of the directory, blueprint and type members decoded during open.
    pub max_metadata_bytes: usize,
    pub max_directory_records: usize,
    pub max_blueprints: usize,
    pub max_asset_entries: usize,
    pub max_label_bytes: usize,
    pub max_path_bytes: usize,
    /// Applied separately to each of the two explicit configuration defaults.
    /// The resolver additionally enforces exactly one parentless layer.
    pub configuration: ValueLimits,
    /// Applied to each resolved localized prototype and its explicit parents.
    pub localized_prototype: ValueLimits,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            sip: SipLimits::default(),
            max_metadata_bytes: 128 * 1024 * 1024,
            max_directory_records: 200_000,
            max_blueprints: 10_000,
            max_asset_entries: 1_000_000,
            max_label_bytes: 1024,
            max_path_bytes: 4096,
            configuration: ValueLimits {
                max_parent_layers: 1,
                max_encoded_bytes: 1024 * 1024,
                max_storage_bytes: 8 * 1024 * 1024,
                max_nodes: 10_000,
                max_groups: 10_000,
                max_fields: 100_000,
                max_values: 100_000,
                ..ValueLimits::default()
            },
            localized_prototype: ValueLimits::default(),
        }
    }
}

/// Explicit source data behind the configured directory; no inferred defaults.
pub struct Configuration {
    globals: PrototypeValues,
    ui_globals: PrototypeValues,
    ui_globals_origin: FieldOrigin,
    directory_origin: FieldOrigin,
    directory_type: GenericRecord,
    type_record_index: usize,
    directory_asset: AssetValue,
    directory_asset_index: usize,
}

impl Configuration {
    pub fn globals(&self) -> &PrototypeValues {
        &self.globals
    }
    pub fn ui_globals(&self) -> &PrototypeValues {
        &self.ui_globals
    }
    pub fn ui_globals_origin(&self) -> FieldOrigin {
        self.ui_globals_origin
    }
    pub fn directory_origin(&self) -> FieldOrigin {
        self.directory_origin
    }
    pub fn directory_type(&self) -> &GenericRecord {
        &self.directory_type
    }
    pub fn type_record_index(&self) -> usize {
        self.type_record_index
    }
    pub fn directory_asset(&self) -> &AssetValue {
        &self.directory_asset
    }
    pub fn directory_asset_index(&self) -> usize {
        self.directory_asset_index
    }
}

/// A member/class-checked result. The public hash is not an unchecked lookup API.
pub struct ResolvedLabel {
    pub label: String,
    pub joined_path: String,
    pub hash_bytes: Vec<u8>,
    pub prototype_id: u64,
    configuration: Arc<Configuration>,
    prototype: PreparedPrototype,
}

impl ResolvedLabel {
    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }
    pub fn prototype(&self) -> &PreparedPrototype {
        &self.prototype
    }
}

/// One pinned seekable source owns configuration, indexes and all later reads.
/// Renaming/replacing the original pathname cannot redirect subsequent loads.
/// In-place mutation of the caller's source remains outside this read-only API.
pub struct ConfiguredLabels<R = BufReader<File>> {
    archive: SipArchive<R>,
    directory: Directory,
    blueprints: Vec<Blueprint>,
    by_id: HashMap<u64, usize>,
    configuration: Arc<Configuration>,
    limits: Limits,
}

impl ConfiguredLabels<BufReader<File>> {
    pub fn open(path: impl AsRef<Path>, limits: Limits) -> io::Result<Self> {
        Self::from_reader(BufReader::new(File::open(path)?), limits)
    }
}

impl<R: Read + Seek> ConfiguredLabels<R> {
    pub fn from_reader(reader: R, limits: Limits) -> io::Result<Self> {
        let mut archive = SipArchive::from_reader_with_limits(reader, limits.sip)?;
        let mut metadata_bytes = 0;
        let directory = read_directory(
            &mut archive,
            "Calligraphy/Prototype.directory",
            DirectoryKind::Prototype,
            limits,
            &mut metadata_bytes,
        )?;
        let mut by_id = HashMap::new();
        let mut paths = HashSet::new();
        by_id
            .try_reserve(directory.records.len())
            .map_err(allocation)?;
        paths
            .try_reserve(directory.records.len())
            .map_err(allocation)?;
        let mut globals_id = None;
        for (index, entry) in directory.records.iter().enumerate() {
            let DirectoryRecord::Prototype(record) = entry else {
                return Err(invalid("non-prototype record in Prototype.directory"));
            };
            if record.id == 0 || record.path.len() > limits.max_path_bytes {
                return Err(invalid("prototype identity is zero or path exceeds budget"));
            }
            if by_id.insert(record.id, index).is_some() || !paths.insert(record.path.as_str()) {
                return Err(invalid("duplicate prototype ID or member path"));
            }
            let member = format!("Calligraphy/{}", record.path);
            if archive.entry(&member).is_none() {
                return Err(missing("prototype directory member is absent"));
            }
            if member == GLOBALS_MEMBER {
                globals_id = Some(record.id);
            }
        }
        let globals_id =
            globals_id.ok_or_else(|| missing("native-selected Globals.defaults is absent"))?;
        let blueprint_indices: Vec<_> = archive
            .entries()
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| entry.name.ends_with(".blueprint").then_some(i))
            .collect();
        if blueprint_indices.len() > limits.max_blueprints {
            return Err(invalid("blueprint count exceeds metadata budget"));
        }
        let mut blueprints = Vec::new();
        blueprints
            .try_reserve(blueprint_indices.len())
            .map_err(allocation)?;
        for index in blueprint_indices {
            let bytes = read_metadata_index(&mut archive, index, limits, &mut metadata_bytes)?;
            blueprints.push(Blueprint::parse(&bytes).map_err(decode)?);
        }
        // The existing schema index rejects duplicate IDs/declarations and
        // validates reference relationships before any source is exposed.
        let schemas = SchemaSet::new(&blueprints, &directory).map_err(decode)?;
        let mut source = Source {
            archive: &mut archive,
            schemas,
            directory: &directory,
            by_id: &by_id,
            limits: limits.configuration,
        };
        let (globals, ui_field) = configuration_default(
            &mut source,
            globals_id,
            "GlobalsPrototype",
            "UIGlobals",
            BaseType::Prototype,
            limits.configuration,
        )?;
        let (ui_id, ui_globals_origin) = explicit_reference(&globals, &ui_field)?;
        let ui_record = source
            .record_by_id(ui_id)
            .ok_or_else(|| missing("configured UIGlobals reference is absent"))?;
        if Some(ui_record.blueprint_id) != ui_field.subtype {
            return Err(unsupported(
                "configured UIGlobals differs from its declared subtype",
            ));
        }
        let (ui_globals, directory_field) = configuration_default(
            &mut source,
            ui_id,
            "UIGlobalsPrototype",
            "LocalizedInfoDirectory",
            BaseType::Asset,
            limits.configuration,
        )?;
        let (asset_id, directory_origin) = explicit_reference(&ui_globals, &directory_field)?;
        drop(source);
        let type_id = directory_field
            .subtype
            .ok_or_else(|| invalid("directory Asset lacks its type"))?;
        let (directory_type, type_record_index, directory_asset, directory_asset_index) =
            directory_asset(&mut archive, type_id, asset_id, limits, &mut metadata_bytes)?;
        ascii(
            &directory_asset.name,
            limits.max_path_bytes,
            "configured directory",
        )?;
        if directory_asset.name.is_empty() {
            return Err(unsupported(
                "empty configured directory is outside the evidenced profile",
            ));
        }
        let configuration = Arc::new(Configuration {
            globals,
            ui_globals,
            ui_globals_origin,
            directory_origin,
            directory_type,
            type_record_index,
            directory_asset,
            directory_asset_index,
        });
        Ok(Self {
            archive,
            directory,
            blueprints,
            by_id,
            configuration,
            limits,
        })
    }

    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }

    /// Resolve exact native-derived membership, then the existing typed scalar
    /// profile. Missing names, unsupported labels and resource failures are errors.
    pub fn resolve(&mut self, label: &str) -> io::Result<ResolvedLabel> {
        ascii(label, self.limits.max_label_bytes, "localization label")?;
        if label.is_empty() {
            return Err(unsupported(
                "empty labels are guarded by the saved script wrapper",
            ));
        }
        let joined_path = join(
            &self.configuration.directory_asset.name,
            label,
            self.limits.max_path_bytes,
        )?;
        let hash_bytes = transform(joined_path.as_bytes());
        let prototype_id = identity(&hash_bytes);
        let record = record(&self.directory, &self.by_id, prototype_id).ok_or_else(|| {
            missing(format!(
                "localized label has no resource at {prototype_id:#x}"
            ))
        })?;
        if record.flags != 0 {
            return Err(unsupported(
                "flagged localized resource is outside the supported profile",
            ));
        }
        let schemas = SchemaSet::new(&self.blueprints, &self.directory).map_err(decode)?;
        let mut source = Source {
            archive: &mut self.archive,
            schemas,
            directory: &self.directory,
            by_id: &self.by_id,
            limits: self.limits.localized_prototype,
        };
        let prototype = localized_prototype::resolve(
            &mut source,
            prototype_id,
            self.limits.localized_prototype,
        )?;
        Ok(ResolvedLabel {
            label: label.to_owned(),
            joined_path,
            hash_bytes,
            prototype_id,
            configuration: Arc::clone(&self.configuration),
            prototype,
        })
    }
}

struct Source<'a, R> {
    archive: &'a mut SipArchive<R>,
    schemas: SchemaSet<'a>,
    directory: &'a Directory,
    by_id: &'a HashMap<u64, usize>,
    limits: ValueLimits,
}

impl<R: Read + Seek> PrototypeSource for Source<'_, R> {
    fn record_by_id(&self, id: u64) -> Option<&PrototypeRecord> {
        record(self.directory, self.by_id, id)
    }
    fn blueprint(&self, id: u64) -> Option<&Blueprint> {
        self.schemas.blueprint(id)
    }
    fn member_uncompressed_size(&self, id: u64) -> Option<usize> {
        let record = self.record_by_id(id)?;
        Some(
            self.archive
                .entry(&format!("Calligraphy/{}", record.path))?
                .uncompressed_size,
        )
    }
    fn load(&mut self, id: u64) -> io::Result<Prototype> {
        let record = self
            .record_by_id(id)
            .ok_or_else(|| missing("prototype source ID is absent"))?;
        if record.flags != u8::from(record.id == record.blueprint_id) {
            return Err(unsupported(
                "prototype directory flags differ from the observed row profile",
            ));
        }
        let blueprint_id = record.blueprint_id;
        let member = format!("Calligraphy/{}", record.path);
        let size = self
            .archive
            .entry(&member)
            .ok_or_else(|| missing("prototype member is absent"))?
            .uncompressed_size;
        if size > self.limits.max_encoded_bytes {
            return Err(invalid("prototype member exceeds encoded byte budget"));
        }
        let bytes = self.archive.read(&member)?;
        Prototype::parse_with_limits(
            &bytes,
            blueprint_id,
            &self.schemas,
            PrototypeLimits {
                max_file_bytes: self.limits.max_encoded_bytes,
                max_depth: self.limits.max_embedded_depth,
                max_prototypes: self.limits.max_nodes,
                max_groups: self.limits.max_groups,
                max_fields: self.limits.max_fields,
                max_values: self.limits.max_values,
                max_schema_steps: self.limits.max_schema_steps,
            },
        )
        .map_err(decode)
    }
}

fn record<'a>(
    directory: &'a Directory,
    by_id: &HashMap<u64, usize>,
    id: u64,
) -> Option<&'a PrototypeRecord> {
    match directory.records.get(*by_id.get(&id)?)? {
        DirectoryRecord::Prototype(record) => Some(record),
        _ => None,
    }
}

fn configuration_default(
    source: &mut impl PrototypeSource,
    id: u64,
    binding: &str,
    field_name: &str,
    base: BaseType,
    mut limits: ValueLimits,
) -> io::Result<(PrototypeValues, BlueprintMember)> {
    let record = source
        .record_by_id(id)
        .ok_or_else(|| missing("configuration source is absent"))?;
    if record.flags != 1 || record.blueprint_id != id {
        return Err(unsupported(
            "configuration must be an explicit class default with observed directory flags",
        ));
    }
    let schema = source
        .blueprint(record.blueprint_id)
        .ok_or_else(|| missing("configuration schema is absent"))?;
    if schema.runtime_binding != binding
        || schema.default_prototype_id != id
        || !schema.parents.is_empty()
        || !schema.contributors.is_empty()
    {
        return Err(unsupported(
            "configuration class or schema ancestry is unproved",
        ));
    }
    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    let mut selected = None;
    for member in &schema.members {
        if member.id == 0 || !ids.insert(member.id) || !names.insert(member.name.as_str()) {
            return Err(invalid(
                "configuration schema has zero or duplicate field identities",
            ));
        }
        if member.name == field_name {
            selected = Some(member.clone());
        }
    }
    let field = selected.ok_or_else(|| missing("required configuration declaration is absent"))?;
    if field.base_type != base
        || field.structure != StructureType::Simple
        || field.subtype.is_none_or(|id| id == 0)
    {
        return Err(unsupported(
            "configuration reference declaration has an unsupported type",
        ));
    }
    limits.max_parent_layers = limits.max_parent_layers.min(1);
    let values = PrototypeValues::resolve_with_limits(source, id, limits).map_err(decode)?;
    if values.layers().len() != 1 || values.layers()[0].parent_id != 0 {
        return Err(unsupported(
            "configuration inheritance is outside the evidenced profile",
        ));
    }
    for group in &values.layers()[0].prototype().groups {
        if group.blueprint_id != id || group.copy_number != 0 {
            return Err(unsupported(
                "configuration assignment group has unproved schema/copy identity",
            ));
        }
        for stored in group.simple_fields.iter().chain(&group.list_fields) {
            if stored.schema_blueprint_id != id {
                return Err(unsupported(
                    "configuration field has a different declaring schema",
                ));
            }
        }
    }
    // Ambiguous duplicates anywhere in this source are rejected; unrelated
    // embedded/list values are retained structurally without runtime evaluation.
    if values.fields().any(|field| field.assignments().len() != 1) {
        return Err(invalid(
            "configuration source contains duplicate field assignments",
        ));
    }
    Ok((values, field))
}

fn explicit_reference(
    values: &PrototypeValues,
    declaration: &BlueprintMember,
) -> io::Result<(u64, FieldOrigin)> {
    let view = values
        .field(FieldKey {
            declaring_blueprint_id: values.target_id(),
            copy_number: 0,
            field_id: declaration.id,
        })
        .ok_or_else(|| missing("configuration reference has no explicit stored assignment"))?;
    let origin = view
        .selected()
        .ok_or_else(|| unsupported("configuration reference assignment is ambiguous"))?;
    let field = values
        .stored(origin)
        .ok_or_else(|| invalid("configuration reference origin is invalid"))?;
    let id = match (&field.data, declaration.base_type) {
        (FieldData::Simple(Value::Prototype(id)), BaseType::Prototype)
        | (FieldData::Simple(Value::Asset(id)), BaseType::Asset) => *id,
        _ => {
            return Err(invalid(
                "configuration reference value differs from its declaration",
            ));
        }
    };
    if id == 0 {
        return Err(unsupported(
            "configuration reference is explicitly zero; no fallback",
        ));
    }
    Ok((id, origin))
}

fn directory_asset<R: Read + Seek>(
    archive: &mut SipArchive<R>,
    type_id: u64,
    asset_id: u64,
    limits: Limits,
    used: &mut usize,
) -> io::Result<(GenericRecord, usize, AssetValue, usize)> {
    let directory = read_directory(
        archive,
        "Calligraphy/Type.directory",
        DirectoryKind::Type,
        limits,
        used,
    )?;
    let mut ids = HashSet::new();
    let mut paths = HashSet::new();
    let mut result = None;
    let mut assets = 0usize;
    let mut declared_type_present = false;
    for (type_index, entry) in directory.records.iter().enumerate() {
        let DirectoryRecord::Generic(record) = entry else {
            return Err(invalid("non-type directory record"));
        };
        if record.id == 0 || !ids.insert(record.id) || !paths.insert(record.path.as_str()) {
            return Err(invalid("duplicate or zero type identity"));
        }
        if record.path.len() > limits.max_path_bytes {
            return Err(invalid("type path exceeds budget"));
        }
        let member = format!("Calligraphy/{}", record.path);
        let bytes = read_metadata(archive, &member, limits, used)?;
        let count = bytes
            .get(4..6)
            .ok_or_else(|| invalid("asset type header is truncated"))?;
        assets = assets
            .checked_add(usize::from(u16::from_le_bytes([count[0], count[1]])))
            .filter(|n| *n <= limits.max_asset_entries)
            .ok_or_else(|| invalid("asset entry budget exceeded"))?;
        let values = AssetType::parse(&bytes).map_err(decode)?;
        let selected_type = record.id == type_id;
        if selected_type && record.flags != 0 {
            return Err(unsupported("configured type has unproved flags"));
        }
        declared_type_present |= selected_type;
        let mut type_ids = HashSet::new();
        let mut type_names = HashSet::new();
        for (asset_index, asset) in values.assets.into_iter().enumerate() {
            if selected_type
                && (!type_ids.insert(asset.id) || !type_names.insert(asset.name.clone()))
            {
                return Err(invalid(
                    "configured type contains duplicate asset identities",
                ));
            }
            if asset.id != asset_id {
                continue;
            }
            if !selected_type || result.is_some() {
                return Err(invalid(
                    "directory Asset ID is ambiguous across type entries",
                ));
            }
            if asset.flags != 0 {
                return Err(unsupported("configured directory asset has unproved flags"));
            }
            result = Some((record.clone(), type_index, asset, asset_index));
        }
    }
    if !declared_type_present {
        return Err(missing("configured directory subtype is absent"));
    }
    result.ok_or_else(|| {
        missing("configured directory Asset reference is absent; no native fallback")
    })
}

fn read_directory<R: Read + Seek>(
    archive: &mut SipArchive<R>,
    member: &str,
    kind: DirectoryKind,
    limits: Limits,
    used: &mut usize,
) -> io::Result<Directory> {
    let bytes = read_metadata(archive, member, limits, used)?;
    let raw = bytes
        .get(4..8)
        .ok_or_else(|| invalid("directory header is truncated"))?;
    let count = u32::from_le_bytes(raw.try_into().expect("checked range")) as usize;
    if count > limits.max_directory_records {
        return Err(invalid("directory count exceeds budget"));
    }
    let result = Directory::parse(&bytes).map_err(decode)?;
    if result.kind != kind {
        return Err(invalid("unexpected Calligraphy directory kind"));
    }
    Ok(result)
}

fn read_metadata<R: Read + Seek>(
    archive: &mut SipArchive<R>,
    member: &str,
    limits: Limits,
    used: &mut usize,
) -> io::Result<Vec<u8>> {
    let entry = archive
        .entry(member)
        .ok_or_else(|| missing(format!("metadata member is absent: {member}")))?;
    *used = used
        .checked_add(entry.uncompressed_size)
        .filter(|n| *n <= limits.max_metadata_bytes)
        .ok_or_else(|| invalid("metadata byte budget exceeded"))?;
    archive.read(member)
}

fn read_metadata_index<R: Read + Seek>(
    archive: &mut SipArchive<R>,
    index: usize,
    limits: Limits,
    used: &mut usize,
) -> io::Result<Vec<u8>> {
    let entry = archive
        .entries()
        .get(index)
        .ok_or_else(|| invalid("metadata index is invalid"))?;
    *used = used
        .checked_add(entry.uncompressed_size)
        .filter(|n| *n <= limits.max_metadata_bytes)
        .ok_or_else(|| invalid("metadata byte budget exceeded"))?;
    archive.read_index(index)
}

fn ascii(text: &str, limit: usize, what: &str) -> io::Result<()> {
    if text.len() > limit {
        return Err(invalid(format!("{what} exceeds byte budget")));
    }
    if !text.is_ascii() || text.as_bytes().contains(&0) {
        return Err(unsupported(format!(
            "{what} requires unproved code-page or NUL conversion"
        )));
    }
    Ok(())
}

fn join(directory: &str, label: &str, limit: usize) -> io::Result<String> {
    let separator = !directory.is_empty() && !directory.ends_with(['/', '\\']);
    let size = directory
        .len()
        .checked_add(usize::from(separator))
        .and_then(|n| n.checked_add(label.len()))
        .and_then(|n| n.checked_add(".prototype".len()))
        .filter(|n| *n <= limit)
        .ok_or_else(|| invalid("joined localization path exceeds budget"))?;
    let mut path = String::new();
    path.try_reserve_exact(size).map_err(allocation)?;
    path.push_str(directory);
    if separator {
        path.push('/');
    }
    path.push_str(label);
    path.push_str(".prototype");
    Ok(path)
}

fn transform(bytes: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .map(|byte| match byte {
            b'/' => b'.',
            b'.' => b'?',
            b'A'..=b'Z' => byte + 32,
            _ => *byte,
        })
        .collect()
}

fn identity(bytes: &[u8]) -> u64 {
    let (mut s1, mut s2, mut crc) = (1u32, 0u32, u32::MAX);
    for byte in bytes {
        s1 = (s1 + u32::from(*byte)) % 65521;
        s2 = (s2 + s1) % 65521;
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ if crc & 1 != 0 { 0xedb88320 } else { 0 };
        }
    }
    ((u64::from(!crc) << 32) | u64::from((s2 << 16) | s1)).wrapping_sub(1)
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn missing(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, message.into())
}
fn unsupported(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message.into())
}
fn allocation(error: impl std::fmt::Display) -> io::Error {
    invalid(format!("localization allocation failed: {error}"))
}
fn decode(error: impl std::fmt::Display) -> io::Error {
    invalid(format!("localization source: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Entirely fabricated format fixtures. The hash vector was independently
    // calculated for this fixture spelling, rather than copied from game data.
    const BUTTON: u64 = 0x5a5569eac4ef0c8c;

    #[derive(Clone, Copy, PartialEq)]
    enum Fault {
        None,
        ZeroUi,
        UiSubtype,
        Parent,
        DuplicateField,
        DuplicatePrototype,
        DuplicateAsset,
        OtherTypeAsset,
        MissingType,
        MissingTarget,
        WrongClass,
    }

    fn string(out: &mut Vec<u8>, text: &str) {
        out.extend((text.len() as u16).to_le_bytes());
        out.extend(text.as_bytes());
    }

    fn blueprint(id: u64, name: &str, fields: &[(u64, &str, u8, Option<u64>)]) -> Vec<u8> {
        let mut out = b"BPT\x0b".to_vec();
        string(&mut out, name);
        out.extend(id.to_le_bytes());
        out.extend(0u16.to_le_bytes()); // No schema parents/contributors.
        out.extend(0u16.to_le_bytes());
        out.extend((fields.len() as u16).to_le_bytes());
        for (id, name, base, subtype) in fields {
            out.extend(id.to_le_bytes());
            string(&mut out, name);
            out.extend([*base, b'S']);
            if let Some(subtype) = subtype {
                out.extend(subtype.to_le_bytes());
            }
        }
        out
    }

    fn prototype(blueprint: u64, parent: u64, fields: &[(u64, u8, u64)]) -> Vec<u8> {
        let mut out = b"PTP\x0b\x03".to_vec();
        out.extend(parent.to_le_bytes());
        out.extend(1u16.to_le_bytes());
        out.extend(blueprint.to_le_bytes());
        out.push(0); // Copy zero.
        out.extend((fields.len() as u16).to_le_bytes());
        for (id, base, value) in fields {
            out.extend(id.to_le_bytes());
            out.push(*base);
            out.extend(value.to_le_bytes());
        }
        out.extend(0u16.to_le_bytes());
        out
    }

    fn asset_type(entries: &[(u64, &str)]) -> Vec<u8> {
        let mut out = b"TYP\x0b".to_vec();
        out.extend((entries.len() as u16).to_le_bytes());
        for (id, name) in entries {
            out.extend(id.to_le_bytes());
            out.extend((id + 10).to_le_bytes());
            out.push(0);
            string(&mut out, name);
        }
        out
    }

    fn fixture(fault: Fault) -> Vec<u8> {
        let mut records = vec![
            (1u64, 1u64, "Globals/Globals.defaults"),
            (2, 2, "Other/ConfiguredUI.defaults"),
            (3, 3, "Other/Text.defaults"),
        ];
        if fault != Fault::MissingTarget {
            records.push((
                BUTTON,
                if fault == Fault::WrongClass { 2 } else { 3 },
                "Example/Labels/Button.prototype",
            ));
        }
        if fault == Fault::DuplicatePrototype {
            records.push((1, 1, "Other/Duplicate.defaults"));
        }
        let mut pdr = b"PDR\x0b".to_vec();
        pdr.extend((records.len() as u32).to_le_bytes());
        for (id, blueprint, name) in records {
            pdr.extend(id.to_le_bytes());
            pdr.extend((id + 100).to_le_bytes());
            pdr.extend(blueprint.to_le_bytes());
            pdr.push(u8::from(id == blueprint));
            string(&mut pdr, name);
        }
        let mut types = vec![(
            if fault == Fault::MissingType { 8u64 } else { 7 },
            "Types/Directory.type",
        )];
        if fault == Fault::OtherTypeAsset {
            types.push((8, "Types/Other.type"));
        }
        let mut tdr = b"TDR\x0b".to_vec();
        tdr.extend((types.len() as u32).to_le_bytes());
        for (id, name) in types {
            tdr.extend(id.to_le_bytes());
            tdr.extend((id + 10).to_le_bytes());
            tdr.push(0);
            string(&mut tdr, name);
        }
        let mut global_fields = vec![(101, b'P', if fault == Fault::ZeroUi { 0 } else { 2 })];
        if fault == Fault::DuplicateField {
            global_fields.push((101, b'P', 2));
        }
        let mut assets = vec![(9, "Example/Labels")];
        if fault == Fault::DuplicateAsset {
            assets.push((9, "Different/Directory"));
        }
        let members = vec![
            ("Calligraphy/Prototype.directory", pdr),
            ("Calligraphy/Type.directory", tdr),
            (
                "Calligraphy/Globals/Globals.blueprint",
                blueprint(
                    1,
                    "GlobalsPrototype",
                    &[(
                        101,
                        "UIGlobals",
                        b'P',
                        Some(if fault == Fault::UiSubtype { 3 } else { 2 }),
                    )],
                ),
            ),
            (
                "Calligraphy/Other/ConfiguredUI.blueprint",
                blueprint(
                    2,
                    "UIGlobalsPrototype",
                    &[(102, "LocalizedInfoDirectory", b'A', Some(7))],
                ),
            ),
            (
                "Calligraphy/Other/Text.blueprint",
                blueprint(
                    3,
                    "UILocalizedInfoPrototype",
                    &[
                        (103, "DisplayText", b'S', None),
                        (104, "TooltipText", b'S', None),
                        (105, "TooltipStyle", b'P', Some(1)),
                        (106, "TooltipFont", b'A', Some(7)),
                    ],
                ),
            ),
            (
                GLOBALS_MEMBER,
                prototype(
                    1,
                    if fault == Fault::Parent { 2 } else { 0 },
                    &global_fields,
                ),
            ),
            (
                "Calligraphy/Other/ConfiguredUI.defaults",
                prototype(2, 0, &[(102, b'A', 9)]),
            ),
            (
                "Calligraphy/Other/Text.defaults",
                prototype(3, 0, &[(103, b'S', 0), (104, b'S', 0)]),
            ),
            (
                "Calligraphy/Example/Labels/Button.prototype",
                prototype(3, 3, &[(103, b'S', 500)]),
            ),
            ("Calligraphy/Types/Directory.type", asset_type(&assets)),
            (
                "Calligraphy/Types/Other.type",
                asset_type(&[(9, "Elsewhere")]),
            ),
        ];
        let mut index = b"KAPG".to_vec();
        index.extend(1u32.to_le_bytes());
        index.extend((members.len() as u32).to_le_bytes());
        let mut payload = Vec::new();
        for (name, bytes) in members {
            let mut compressed = vec![(bytes.len().min(15) as u8) << 4];
            if bytes.len() >= 15 {
                let mut remaining = bytes.len() - 15;
                while remaining >= 255 {
                    compressed.push(255);
                    remaining -= 255;
                }
                compressed.push(remaining as u8);
            }
            compressed.extend(&bytes);
            index.extend(0u64.to_le_bytes());
            index.extend((name.len() as i32).to_le_bytes());
            index.extend(name.as_bytes());
            index.extend(0i32.to_le_bytes());
            index.extend((payload.len() as i32).to_le_bytes());
            index.extend((compressed.len() as i32).to_le_bytes());
            index.extend((bytes.len() as i32).to_le_bytes());
            payload.extend(compressed);
        }
        index.extend(payload);
        index
    }

    #[test]
    fn configured_sources_are_used_and_origins_survive_resolution() {
        let mut labels =
            ConfiguredLabels::from_reader(Cursor::new(fixture(Fault::None)), Limits::default())
                .unwrap();
        let result = labels.resolve("Button").unwrap();
        assert_eq!(result.prototype_id, BUTTON);
        assert_eq!(result.joined_path, "Example/Labels/Button.prototype");
        assert_eq!(result.hash_bytes, b"example.labels.button?prototype");
        assert_eq!(result.prototype().references().display_text_id, 500);
        assert_eq!(result.prototype().references().tooltip_text_id, 0);
        assert_eq!(result.prototype().display_origin().layer, 1);
        assert_eq!(result.prototype().tooltip_origin().layer, 0);
        assert_eq!(result.configuration().directory_asset().id, 9);
        assert_eq!(result.configuration().directory_type().id, 7);
        assert_eq!(result.configuration().ui_globals().target_id(), 2);
        assert_eq!(labels.resolve("bUtToN").unwrap().prototype_id, BUTTON);
        drop(labels);
        assert_eq!(result.configuration().globals().target_id(), 1);
    }

    #[test]
    fn invalid_configuration_cannot_become_an_empty_or_default_success() {
        for fault in [
            Fault::ZeroUi,
            Fault::UiSubtype,
            Fault::Parent,
            Fault::DuplicateField,
            Fault::DuplicatePrototype,
            Fault::DuplicateAsset,
            Fault::OtherTypeAsset,
            Fault::MissingType,
        ] {
            assert!(
                ConfiguredLabels::from_reader(Cursor::new(fixture(fault)), Limits::default())
                    .is_err()
            );
        }
        let mut labels = ConfiguredLabels::from_reader(
            Cursor::new(fixture(Fault::MissingTarget)),
            Limits::default(),
        )
        .unwrap();
        assert_eq!(
            labels.resolve("Button").err().unwrap().kind(),
            io::ErrorKind::NotFound
        );
        let mut labels = ConfiguredLabels::from_reader(
            Cursor::new(fixture(Fault::WrongClass)),
            Limits::default(),
        )
        .unwrap();
        assert_eq!(
            labels.resolve("Button").err().unwrap().kind(),
            io::ErrorKind::Unsupported
        );
    }

    #[test]
    fn identity_vectors_and_literal_join_preserve_native_distinctions() {
        assert_eq!(identity(b"123456789"), 0xcbf43926091e01dd);
        assert_eq!(identity(b""), 0);
        assert_eq!(transform(b"A/B.C\\D"), b"a.b?c\\d");
        assert_eq!(
            join("Example/Labels/", "/Button", 128).unwrap(),
            "Example/Labels//Button.prototype"
        );
        assert_eq!(
            join("Example\\Labels\\", "Button.prototype", 128).unwrap(),
            "Example\\Labels\\Button.prototype.prototype"
        );
    }

    #[test]
    fn unsupported_labels_and_resource_budgets_fail_before_lookup() {
        let mut limits = Limits::default();
        limits.max_label_bytes = 8;
        let mut labels =
            ConfiguredLabels::from_reader(Cursor::new(fixture(Fault::None)), limits).unwrap();
        for label in ["", "\0Button", "Button\0", "Büttön", "😀"] {
            assert_eq!(
                labels.resolve(label).err().unwrap().kind(),
                io::ErrorKind::Unsupported
            );
        }
        assert_eq!(
            labels.resolve("123456789").err().unwrap().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(
            labels.resolve("Absent").err().unwrap().kind(),
            io::ErrorKind::NotFound
        );
        assert!(join("Example/Labels", "Button", 20).is_err());
        for limited in [
            Limits {
                max_metadata_bytes: 1,
                ..Limits::default()
            },
            Limits {
                max_directory_records: 1,
                ..Limits::default()
            },
            Limits {
                max_blueprints: 1,
                ..Limits::default()
            },
            Limits {
                max_asset_entries: 0,
                ..Limits::default()
            },
        ] {
            assert!(
                ConfiguredLabels::from_reader(Cursor::new(fixture(Fault::None)), limited).is_err()
            );
        }
    }
}
