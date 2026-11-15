//! Bounded integer phase of the owned ModelComponent index writer.
//!
//! Resolved identities and the map/buffer snapshot are explicit caller inputs.
//! This neither invokes the native lifecycle nor evaluates materials or floats.
//! Exact branch evidence and restrictions: docs/MODEL_INDEX_RESOURCE.md.

use crate::upk::{
    Package,
    model::{ModelArraySlot, ModelBulkArray, ModelC8Record},
    model_component::{ModelComponentLink, ModelComponentObject},
    model_node_geometry::{Corner, Position},
    model_render_resource::{self, ModelRenderResource},
    objects::Span,
};
use std::{collections::HashMap, io};

/// Caller-owned equality token, never dereferenced or interpreted as an address.
/// Different serialized references may resolve to the same token, including null.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObjectKey {
    Null,
    Resolved(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceKey {
    pub reference: i32,
    pub key: ObjectKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportInput<'a> {
    /// Zero-based export index in the single supplied Package namespace.
    pub index: usize,
    pub bytes: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitialBuffer<'a> {
    pub indices: &'a [u32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapEntry {
    pub key: ObjectKey,
    /// Zero-based logical buffer ordinal. Distinct keys may alias one buffer.
    pub buffer: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotPhase {
    BeforeElementCallbacks,
    /// Caller asserts a stable snapshot after bf5a30's first callback loop.
    /// This assertion does not establish that the native path ran or will run.
    AfterElementCallbacks,
}

#[derive(Clone, Copy, Debug)]
pub struct Request<'a> {
    pub model: ExportInput<'a>,
    /// Ordered writer invocations; repeated component selections append again.
    pub components: &'a [ExportInput<'a>],
    pub phase: SnapshotPhase,
    pub references: &'a [ReferenceKey],
    pub initial_buffers: &'a [InitialBuffer<'a>],
    pub initial_map: &'a [MapEntry],
}

