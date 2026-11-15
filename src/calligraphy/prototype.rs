//! Structural prototype decoding. Runtime class assignment and inheritance are separate.

use std::collections::{HashMap, HashSet};
use std::fmt;

use super::{
    BaseType, Blueprint, BlueprintMember, Directory, DirectoryKind, DirectoryRecord, FileKind,
    PrototypeDataHeader, Reader, StructureType,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    pub offset: usize,
    pub message: &'static str,
    pub blueprint_id: Option<u64>,
    pub field_id: Option<u64>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "prototype at byte {}: {}", self.offset, self.message)?;
        if let Some(id) = self.blueprint_id {
            write!(f, " (blueprint {id:#x})")?;
        }
        if let Some(id) = self.field_id {
            write!(f, " (field {id:#x})")?;
        }
        Ok(())
    }
}
impl std::error::Error for Error {}
impl From<super::Error> for Error {
    fn from(error: super::Error) -> Self {
        Self {
            offset: error.offset,
            message: error.message,
            blueprint_id: None,
            field_id: None,
        }
    }
}
type Result<T> = std::result::Result<T, Error>;

fn error(offset: usize, message: &'static str) -> Error {
    Error {
        offset,
        message,
        blueprint_id: None,
        field_id: None,
    }
}

/// Budgets apply to one complete file, including every nested value.
#[derive(Clone, Copy, Debug)]
pub struct PrototypeLimits {
    pub max_file_bytes: usize,
    pub max_depth: usize,
    pub max_prototypes: usize,
    pub max_groups: usize,
    pub max_fields: usize,
    pub max_values: usize,
    pub max_schema_steps: usize,
}
impl Default for PrototypeLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: 64 * 1024 * 1024,
            max_depth: 64,
            max_prototypes: 100_000,
            max_groups: 200_000,
            max_fields: 1_000_000,
            max_values: 1_000_000,
            max_schema_steps: 1_000_000,
        }
    }
}

struct Schema<'a> {
    blueprint: &'a Blueprint,
    members: HashMap<u64, &'a BlueprintMember>,
}

/// A borrowed blueprint index and owned prototype-to-blueprint ID index.
/// This does not load prototype parent values or resolve inheritance.
pub struct SchemaSet<'a> {
    schemas: HashMap<u64, Schema<'a>>,
    prototype_blueprints: HashMap<u64, u64>,
}

impl<'a> SchemaSet<'a> {
    pub fn new(blueprints: &'a [Blueprint], prototype_directory: &Directory) -> Result<Self> {
        if prototype_directory.kind != DirectoryKind::Prototype {
            return Err(error(0, "expected Prototype.directory"));
        }
        let mut schemas = HashMap::new();
        schemas
            .try_reserve(blueprints.len())
            .map_err(|_| error(0, "schema allocation failed"))?;
        for blueprint in blueprints {
            let id = blueprint.default_prototype_id;
            let mut members = HashMap::new();
            members
                .try_reserve(blueprint.members.len())
                .map_err(|_| error(0, "member allocation failed"))?;
            for member in &blueprint.members {
                if member.id == 0 || members.insert(member.id, member).is_some() {
                    return Err(Error {
                        offset: 0,
                        message: "zero or duplicate schema field ID",
                        blueprint_id: Some(id),
                        field_id: Some(member.id),
                    });
                }
            }
            if id == 0 || schemas.insert(id, Schema { blueprint, members }).is_some() {
                return Err(Error {
                    offset: 0,
                    message: "zero or duplicate blueprint ID",
                    blueprint_id: Some(id),
                    field_id: None,
                });
            }
        }
        let mut prototype_blueprints = HashMap::new();
        prototype_blueprints
            .try_reserve(prototype_directory.records.len())
            .map_err(|_| error(0, "prototype index allocation failed"))?;
        for record in &prototype_directory.records {
            let DirectoryRecord::Prototype(record) = record else {
                return Err(error(0, "non-prototype record in Prototype.directory"));
            };
            if record.id == 0
                || prototype_blueprints
                    .insert(record.id, record.blueprint_id)
                    .is_some()
            {
                return Err(error(0, "zero or duplicate prototype ID"));
            }
            if !schemas.contains_key(&record.blueprint_id) {
                return Err(Error {
                    offset: 0,
                    message: "prototype directory names an unknown blueprint",
                    blueprint_id: Some(record.blueprint_id),
                    field_id: None,
                });
            }
        }
        Ok(Self {
            schemas,
            prototype_blueprints,
        })
    }

