//! Checked selected Model Node -> vertex-pool -> position/surface-record joins.
//! This preserves stored geometry records; it does not triangulate or render.
//! Native proof and limits: docs/UPK_MODEL_NODE_GEOMETRY.md.

use crate::upk::{
    Package,
    model::{ModelArraySlot, ModelBulkArray, ModelC8Record, ModelObject},
    objects::Span,
};
use std::io;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// Inspection limits, not native-format acceptance rules.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub serialized_bytes: usize,
    pub selected_nodes: usize,
    /// Counts each occurrence, including repeated selected nodes/point indices.
    pub corners: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            serialized_bytes: 32 * 1024 * 1024,
            selected_nodes: 65_536,
            corners: 262_144,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub serialized_bytes: usize,
    pub selected_nodes: usize,
    pub corners: usize,
}

/// A position triple from Model +0xb0; original member spelling is unproved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub index: usize,
    pub span: Span,
    /// Exact IEEE-754 single-precision coordinate bits, including nonfinite bits.
    pub bits: [u32; 3],
}
impl Position {
    pub fn coordinates(&self) -> [f32; 3] {
        self.bits.map(f32::from_bits)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Corner {
    /// Zero-based index in Model's admitted 24-byte +0x80 pool.
    pub pool_index: usize,
    pub pool_span: Span,
    /// All six stored words; only the first word's position-index role is used.
    pub pool_words: [u32; 6],
    pub point_index: i32,
    pub position: Position,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeGeometry {
    pub index: usize,
    pub span: Span,
    /// Entire original record, including fields/flag bits not interpreted here.
    pub raw: [u8; 64],
    /// Native/stored Node +0x10. Unused when corner_count is zero.
    pub vertex_pool_start: i32,
    /// Native/stored Node +0x14, selecting Model's +0xc8 records.
    pub surface_index: i32,
    /// Native/stored Node +0x36; not a u16 value or inferred polygon default.
    pub corner_count: u8,
    /// Previously established native Node.ComponentIndex at +0x1c.
    pub component_index: u16,
    /// Exact 60-byte serialized record (96 bytes native), with its two raw refs
    /// and thirteen raw words. Material evaluation/type checks are not implied.
    pub surface: ModelC8Record,
    pub corners: Vec<Corner>,
}

/// Selected joins and the original complete Model parser result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelNodeGeometry<'a> {
    pub model: ModelObject,
    /// Same export bytes used by the parser; every span is relative to these.
    pub bytes: &'a [u8],
    /// Caller order and duplicate selections are preserved.
    pub nodes: Vec<NodeGeometry>,
    pub usage: Usage,
}
impl<'a> ModelNodeGeometry<'a> {
    /// Parse one exact Model export and bind only explicitly selected zero-based
    /// Node indices. Passing an empty list does not discover or bind any nodes.
    pub fn parse_export(
        package: &Package,
        export_index: usize,
        bytes: &'a [u8],
        selected: &[usize],
        limits: Limits,
    ) -> io::Result<Self> {
        check_limit(bytes.len(), limits.serialized_bytes, "serialized bytes")?;
        check_limit(selected.len(), limits.selected_nodes, "selected nodes")?;
        let model = ModelObject::parse_export(package, export_index, bytes)?;
        let node_array = model
            .arrays
            .iter()
            .find(|a| a.slot == ModelArraySlot::Nodes)
            .copied()
            .ok_or_else(|| invalid("Model Nodes descriptor is missing"))?;
        let pool = bulk(&model.native_fields.arrays, 0x80)?;
        let positions = bulk(&model.arrays, 0xb0)?;
        let mut usage = Usage {
            serialized_bytes: bytes.len(),
            selected_nodes: selected.len(),
            corners: 0,
        };
        let mut nodes = Vec::with_capacity(selected.len());
        for &index in selected {
            let span = node_array.record_span(index)?;
            let raw: [u8; 64] = span
                .bytes(bytes)?
                .try_into()
                .map_err(|_| invalid("Model Node record is not 64 bytes"))?;
            let vertex_pool_start = word(&raw, 0x10) as i32;
            let surface_index = word(&raw, 0x14) as i32;
            let corner_count = raw[0x36];
            let component_index = u16::from_le_bytes(raw[0x1c..0x1e].try_into().unwrap());
            let surface = model
                .native_fields
                .records_c8
                .records
                .get(nonnegative(surface_index)?)
                .copied()
                .ok_or_else(|| invalid("Node surface index is outside Model +0xc8 records"))?;
            usage.corners = usage
                .corners
                .checked_add(usize::from(corner_count))
                .ok_or_else(|| invalid("selected corner count overflow"))?;
            check_limit(usage.corners, limits.corners, "corners")?;
            // The typed native consumer bypasses pool/position dereferences for
            // zero corners. Preserve even a negative unused start in that case.
            let start = if corner_count == 0 {
                0
            } else {
                nonnegative(vertex_pool_start)?
            };
            let end = start
                .checked_add(usize::from(corner_count))
                .ok_or_else(|| invalid("Node vertex-pool range overflow"))?;
            if end > pool.count {
                return Err(invalid("Node corner range exceeds Model vertex pool"));
            }
            let mut corners = Vec::with_capacity(usize::from(corner_count));
            for pool_index in start..end {
                let pool_span = pool.record_span(pool_index)?;
                let record = pool_span.bytes(bytes)?;
                let pool_words = [
                    word(record, 0),
                    word(record, 4),
                    word(record, 8),
                    word(record, 12),
                    word(record, 16),
                    word(record, 20),
                ];
                let point_index = pool_words[0] as i32;
                let index = nonnegative(point_index)?;
                let span = positions.record_span(index)?;
                let point = span.bytes(bytes)?;
                let position = Position {
                    index,
                    span,
                    bits: [word(point, 0), word(point, 4), word(point, 8)],
                };
                corners.push(Corner {
                    pool_index,
                    pool_span,
                    pool_words,
                    point_index,
                    position,
                });
            }
            nodes.push(NodeGeometry {
                index,
                span,
                raw,
                vertex_pool_start,
                surface_index,
                corner_count,
                component_index,
                surface,
                corners,
            });
        }
        Ok(Self {
            model,
            bytes,
            nodes,
            usage,
        })
    }
}

fn word(bytes: &[u8], offset: usize) -> u32 {
    // All callers use records whose complete fixed widths were checked by the
    // existing Model parser and record_span before reaching these offsets.
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn nonnegative(index: i32) -> io::Result<usize> {
    usize::try_from(index).map_err(|_| invalid("negative index in a selected Model geometry join"))
}
fn bulk(arrays: &[ModelBulkArray], native_offset: usize) -> io::Result<ModelBulkArray> {
    arrays
        .iter()
        .find(|a| a.slot == ModelArraySlot::Field { native_offset })
        .copied()
        .ok_or_else(|| invalid("Model geometry bulk descriptor is missing"))
}
fn check_limit(count: usize, limit: usize, resource: &str) -> io::Result<()> {
    if count > limit {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("Model geometry {resource} limit exceeded ({limit})"),
        ))
    } else {
        Ok(())
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
    fn push(b: &mut Vec<u8>, v: u32) {
        b.extend(v.to_le_bytes());
    }
    fn patch(b: &mut [u8], offset: usize, v: i32) {
        b[offset..offset + 4].copy_from_slice(&v.to_le_bytes());
    }
    fn bulk(b: &mut Vec<u8>, stride: u32, records: &[u8]) {
        assert_eq!(records.len() % stride as usize, 0);
        push(b, stride);
        push(b, (records.len() / stride as usize) as u32);
        b.extend(records);
    }
    fn fixture() -> Vec<u8> {
        let mut b = Vec::new();
        for v in [u32::MAX, 0, 0] {
            push(&mut b, v);
        }
        for _ in 0..7 {
            push(&mut b, 0);
        }
        bulk(&mut b, 12, &[0; 12]);
        let mut points = Vec::new();
        for v in [
            0x80000000, 0x7fc00001, 0x7f800000, 0x3f800000, 0x40000000, 0x40400000,
        ] {
            push(&mut points, v);
        }
        bulk(&mut b, 12, &points);
        let mut nodes = [0xa5; 128];
        for (at, start, count) in [(0, 1, 3), (64, -1, 0)] {
            patch(&mut nodes, at + 0x10, start);
            patch(&mut nodes, at + 0x14, 0);
            nodes[at + 0x1c..at + 0x1e].copy_from_slice(&u16::MAX.to_le_bytes());
            nodes[at + 0x36] = count;
            nodes[at + 0x37] = 0xe0;
        }
        bulk(&mut b, 64, &nodes);
        push(&mut b, 1);
        push(&mut b, 1);
        // +0xc8 wire record: two valid generic refs, arbitrary unused raw words.
        push(&mut b, (-2i32) as u32);
        for _ in 0..6 {
            push(&mut b, u32::MAX);
        }
        push(&mut b, 1);
        for _ in 0..7 {
            push(&mut b, 0x7fc00001);
        }
        let mut pool = Vec::new();
        for index in [-99i32, 1, 0, 1, i32::MAX] {
            for value in [index as u32, u32::MAX, 0x7fc00001, 0x80000000, 3, 4] {
                push(&mut pool, value);
            }
        }
        bulk(&mut b, 24, &pool);
        for _ in 0..3 {
            push(&mut b, 0);
        }
        bulk(&mut b, 4, &[]);
        bulk(&mut b, 4, &[]);
        push(&mut b, 0);
        push(&mut b, 0);
        bulk(&mut b, 4, &[]);
        push(&mut b, 0);
        bulk(&mut b, 36, &[]);
        for _ in 0..5 {
            push(&mut b, 0);
        }
        b.extend([0xab, 0xcd]);
        b
    }

    #[test]
    fn preserves_order_repeated_corners_raw_fields_and_nonfinite_position_bits() {
        let b = fixture();
        let p = package(b.len());
        let g = ModelNodeGeometry::parse_export(&p, 0, &b, &[0, 1, 0], Limits::default()).unwrap();
        assert_eq!(
            g.nodes.iter().map(|n| n.index).collect::<Vec<_>>(),
            vec![0, 1, 0]
        );
        assert_eq!(g.usage.corners, 6);
        let n = &g.nodes[0];
        assert_eq!(n.corner_count, 3);
        assert_eq!(n.component_index, u16::MAX);
        assert_eq!(n.raw[0x37], 0xe0);
        assert_eq!(word(&n.raw, 0x18), 0xa5a5a5a5);
        assert_eq!(n.surface.links[0].reference, -2);
        assert_eq!(n.surface.links[1].reference, 1);
        assert_eq!(n.surface.words[0], u32::MAX);
        assert_eq!(
            n.corners.iter().map(|c| c.point_index).collect::<Vec<_>>(),
            vec![1, 0, 1]
        );
        assert_eq!(
            n.corners[1].position.bits,
            [0x80000000, 0x7fc00001, 0x7f800000]
        );
        let coordinates = n.corners[1].position.coordinates();
        assert_eq!(coordinates[0].to_bits(), 0x80000000);
        assert!(coordinates[1].is_nan());
        assert!(coordinates[2].is_infinite());
        for c in &n.corners {
            assert_eq!(c.pool_words[1..], [u32::MAX, 0x7fc00001, 0x80000000, 3, 4]);
            assert_eq!(
                word(c.pool_span.bytes(&b).unwrap(), 0) as i32,
                c.point_index
            );
            assert_eq!(
                c.position.span.bytes(&b).unwrap(),
                c.position
                    .bits
                    .into_iter()
                    .flat_map(u32::to_le_bytes)
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(
            g.model.unparsed_native_tail.bytes(g.bytes).unwrap(),
            &[0xab, 0xcd]
        );
        assert_eq!(g.nodes[1].vertex_pool_start, -1);
        assert!(g.nodes[1].corners.is_empty());
    }

    #[test]
    fn checks_selected_node_surface_pool_range_and_point_indices() {
        let original = fixture();
        let p = package(original.len());
        let m = ModelObject::parse_export(&p, 0, &original).unwrap();
        let node = m.arrays[2].record_span(0).unwrap().offset;
        let pool = m.native_fields.arrays[0].record_span(1).unwrap().offset;
        for (offset, value) in [
            (node + 0x10, -1),
            (node + 0x10, 4),
            (node + 0x10, i32::MAX),
            (node + 0x14, -1),
            (node + 0x14, 1),
            (pool, -1),
            (pool, 2),
        ] {
            let mut b = original.clone();
            patch(&mut b, offset, value);
            assert_eq!(
                ModelNodeGeometry::parse_export(&p, 0, &b, &[0], Limits::default())
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidData
            );
            // No automatic discovery or validation of an unselected node's joins.
            assert!(ModelNodeGeometry::parse_export(&p, 0, &b, &[1], Limits::default()).is_ok());
        }
        assert_eq!(
            ModelNodeGeometry::parse_export(&p, 0, &original, &[2], Limits::default())
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn zero_corner_skips_unused_start_but_still_checks_the_selected_surface() {
        let mut b = fixture();
        let p = package(b.len());
        let m = ModelObject::parse_export(&p, 0, &b).unwrap();
        let node = m.arrays[2].record_span(1).unwrap().offset;
        let g = ModelNodeGeometry::parse_export(&p, 0, &b, &[1], Limits::default()).unwrap();
        assert_eq!(g.nodes[0].vertex_pool_start, -1);
        assert_eq!(g.usage.corners, 0);
        patch(&mut b, node + 0x14, -1);
        assert_eq!(
            ModelNodeGeometry::parse_export(&p, 0, &b, &[1], Limits::default())
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        let empty = ModelNodeGeometry::parse_export(&p, 0, &b, &[], Limits::default()).unwrap();
        assert!(empty.nodes.is_empty());
    }

    #[test]
    fn bounds_work_and_retains_existing_class_profile_stride_admission() {
        let b = fixture();
        let p = package(b.len());
        let exact = Limits {
            serialized_bytes: b.len(),
            selected_nodes: 3,
            corners: 6,
        };
        assert!(ModelNodeGeometry::parse_export(&p, 0, &b, &[0, 1, 0], exact).is_ok());
        for which in 0..3 {
            let mut limits = exact;
            match which {
                0 => limits.serialized_bytes -= 1,
                1 => limits.selected_nodes -= 1,
                _ => limits.corners -= 1,
            }
            assert_eq!(
                ModelNodeGeometry::parse_export(&p, 0, &b, &[0, 1, 0], limits)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::Unsupported
            );
        }
        let mut wrong = p.clone();
        wrong.summary.file_version = 867;
        assert_eq!(
            ModelNodeGeometry::parse_export(&wrong, 0, &b, &[], exact)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        let mut wrong = p.clone();
        wrong.names[5].value = "World".into();
        assert_eq!(
            ModelNodeGeometry::parse_export(&wrong, 0, &b, &[], exact)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
        let m = ModelObject::parse_export(&p, 0, &b).unwrap();
        let mut wrong = b.clone();
        patch(&mut wrong, m.native_fields.arrays[0].stride_span.offset, 16);
        assert_eq!(
            ModelNodeGeometry::parse_export(&p, 0, &wrong, &[], exact)
                .unwrap_err()
                .kind(),
            io::ErrorKind::Unsupported
        );
    }
}
