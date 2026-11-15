//! Original saved Model +0x19c resource records for the admitted 868/3 profile.
//! This is a raw inspection view, not resource creation or a draw-index binding.
//! Native consumer evidence and boundaries: docs/MODEL_RENDER_RESOURCE.md.

use crate::upk::{
    Package,
    model::{MAX_MODEL_ARRAY_ELEMENTS, ModelArraySlot, ModelBulkArray, ModelObject},
    objects::Span,
};
use std::io;

pub const MODEL_RESOURCE_RECORD_BYTES: usize = 36;

/// Inspection limits; these do not claim native rejection behavior.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub serialized_bytes: usize,
    pub records: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            serialized_bytes: 32 * 1024 * 1024,
            records: MAX_MODEL_ARRAY_ELEMENTS,
        }
    }
}

/// Exact saved fields, with no float normalization, packed-byte conversion,
/// material evaluation or shader semantic-name assignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelResourceRecord {
    pub index: usize,
    pub span: Span,
    pub raw: [u8; MODEL_RESOURCE_RECORD_BYTES],
    /// IEEE32 words at offsets 0, 4 and 8; nonfinite and signed-zero bits survive.
    pub position_bits: [u32; 3],
    /// Native packed output at offsets 12..20, preserved without decoding.
    pub basis_bytes: [u8; 8],
    /// IEEE32 coordinate words at offsets 20 and 24; never recomputed here.
    pub coordinate_bits: [u32; 2],
    /// The separate paired-data bytes at offsets 28..36.
    pub paired_bytes: [u8; 8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRenderResource<'a> {
    pub model: ModelObject,
    /// The original export backing every retained span, including opaque tails.
    pub bytes: &'a [u8],
    /// The saved +0x19c descriptor; order is independent of component ordering.
    pub records: ModelBulkArray,
}
impl<'a> ModelRenderResource<'a> {
    /// `export_index` is zero-based. Exact class, version, prefix, bulk framing
    /// and reference checks are inherited from the existing Model reader.
    pub fn parse_export(
        package: &Package,
        export_index: usize,
        bytes: &'a [u8],
        limits: Limits,
    ) -> io::Result<Self> {
        check_limit(bytes.len(), limits.serialized_bytes, "serialized bytes")?;
        let model = ModelObject::parse_export(package, export_index, bytes)?;
        let records = model
            .native_fields
            .arrays
            .iter()
            .find(|a| {
                a.slot
                    == ModelArraySlot::Field {
                        native_offset: 0x19c,
                    }
            })
            .copied()
            .ok_or_else(|| invalid("Model +0x19c resource descriptor is missing"))?;
        check_limit(records.count, limits.records, "records")?;
        Ok(Self {
            model,
            bytes,
            records,
        })
    }

