//! Checked Engine.Model native prefix for the observed UE3 868/3 profile.
//!
//! Raw bulk records are retained without geometry interpretation. Containers
//! after the +0xd8 reference remain opaque. See docs/UPK_MODEL_POLYS.md.

use crate::upk::Package;
use crate::upk::objects::{ObjectProperties, Span};
use std::io;

/// Inspection limit, not an asserted native format limit.
pub const MAX_MODEL_ARRAY_ELEMENTS: usize = 1_048_576;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn unsupported(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelRecord {
    pub native_offset: usize,
    pub span: Span,
    /// Seven serialized words with their exact bits; no scalar type imposed.
    pub words: [u32; 7],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelArraySlot {
    /// Field names for the two twelve-byte containers remain unproved.
    Field { native_offset: usize },
    /// Native +0x68, identified by the owned Nodes diagnostic.
    Nodes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelBulkArray {
    pub slot: ModelArraySlot,
    pub stride: usize,
    pub stride_span: Span,
    pub count: usize,
    pub count_span: Span,
    /// All stored records; no records are copied, normalized or interpreted.
    pub records_span: Span,
}

impl ModelBulkArray {
    /// Zero-based record span relative to the complete export.
    pub fn record_span(&self, index: usize) -> io::Result<Span> {
        if index >= self.count {
            return Err(invalid("Model bulk record index is outside the array"));
        }
        let start = index
            .checked_mul(self.stride)
            .ok_or_else(|| invalid("Model bulk record offset overflow"))?;
        let end = start
            .checked_add(self.stride)
            .ok_or_else(|| invalid("Model bulk record end overflow"))?;
        if end > self.records_span.length {
            return Err(invalid("Model bulk record exceeds retained span"));
        }
        Ok(Span {
            offset: self
                .records_span
                .offset
                .checked_add(start)
                .ok_or_else(|| invalid("Model bulk record absolute offset overflow"))?,
            length: self.stride,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelLink {
    pub native_offset: usize,
    /// Null=0, positive=one-based export, negative=one-based import.
    pub reference: i32,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelObject {
    pub object: ObjectProperties,
    pub record_c5c: ModelRecord,
    /// Serialization order: +0x98, +0xb0, +0x68 (Nodes).
    pub arrays: [ModelBulkArray; 3],
    /// This is not an inventory of every Model dependency.
    pub link_d8: ModelLink,
    /// Starts before the +0xc8 container; its completeness is not checked.
    pub unparsed_native_tail: Span,
}

impl ModelObject {
    pub fn parse_export(package: &Package, export_index: usize, bytes: &[u8]) -> io::Result<Self> {
        if (
            package.summary.file_version,
            package.summary.licensee_version,
        ) != (868, 3)
        {
            return Err(unsupported(
                "Model native prefix requires UE3 profile 868/3",
            ));
        }
        let class = package
            .export_class_name(export_index)
            .ok_or_else(|| invalid("Model export class is unresolved"))?;
        if !class.eq_ignore_ascii_case("Model") {
            return Err(unsupported("native Model reader requires Engine.Model"));
        }
        // Establishes exact Engine class-import identity and rejects state frames.
        let object = ObjectProperties::parse_export(package, export_index, bytes)?;
        let mut input = Reader {
            bytes,
            position: object.native_payload.offset,
        };
        let span = input.take(28)?;
        let mut words = [0; 7];
        for (word, raw) in words.iter_mut().zip(span.bytes(bytes)?.chunks_exact(4)) {
            *word = u32::from_le_bytes(raw.try_into().unwrap());
        }
        let record_c5c = ModelRecord {
            native_offset: 0xc5c,
            span,
            words,
        };
        let arrays = [
            input.bulk(
                ModelArraySlot::Field {
                    native_offset: 0x98,
                },
                12,
            )?,
            input.bulk(
                ModelArraySlot::Field {
                    native_offset: 0xb0,
                },
                12,
            )?,
            input.bulk(ModelArraySlot::Nodes, 64)?,
        ];
        let (bits, span) = input.word()?;
        let reference = bits as i32;
        if (reference > 0 && reference as usize > package.exports.len())
            || (reference < 0 && reference.unsigned_abs() as usize > package.imports.len())
        {
            return Err(invalid("Model object reference is outside package tables"));
        }
        let link_d8 = ModelLink {
            native_offset: 0xd8,
            reference,
            span,
        };
        let unparsed_native_tail = Span {
            offset: input.position,
            length: bytes.len() - input.position,
        };
        Ok(Self {
            object,
            record_c5c,
            arrays,
            link_d8,
            unparsed_native_tail,
        })
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl Reader<'_> {
    fn take(&mut self, length: usize) -> io::Result<Span> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| invalid("Model field offset overflow"))?;
        if end > self.bytes.len() {
            return Err(invalid("truncated Model native prefix"));
        }
        let span = Span {
            offset: self.position,
            length,
        };
        self.position = end;
        Ok(span)
    }
    fn word(&mut self) -> io::Result<(u32, Span)> {
        let span = self.take(4)?;
        Ok((
            u32::from_le_bytes(span.bytes(self.bytes)?.try_into().unwrap()),
            span,
        ))
    }
    fn bulk(&mut self, slot: ModelArraySlot, expected_stride: usize) -> io::Result<ModelBulkArray> {
        let (encoded_stride, stride_span) = self.word()?;
        if encoded_stride as usize != expected_stride {
            return Err(unsupported("Model encoded bulk stride is unsupported"));
        }
        let (bits, count_span) = self.word()?;
        let signed_count = bits as i32;
        if signed_count < 0 {
            return Err(invalid("negative Model bulk array count"));
        }
        let count = signed_count as usize;
        if count > MAX_MODEL_ARRAY_ELEMENTS {
            return Err(unsupported("Model bulk count exceeds inspection limit"));
        }
        let length = count
            .checked_mul(expected_stride)
            .ok_or_else(|| invalid("Model bulk array size overflow"))?;
        let records_span = self.take(length)?;
        Ok(ModelBulkArray {
            slot,
            stride: expected_stride,
            stride_span,
            count,
            count_span,
            records_span,
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
                "None", "Core", "Class", "Package", "Engine", "Model", "Instance",
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

    fn fixture() -> Vec<u8> {
        let mut b = Vec::new();
        for value in [-1i32, 0, 0] {
            b.extend(value.to_le_bytes());
        }
        for value in [0x80000000u32, 0x7fc00001, 3, 4, 5, 6, 0xffffffff] {
            b.extend(value.to_le_bytes());
        }
        for (stride, count) in [(12u32, 2i32), (12, 0), (64, 1)] {
            b.extend(stride.to_le_bytes());
            b.extend(count.to_le_bytes());
            b.extend(std::iter::repeat_n(0xff, stride as usize * count as usize));
        }
        b.extend((-1i32).to_le_bytes());
        b
    }
    #[test]
    fn retains_raw_records_spans_links_and_tail() {
        let mut b = fixture();
        let end = b.len();
        b.extend([17, 29, 47]);
        let m = ModelObject::parse_export(&package(b.len()), 0, &b).unwrap();
        assert_eq!(
            m.record_c5c.words,
            [0x80000000, 0x7fc00001, 3, 4, 5, 6, 0xffffffff]
        );
        assert_eq!(m.arrays.map(|a| a.count), [2, 0, 1]);
        assert_eq!(m.arrays[2].slot, ModelArraySlot::Nodes);
        assert_eq!(
            m.arrays[0].record_span(1).unwrap().bytes(&b).unwrap(),
            &[0xff; 12]
        );
        assert_eq!(
            m.arrays[2].record_span(0).unwrap().bytes(&b).unwrap(),
            &[0xff; 64]
        );
        for a in m.arrays {
            assert!(a.record_span(a.count).is_err());
            assert!(a.record_span(usize::MAX).is_err());
        }
        assert_eq!(m.link_d8.reference, -1);
        assert_eq!(
            m.unparsed_native_tail,
            Span {
                offset: end,
                length: 3
            }
        );
        assert_eq!(m.unparsed_native_tail.bytes(&b).unwrap(), &[17, 29, 47]);
    }
    #[test]
    fn bounds_each_known_prefix_field_and_rejects_bad_headers() {
        let b = fixture();
        let m = ModelObject::parse_export(&package(b.len()), 0, &b).unwrap();
        for n in 0..b.len() {
            assert!(
                ModelObject::parse_export(&package(n), 0, &b[..n]).is_err(),
                "length {n}"
            );
        }
        for a in m.arrays {
            for (at, bits, kind) in [
                (a.stride_span.offset, 0u32, io::ErrorKind::Unsupported),
                (a.stride_span.offset, 0xffffffff, io::ErrorKind::Unsupported),
                (a.count_span.offset, 0xffffffff, io::ErrorKind::InvalidData),
                (
                    a.count_span.offset,
                    MAX_MODEL_ARRAY_ELEMENTS as u32 + 1,
                    io::ErrorKind::Unsupported,
                ),
                (
                    a.count_span.offset,
                    MAX_MODEL_ARRAY_ELEMENTS as u32,
                    io::ErrorKind::InvalidData,
                ),
            ] {
                let mut changed = b.clone();
                changed[at..at + 4].copy_from_slice(&bits.to_le_bytes());
                assert_eq!(
                    ModelObject::parse_export(&package(changed.len()), 0, &changed)
                        .unwrap_err()
                        .kind(),
                    kind
                );
            }
        }
        for reference in [2i32, -3, i32::MIN] {
            let mut changed = b.clone();
            let at = m.link_d8.span.offset;
            changed[at..at + 4].copy_from_slice(&reference.to_le_bytes());
            assert_eq!(
                ModelObject::parse_export(&package(changed.len()), 0, &changed)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidData
            );
        }
    }

    #[test]
    fn requires_exact_engine_class_profile_and_supported_object_prefix() {
        let b = fixture();
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
            assert!(ModelObject::parse_export(&p, 0, &b).is_err());
        }
        assert!(ModelObject::parse_export(&package(b.len()), 1, &b).is_err());
    }
}
