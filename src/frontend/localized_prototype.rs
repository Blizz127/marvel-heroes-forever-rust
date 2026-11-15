//! Effective text references for the evidenced UILocalizedInfoPrototype profile.
//! Native contract and limits: docs/LOCALIZED_PROTOTYPE_FIELDS.md.

use super::localized_data::{self, LocalizedObjectData, References};
use crate::{
    calligraphy::{
        BaseType, Blueprint, StructureType,
        prototype::{FieldData, Value},
    },
    localization::LocaleCatalog,
    prototype_catalog::PrototypeReader,
    prototype_values::{FieldKey, FieldOrigin, PrototypeSource, PrototypeValues, ValueLimits},
};
use std::io;

/// Keeps the complete bounded parent/assignment provenance behind each result.
pub struct PreparedPrototype {
    references: References,
    values: PrototypeValues,
    display_origin: FieldOrigin,
    tooltip_origin: FieldOrigin,
}

impl PreparedPrototype {
    pub fn references(&self) -> References {
        self.references
    }
    pub fn values(&self) -> &PrototypeValues {
        &self.values
    }
    pub fn display_origin(&self) -> FieldOrigin {
        self.display_origin
    }
    pub fn tooltip_origin(&self) -> FieldOrigin {
        self.tooltip_origin
    }
    /// Select stored locale defaults through the separate no-marker profile.
    pub fn localized_data(&self, catalog: &LocaleCatalog) -> io::Result<LocalizedObjectData> {
        localized_data::resolve(catalog, self.references)
    }
}

/// Exact archive-member selection. The caller supplies the resource identity;
/// no native label-directory default or case/path normalization is inferred.
pub fn read_member(
    reader: &mut PrototypeReader<'_>,
    member: &str,
    limits: ValueLimits,
) -> io::Result<Option<PreparedPrototype>> {
    let Some(id) = reader.record_by_member(member).map(|record| record.id) else {
        return Ok(None);
    };
    resolve(reader, id, limits).map(Some)
}

/// Resolve only the recovered simple, same-blueprint native class profile.
/// Missing stored fields are errors, including at a terminal default: allocation
/// clearing is not promoted into a general class-default policy here.
pub fn resolve(
    source: &mut impl PrototypeSource,
    id: u64,
    limits: ValueLimits,
) -> io::Result<PreparedPrototype> {
    let blueprint_id = source
        .record_by_id(id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "localized prototype ID is absent"))?
        .blueprint_id;
    let (display_id, tooltip_id) = validate_schema(
        source
            .blueprint(blueprint_id)
            .ok_or_else(|| invalid("localized prototype schema is absent"))?,
    )?;
    let values = PrototypeValues::resolve_with_limits(source, id, limits)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    // The native copy requires matching runtime class IDs. This bounded profile
    // admits one schema identity throughout, rather than equating arbitrary
    // classes from similar binding names or substituting a blueprint parent.
    for layer in values.layers() {
        if layer.blueprint_id != blueprint_id {
            return Err(unsupported("localized parent uses a different blueprint"));
        }
        let schema = source
            .blueprint(layer.blueprint_id)
            .ok_or_else(|| invalid("localized parent schema is absent"))?;
        if validate_schema(schema)? != (display_id, tooltip_id) {
            return Err(invalid("localized parent field declarations changed"));
        }
        for group in &layer.prototype().groups {
            if group.blueprint_id != blueprint_id || group.copy_number != 0 {
                return Err(unsupported(
                    "localized field group uses an unproved assignment context",
                ));
            }
            if !group.list_fields.is_empty() {
                return Err(unsupported(
                    "localized list assignment is outside the scalar profile",
                ));
            }
            for field in &group.simple_fields {
                if field.schema_blueprint_id != blueprint_id {
                    return Err(unsupported(
                        "localized field has a different declaring schema",
                    ));
                }
            }
        }
    }
    // Require every stored class field to have an unambiguous scalar assignment
    // context, even when the callback does not read that field.
    for field in values.fields() {
        if field.selected().is_none() {
            return Err(unsupported(
                "localized prototype has an unresolved field assignment",
            ));
        }
    }
    let (display_text_id, display_origin) = selected_string(&values, blueprint_id, display_id)?;
    let (tooltip_text_id, tooltip_origin) = selected_string(&values, blueprint_id, tooltip_id)?;
    Ok(PreparedPrototype {
        references: References {
            prototype_id: id,
            display_text_id,
            tooltip_text_id,
        },
        values,
        display_origin,
        tooltip_origin,
    })
}

fn validate_schema(schema: &Blueprint) -> io::Result<(u64, u64)> {
    if schema.runtime_binding != "UILocalizedInfoPrototype"
        || !schema.parents.is_empty()
        || !schema.contributors.is_empty()
        || schema.members.len() != 4
    {
        return Err(unsupported(
            "unproved localized prototype schema/class profile",
        ));
    }
    let mut display = None;
    let mut tooltip = None;
    let mut style = false;
    let mut font = false;
    for (index, member) in schema.members.iter().enumerate() {
        if schema.members[..index]
            .iter()
            .any(|prior| prior.id == member.id)
        {
            return Err(invalid("localized schema has duplicate field identities"));
        }
        if member.structure != StructureType::Simple {
            return Err(unsupported(
                "localized class declaration is not a simple field",
            ));
        }
        let valid = match member.name.as_str() {
            "DisplayText" => {
                member.base_type == BaseType::String
                    && member.subtype.is_none()
                    && display.replace(member.id).is_none()
            }
            "TooltipText" => {
                member.base_type == BaseType::String
                    && member.subtype.is_none()
                    && tooltip.replace(member.id).is_none()
            }
            "TooltipStyle" => {
                let fresh = !style;
                style = true;
                fresh && member.base_type == BaseType::Prototype && member.subtype.is_some()
            }
            "TooltipFont" => {
                let fresh = !font;
                font = true;
                fresh && member.base_type == BaseType::Asset && member.subtype.is_some()
            }
            _ => false,
        };
        if !valid || member.id == 0 {
            return Err(unsupported(
                "localized field declaration differs from the recovered profile",
            ));
        }
    }
    match (display, tooltip, style, font) {
        (Some(display), Some(tooltip), true, true) if display != tooltip => Ok((display, tooltip)),
        _ => Err(invalid("localized schema lacks distinct required fields")),
    }
}

fn selected_string(
    values: &PrototypeValues,
    blueprint_id: u64,
    field_id: u64,
) -> io::Result<(u64, FieldOrigin)> {
    let key = FieldKey {
        declaring_blueprint_id: blueprint_id,
        copy_number: 0,
        field_id,
    };
    let field = values
        .field(key)
        .ok_or_else(|| unsupported("localized text reference has no explicit stored assignment"))?;
    let origin = field
        .selected()
        .ok_or_else(|| unsupported("localized text assignment is unresolved"))?;
    let stored = values
        .stored(origin)
        .ok_or_else(|| invalid("localized field origin is invalid"))?;
    match stored.data {
        FieldData::Simple(Value::String(id)) => Ok((id, origin)),
        _ => Err(invalid(
            "localized text reference is not a scalar String ID",
        )),
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn unsupported(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}