/// Loader policy, not native-format or native-allocation acceptance rules.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub serialized_bytes: usize,
    pub components: usize,
    pub elements: usize,
    pub node_visits: usize,
    pub source_corners: usize,
    pub index_values: usize,
    pub buffers: usize,
    pub reference_keys: usize,
    pub map_entries: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            serialized_bytes: 32 * 1024 * 1024,
            components: 4096,
            elements: 65_536,
            node_visits: 262_144,
            source_corners: 262_144,
            index_values: 1_048_576,
            buffers: 65_536,
            reference_keys: 65_536,
            map_entries: 65_536,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub serialized_bytes: usize,
    pub components: usize,
    pub elements: usize,
    pub node_visits: usize,
    /// Once per accepted node visit, shared by that visit's accepted passes.
    pub source_corners: usize,
    /// Sum across unique buffers, including preserved initial contents.
    pub index_values: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexBuffer {
    pub initial_count: usize,
    pub indices: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentInput<'a> {
    pub export: ExportInput<'a>,
    pub object: ModelComponentObject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PassResult {
    pub pass: u8,
    /// The exact Node byte selected at +0x34+(1-pass).
    pub selector: u8,
    pub accepted: bool,
    pub first_index: usize,
    pub index_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeVisit {
    pub index: usize,
    pub span: Span,
    pub raw: [u8; 64],
    pub surface_index: i32,
    pub surface: ModelC8Record,
    pub base: u32,
    pub count: u8,
    pub skipped_bit_26: bool,
    /// Empty for skipped or wholly rejected visits. Raw source positions are
    /// retained without regenerating packed vertices or floating bounds.
    pub corners: Vec<Corner>,
    pub passes: Vec<PassResult>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementResult {
    pub component_ordinal: usize,
    pub component_export: usize,
    pub element_ordinal: usize,
    pub span: Span,
    pub key_reference: ModelComponentLink,
    pub key: ObjectKey,
    pub buffer: usize,
    /// Native +0x50, +0x54, +0x58 and +0x5c integer results, respectively.
    pub first_index: usize,
    pub triangle_count: u32,
    pub min_vertex: u32,
    pub max_vertex: u32,
    pub nodes: Vec<NodeVisit>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelIndexResource<'a> {
    pub model_export: usize,
    pub resource: ModelRenderResource<'a>,
    pub phase: SnapshotPhase,
    pub references: Vec<ReferenceKey>,
    pub components: Vec<ComponentInput<'a>>,
    /// Initial order followed by first encounter order for new buffers/keys.
    /// These ordinals do not represent native sparse slots or hash iteration.
    pub buffers: Vec<IndexBuffer>,
    /// Prefix length in `map` belonging to the supplied initial snapshot.
    pub initial_map_entries: usize,
    pub map: Vec<MapEntry>,
    pub elements: Vec<ElementResult>,
    pub usage: Usage,
}

impl<'a> ModelIndexResource<'a> {
    pub fn construct(package: &Package, request: Request<'a>, limits: Limits) -> io::Result<Self> {
        if request.phase != SnapshotPhase::AfterElementCallbacks {
            return Err(unsupported("index snapshot must follow element callbacks"));
        }
        check(request.components.len(), limits.components, "components")?;
        check(
            request.references.len(),
            limits.reference_keys,
            "reference keys",
        )?;
        check(request.initial_map.len(), limits.map_entries, "map entries")?;
        check(request.initial_buffers.len(), limits.buffers, "buffers")?;
        let mut usage = Usage {
            serialized_bytes: request.model.bytes.len(),
            components: request.components.len(),
            ..Usage::default()
        };
        for component in request.components {
            add(
                &mut usage.serialized_bytes,
                component.bytes.len(),
                limits.serialized_bytes,
                "serialized bytes",
            )?;
        }
        check(
            usage.serialized_bytes,
            limits.serialized_bytes,
            "serialized bytes",
        )?;
        let resource = ModelRenderResource::parse_export(
            package,
            request.model.index,
            request.model.bytes,
            model_render_resource::Limits {
                serialized_bytes: limits.serialized_bytes,
                ..Default::default()
            },
        )?;
        let model_reference = request
            .model
            .index
            .checked_add(1)
            .and_then(|n| i32::try_from(n).ok())
            .ok_or_else(|| invalid("Model export cannot be represented by a signed reference"))?;
        let mut reference_keys = HashMap::new();
        for binding in request.references {
            validate_reference(package, binding.reference)?;
            if binding.reference == 0 && binding.key != ObjectKey::Null {
                return Err(invalid("null serialized reference requires the null key"));
            }
            if reference_keys
                .insert(binding.reference, binding.key)
                .is_some()
            {
                return Err(invalid("duplicate serialized reference binding"));
            }
        }
        let mut buffers = Vec::with_capacity(request.initial_buffers.len());
        for buffer in request.initial_buffers {
            native_count(buffer.indices.len())?;
            add(
                &mut usage.index_values,
                buffer.indices.len(),
                limits.index_values,
                "index values",
            )?;
            for &index in buffer.indices {
                validate_index(index, resource.records.count)?;
            }
            buffers.push(IndexBuffer {
                initial_count: buffer.indices.len(),
                indices: buffer.indices.to_vec(),
            });
        }
        let mut key_buffers = HashMap::new();
        for entry in request.initial_map {
            if entry.buffer >= buffers.len() {
                return Err(invalid("initial map buffer is out of range"));
            }
            if key_buffers.insert(entry.key, entry.buffer).is_some() {
                return Err(invalid("duplicate initial map key"));
            }
        }
        let mut map = request.initial_map.to_vec();
        let mut components = Vec::with_capacity(request.components.len());
        for &export in request.components {
            let object = ModelComponentObject::parse_export(package, export.index, export.bytes)?;
            if object.model.reference != model_reference {
                return Err(invalid("component does not link the selected local Model"));
            }
            add(
                &mut usage.elements,
                object.elements.len(),
                limits.elements,
                "elements",
            )?;
            for element in &object.elements {
                add(
                    &mut usage.node_visits,
                    element.nodes.values.len(),
                    limits.node_visits,
                    "node visits",
                )?;
            }
            components.push(ComponentInput { export, object });
        }
        let nodes = resource
            .model
            .arrays
            .iter()
            .find(|a| a.slot == ModelArraySlot::Nodes)
            .copied()
            .ok_or_else(|| invalid("Model Nodes descriptor is missing"))?;
        let pool = field(&resource.model.native_fields.arrays, 0x80)?;
        let positions = field(&resource.model.arrays, 0xb0)?;
        let mut elements = Vec::with_capacity(usage.elements);
        for (component_ordinal, component) in components.iter().enumerate() {
            for (element_ordinal, element) in component.object.elements.iter().enumerate() {
                let key_reference = element.references[1];
                let key = *reference_keys
                    .get(&key_reference.reference)
                    .ok_or_else(|| invalid("element key lacks an explicit resolved identity"))?;
                let buffer = if let Some(&buffer) = key_buffers.get(&key) {
                    buffer
                } else {
                    check(
                        buffers
                            .len()
                            .checked_add(1)
                            .ok_or_else(|| invalid("buffer count overflow"))?,
                        limits.buffers,
                        "buffers",
                    )?;
                    check(
                        map.len()
                            .checked_add(1)
                            .ok_or_else(|| invalid("map count overflow"))?,
                        limits.map_entries,
                        "map entries",
                    )?;
                    let buffer = buffers.len();
                    buffers.push(IndexBuffer {
                        initial_count: 0,
                        indices: Vec::new(),
                    });
                    key_buffers.insert(key, buffer);
                    map.push(MapEntry { key, buffer });
                    buffer
                };
                let output = &mut buffers[buffer].indices;
                let mut result = ElementResult {
                    component_ordinal,
                    component_export: component.export.index,
                    element_ordinal,
                    span: element.span,
                    key_reference,
                    key,
                    buffer,
                    first_index: output.len(),
                    triangle_count: 0,
                    min_vertex: u32::MAX,
                    max_vertex: 0,
                    nodes: Vec::with_capacity(element.nodes.values.len()),
                };
                for &node in &element.nodes.values {
                    let index = usize::from(node);
                    let span = nodes.record_span(index)?;
                    let raw: [u8; 64] = span
                        .bytes(resource.bytes)?
                        .try_into()
                        .map_err(|_| invalid("Node record is not 64 bytes"))?;
                    let surface_index = word(&raw, 0x14) as i32;
                    let surface = *resource
                        .model
                        .native_fields
                        .records_c8
                        .records
                        .get(nonnegative(surface_index)?)
                        .ok_or_else(|| invalid("Node surface is outside Model +0xc8 records"))?;
                    let flags = surface.words[0];
                    let base = word(&raw, 0x18);
                    let count = raw[0x36];
                    let mut visit = NodeVisit {
                        index,
                        span,
                        raw,
                        surface_index,
                        surface,
                        base,
                        count,
                        skipped_bit_26: flags & (1 << 26) != 0,
                        corners: Vec::new(),
                        passes: Vec::new(),
                    };
                    if !visit.skipped_bit_26 {
                        let mut source_checked = false;
                        for pass in 0..=u8::from(flags & 0x100 != 0) {
                            let selector = raw[0x34 + usize::from(1 - pass)];
                            let accepted = component.object.word_240.bits == u32::MAX
                                || component.object.word_240.bits == u32::from(selector);
                            let first_index = output.len();
                            if accepted {
                                if !source_checked {
                                    add(
                                        &mut usage.source_corners,
                                        usize::from(count),
                                        limits.source_corners,
                                        "source corners",
                                    )?;
                                    visit.corners = source_corners(
                                        resource.bytes,
                                        pool,
                                        positions,
                                        word(&raw, 0x10) as i32,
                                        count,
                                    )?;
                                    source_checked = true;
                                }
                                if count >= 3 {
                                    let added = usize::from(count - 2) * 3;
                                    let end = output
                                        .len()
                                        .checked_add(added)
                                        .ok_or_else(|| invalid("index count overflow"))?;
                                    native_count(end)?;
                                    add(
                                        &mut usage.index_values,
                                        added,
                                        limits.index_values,
                                        "index values",
                                    )?;
                                    let first = base
                                        .checked_add(u32::from(pass) * u32::from(count))
                                        .ok_or_else(|| invalid("packed vertex base overflow"))?;
                                    let last = first.checked_add(u32::from(count - 1)).ok_or_else(
                                        || invalid("packed vertex endpoint overflow"),
                                    )?;
                                    validate_index(first, resource.records.count)?;
                                    validate_index(last, resource.records.count)?;
                                    for d in 2..u32::from(count) {
                                        output.extend([first, first + d, first + d - 1]);
                                        result.triangle_count =
                                            result.triangle_count.checked_add(1).ok_or_else(
                                                || invalid("element triangle count overflow"),
                                            )?;
                                        result.min_vertex = result.min_vertex.min(first);
                                        result.max_vertex = result.max_vertex.max(first + d);
                                    }
                                }
                            }
                            visit.passes.push(PassResult {
                                pass,
                                selector,
                                accepted,
                                first_index,
                                index_count: output.len() - first_index,
                            });
                        }
                    }
                    result.nodes.push(visit);
                }
                elements.push(result);
            }
        }
        Ok(Self {
            model_export: request.model.index,
            resource,
            phase: request.phase,
            references: request.references.to_vec(),
            components,
            buffers,
            initial_map_entries: request.initial_map.len(),
            map,
            elements,
            usage,
        })
    }
}

fn source_corners(
    bytes: &[u8],
    pool: ModelBulkArray,
    positions: ModelBulkArray,
    start: i32,
    count: u8,
) -> io::Result<Vec<Corner>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    let start = nonnegative(start)?;
    let end = start
        .checked_add(usize::from(count))
        .ok_or_else(|| invalid("source pool range overflow"))?;
    if end > pool.count {
        return Err(invalid("source corners exceed Model pool"));
    }
    let mut corners = Vec::with_capacity(usize::from(count));
    for pool_index in start..end {
        let pool_span = pool.record_span(pool_index)?;
        let bytes_pool = pool_span.bytes(bytes)?;
        let pool_words = std::array::from_fn(|i| word(bytes_pool, i * 4));
        let point_index = pool_words[0] as i32;
        let index = nonnegative(point_index)?;
        let span = positions.record_span(index)?;
        let bytes_point = span.bytes(bytes)?;
        corners.push(Corner {
            pool_index,
            pool_span,
            pool_words,
            point_index,
            position: Position {
                index,
                span,
                bits: std::array::from_fn(|i| word(bytes_point, i * 4)),
            },
        });
    }
    Ok(corners)
}
fn field(arrays: &[ModelBulkArray], offset: usize) -> io::Result<ModelBulkArray> {
    arrays
        .iter()
        .find(|a| {
            a.slot
                == ModelArraySlot::Field {
                    native_offset: offset,
                }
        })
        .copied()
        .ok_or_else(|| invalid("required Model bulk descriptor is missing"))
}
fn word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn nonnegative(value: i32) -> io::Result<usize> {
    usize::try_from(value).map_err(|_| invalid("negative Model source index"))
}
fn validate_reference(package: &Package, reference: i32) -> io::Result<()> {
    let valid = if reference > 0 {
        reference as usize <= package.exports.len()
    } else if reference < 0 {
        reference.unsigned_abs() as usize <= package.imports.len()
    } else {
        true
    };
    if !valid {
        return Err(invalid(
            "resolved binding reference is outside Package tables",
        ));
    }
    Ok(())
}
fn validate_index(index: u32, records: usize) -> io::Result<()> {
    if usize::try_from(index).map_or(true, |i| i >= records) {
        return Err(invalid(
            "index is outside the selected packed Model resource",
        ));
    }
    Ok(())
}
fn native_count(value: usize) -> io::Result<()> {
    if value > i32::MAX as usize {
        return Err(invalid("buffer count exceeds signed native domain"));
    }
    Ok(())
}
fn add(value: &mut usize, extra: usize, limit: usize, label: &str) -> io::Result<()> {
    *value = value
        .checked_add(extra)
        .ok_or_else(|| invalid("inspection count overflow"))?;
    check(*value, limit, label)
}
fn check(value: usize, limit: usize, label: &str) -> io::Result<()> {
    if value > limit {
        return Err(unsupported(&format!(
            "index resource {label} inspection limit exceeded ({limit})"
        )));
    }
    Ok(())
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn unsupported(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upk::{Export, Import, NameEntry, NameRef, Summary, TableLocation};
    use std::io::ErrorKind;

    fn name(index: usize) -> NameRef {
        NameRef { index, number: 0 }
    }
    fn push(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend(value.to_le_bytes());
    }
    fn patch(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn bulk(bytes: &mut Vec<u8>, stride: usize, records: &[u8]) {
        assert_eq!(records.len() % stride, 0);
        push(bytes, stride as u32);
        push(bytes, (records.len() / stride) as u32);
        bytes.extend(records);
    }
    fn component(selector: u32, elements: &[(i32, &[u16])]) -> Vec<u8> {
        let mut out = Vec::new();
        // Opaque component prefix, NetIndex, None tag, local Model ref, selector.
        for value in [0, u32::MAX, 0, 0, 1, selector, elements.len() as u32] {
            push(&mut out, value);
        }
        for &(key, nodes) in elements {
            for value in [0, 0, key as u32, nodes.len() as u32] {
                push(&mut out, value);
            }
            for node in nodes {
                out.extend(node.to_le_bytes());
            }
            push(&mut out, 0); // Native +20 reference array.
            push(&mut out, 0); // Native +38 record array.
        }
        out.extend(0u16.to_le_bytes());
        push(&mut out, 0); // Separate component node list, not a partition rule.
        out
    }

    #[derive(Clone)]
    struct Fixture {
        package: Package,
        model: Vec<u8>,
        components: Vec<Vec<u8>>,
        nodes_at: usize,
        pool_at: usize,
    }
    impl Fixture {
        fn new(components: Vec<Vec<u8>>) -> Self {
            let mut model = Vec::new();
            for value in [u32::MAX, 0, 0] {
                push(&mut model, value);
            }
            for _ in 0..7 {
                push(&mut model, 0);
            } // Native +c5c record.
            bulk(&mut model, 12, &[0; 12]); // +98 raw vector record.
            let mut points = Vec::new();
            for point in [[0f32, 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]] {
                for value in point {
                    push(&mut points, value.to_bits());
                }
            }
            bulk(&mut model, 12, &points); // +b0 positions.
            // N, packed base, pool start, surface flags. Node4's bad source
            // is deliberate: bit26 skips source traversal before any access.
            let specs = [
                (3u8, 0u32, 0i32, 0u32),
                (4, 3, 0, 0x100),
                (0, 11, -1, 0),
                (2, 11, 0, 0),
                (3, 13, i32::MAX, 1 << 26),
            ];
            let mut nodes = vec![0; specs.len() * 64];
            for (i, &(count, base, start, _)) in specs.iter().enumerate() {
                patch(&mut nodes, i * 64 + 0x10, start as u32);
                patch(&mut nodes, i * 64 + 0x14, i as u32);
                patch(&mut nodes, i * 64 + 0x18, base);
                nodes[i * 64 + 0x34] = 8;
                nodes[i * 64 + 0x35] = 7;
                nodes[i * 64 + 0x36] = count;
            }
            let nodes_at = model.len() + 8;
            bulk(&mut model, 64, &nodes);
            push(&mut model, 0); // +d8 reference.
            push(&mut model, specs.len() as u32);
            for &(_, _, _, flags) in &specs {
                push(&mut model, 0); // +c8 record's first reference.
                for value in [flags, 0, 0, 0, 0, 0] {
                    push(&mut model, value);
                }
                push(&mut model, 0); // Second reference.
                for _ in 0..7 {
                    push(&mut model, 0);
                }
            }
            let mut pool = Vec::new();
            for point in 0..4 {
                for value in [point, 0, 0, 0, 0, 0] {
                    push(&mut pool, value);
                }
            }
            let pool_at = model.len() + 8;
            bulk(&mut model, 24, &pool);
            for _ in 0..3 {
                push(&mut model, 0);
            } // +554 word, +55c count, Polys reference.
            bulk(&mut model, 4, &[]);
            bulk(&mut model, 4, &[]);
            push(&mut model, 0);
            push(&mut model, 0);
            bulk(&mut model, 4, &[]);
            push(&mut model, 0);
            bulk(&mut model, 36, &[0; 16 * 36]); // Packed records, count16.
            for _ in 0..5 {
                push(&mut model, 0);
            } // Final four words/+110 count.

            let table = TableLocation {
                count: 0,
                offset: 0,
            };
            let mut package = Package {
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
                    "None",
                    "Core",
                    "Class",
                    "Package",
                    "Engine",
                    "Model",
                    "Level",
                    "ModelComponent",
                    "Material",
                    "MaterialA",
                    "MaterialB",
                    "ModelInstance",
                    "LevelInstance",
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
                    Import {
                        class_package: name(1),
                        class_name: name(2),
                        outer_index: -2,
                        object_name: name(6),
                    },
                    Import {
                        class_package: name(1),
                        class_name: name(2),
                        outer_index: -2,
                        object_name: name(7),
                    },
                    Import {
                        class_package: name(4),
                        class_name: name(8),
                        outer_index: -2,
                        object_name: name(9),
                    },
                    Import {
                        class_package: name(4),
                        class_name: name(8),
                        outer_index: -2,
                        object_name: name(10),
                    },
                ],
                exports: Vec::new(),
            };
            let export = |class_index, outer_index, object_name, serial_size| Export {
                class_index,
                super_index: 0,
                outer_index,
                object_name: name(object_name),
                archetype_index: 0,
                object_flags: 0,
                serial_size,
                serial_offset: 4096,
                export_flags: 0,
                generation_net_object_counts: Vec::new(),
                guid: [0; 16],
                package_flags: 0,
            };
            package.exports.push(export(-1, 2, 11, model.len()));
            package.exports.push(export(-3, 0, 12, 0));
            for (i, bytes) in components.iter().enumerate() {
                let object_name = package.names.len();
                package.names.push(NameEntry {
                    value: format!("Component{i}"),
                    flags: 0,
                });
                package
                    .exports
                    .push(export(-4, 2, object_name, bytes.len()));
            }
            Self {
                package,
                model,
                components,
                nodes_at,
                pool_at,
            }
        }
        fn selected(&self, ordinals: &[usize]) -> Vec<ExportInput<'_>> {
            ordinals
                .iter()
                .map(|&i| ExportInput {
                    index: i + 2,
                    bytes: &self.components[i],
                })
                .collect()
        }
        fn request<'a>(
            &'a self,
            components: &'a [ExportInput<'a>],
            references: &'a [ReferenceKey],
            initial_buffers: &'a [InitialBuffer<'a>],
            initial_map: &'a [MapEntry],
        ) -> Request<'a> {
            Request {
                model: ExportInput {
                    index: 0,
                    bytes: &self.model,
                },
                components,
                phase: SnapshotPhase::AfterElementCallbacks,
                references,
                initial_buffers,
                initial_map,
            }
        }
        fn node_word(&mut self, node: usize, field: usize, value: u32) {
            patch(&mut self.model, self.nodes_at + node * 64 + field, value);
        }
    }
    fn key(reference: i32, identity: u64) -> ReferenceKey {
        ReferenceKey {
            reference,
            key: ObjectKey::Resolved(identity),
        }
    }
    fn ordinary_error(f: &Fixture, limits: Limits) -> ErrorKind {
        let components = f.selected(&[0]);
        let references = [key(-5, 10)];
        ModelIndexResource::construct(
            &f.package,
            f.request(&components, &references, &[], &[]),
            limits,
        )
        .unwrap_err()
        .kind()
    }

    #[test]
    fn preserves_seed_aliases_and_repeated_component_and_node_visits() {
        let f = Fixture::new(vec![
            component(u32::MAX, &[(-5, &[0, 0]), (-6, &[1])]),
            component(7, &[(-5, &[1])]),
        ]);
        let components = f.selected(&[0, 1, 0]);
        let references = [key(-5, 10), key(-6, 20)];
        let initial = [
            InitialBuffer { indices: &[2, 1] },
            InitialBuffer { indices: &[0] },
        ];
        let map = [
            MapEntry {
                key: ObjectKey::Resolved(10),
                buffer: 0,
            },
            MapEntry {
                key: ObjectKey::Resolved(20),
                buffer: 0,
            },
        ];
        let got = ModelIndexResource::construct(
            &f.package,
            f.request(&components, &references, &initial, &map),
            Limits::default(),
        )
        .unwrap();
        let repeated = [0, 2, 1, 0, 2, 1];
        let front = [3, 5, 4, 3, 6, 5];
        let both = [3, 5, 4, 3, 6, 5, 7, 9, 8, 7, 10, 9];
        let expected = [
            vec![2, 1],
            repeated.to_vec(),
            both.to_vec(),
            front.to_vec(),
            repeated.to_vec(),
            both.to_vec(),
        ]
        .concat();
        assert_eq!(got.buffers.len(), 2);
        assert_eq!(got.buffers[0].indices, expected);
        assert_eq!(got.buffers[0].initial_count, 2);
        assert_eq!(got.buffers[1].indices, [0]); // Unmapped initial buffer is retained.
        assert_eq!(got.map, map);
        assert_eq!(
            got.elements
                .iter()
                .map(|e| (
                    e.buffer,
                    e.first_index,
                    e.triangle_count,
                    e.min_vertex,
                    e.max_vertex
                ))
                .collect::<Vec<_>>(),
            [
                (0, 2, 2, 0, 2),
                (0, 8, 4, 3, 10),
                (0, 20, 2, 3, 6),
                (0, 26, 2, 0, 2),
                (0, 32, 4, 3, 10)
            ]
        );
        assert_eq!(
            got.elements[0]
                .nodes
                .iter()
                .map(|n| n.index)
                .collect::<Vec<_>>(),
            [0, 0]
        );
        assert_eq!(got.usage.index_values, 45); // Each logical buffer counted once.
    }

    #[test]
    fn gates_source_access_and_retains_two_pass_and_small_count_results() {
        let all = [0, 1, 2, 3, 4];
        let f = Fixture::new(vec![
            component(u32::MAX, &[(-5, &all), (0, &[2, 3])]),
            component(7, &[(-5, &all)]),
        ]);
        let components = f.selected(&[0, 1]);
        let references = [
            key(-5, 10),
            ReferenceKey {
                reference: 0,
                key: ObjectKey::Null,
            },
        ];
        let got = ModelIndexResource::construct(
            &f.package,
            f.request(&components, &references, &[], &[]),
            Limits::default(),
        )
        .unwrap();
        assert_eq!(
            got.buffers[0].indices,
            [
                0, 2, 1, 3, 5, 4, 3, 6, 5, 7, 9, 8, 7, 10, 9, 0, 2, 1, 3, 5, 4, 3, 6, 5,
            ]
        );
        assert!(got.buffers[1].indices.is_empty());
        assert_eq!(
            got.map,
            [
                MapEntry {
                    key: ObjectKey::Resolved(10),
                    buffer: 0
                },
                MapEntry {
                    key: ObjectKey::Null,
                    buffer: 1
                }
            ]
        );
        assert_eq!(
            got.elements
                .iter()
                .map(|e| (e.first_index, e.triangle_count, e.min_vertex, e.max_vertex))
                .collect::<Vec<_>>(),
            [(0, 5, 0, 10), (0, 0, u32::MAX, 0), (15, 3, 0, 6)]
        );
        assert_eq!(
            got.elements[0].nodes[1]
                .passes
                .iter()
                .map(|p| (p.pass, p.accepted, p.index_count))
                .collect::<Vec<_>>(),
            [(0, true, 6), (1, true, 6)]
        );
        assert!(got.elements[0].nodes[2].corners.is_empty()); // N0 permits start -1.
        assert_eq!(got.elements[0].nodes[3].corners.len(), 2); // N2 still reads source.
        assert!(got.elements[0].nodes[4].skipped_bit_26);
        assert!(got.elements[0].nodes[4].corners.is_empty()); // Bad pool start not read.
        assert_eq!(
            got.elements[2].nodes[1]
                .passes
                .iter()
                .map(|p| p.accepted)
                .collect::<Vec<_>>(),
            [true, false]
        );

        let mut rejected = Fixture::new(vec![component(99, &[(-5, &[0, 1])])]);
        rejected.node_word(0, 0x10, u32::MAX);
        rejected.node_word(1, 0x10, u32::MAX);
        let components = rejected.selected(&[0]);
        let got = ModelIndexResource::construct(
            &rejected.package,
            rejected.request(&components, &references, &[], &[]),
            Limits::default(),
        )
        .unwrap();
        assert!(got.buffers[0].indices.is_empty());
        assert_eq!(got.usage.source_corners, 0); // Wholly rejected selectors read no pool.

        for count in 0..=2 {
            let mut small = Fixture::new(vec![component(7, &[(-5, &[0])])]);
            small.model[small.nodes_at + 0x36] = count;
            small.node_word(0, 0x18, u32::MAX); // Unused base must not be range checked.
            if count == 0 {
                small.node_word(0, 0x10, u32::MAX);
            }
            let components = small.selected(&[0]);
            let got = ModelIndexResource::construct(
                &small.package,
                small.request(&components, &references, &[], &[]),
                Limits::default(),
            )
            .unwrap();
            assert!(got.buffers[0].indices.is_empty());
            assert_eq!(
                (
                    got.elements[0].triangle_count,
                    got.elements[0].min_vertex,
                    got.elements[0].max_vertex
                ),
                (0, u32::MAX, 0)
            );
            assert_eq!(got.elements[0].nodes[0].corners.len(), count as usize);
        }
    }

    #[test]
    fn rejects_unasserted_phase_and_inconsistent_bindings_or_initial_map() {
        let f = Fixture::new(vec![component(7, &[(-5, &[0])])]);
        let components = f.selected(&[0]);
        let references = [key(-5, 10)];
        let initial = [InitialBuffer { indices: &[2] }];
        let map = [MapEntry {
            key: ObjectKey::Resolved(10),
            buffer: 0,
        }];
        let request = f.request(&components, &references, &initial, &map);
        let duplicate_refs = [key(-5, 10), key(-5, 20)];
        let duplicate_map = [map[0], map[0]];
        let missing_buffer = [MapEntry {
            key: ObjectKey::Resolved(10),
            buffer: 1,
        }];
        let wrong_null = [key(-5, 10), key(0, 20)];
        let bad_reference = [key(-5, 10), key(-7, 20)];
        let bad_initial = [InitialBuffer { indices: &[16] }];
        for bad in [
            Request {
                references: &[],
                ..request
            },
            Request {
                references: &duplicate_refs,
                ..request
            },
            Request {
                references: &wrong_null,
                ..request
            },
            Request {
                references: &bad_reference,
                ..request
            },
            Request {
                initial_buffers: &bad_initial,
                ..request
            },
            Request {
                initial_map: &duplicate_map,
                ..request
            },
            Request {
                initial_map: &missing_buffer,
                ..request
            },
        ] {
            assert_eq!(
                ModelIndexResource::construct(&f.package, bad, Limits::default())
                    .unwrap_err()
                    .kind(),
                ErrorKind::InvalidData
            );
        }
        assert_eq!(
            ModelIndexResource::construct(
                &f.package,
                Request {
                    phase: SnapshotPhase::BeforeElementCallbacks,
                    ..request
                },
                Limits::default()
            )
            .unwrap_err()
            .kind(),
            ErrorKind::Unsupported
        );
        let mut wrong_model = f.clone();
        patch(&mut wrong_model.components[0], 16, 2); // Valid reference, wrong selected object.
        assert_eq!(
            ordinary_error(&wrong_model, Limits::default()),
            ErrorKind::InvalidData
        );
        let mut wrong_class = f.clone();
        wrong_class.package.exports[2].class_index = -3; // Level, not ModelComponent.
        assert_eq!(
            ordinary_error(&wrong_class, Limits::default()),
            ErrorKind::Unsupported
        );
    }

    #[test]
    fn bounds_selected_sources_packed_indices_and_aggregate_inspection_budgets() {
        let original = Fixture::new(vec![component(7, &[(-5, &[0])])]);
        for (field, bits) in [(0x10, 4), (0x14, 5), (0x18, 15), (0x18, u32::MAX)] {
            let mut f = original.clone();
            f.node_word(0, field, bits);
            assert_eq!(
                ordinary_error(&f, Limits::default()),
                ErrorKind::InvalidData
            );
        }
        let mut point = original.clone();
        let at = point.pool_at;
        patch(&mut point.model, at, 4); // Point equal to position count is outside.
        assert_eq!(
            ordinary_error(&point, Limits::default()),
            ErrorKind::InvalidData
        );
        let mut node = original.clone();
        node.components[0][44..46].copy_from_slice(&5u16.to_le_bytes());
        assert_eq!(
            ordinary_error(&node, Limits::default()),
            ErrorKind::InvalidData
        );
        let mut small = Fixture::new(vec![component(7, &[(-5, &[3])])]);
        small.node_word(3, 0x10, 3); // Accepted N2 must still have two valid pool records.
        assert_eq!(
            ordinary_error(&small, Limits::default()),
            ErrorKind::InvalidData
        );

        let f = Fixture::new(vec![component(u32::MAX, &[(-5, &[1])])]);
        let components = f.selected(&[0]);
        let references = [key(-5, 10)];
        let request = f.request(&components, &references, &[], &[]);
        let exact = Limits {
            source_corners: 4,
            index_values: 12,
            ..Limits::default()
        };
        let got = ModelIndexResource::construct(&f.package, request, exact).unwrap();
        assert_eq!(got.usage.source_corners, 4); // One source list, two accepted passes.
        assert_eq!(got.usage.index_values, 12);
        for limits in [
            Limits {
                source_corners: 3,
                ..exact
            },
            Limits {
                index_values: 11,
                ..exact
            },
            Limits {
                node_visits: 0,
                ..exact
            },
            Limits {
                serialized_bytes: f.model.len() + f.components[0].len() - 1,
                ..exact
            },
            Limits {
                components: 0,
                ..exact
            },
            Limits {
                elements: 0,
                ..exact
            },
            Limits {
                buffers: 0,
                ..exact
            },
            Limits {
                reference_keys: 0,
                ..exact
            },
            Limits {
                map_entries: 0,
                ..exact
            },
        ] {
            assert_eq!(
                ModelIndexResource::construct(&f.package, request, limits)
                    .unwrap_err()
                    .kind(),
                ErrorKind::Unsupported
            );
        }
    }
}
