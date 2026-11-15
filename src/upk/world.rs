//! Native Engine.World file payload for the observed UE3 868/3 profile.
//!
//! Links remain package-local signed references. Seven-word native records and
//! any trailing bytes retain their exact representation without runtime meaning.
//! Evidence and limits are recorded in docs/UPK_WORLD.md.

use crate::upk::Package;
use crate::upk::objects::{ObjectProperties, Span};
use std::io;

pub const MAX_WORLD_ARRAY_REFERENCES: usize = 65_536;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn unsupported(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message.into())
}

/// Native slots identify serialization roles without guessing gameplay names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldLinkSlot {
    /// Native +0x80; construction binds a Level named PersistentLevel here.
    PersistentLevel,
    Field {
        native_offset: usize,
    },
    ArrayElement {
        native_offset: usize,
        index: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldLink {
    pub slot: WorldLinkSlot,
    /// Zero is null, negative is a one-based import, positive a one-based export.
    pub reference: i32,
    /// Relative to the complete serialized export.
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldRecord {
    pub native_offset: usize,
    /// Relative to the complete serialized export.
    pub span: Span,
    /// Seven successive serialized 32-bit words; no scalar type is inferred.
    pub words: [u32; 7],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldObject {
    pub object: ObjectProperties,
    /// Serialization order: fields +0x80, +0x88, +0x118, then array +0x380.
    /// Duplicates and null references are preserved. Nothing is loaded or run.
    pub links: Vec<WorldLink>,
    /// Native fields +0xa8, +0xc4, +0xe0, +0xfc in serialization order.
    pub records: [WorldRecord; 4],
    pub reference_array_count: usize,
    pub reference_array_count_span: Span,
    /// Bytes beyond the established native file layout, retained without meaning.
    pub unknown_tail: Span,
}

impl WorldObject {
    pub fn parse_export(package: &Package, export_index: usize, bytes: &[u8]) -> io::Result<Self> {
        if (
            package.summary.file_version,
            package.summary.licensee_version,
        ) != (868, 3)
        {
            return Err(unsupported("World native fields require UE3 profile 868/3"));
        }
        let class = package
            .export_class_name(export_index)
            .ok_or_else(|| invalid("World export class is unresolved"))?;
        if !class.eq_ignore_ascii_case("World") {
            return Err(unsupported("native World reader requires Engine.World"));
        }
        // Includes exact Engine class-import identity and state-frame rejection.
        let object = ObjectProperties::parse_export(package, export_index, bytes)?;
        let mut input = Reader {
            bytes,
            position: object.native_payload.offset,
        };
        let mut links = vec![
            input.link(package, WorldLinkSlot::PersistentLevel)?,
            input.link(
                package,
                WorldLinkSlot::Field {
                    native_offset: 0x88,
                },
            )?,
        ];
        let records = [
            input.record(0xa8)?,
            input.record(0xc4)?,
            input.record(0xe0)?,
            input.record(0xfc)?,
        ];
        links.push(input.link(
            package,
            WorldLinkSlot::Field {
                native_offset: 0x118,
            },
        )?);
        let reference_array_count_span = Span {
            offset: input.position,
            length: 4,
        };
        let signed_count = input.word()? as i32;
        if signed_count < 0 {
            return Err(invalid("negative World reference-array count"));
        }
        let reference_array_count = signed_count as usize;
        if reference_array_count > MAX_WORLD_ARRAY_REFERENCES {
            return Err(unsupported(
                "World reference array exceeds inspection limit",
            ));
        }
        let array_bytes = reference_array_count
            .checked_mul(4)
            .ok_or_else(|| invalid("World reference-array size overflow"))?;
        if array_bytes > bytes.len() - input.position {
            return Err(invalid("truncated World reference array"));
        }
        links.reserve(reference_array_count);
        for index in 0..reference_array_count {
            links.push(input.link(
                package,
                WorldLinkSlot::ArrayElement {
                    native_offset: 0x380,
                    index,
                },
            )?);
        }
        let unknown_tail = Span {
            offset: input.position,
            length: bytes.len() - input.position,
        };
        Ok(Self {
            object,
            links,
            records,
            reference_array_count,
            reference_array_count_span,
            unknown_tail,
        })
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl Reader<'_> {
    fn word(&mut self) -> io::Result<u32> {
        let end = self
            .position
            .checked_add(4)
            .ok_or_else(|| invalid("World field offset overflow"))?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| invalid("truncated World native field"))?;
        self.position = end;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    fn link(&mut self, package: &Package, slot: WorldLinkSlot) -> io::Result<WorldLink> {
        let span = Span {
            offset: self.position,
            length: 4,
        };
        let reference = self.word()? as i32;
        if (reference > 0 && reference as usize > package.exports.len())
            || (reference < 0 && reference.unsigned_abs() as usize > package.imports.len())
        {
            return Err(invalid("World object reference is outside package tables"));
        }
        Ok(WorldLink {
            slot,
            reference,
            span,
        })
    }

    fn record(&mut self, native_offset: usize) -> io::Result<WorldRecord> {
        let span = Span {
            offset: self.position,
            length: 28,
        };
        let mut words = [0; 7];
        for word in &mut words {
            *word = self.word()?;
        }
        Ok(WorldRecord {
            native_offset,
            span,
            words,
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
                "None", "Core", "Class", "Package", "Engine", "World", "Instance",
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
                serial_offset: 1234,
                export_flags: 0,
                generation_net_object_counts: Vec::new(),
                guid: [0; 16],
                package_flags: 0,
            }],
        }
    }

    fn fixture(array: &[i32]) -> Vec<u8> {
        let mut words = vec![u32::MAX, 0, 0, 1, u32::MAX]; // NetIndex, None, two links
        words.extend((0..28).map(|i| 0x7fc0_0001 + i));
        words.extend([0, array.len() as u32]);
        words.extend(array.iter().map(|value| *value as u32));
        words.into_iter().flat_map(u32::to_le_bytes).collect()
    }

    #[test]
    fn preserves_link_order_record_bits_and_unknown_tail() {
        let mut bytes = fixture(&[1, -1, 0, 1]);
        bytes.extend([19, 83, 127]);
        let parsed = WorldObject::parse_export(&package(bytes.len()), 0, &bytes).unwrap();
        assert_eq!(
            parsed
                .links
                .iter()
                .map(|link| link.reference)
                .collect::<Vec<_>>(),
            [1, -1, 0, 1, -1, 0, 1]
        );
        assert_eq!(parsed.links[0].slot, WorldLinkSlot::PersistentLevel);
        assert_eq!(
            parsed
                .links
                .iter()
                .map(|link| link.span.offset)
                .collect::<Vec<_>>(),
            [12, 16, 132, 140, 144, 148, 152]
        );
        assert_eq!(parsed.records[0].words[0], 0x7fc0_0001);
        assert_eq!(parsed.records[3].words[6], 0x7fc0_001c);
        assert_eq!(
            parsed.records[3].span,
            Span {
                offset: 104,
                length: 28
            }
        );
        assert_eq!(
            parsed.links[6].slot,
            WorldLinkSlot::ArrayElement {
                native_offset: 0x380,
                index: 3
            }
        );
        assert_eq!(
            parsed.reference_array_count_span,
            Span {
                offset: 136,
                length: 4
            }
        );
        assert_eq!(parsed.reference_array_count, 4);
        assert_eq!(parsed.unknown_tail.bytes(&bytes).unwrap(), &[19, 83, 127]);
    }

    #[test]
    fn rejects_truncated_fields_bad_counts_and_invalid_references() {
        let bytes = fixture(&[1]);
        for length in 0..bytes.len() {
            assert!(
                WorldObject::parse_export(&package(length), 0, &bytes[..length]).is_err(),
                "accepted truncation {length}"
            );
        }
        for (offset, value) in [
            (12, 2i32),
            (16, -3),
            (132, i32::MIN),
            (136, -1),
            (136, i32::MAX),
            (136, 2),
            (140, 2),
        ] {
            let mut changed = bytes.clone();
            changed[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            assert!(WorldObject::parse_export(&package(changed.len()), 0, &changed).is_err());
        }
    }

    #[test]
    fn rejects_unproved_class_profile_and_state_frame() {
        let bytes = fixture(&[]);
        for mutation in 0..6 {
            let mut changed = package(bytes.len());
            match mutation {
                0 => changed.names[5].value = "Level".into(),
                1 => changed.names[4].value = "OtherPackage".into(),
                2 => changed.exports[0].object_flags = 0x0200_0000_0000_0000,
                3 => changed.summary.file_version = 867,
                4 => changed.summary.licensee_version = 4,
                5 => changed.imports[0].outer_index = 0,
                _ => unreachable!(),
            }
            assert!(WorldObject::parse_export(&changed, 0, &bytes).is_err());
        }
        assert!(WorldObject::parse_export(&package(bytes.len()), 1, &bytes).is_err());
    }
}
