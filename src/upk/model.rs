//! Checked Engine.Model native prefix for the observed UE3 868/3 profile.
//!
//! Raw bulk records and fields are retained without geometry interpretation.
//! Bytes beyond the final +0x110 array remain opaque. See docs/UPK_MODEL_POLYS.md.

use crate::upk::Package;
use crate::upk::objects::{ObjectProperties, Span};
use std::io;

/// Inspection limit, not an asserted native format limit.
pub const MAX_MODEL_ARRAY_ELEMENTS: usize = 1_048_576;
/// Bound for arrays whose individual fields are retained as Rust records.
pub const MAX_MODEL_STRUCTURED_RECORDS: usize = 65_536;
/// Number of inline native records before the separate +0xc5c field.
pub const MODEL_55C_RECORD_CAPACITY: usize = 64;

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
    /// Relative to the containing native Model or record, as documented.
    pub native_offset: usize,
    /// Null=0, positive=one-based export, negative=one-based import.
    pub reference: i32,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelWord {
    pub native_offset: usize,
    pub span: Span,
    pub bits: u32,
}

/// One 60-byte file record from the native +0xc8 array (96 bytes in memory).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelC8Record {
    pub span: Span,
    /// Native record offsets +0 and +0x20; meanings remain unproved.
    pub links: [ModelLink; 2],
    /// Six fields at native +0x08..+0x1c, then seven at +0x30..+0x48.
    /// These are raw bits, not asserted indices, flags or scalar values.
    pub words: [u32; 13],
}