    /// Inspect one zero-based saved record. No Node base/count relationship or
    /// runtime resource availability is implied by a successful lookup.
    pub fn record(&self, index: usize) -> io::Result<ModelResourceRecord> {
        if self.records.stride != MODEL_RESOURCE_RECORD_BYTES {
            return Err(invalid("Model resource record stride is not 36 bytes"));
        }
        let span = self.records.record_span(index)?;
        let raw: [u8; MODEL_RESOURCE_RECORD_BYTES] = span
            .bytes(self.bytes)?
            .try_into()
            .map_err(|_| invalid("Model resource record is not 36 bytes"))?;
        let word = |offset| u32::from_le_bytes(raw[offset..offset + 4].try_into().unwrap());
        Ok(ModelResourceRecord {
            index,
            span,
            raw,
            position_bits: [word(0), word(4), word(8)],
            basis_bytes: raw[12..20].try_into().unwrap(),
            coordinate_bits: [word(20), word(24)],
            paired_bytes: raw[28..36].try_into().unwrap(),
        })
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn check_limit(count: usize, limit: usize, field: &str) -> io::Result<()> {
    if count > limit {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("Model resource {field} inspection limit exceeded ({limit})"),
        ));
    }
    Ok(())
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
            .map(|s| NameEntry {
                value: s.into(),
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

    fn word(out: &mut Vec<u8>, value: u32) {
        out.extend(value.to_le_bytes());
    }
    fn bulk(out: &mut Vec<u8>, stride: u32, records: &[u8]) {
        assert_eq!(records.len() % stride as usize, 0);
        word(out, stride);
        word(out, (records.len() / stride as usize) as u32);
        out.extend(records);
    }
    fn fixture(records: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        for value in [u32::MAX, 0, 0] {
            word(&mut out, value);
        }
        out.extend([0; 28]);
        bulk(&mut out, 12, &[]);
        bulk(&mut out, 12, &[]);
        bulk(&mut out, 64, &[]);
        word(&mut out, 0); // +0xd8 reference
        word(&mut out, 0); // +0xc8 count
        bulk(&mut out, 24, &[]);
        word(&mut out, 0); // +0x554
        word(&mut out, 0); // +0x55c count
        word(&mut out, 0); // Polys reference
        bulk(&mut out, 4, &[]);
        bulk(&mut out, 4, &[]);
        word(&mut out, 0); // +0x548
        word(&mut out, 0); // +0x54c
        bulk(&mut out, 4, &[]);
        word(&mut out, 0); // +0x4d0
        bulk(&mut out, 36, records);
        out.extend([0; 20]); // four final words and +0x110 count
        out.extend([0xab, 0xcd]);
        out
    }

    #[test]
    fn preserves_all_record_bits_order_backing_identity_and_opaque_tail() {
        let words = [
            0x80000000u32,
            0x7fc12345,
            0x7f800000,
            0x010080ff,
            0xfe7f0080,
            0xff800000,
            0x7fa12345,
            0xdeadbeef,
            0x80000000,
        ];
        let first: Vec<u8> = words.into_iter().flat_map(u32::to_le_bytes).collect();
        let mut records = first.clone();
        records.extend([0x69; 36]);
        let bytes = fixture(&records);
        let package = package(bytes.len());
        let resource =
            ModelRenderResource::parse_export(&package, 0, &bytes, Limits::default()).unwrap();
        assert_eq!(resource.records.count, 2);
        assert_eq!(resource.bytes.as_ptr(), bytes.as_ptr());
        let a = resource.record(0).unwrap();
        let b = resource.record(1).unwrap();
        assert_eq!(a.raw.as_slice(), first);
        assert_eq!(a.position_bits, words[..3]);
        assert_eq!(a.basis_bytes, [0xff, 0x80, 0, 1, 0x80, 0, 0x7f, 0xfe]);
        assert_eq!(a.coordinate_bits, words[5..7]);
        assert_eq!(a.paired_bytes, [0xef, 0xbe, 0xad, 0xde, 0, 0, 0, 0x80]);
        assert_eq!(b.index, 1);
        assert_eq!(b.raw, [0x69; 36]);
        assert_eq!(b.span.offset, a.span.offset + 36);
        assert_eq!(a.span.bytes(resource.bytes).unwrap(), a.raw);
        assert_eq!(resource.record(0).unwrap(), a);
        assert_eq!(
            resource.model,
            ModelObject::parse_export(&package, 0, &bytes).unwrap()
        );
        assert_eq!(
            resource.model.unparsed_native_tail.bytes(&bytes).unwrap(),
            [0xab, 0xcd]
        );
    }

    #[test]
    fn rejects_bad_framing_limits_and_out_of_bounds_record_views() {
        let bytes = fixture(&[0x55; 36]);
        let package = package(bytes.len());
        let mut resource =
            ModelRenderResource::parse_export(&package, 0, &bytes, Limits::default()).unwrap();
        assert_eq!(
            resource.record(1).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(
            resource.record(usize::MAX).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        let descriptor = resource.records;
        for (offset, value, expected) in [
            (
                descriptor.stride_span.offset,
                35,
                io::ErrorKind::Unsupported,
            ),
            (
                descriptor.count_span.offset,
                u32::MAX,
                io::ErrorKind::InvalidData,
            ),
            (descriptor.count_span.offset, 20, io::ErrorKind::InvalidData),
        ] {
            let mut corrupt = bytes.clone();
            corrupt[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            assert_eq!(
                ModelRenderResource::parse_export(&package, 0, &corrupt, Limits::default())
                    .unwrap_err()
                    .kind(),
                expected
            );
        }
        for limits in [
            Limits {
                serialized_bytes: bytes.len() - 1,
                ..Limits::default()
            },
            Limits {
                records: 0,
                ..Limits::default()
            },
        ] {
            assert_eq!(
                ModelRenderResource::parse_export(&package, 0, &bytes, limits)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::Unsupported
            );
        }
        resource.records.records_span.offset = usize::MAX;
        assert_eq!(
            resource.record(0).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        resource.records = descriptor;
        resource.bytes = &bytes[..descriptor.records_span.offset + 35];
        assert_eq!(
            resource.record(0).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn admits_empty_resource_and_preserves_model_class_and_profile_checks() {
        let bytes = fixture(&[]);
        let mut package = package(bytes.len());
        let resource =
            ModelRenderResource::parse_export(&package, 0, &bytes, Limits::default()).unwrap();
        assert_eq!(resource.records.count, 0);
        assert_eq!(
            resource.record(0).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        package.summary.file_version = 867;
        assert_eq!(
            ModelRenderResource::parse_export(&package, 0, &bytes, Limits::default())
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        package.summary.file_version = 868;
        package.names[4].value = "UnrelatedPackage".into();
        assert_eq!(
            ModelRenderResource::parse_export(&package, 0, &bytes, Limits::default())
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }
}
