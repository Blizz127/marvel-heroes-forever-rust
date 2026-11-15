//! Checked UE3 property tags for the observed 868/3 asset profile.
//!
//! Unknown property values retain their type and byte span. Arrays and native
//! class payloads require a class/schema-specific reader and are not guessed.

use super::{invalid, unsupported, Cursor, NameRef, Package, MAX_EXPORT_BYTES};
use std::io;

pub const MAX_PROPERTIES: usize = 16_384;
// Array elements remain opaque and are not individually allocated. Byte-array
// assets can legitimately contain more than one million elements.
pub const MAX_ARRAY_ITEMS: usize = MAX_EXPORT_BYTES;

/// A byte range relative to the supplied serialized export, never a file path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub offset: usize,
    pub length: usize,
}

impl Span {
    pub fn bytes<'a>(&self, export: &'a [u8]) -> io::Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(self.length)
            .ok_or_else(|| invalid("property span overflow"))?;
        export
            .get(self.offset..end)
            .ok_or_else(|| invalid("property span is outside export"))
    }

    pub fn logical_offset(&self, export_offset: usize) -> io::Result<usize> {
        export_offset
            .checked_add(self.offset)
            .ok_or_else(|| invalid("logical property offset overflow"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagDetail {
    None,
    Structure(NameRef),
    Enumeration(NameRef),
    Boolean(bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Integer(i32),
    /// Exact IEEE-754 bits, preserving NaNs and signed zero.
    FloatBits(u32),
    Boolean(bool),
    Byte(u8),
    Name(NameRef),
    Object(i32),
    String(String),
    /// UE3 array tags do not carry their element type. The remaining bytes are
    /// opaque until a class/schema supplies that type.
    Array {
        count: usize,
        elements: Span,
    },
    Structure {
        name: NameRef,
    },
    /// The property type and exact payload span remain available on Property.
    Opaque,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property {
    pub name: NameRef,
    pub property_type: NameRef,
    pub array_index: usize,
    /// Includes the property name, type, size/index and type-specific tag fields.
    pub tag: Span,
    pub payload: Span,
    pub detail: TagDetail,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyList {
    pub entries: Vec<Property>,
    pub start_offset: usize,
    /// First byte after the terminating None FName.
    pub end_offset: usize,
    pub terminator: Span,
}

impl PropertyList {
    pub fn find<'a>(&'a self, package: &Package, name: &str) -> Option<&'a Property> {
        self.entries.iter().find(|property| {
            package
                .resolve_name(property.name)
                .is_some_and(|value| value.eq_ignore_ascii_case(name))
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectProperties {
    pub export_index: usize,
    pub class_name: String,
    pub net_index: i32,
    pub serial_offset: usize,
    pub properties: PropertyList,
    /// Class-specific trailing data. This module does not reinterpret it.
    pub native_payload: Span,
}

impl ObjectProperties {
    /// Decode the standard UObject prefix and tags for the asset classes
    /// established by reference layout and private sample observation.
    /// UClass, actors and custom gameplay classes are intentionally unsupported.
    pub fn parse_export(package: &Package, export_index: usize, bytes: &[u8]) -> io::Result<Self> {
        let export = package
            .exports
            .get(export_index)
            .ok_or_else(|| invalid("export index is out of bounds"))?;
        if bytes.len() != export.serial_size {
            return Err(invalid("serialized export length disagrees with metadata"));
        }
        let class_name = package
            .export_class_name(export_index)
            .ok_or_else(|| invalid("export class cannot be resolved"))?;
        if !supports_asset_class(&class_name) {
            return Err(unsupported(format!(
                "UObject preamble is not established for class {class_name}"
            )));
        }
        let mut input = Cursor::new(bytes);
        let net_index = input.i32()?;
        let properties = parse_tagged_properties(package, bytes, input.position)?;
        let native_payload = Span {
            offset: properties.end_offset,
            length: bytes.len() - properties.end_offset,
        };
        Ok(Self {
            export_index,
            class_name,
            net_index,
            serial_offset: export.serial_offset,
            properties,
            native_payload,
        })
    }
}

pub fn supports_asset_class(name: &str) -> bool {
    [
        "texture2d",
        "swfmovie",
        "material",
        "materialinstanceconstant",
        "materialinstancetimevarying",
        "package",
        "objectreferencer",
        "gazcookedobjectreferencer",
    ]
    .iter()
    .any(|known| name.eq_ignore_ascii_case(known))
}

/// Parse a property list at a caller-established byte offset. This makes no
/// claim about the bytes preceding the list or its class-specific trailing data.
pub fn parse_tagged_properties(
    package: &Package,
    bytes: &[u8],
    start_offset: usize,
) -> io::Result<PropertyList> {
    if bytes.len() > MAX_EXPORT_BYTES {
        return Err(invalid(
            "serialized property input exceeds the 64 MiB limit",
        ));
    }
    if start_offset > bytes.len() {
        return Err(invalid("property start is outside export"));
    }
    let mut input = Cursor::new(bytes);
    input.position = start_offset;
    let mut entries = Vec::new();
    loop {
        let tag_start = input.position;
        let name = input.name_ref(package.names.len())?;
        let name_text = package
            .resolve_name(name)
            .ok_or_else(|| invalid("property name is unresolved"))?;
        if name_text.eq_ignore_ascii_case("none") {
            return Ok(PropertyList {
                entries,
                start_offset,
                end_offset: input.position,
                terminator: Span {
                    offset: tag_start,
                    length: input.position - tag_start,
                },
            });
        }
        if entries.len() == MAX_PROPERTIES {
            return Err(invalid("property count exceeds inspection limit"));
        }
        let property_type = input.name_ref(package.names.len())?;
        if property_type.number != 0 {
            return Err(unsupported("numbered property type names are unsupported"));
        }
        let type_text = package
            .resolve_name(property_type)
            .ok_or_else(|| invalid("property type is unresolved"))?;
        let size = input.nonnegative("property payload size")?;
        let array_index = input.nonnegative("property array index")?;
        if array_index > MAX_ARRAY_ITEMS {
            return Err(invalid("property array index exceeds inspection limit"));
        }
        let detail = match type_text.to_ascii_lowercase().as_str() {
            "structproperty" => TagDetail::Structure(input.name_ref(package.names.len())?),
            "byteproperty" => TagDetail::Enumeration(input.name_ref(package.names.len())?),
            "boolproperty" => {
                if size != 0 {
                    return Err(unsupported(
                        "BoolProperty payload size must be zero for this profile",
                    ));
                }
                let value = input.take(1)?[0];
                if value > 1 {
                    return Err(invalid("noncanonical boolean property value"));
                }
                TagDetail::Boolean(value == 1)
            }
            _ => TagDetail::None,
        };
        let payload = Span {
            offset: input.position,
            length: size,
        };
        let tag = Span {
            offset: tag_start,
            length: input.position - tag_start,
        };
        let raw = input.take(size)?;
        let value = parse_value(package, &type_text, &detail, payload, raw)?;
        entries.push(Property {
            name,
            property_type,
            array_index,
            tag,
            payload,
            detail,
            value,
        });
    }
}

fn parse_value(
    package: &Package,
    property_type: &str,
    detail: &TagDetail,
    span: Span,
    bytes: &[u8],
) -> io::Result<Value> {
    let mut input = Cursor::new(bytes);
    let value = match property_type.to_ascii_lowercase().as_str() {
        "intproperty" => Value::Integer(input.i32()?),
        "floatproperty" => Value::FloatBits(input.u32()?),
        "nameproperty" => Value::Name(input.name_ref(package.names.len())?),
        "objectproperty" | "componentproperty" | "classproperty" => {
            Value::Object(input.object_index(&package.summary)?)
        }
        "strproperty" => Value::String(input.string()?),
        "boolproperty" => match detail {
            TagDetail::Boolean(value) => Value::Boolean(*value),
            _ => return Err(invalid("boolean property lacks tag value")),
        },
        "byteproperty" => match bytes.len() {
            1 => Value::Byte(input.take(1)?[0]),
            8 => Value::Name(input.name_ref(package.names.len())?),
            _ => {
                return Err(unsupported(
                    "ByteProperty must contain one byte or an enum FName",
                ))
            }
        },
        "arrayproperty" => {
            let count = input.count("array element count", MAX_ARRAY_ITEMS)?;
            let elements = Span {
                offset: span.offset + input.position,
                length: bytes.len() - input.position,
            };
            return Ok(Value::Array { count, elements });
        }
        "structproperty" => match detail {
            TagDetail::Structure(name) => return Ok(Value::Structure { name: *name }),
            _ => return Err(invalid("struct property lacks struct name")),
        },
        _ => return Ok(Value::Opaque),
    };
    if input.position != bytes.len() {
        return Err(invalid(
            "property payload size disagrees with its scalar type",
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upk::{NameEntry, NameRef};

    const NAMES: &[&str] = &[
        "None",
        "SizeX",
        "IntProperty",
        "NeverStream",
        "BoolProperty",
        "Format",
        "ByteProperty",
        "EPixelFormat",
        "PF_DXT1",
        "TextureFileCacheName",
        "NameProperty",
        "Textures",
        "RawData",
        "ArrayProperty",
        "SourceFile",
        "StrProperty",
        "Unknown",
        "MysteryProperty",
        "Tint",
        "StructProperty",
        "Vector",
        "Texture2D",
    ];

    fn word(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fn name(bytes: &mut Vec<u8>, value: u32) {
        word(bytes, value);
        word(bytes, 0);
    }
    fn tag(bytes: &mut Vec<u8>, property: u32, kind: u32, size: u32) {
        name(bytes, property);
        name(bytes, kind);
        word(bytes, size);
        word(bytes, 0);
    }
    fn end(bytes: &mut Vec<u8>) {
        name(bytes, 0);
    }

    fn package(serial_size: usize) -> Package {
        let mut package = Package::parse(&super::super::tests::fixture()).unwrap();
        package.names = NAMES
            .iter()
            .map(|name| NameEntry {
                value: (*name).into(),
                flags: 0,
            })
            .collect();
        package.summary.names.count = package.names.len();
        package.imports[0].object_name = NameRef {
            index: 21,
            number: 0,
        };
        package.exports[0].serial_size = serial_size;
        package
    }

    fn fixture() -> Vec<u8> {
        let mut bytes = (-1i32).to_le_bytes().to_vec();
        tag(&mut bytes, 1, 2, 4);
        word(&mut bytes, 128);
        tag(&mut bytes, 3, 4, 0);
        bytes.push(1);
        tag(&mut bytes, 5, 6, 8);
        name(&mut bytes, 7);
        name(&mut bytes, 8);
        tag(&mut bytes, 9, 10, 8);
        name(&mut bytes, 11);
        tag(&mut bytes, 12, 13, 7);
        word(&mut bytes, 3);
        bytes.extend_from_slice(&[1, 2, 3]);
        tag(&mut bytes, 16, 17, 3);
        bytes.extend_from_slice(&[9, 8, 7]);
        end(&mut bytes);
        bytes.extend_from_slice(&[4, 5, 6]);
        bytes
    }

    #[test]
    fn reads_asset_scalars_and_preserves_opaque_spans() {
        let bytes = fixture();
        let package = package(bytes.len());
        let object = ObjectProperties::parse_export(&package, 0, &bytes).unwrap();
        assert_eq!(object.net_index, -1);
        assert_eq!(object.properties.entries.len(), 6);
        assert_eq!(
            object.properties.find(&package, "sizex").unwrap().value,
            Value::Integer(128)
        );
        assert_eq!(object.properties.entries[1].value, Value::Boolean(true));
        assert_eq!(
            object.properties.entries[2].value,
            Value::Name(NameRef {
                index: 8,
                number: 0
            })
        );
        let Value::Array { count, elements } = object.properties.entries[4].value else {
            panic!("array not parsed")
        };
        assert_eq!(count, 3);
        assert_eq!(elements.bytes(&bytes).unwrap(), &[1, 2, 3]);
        assert_eq!(object.properties.entries[5].value, Value::Opaque);
        assert_eq!(
            object.properties.entries[5].payload.bytes(&bytes).unwrap(),
            &[9, 8, 7]
        );
        assert_eq!(object.native_payload.bytes(&bytes).unwrap(), &[4, 5, 6]);
    }

    #[test]
    fn rejects_truncated_lists_and_malformed_property_fields() {
        let bytes = fixture();
        let package = package(bytes.len());
        let list = parse_tagged_properties(&package, &bytes, 4).unwrap();
        for length in 4..list.end_offset {
            assert!(
                parse_tagged_properties(&package, &bytes[..length], 4).is_err(),
                "accepted prefix {length}"
            );
        }
        for (offset, value) in [(4, 100u32), (20, u32::MAX), (24, u32::MAX), (20, 5)] {
            let mut corrupt = bytes.clone();
            corrupt[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            assert!(parse_tagged_properties(&package, &corrupt, 4).is_err());
        }
        assert!(ObjectProperties::parse_export(&package, 0, &bytes[..bytes.len() - 1]).is_err());
        assert!(ObjectProperties::parse_export(&package, 1, &bytes).is_err());
    }

    #[test]
    fn rejects_unknown_asset_preambles_and_invalid_booleans() {
        let mut bytes = Vec::new();
        tag(&mut bytes, 3, 4, 0);
        bytes.push(2);
        end(&mut bytes);
        let mut package = package(bytes.len());
        assert!(parse_tagged_properties(&package, &bytes, 0).is_err());
        bytes[24] = 1;
        assert!(parse_tagged_properties(&package, &bytes, 0).is_ok());
        package.imports[0].object_name.index = 0;
        assert_eq!(
            ObjectProperties::parse_export(&package, 0, &bytes)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }

    #[test]
    fn preserves_struct_types_and_exact_float_bits() {
        let mut bytes = Vec::new();
        tag(&mut bytes, 18, 19, 12);
        name(&mut bytes, 20);
        bytes.extend_from_slice(&[0; 12]);
        end(&mut bytes);
        let package = package(bytes.len());
        let list = parse_tagged_properties(&package, &bytes, 0).unwrap();
        assert_eq!(
            list.entries[0].value,
            Value::Structure {
                name: NameRef {
                    index: 20,
                    number: 0
                }
            }
        );
        let bits = 0x7fc0_1234u32;
        assert_eq!(
            parse_value(
                &package,
                "FloatProperty",
                &TagDetail::None,
                Span {
                    offset: 0,
                    length: 4
                },
                &bits.to_le_bytes()
            )
            .unwrap(),
            Value::FloatBits(bits)
        );
        assert!(Span {
            offset: usize::MAX,
            length: 1
        }
        .bytes(&[])
        .is_err());
        assert!(Span {
            offset: 1,
            length: 0
        }
        .logical_offset(usize::MAX)
        .is_err());
    }
}