/// One 24-byte file record from native +0x55c (28 bytes in memory).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Model55cRecord {
    pub span: Span,
    pub link: ModelLink,
    /// Raw 64-bit fields at native record +0x0c and +0x14, in wire order.
    pub words_64: [u64; 2],
    /// Raw 32-bit field at native record +0x08, serialized last.
    pub word_08: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRecordArray<T> {
    pub native_offset: usize,
    pub count_span: Span,
    pub records_span: Span,
    pub records: Vec<T>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelNativeFields {
    pub records_c8: ModelRecordArray<ModelC8Record>,
    /// Serialization order: +0x80, +0xe0, +0xf0, +0x100, +0x19c.
    /// Raw strides: 24, 4, 4, 4, 36; field names remain unproved.
    pub arrays: [ModelBulkArray; 5],
    /// Native +0x554, +0x548, +0x54c, +0x4d0 in wire order.
    pub words: [ModelWord; 4],
    pub records_55c: ModelRecordArray<Model55cRecord>,
    /// Native +0x60, bound to Polys by owned constructor evidence.
    /// The saved reference is validated but not required to name a Polys class.
    pub polys_link: ModelLink,
}

/// Final array record: 36 serialized bytes, 28 bytes in native memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Model110Record {
    pub span: Span,
    /// Wire order: flag bit 0 word, flag bit 1 word, native +0x18 word,
    /// flag bit 2 word, then native +0x04/+0x08/+0x0c/+0x10/+0x14 words.
    /// All bits are retained; native flag masking is not applied.
    pub words: [u32; 9],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelFinalFields {
    /// Native +0x4d4/+0x4d8/+0x4dc/+0x4e0, without an inferred type name.
    pub words: [ModelWord; 4],
    pub records_110: ModelRecordArray<Model110Record>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelObject {
    pub object: ObjectProperties,
    pub record_c5c: ModelRecord,
    /// Serialization order: +0x98, +0xb0, +0x68 (Nodes).
    pub arrays: [ModelBulkArray; 3],
    /// This is not an inventory of every Model dependency.
    pub link_d8: ModelLink,
    pub native_fields: ModelNativeFields,
    pub final_fields: ModelFinalFields,
    /// Bytes beyond the established ordinary-file serialization, if any.
    /// These are retained without meaning or completeness validation.
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
        let link_d8 = input.reference(package, 0xd8)?;
        let records_c8 = input.records_c8(package)?;
        let array_80 = input.bulk(
            ModelArraySlot::Field {
                native_offset: 0x80,
            },
            24,
        )?;
        let word_554 = input.field_word(0x554)?;
        let records_55c = input.records_55c(package)?;
        let polys_link = input.reference(package, 0x60)?;
        let array_e0 = input.bulk(
            ModelArraySlot::Field {
                native_offset: 0xe0,
            },
            4,
        )?;
        let array_f0 = input.bulk(
            ModelArraySlot::Field {
                native_offset: 0xf0,
            },
            4,
        )?;
        let word_548 = input.field_word(0x548)?;
        let word_54c = input.field_word(0x54c)?;
        let array_100 = input.bulk(
            ModelArraySlot::Field {
                native_offset: 0x100,
            },
            4,
        )?;
        // The additional pre-686 container is absent from the admitted profile.
        let word_4d0 = input.field_word(0x4d0)?;
        let array_19c = input.bulk(
            ModelArraySlot::Field {
                native_offset: 0x19c,
            },
            36,
        )?;
        let native_fields = ModelNativeFields {
            records_c8,
            arrays: [array_80, array_e0, array_f0, array_100, array_19c],
            words: [word_554, word_548, word_54c, word_4d0],
            records_55c,
            polys_link,
        };
        // The intervening native helper can change in-memory +0x4d0; this
        // reader preserves its saved word and does not execute that helper.
        let final_fields = ModelFinalFields {
            words: [
                input.field_word(0x4d4)?,
                input.field_word(0x4d8)?,
                input.field_word(0x4dc)?,
                input.field_word(0x4e0)?,
            ],
            records_110: input.records_110()?,
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
            native_fields,
            final_fields,
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
    fn field_word(&mut self, native_offset: usize) -> io::Result<ModelWord> {
        let (bits, span) = self.word()?;
        Ok(ModelWord {
            native_offset,
            span,
            bits,
        })
    }
    fn reference(&mut self, package: &Package, native_offset: usize) -> io::Result<ModelLink> {
        let (bits, span) = self.word()?;
        let reference = bits as i32;
        if (reference > 0 && reference as usize > package.exports.len())
            || (reference < 0 && reference.unsigned_abs() as usize > package.imports.len())
        {
            return Err(invalid("Model object reference is outside package tables"));
        }
        Ok(ModelLink {
            native_offset,
            reference,
            span,
        })
    }
    fn structured_header(
        &mut self,
        stride: usize,
        maximum: usize,
        limit_error: io::ErrorKind,
    ) -> io::Result<(usize, Span, Span)> {
        let (bits, count_span) = self.word()?;
        let signed = bits as i32;
        if signed < 0 {
            return Err(invalid("negative Model structured-record count"));
        }
        let count = signed as usize;
        if count > maximum {
            return Err(io::Error::new(
                limit_error,
                "Model structured-record count exceeds admitted limit",
            ));
        }
        let length = count
            .checked_mul(stride)
            .ok_or_else(|| invalid("Model structured-record byte count overflow"))?;
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| invalid("Model structured-record end overflow"))?;
        if end > self.bytes.len() {
            return Err(invalid("truncated Model structured records"));
        }
        Ok((
            count,
            count_span,
            Span {
                offset: self.position,
                length,
            },
        ))
    }
    fn records_c8(&mut self, package: &Package) -> io::Result<ModelRecordArray<ModelC8Record>> {
        let (count, count_span, records_span) =
            self.structured_header(60, MAX_MODEL_STRUCTURED_RECORDS, io::ErrorKind::Unsupported)?;
        let mut records = Vec::with_capacity(count);
        for _ in 0..count {
            let start = self.position;
            let first = self.reference(package, 0)?;
            let mut words = [0; 13];
            for word in &mut words[..6] {
                *word = self.word()?.0;
            }
            let second = self.reference(package, 0x20)?;
            for word in &mut words[6..] {
                *word = self.word()?.0;
            }
            records.push(ModelC8Record {
                span: Span {
                    offset: start,
                    length: 60,
                },
                links: [first, second],
                words,
            });
        }
        Ok(ModelRecordArray {
            native_offset: 0xc8,
            count_span,
            records_span,
            records,
        })
    }
    fn records_55c(&mut self, package: &Package) -> io::Result<ModelRecordArray<Model55cRecord>> {
        let (count, count_span, records_span) =
            self.structured_header(24, MODEL_55C_RECORD_CAPACITY, io::ErrorKind::InvalidData)?;
        let mut records = Vec::with_capacity(count);
        for _ in 0..count {
            let start = self.position;
            let link = self.reference(package, 0)?;
            let mut words_64 = [0; 2];
            for word in &mut words_64 {
                let span = self.take(8)?;
                *word = u64::from_le_bytes(span.bytes(self.bytes)?.try_into().unwrap());
            }
            let word_08 = self.word()?.0;
            records.push(Model55cRecord {
                span: Span {
                    offset: start,
                    length: 24,
                },
                link,
                words_64,
                word_08,
            });
        }
        Ok(ModelRecordArray {
            native_offset: 0x55c,
            count_span,
            records_span,
            records,
        })
    }
    fn records_110(&mut self) -> io::Result<ModelRecordArray<Model110Record>> {
        let (count, count_span, records_span) =
            self.structured_header(36, MAX_MODEL_STRUCTURED_RECORDS, io::ErrorKind::Unsupported)?;
        let mut records = Vec::with_capacity(count);
        for _ in 0..count {
            let start = self.position;
            let mut words = [0; 9];
            for word in &mut words {
                *word = self.word()?.0;
            }
            records.push(Model110Record {
                span: Span {
                    offset: start,
                    length: 36,
                },
                words,
            });
        }
        Ok(ModelRecordArray {
            native_offset: 0x110,
            count_span,
            records_span,
            records,
        })
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
        fixture_with_inline_count(2)
    }
    fn fixture_with_inline_count(inline_count: u32) -> Vec<u8> {
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
        b.extend(2i32.to_le_bytes());
        for second_reference in [0i32, 1] {
            b.extend((-1i32).to_le_bytes());
            for word in [0xffffffffu32, 0x7fc00001, 2, 3, 4, 5] {
                b.extend(word.to_le_bytes());
            }
            b.extend(second_reference.to_le_bytes());
            for word in [6u32, 7, 8, 9, 10, 11, 0x80000000] {
                b.extend(word.to_le_bytes());
            }
        }
        bulk_fixture(&mut b, 24, 1);
        b.extend(0xffffffffu32.to_le_bytes());
        b.extend(inline_count.to_le_bytes());
        for reference in [0i32, -2].into_iter().cycle().take(inline_count as usize) {
            b.extend(reference.to_le_bytes());
            b.extend(0x8000000000000000u64.to_le_bytes());
            b.extend(0x0123456789abcdefu64.to_le_bytes());
            b.extend(0x7fc00001u32.to_le_bytes());
        }
        b.extend(1i32.to_le_bytes());
        bulk_fixture(&mut b, 4, 1);
        bulk_fixture(&mut b, 4, 0);
        b.extend(0x80000000u32.to_le_bytes());
        b.extend(0x7fc00001u32.to_le_bytes());
        bulk_fixture(&mut b, 4, 1);
        b.extend(0xffffffffu32.to_le_bytes());
        bulk_fixture(&mut b, 36, 1);
        for word in [0x80000000u32, 0x7fc00001, 0xffffffff, 0x01234567] {
            b.extend(word.to_le_bytes());
        }
        b.extend(2i32.to_le_bytes());
        for _ in 0..2 {
            for word in [
                0xfffffffeu32,
                0xffffffff,
                0x7fc00001,
                2,
                0x80000000,
                5,
                6,
                7,
                8,
            ] {
                b.extend(word.to_le_bytes());
            }
        }
        b
    }
    fn bulk_fixture(bytes: &mut Vec<u8>, stride: u32, count: u32) {
        bytes.extend(stride.to_le_bytes());
        bytes.extend(count.to_le_bytes());
        bytes.extend(std::iter::repeat_n(0xa5, stride as usize * count as usize));
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
        for a in m.arrays.into_iter().chain(m.native_fields.arrays) {
            assert!(a.record_span(a.count).is_err());
            assert!(a.record_span(usize::MAX).is_err());
        }
        assert_eq!(m.link_d8.reference, -1);
        assert_eq!(m.native_fields.arrays.map(|a| a.count), [1, 1, 0, 1, 1]);
        assert_eq!(
            m.native_fields.words.map(|w| w.native_offset),
            [0x554, 0x548, 0x54c, 0x4d0]
        );
        assert_eq!(
            m.native_fields.words.map(|w| w.bits),
            [0xffffffff, 0x80000000, 0x7fc00001, 0xffffffff]
        );
        let c8 = &m.native_fields.records_c8;
        assert_eq!(c8.records_span.length, 120);
        assert_eq!(c8.records[0].span.length, 60);
        assert_eq!(c8.records[1].span.offset, c8.records[0].span.offset + 60);
        assert_eq!(c8.records[0].links.map(|l| l.reference), [-1, 0]);
        assert_eq!(c8.records[1].links.map(|l| l.reference), [-1, 1]);
        assert_eq!(
            c8.records[0].words,
            [
                0xffffffff, 0x7fc00001, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0x80000000
            ]
        );
        let records = &m.native_fields.records_55c;
        assert_eq!(records.records_span.length, 48);
        assert_eq!(records.records[1].link.reference, -2);
        assert_eq!(
            records.records[1].words_64,
            [0x8000000000000000, 0x0123456789abcdef]
        );
        assert_eq!(records.records[1].word_08, 0x7fc00001);
        assert_eq!(m.native_fields.polys_link.reference, 1);
        assert_eq!(
            m.final_fields.words.map(|w| w.native_offset),
            [0x4d4, 0x4d8, 0x4dc, 0x4e0]
        );
        assert_eq!(
            m.final_fields.words.map(|w| w.bits),
            [0x80000000, 0x7fc00001, 0xffffffff, 0x01234567]
        );
        let last = &m.final_fields.records_110;
        assert_eq!(last.records_span.length, 72);
        assert_eq!(last.records.len(), 2);
        assert_eq!(
            last.records[1].span.offset,
            last.records[0].span.offset + 36
        );
        assert_eq!(
            last.records[0].words,
            [
                0xfffffffe, 0xffffffff, 0x7fc00001, 2, 0x80000000, 5, 6, 7, 8
            ]
        );
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
        for a in m.arrays.into_iter().chain(m.native_fields.arrays) {
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
    fn checks_every_structured_reference_and_retains_raw_non_reference_words() {
        let b = fixture();
        let m = ModelObject::parse_export(&package(b.len()), 0, &b).unwrap();
        let mut links = vec![m.native_fields.polys_link];
        for record in &m.native_fields.records_c8.records {
            links.extend(record.links);
        }
        for record in &m.native_fields.records_55c.records {
            links.push(record.link);
        }
        for link in links {
            for reference in [2i32, -3, i32::MIN] {
                let mut changed = b.clone();
                changed[link.span.offset..link.span.offset + 4]
                    .copy_from_slice(&reference.to_le_bytes());
                assert_eq!(
                    ModelObject::parse_export(&package(changed.len()), 0, &changed)
                        .unwrap_err()
                        .kind(),
                    io::ErrorKind::InvalidData
                );
            }
        }
        // Opaque arrays and scalar words may contain bit patterns which would
        // be out-of-range object references; they must not be reinterpreted.
        for array in m.native_fields.arrays {
            if array.count > 0 {
                assert!(
                    array
                        .record_span(0)
                        .unwrap()
                        .bytes(&b)
                        .unwrap()
                        .iter()
                        .all(|&x| x == 0xa5)
                );
            }
        }
    }

    #[test]
    fn bounds_dynamic_records_and_the_fixed_inline_table() {
        let b = fixture();
        let m = ModelObject::parse_export(&package(b.len()), 0, &b).unwrap();
        for (at, bits, kind) in [
            (
                m.native_fields.records_c8.count_span.offset,
                u32::MAX,
                io::ErrorKind::InvalidData,
            ),
            (
                m.native_fields.records_c8.count_span.offset,
                MAX_MODEL_STRUCTURED_RECORDS as u32 + 1,
                io::ErrorKind::Unsupported,
            ),
            (
                m.native_fields.records_c8.count_span.offset,
                MAX_MODEL_STRUCTURED_RECORDS as u32,
                io::ErrorKind::InvalidData,
            ),
            (
                m.native_fields.records_55c.count_span.offset,
                u32::MAX,
                io::ErrorKind::InvalidData,
            ),
            (
                m.native_fields.records_55c.count_span.offset,
                65,
                io::ErrorKind::InvalidData,
            ),
            (
                m.native_fields.records_55c.count_span.offset,
                i32::MAX as u32,
                io::ErrorKind::InvalidData,
            ),
            (
                m.native_fields.arrays[0].stride_span.offset,
                16,
                io::ErrorKind::Unsupported,
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
        for (count, kind) in [
            (-1i32, io::ErrorKind::InvalidData),
            (65_537, io::ErrorKind::Unsupported),
            (65_536, io::ErrorKind::InvalidData),
        ] {
            let mut changed = b.clone();
            let at = m.final_fields.records_110.count_span.offset;
            changed[at..at + 4].copy_from_slice(&count.to_le_bytes());
            assert_eq!(
                ModelObject::parse_export(&package(changed.len()), 0, &changed)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        let mut empty_final_array = b.clone();
        let at = m.final_fields.records_110.count_span.offset;
        empty_final_array[at..at + 4].copy_from_slice(&0i32.to_le_bytes());
        let empty =
            ModelObject::parse_export(&package(empty_final_array.len()), 0, &empty_final_array)
                .unwrap();
        assert!(empty.final_fields.records_110.records.is_empty());
        assert_eq!(empty.unparsed_native_tail.offset, at + 4);
        assert_eq!(
            empty
                .unparsed_native_tail
                .bytes(&empty_final_array)
                .unwrap(),
            &b[at + 4..]
        );
        let full = fixture_with_inline_count(64);
        assert_eq!(
            ModelObject::parse_export(&package(full.len()), 0, &full)
                .unwrap()
                .native_fields
                .records_55c
                .records
                .len(),
            64
        );
        let overflow = fixture_with_inline_count(65);
        assert_eq!(
            ModelObject::parse_export(&package(overflow.len()), 0, &overflow)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
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