    pub fn blueprint(&self, id: u64) -> Option<&'a Blueprint> {
        self.schemas.get(&id).map(|schema| schema.blueprint)
    }

    pub fn prototype_blueprint(&self, id: u64) -> Option<u64> {
        self.prototype_blueprints
            .get(&id)
            .copied()
            .or_else(|| self.schemas.contains_key(&id).then_some(id))
    }

    /// Lookup a field declared directly by this blueprint, without parent traversal.
    pub fn declared_member(&self, blueprint_id: u64, field_id: u64) -> Option<&'a BlueprintMember> {
        self.schemas
            .get(&blueprint_id)?
            .members
            .get(&field_id)
            .copied()
    }

    fn member(
        &self,
        blueprint_id: u64,
        field_id: u64,
        budget: &mut Budget,
        offset: usize,
    ) -> Result<Option<(u64, &'a BlueprintMember)>> {
        // Explicit DFS preserves declared parent order without recursive call stacks.
        let mut stack = vec![(blueprint_id, false)];
        let mut active = HashSet::new();
        let mut finished = HashSet::new();
        while let Some((id, leaving)) = stack.pop() {
            if leaving {
                active.remove(&id);
                finished.insert(id);
                continue;
            }
            budget.schema_step(offset)?;
            if finished.contains(&id) {
                continue;
            }
            if !active.insert(id) {
                return Err(Error {
                    offset,
                    message: "cycle in blueprint parents",
                    blueprint_id: Some(id),
                    field_id: Some(field_id),
                });
            }
            let schema = self.schemas.get(&id).ok_or(Error {
                offset,
                message: "unknown blueprint in parent hierarchy",
                blueprint_id: Some(id),
                field_id: Some(field_id),
            })?;
            if let Some(member) = schema.members.get(&field_id) {
                return Ok(Some((id, member)));
            }
            stack.push((id, true));
            if schema.blueprint.parents.len() > budget.schema_steps
                || stack.len() > budget.schema_steps.saturating_mul(2).saturating_add(1)
            {
                return Err(error(offset, "schema traversal frontier exceeds budget"));
            }
            stack.extend(
                schema
                    .blueprint
                    .parents
                    .iter()
                    .rev()
                    .map(|parent| (parent.id, false)),
            );
        }
        Ok(None)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prototype {
    pub header: PrototypeDataHeader,
    pub groups: Vec<FieldGroup>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldGroup {
    pub blueprint_id: u64,
    pub copy_number: u8,
    pub simple_fields: Vec<Field>,
    pub list_fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub id: u64,
    /// The blueprint that declares the matched field schema.
    pub schema_blueprint_id: u64,
    pub data: FieldData,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FieldData {
    Simple(Value),
    List(Vec<Value>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Asset(u64),
    /// Preserve the serialized bits; reference source interprets any nonzero value as true.
    Boolean(u64),
    Curve(u64),
    Double(f64),
    Long(i64),
    Prototype(u64),
    EmbeddedPrototype(Box<Prototype>),
    String(u64),
    Type(u64),
}

impl Prototype {
    /// Decode a complete PTP file with its containing blueprint ID from Prototype.directory.
    pub fn parse(bytes: &[u8], blueprint_id: u64, schemas: &SchemaSet<'_>) -> Result<Self> {
        Self::parse_with_limits(bytes, blueprint_id, schemas, PrototypeLimits::default())
    }

    pub fn parse_with_limits(
        bytes: &[u8],
        blueprint_id: u64,
        schemas: &SchemaSet<'_>,
        limits: PrototypeLimits,
    ) -> Result<Self> {
        if limits.max_depth > 256 {
            return Err(error(0, "maximum supported nesting budget is 256"));
        }
        if bytes.len() > limits.max_file_bytes {
            return Err(error(0, "file exceeds byte budget"));
        }
        if !schemas.schemas.contains_key(&blueprint_id) {
            return Err(Error {
                offset: 0,
                message: "unknown containing blueprint",
                blueprint_id: Some(blueprint_id),
                field_id: None,
            });
        }
        let mut reader = Reader::new(bytes);
        reader.expect(FileKind::Prototype)?;
        let mut decoder = Decoder {
            reader,
            schemas,
            limits,
            budget: Budget::new(limits),
        };
        let prototype = decoder.prototype(Some(blueprint_id), 0)?;
        if prototype.header.reference.is_none() || prototype.header.polymorphic {
            return Err(error(
                4,
                "top-level prototype must carry a reference and cannot be polymorphic",
            ));
        }
        decoder.reader.finish()?;
        Ok(prototype)
    }
}

struct Budget {
    prototypes: usize,
    groups: usize,
    fields: usize,
    values: usize,
    schema_steps: usize,
}
impl Budget {
    fn new(limits: PrototypeLimits) -> Self {
        Self {
            prototypes: limits.max_prototypes,
            groups: limits.max_groups,
            fields: limits.max_fields,
            values: limits.max_values,
            schema_steps: limits.max_schema_steps,
        }
    }
    fn take(
        remaining: &mut usize,
        count: usize,
        offset: usize,
        message: &'static str,
    ) -> Result<()> {
        *remaining = remaining
            .checked_sub(count)
            .ok_or_else(|| error(offset, message))?;
        Ok(())
    }
    fn schema_step(&mut self, offset: usize) -> Result<()> {
        Self::take(
            &mut self.schema_steps,
            1,
            offset,
            "schema lookup budget exceeded",
        )
    }
}

struct Decoder<'bytes, 'schemas, 'metadata> {
    reader: Reader<'bytes>,
    schemas: &'schemas SchemaSet<'metadata>,
    limits: PrototypeLimits,
    budget: Budget,
}

impl Decoder<'_, '_, '_> {
    fn prototype(&mut self, containing_blueprint: Option<u64>, depth: usize) -> Result<Prototype> {
        let offset = self.reader.position;
        if depth > self.limits.max_depth {
            return Err(error(offset, "prototype nesting budget exceeded"));
        }
        Budget::take(
            &mut self.budget.prototypes,
            1,
            offset,
            "prototype count budget exceeded",
        )?;
        let flags = self.reader.flags(7)?;
        let reference = if flags & 1 != 0 {
            Some(self.reader.u64()?)
        } else {
            None
        };
        let header = PrototypeDataHeader {
            reference,
            data_exists: flags & 2 != 0,
            polymorphic: flags & 4 != 0,
        };
        if !header.data_exists {
            return Ok(Prototype {
                header,
                groups: Vec::new(),
            });
        }
        let blueprint_id = containing_blueprint
            .or_else(|| reference.and_then(|id| self.schemas.prototype_blueprint(id)))
            .ok_or_else(|| {
                error(
                    offset,
                    "cannot resolve containing blueprint for embedded data",
                )
            })?;
        let count = usize::from(self.reader.u16()?);
        Budget::take(
            &mut self.budget.groups,
            count,
            self.reader.position,
            "field group budget exceeded",
        )?;
        let mut groups = self.reader.records(count, 13)?;
        for _ in 0..count {
            let group_blueprint = self.reader.u64()?;
            if !self.schemas.schemas.contains_key(&group_blueprint) {
                return Err(Error {
                    offset: self.reader.position - 8,
                    message: "unknown declaring blueprint",
                    blueprint_id: Some(group_blueprint),
                    field_id: None,
                });
            }
            let copy_number = self.reader.u8()?;
            let simple_fields =
                self.fields(blueprint_id, group_blueprint, StructureType::Simple, depth)?;
            let list_fields =
                self.fields(blueprint_id, group_blueprint, StructureType::List, depth)?;
            groups.push(FieldGroup {
                blueprint_id: group_blueprint,
                copy_number,
                simple_fields,
                list_fields,
            });
        }
        Ok(Prototype { header, groups })
    }

    fn fields(
        &mut self,
        containing_blueprint: u64,
        group_blueprint: u64,
        structure: StructureType,
        depth: usize,
    ) -> Result<Vec<Field>> {
        let count = usize::from(self.reader.u16()?);
        Budget::take(
            &mut self.budget.fields,
            count,
            self.reader.position,
            "field count budget exceeded",
        )?;
        let minimum = if structure == StructureType::Simple {
            10
        } else {
            11
        };
        let mut fields = self.reader.records(count, minimum)?;
        for _ in 0..count {
            let offset = self.reader.position;
            let id = self.reader.u64()?;
            let kind = self.reader.base_type()?;
            // Both contexts are retained in the file. Matching the group handles property
            // records; matching the container handles ordinary inherited/mixin fields.
            let group_match = self
                .schemas
                .member(group_blueprint, id, &mut self.budget, offset)?;
            let container_match = if group_blueprint == containing_blueprint {
                None
            } else {
                self.schemas
                    .member(containing_blueprint, id, &mut self.budget, offset)?
            };
            if let (Some((_, group)), Some((_, container))) = (group_match, container_match) {
                if group.base_type != container.base_type
                    || group.structure != container.structure
                    || group.subtype != container.subtype
                {
                    return Err(Error {
                        offset,
                        message: "ambiguous group/container field schemas",
                        blueprint_id: Some(group_blueprint),
                        field_id: Some(id),
                    });
                }
            }
            let (schema_blueprint_id, member) = group_match.or(container_match).ok_or(Error {
                offset,
                message: "unknown field in declaring and containing blueprint hierarchies",
                blueprint_id: Some(group_blueprint),
                field_id: Some(id),
            })?;
            if member.base_type != kind || member.structure != structure {
                return Err(Error {
                    offset,
                    message: "field type or structure disagrees with blueprint",
                    blueprint_id: Some(schema_blueprint_id),
                    field_id: Some(id),
                });
            }
            if structure == StructureType::List
                && !matches!(
                    kind,
                    BaseType::Asset
                        | BaseType::Prototype
                        | BaseType::EmbeddedPrototype
                        | BaseType::Type
                )
            {
                return Err(Error {
                    offset,
                    message: "unsupported list element type",
                    blueprint_id: Some(schema_blueprint_id),
                    field_id: Some(id),
                });
            }
            let data = match structure {
                StructureType::Simple => FieldData::Simple(self.value(kind, depth)?),
                StructureType::List => {
                    let length = usize::from(self.reader.u16()?);
                    if length > self.budget.values {
                        return Err(error(self.reader.position, "value count budget exceeded"));
                    }
                    let minimum = if kind == BaseType::EmbeddedPrototype {
                        1
                    } else {
                        8
                    };
                    let mut values = self.reader.records(length, minimum)?;
                    for _ in 0..length {
                        values.push(self.value(kind, depth)?);
                    }
                    FieldData::List(values)
                }
            };
            fields.push(Field {
                id,
                schema_blueprint_id,
                data,
            });
        }
        Ok(fields)
    }

    fn value(&mut self, kind: BaseType, depth: usize) -> Result<Value> {
        Budget::take(
            &mut self.budget.values,
            1,
            self.reader.position,
            "value count budget exceeded",
        )?;
        Ok(match kind {
            BaseType::Asset => Value::Asset(self.reader.u64()?),
            BaseType::Boolean => Value::Boolean(self.reader.u64()?),
            BaseType::Curve => Value::Curve(self.reader.u64()?),
            BaseType::Double => Value::Double(f64::from_bits(self.reader.u64()?)),
            BaseType::Long => Value::Long(self.reader.u64()? as i64),
            BaseType::Prototype => Value::Prototype(self.reader.u64()?),
            BaseType::EmbeddedPrototype => {
                Value::EmbeddedPrototype(Box::new(self.prototype(None, depth + 1)?))
            }
            BaseType::String => Value::String(self.reader.u64()?),
            BaseType::Type => Value::Type(self.reader.u64()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calligraphy::{BlueprintReference, PrototypeRecord};

    fn blueprints() -> Vec<Blueprint> {
        vec![Blueprint {
            runtime_binding: "Synthetic".into(),
            default_prototype_id: 1,
            parents: vec![],
            contributors: vec![],
            members: vec![
                member(101, BaseType::Long, StructureType::Simple),
                member(102, BaseType::EmbeddedPrototype, StructureType::List),
                member(103, BaseType::Boolean, StructureType::Simple),
                member(104, BaseType::Long, StructureType::List),
                member(105, BaseType::EmbeddedPrototype, StructureType::Simple),
                member(106, BaseType::Type, StructureType::List),
            ],
        }]
    }
    fn member(id: u64, base_type: BaseType, structure: StructureType) -> BlueprintMember {
        BlueprintMember {
            id,
            name: format!("Field{id}"),
            base_type,
            structure,
            subtype: None,
        }
    }
    fn directory() -> Directory {
        Directory {
            kind: DirectoryKind::Prototype,
            records: vec![DirectoryRecord::Prototype(PrototypeRecord {
                id: 10,
                guid: 20,
                blueprint_id: 1,
                flags: 0,
                path: "Synthetic.prototype".into(),
            })],
        }
    }
    fn push64(bytes: &mut Vec<u8>, value: u64) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fn body(simple: &[u8], simple_count: u16, list: &[u8], list_count: u16) -> Vec<u8> {
        let mut bytes = vec![3];
        push64(&mut bytes, 10);
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        push64(&mut bytes, 1);
        bytes.push(0);
        bytes.extend_from_slice(&simple_count.to_le_bytes());
        bytes.extend_from_slice(simple);
        bytes.extend_from_slice(&list_count.to_le_bytes());
        bytes.extend_from_slice(list);
        bytes
    }
    fn file(body: &[u8]) -> Vec<u8> {
        let mut bytes = b"PTP\x0b".to_vec();
        bytes.extend_from_slice(body);
        bytes
    }
    fn scalar(id: u64, kind: u8, value: u64) -> Vec<u8> {
        let mut bytes = id.to_le_bytes().to_vec();
        bytes.push(kind);
        push64(&mut bytes, value);
        bytes
    }

    #[test]
    fn preserves_signed_and_boolean_values() {
        let b = blueprints();
        let schemas = SchemaSet::new(&b, &directory()).unwrap();
        let mut fields = scalar(101, b'L', (-7_i64) as u64);
        fields.extend_from_slice(&scalar(103, b'B', u64::MAX));
        let parsed = Prototype::parse(&file(&body(&fields, 2, &[], 0)), 1, &schemas).unwrap();
        assert_eq!(
            parsed.groups[0].simple_fields[0].data,
            FieldData::Simple(Value::Long(-7))
        );
        assert_eq!(
            parsed.groups[0].simple_fields[1].data,
            FieldData::Simple(Value::Boolean(u64::MAX))
        );
    }

    #[test]
    fn decode_empty_reference_and_nested_list_values() {
        let b = blueprints();
        let schemas = SchemaSet::new(&b, &directory()).unwrap();
        let mut list = 102_u64.to_le_bytes().to_vec();
        list.push(b'R');
        list.extend_from_slice(&3_u16.to_le_bytes());
        list.push(0); // Empty embedded value.
        list.push(1);
        push64(&mut list, 10); // Reference-only embedded value.
        list.extend_from_slice(&body(&scalar(101, b'L', 42), 1, &[], 0));
        let parsed = Prototype::parse(&file(&body(&[], 0, &list, 1)), 1, &schemas).unwrap();
        let FieldData::List(values) = &parsed.groups[0].list_fields[0].data else {
            panic!()
        };
        assert_eq!(values.len(), 3);
        let Value::EmbeddedPrototype(last) = &values[2] else {
            panic!()
        };
        assert_eq!(
            last.groups[0].simple_fields[0].data,
            FieldData::Simple(Value::Long(42))
        );
    }

    #[test]
    fn every_truncation_and_trailing_bytes_rejected() {
        let b = blueprints();
        let schemas = SchemaSet::new(&b, &directory()).unwrap();
        let mut bytes = file(&body(&scalar(101, b'L', 42), 1, &[], 0));
        for end in 0..bytes.len() {
            assert!(
                Prototype::parse(&bytes[..end], 1, &schemas).is_err(),
                "{end}"
            );
        }
        bytes.push(0);
        assert!(Prototype::parse(&bytes, 1, &schemas).is_err());
    }

    #[test]
    fn unknown_fields_type_mismatch_and_invalid_lists_fail() {
        let b = blueprints();
        let schemas = SchemaSet::new(&b, &directory()).unwrap();
        assert!(
            Prototype::parse(&file(&body(&scalar(999, b'L', 0), 1, &[], 0)), 1, &schemas).is_err()
        );
        assert!(
            Prototype::parse(&file(&body(&scalar(101, b'D', 0), 1, &[], 0)), 1, &schemas).is_err()
        );
        let mut list = 104_u64.to_le_bytes().to_vec();
        list.extend_from_slice(&[b'L', 0, 0]);
        assert!(Prototype::parse(&file(&body(&[], 0, &list, 1)), 1, &schemas).is_err());
        let unknown = scalar(101, b'?', 0);
        assert!(Prototype::parse(&file(&body(&unknown, 1, &[], 0)), 1, &schemas).is_err());
    }

    #[test]
    fn limits_bound_each_kind_of_work() {
        let b = blueprints();
        let schemas = SchemaSet::new(&b, &directory()).unwrap();
        let bytes = file(&body(&scalar(101, b'L', 42), 1, &[], 0));
        let defaults = PrototypeLimits::default();
        for limits in [
            PrototypeLimits {
                max_file_bytes: bytes.len() - 1,
                ..defaults
            },
            PrototypeLimits {
                max_prototypes: 0,
                ..defaults
            },
            PrototypeLimits {
                max_groups: 0,
                ..defaults
            },
            PrototypeLimits {
                max_fields: 0,
                ..defaults
            },
            PrototypeLimits {
                max_values: 0,
                ..defaults
            },
            PrototypeLimits {
                max_schema_steps: 0,
                ..defaults
            },
            PrototypeLimits {
                max_depth: 257,
                ..defaults
            },
        ] {
            assert!(Prototype::parse_with_limits(&bytes, 1, &schemas, limits).is_err());
        }
        let mut nested = vec![0];
        for _ in 0..3 {
            let mut field = 105_u64.to_le_bytes().to_vec();
            field.push(b'R');
            field.extend_from_slice(&nested);
            nested = body(&field, 1, &[], 0);
        }
        let error = Prototype::parse_with_limits(
            &file(&nested),
            1,
            &schemas,
            PrototypeLimits {
                max_depth: 1,
                ..defaults
            },
        )
        .unwrap_err();
        assert_eq!(error.message, "prototype nesting budget exceeded");
    }

    #[test]
    fn schema_parent_cycles_and_missing_parents_fail() {
        let mut b = blueprints();
        b[0].parents.push(BlueprintReference { id: 1, copies: 1 });
        let schemas = SchemaSet::new(&b, &directory()).unwrap();
        let bytes = file(&body(&scalar(999, b'L', 0), 1, &[], 0));
        assert_eq!(
            Prototype::parse(&bytes, 1, &schemas).unwrap_err().message,
            "cycle in blueprint parents"
        );
        b[0].parents[0].id = 99;
        let schemas = SchemaSet::new(&b, &directory()).unwrap();
        assert_eq!(
            Prototype::parse(&bytes, 1, &schemas).unwrap_err().message,
            "unknown blueprint in parent hierarchy"
        );
    }

    #[test]
    fn containing_blueprint_and_group_schema_conflicts_fail() {
        let mut b = blueprints();
        let mut other = b[0].clone();
        other.default_prototype_id = 2;
        other.members[0].base_type = BaseType::Double;
        b.push(other);
        let schemas = SchemaSet::new(&b, &directory()).unwrap();
        let mut bytes = body(&scalar(101, b'L', 0), 1, &[], 0);
        bytes[11..19].copy_from_slice(&2_u64.to_le_bytes());
        assert_eq!(
            Prototype::parse(&file(&bytes), 1, &schemas)
                .unwrap_err()
                .message,
            "ambiguous group/container field schemas"
        );
    }

    #[test]
    fn schema_lookup_preserves_parent_order() {
        let mut b = blueprints();
        let mut other = b[0].clone();
        other.default_prototype_id = 2;
        b[0].members.clear();
        b[0].parents.push(BlueprintReference { id: 2, copies: 1 });
        b.push(other);
        let schemas = SchemaSet::new(&b, &directory()).unwrap();
        let parsed =
            Prototype::parse(&file(&body(&scalar(101, b'L', 42), 1, &[], 0)), 1, &schemas).unwrap();
        assert_eq!(parsed.groups[0].simple_fields[0].schema_blueprint_id, 2);
    }

    #[test]
    fn invalid_counts_and_top_level_headers_fail() {
        let b = blueprints();
        let schemas = SchemaSet::new(&b, &directory()).unwrap();
        let mut bytes = file(&body(&[], 0, &[], 0));
        bytes[13..15].copy_from_slice(&u16::MAX.to_le_bytes());
        assert!(Prototype::parse(&bytes, 1, &schemas).is_err());
        assert!(Prototype::parse(b"PTP\x0b\x00", 1, &schemas).is_err());
        let mut reference = vec![5];
        push64(&mut reference, 10);
        assert!(Prototype::parse(&file(&reference), 1, &schemas).is_err());
        assert!(Prototype::parse(b"PTP\x0b\x80", 1, &schemas).is_err());
    }
}
