//! Checked Engine.Polys header for the observed UE3 868/3 profile.
//!
//! Polygon records remain opaque, even when the header count is nonzero.
//! Parsing this header does not validate that count against its record payload.

use crate::upk::Package;
use crate::upk::objects::{ObjectProperties, Span};
use std::io;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn unsupported(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolysObject {
    pub object: ObjectProperties,
    pub count: usize,
    pub count_span: Span,
    /// Stored from the same count on save, ignored on load; not called capacity.
    pub secondary_word: u32,
    pub secondary_span: Span,
    /// Native +0x70 array owner, initialized to self by the native constructor.
    /// Null=0, positive=one-based export, negative=one-based import.
    pub owner_reference: i32,
    pub owner_span: Span,
    /// Undecoded polygon serialization, not fixed-size native memory records.
    pub unparsed_native_tail: Span,
}

impl PolysObject {
    pub fn parse_export(package: &Package, export_index: usize, bytes: &[u8]) -> io::Result<Self> {
        if (
            package.summary.file_version,
            package.summary.licensee_version,
        ) != (868, 3)
        {
            return Err(unsupported(
                "Polys native header requires UE3 profile 868/3",
            ));
        }
        let class = package
            .export_class_name(export_index)
            .ok_or_else(|| invalid("Polys export class is unresolved"))?;
        if !class.eq_ignore_ascii_case("Polys") {
            return Err(unsupported("native Polys reader requires Engine.Polys"));
        }
        let object = ObjectProperties::parse_export(package, export_index, bytes)?;
        let start = object.native_payload.offset;
        let end = start
            .checked_add(12)
            .ok_or_else(|| invalid("Polys header offset overflow"))?;
        let header = bytes
            .get(start..end)
            .ok_or_else(|| invalid("truncated Polys native header"))?;
        let count = i32::from_le_bytes(header[..4].try_into().unwrap());
        if count < 0 {
            return Err(invalid("negative Polys record count"));
        }
        let secondary_word = u32::from_le_bytes(header[4..8].try_into().unwrap());
        let owner_reference = i32::from_le_bytes(header[8..12].try_into().unwrap());
        if (owner_reference > 0 && owner_reference as usize > package.exports.len())
            || (owner_reference < 0
                && owner_reference.unsigned_abs() as usize > package.imports.len())
        {
            return Err(invalid("Polys owner reference is outside package tables"));
        }
        // No allocation/iteration depends on count: do not invent a count cap,
        // polygon wire stride, secondary-word equality or record EOF contract.
        Ok(Self {
            object,
            count: count as usize,
            count_span: Span {
                offset: start,
                length: 4,
            },
            secondary_word,
            secondary_span: Span {
                offset: start + 4,
                length: 4,
            },
            owner_reference,
            owner_span: Span {
                offset: start + 8,
                length: 4,
            },
            unparsed_native_tail: Span {
                offset: end,
                length: bytes.len() - end,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upk::{Export, Import, NameEntry, NameRef, Summary, TableLocation};
    fn name(index: usize) -> NameRef {
        NameRef { index, number: 0 }
    }
    fn package(size: usize) -> Package {
        let table = TableLocation {
            count: 0,
            offset: 0,
        };
        Package {
            summary: Summary {
                file_version: 868,
                licensee_version: 3,
                header_size: 0,
                group: String::new(),
                package_flags: 0,
                names: table,
                exports: table,
                imports: table,
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
            names: [
                "None", "Core", "Class", "Package", "Engine", "Polys", "Instance",
            ]
            .into_iter()
            .map(|value| NameEntry {
                value: value.into(),
                flags: 0,
            })
            .collect(),
            imports: vec![
                Import {
                    class_package: name(1),
                    class_name: name(2),
                    outer_index: -2,
                    object_name: name(5),
                },
                Import {
                    class_package: name(1),
                    class_name: name(3),
                    outer_index: 0,
                    object_name: name(4),
                },
            ],
            exports: vec![Export {
                class_index: -1,
                super_index: 0,
                outer_index: 0,
                object_name: name(6),
                archetype_index: 0,
                object_flags: 0,
                serial_size: size,
                serial_offset: 4096,
                export_flags: 0,
                generation_net_object_counts: Vec::new(),
                guid: [0; 16],
                package_flags: 0,
            }],
        }
    }

    fn fixture(count: i32, secondary: u32, owner: i32) -> Vec<u8> {
        [-1i32, 0, 0, count, secondary as i32, owner]
            .into_iter()
            .flat_map(i32::to_le_bytes)
            .collect()
    }
    #[test]
    fn retains_independent_header_words_and_opaque_polygon_bytes() {
        for (count, secondary, owner) in [(0, 7, -1), (2, 0, 1), (i32::MAX, 0xffffffff, 0)] {
            let mut b = fixture(count, secondary, owner);
            b.extend([19, 31, 53]);
            let p = PolysObject::parse_export(&package(b.len()), 0, &b).unwrap();
            assert_eq!(
                (p.count, p.secondary_word, p.owner_reference),
                (count as usize, secondary, owner)
            );
            assert_eq!(
                p.count_span,
                Span {
                    offset: 12,
                    length: 4
                }
            );
            assert_eq!(
                p.secondary_span,
                Span {
                    offset: 16,
                    length: 4
                }
            );
            assert_eq!(
                p.owner_span,
                Span {
                    offset: 20,
                    length: 4
                }
            );
            assert_eq!(p.unparsed_native_tail.bytes(&b).unwrap(), &[19, 31, 53]);
            b.truncate(24);
            assert_eq!(
                PolysObject::parse_export(&package(b.len()), 0, &b)
                    .unwrap()
                    .unparsed_native_tail
                    .length,
                0
            );
        }
    }
    #[test]
    fn rejects_incomplete_headers_negative_counts_and_bad_references() {
        let b = fixture(0, 0, 1);
        for n in 0..b.len() {
            assert!(
                PolysObject::parse_export(&package(n), 0, &b[..n]).is_err(),
                "length {n}"
            );
        }
        for (count, reference) in [(-1, 1), (i32::MIN, 1), (0, 2), (0, -3), (0, i32::MIN)] {
            let b = fixture(count, 0, reference);
            assert_eq!(
                PolysObject::parse_export(&package(b.len()), 0, &b)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidData
            );
        }
    }

    #[test]
    fn requires_exact_engine_class_profile_and_supported_object_prefix() {
        let b = fixture(0, 0, 1);
        for mutation in 0..6 {
            let mut p = package(b.len());
            match mutation {
                0 => p.names[5].value = "World".into(),
                1 => p.names[4].value = "Custom".into(),
                2 => p.exports[0].object_flags = 0x0200_0000_0000_0000,
                3 => p.summary.file_version = 867,
                4 => p.summary.licensee_version = 4,
                5 => p.imports[0].outer_index = 0,
                _ => unreachable!(),
            }
            assert!(PolysObject::parse_export(&p, 0, &b).is_err());
        }
        assert!(PolysObject::parse_export(&package(b.len()), 1, &b).is_err());
    }
}
