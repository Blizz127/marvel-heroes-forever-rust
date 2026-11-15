//! Original bounded reader for the observed UE3 868/3 PC GPU-skinned mesh.
//!
//! Reference poses and geometry remain in their serialized coordinates. This
//! does not evaluate animation, compose bone transforms, infer material defaults,
//! or decode clothing, physics, morph data or the post-LOD native trailer.

use crate::upk::objects::{PropertyList, Span, Value, parse_tagged_properties};
use crate::upk::{MAX_EXPORT_BYTES, NameRef, Package};
use std::io;

const MAX_BONES: usize = 4096;
const MAX_LODS: usize = 32;
const MAX_SECTIONS: usize = 16_384;
const MAX_CHUNKS: usize = 4096;
const MAX_VERTICES: usize = 2_000_000;
const MAX_INDICES: usize = 12_000_000;
const MAX_MATERIALS: usize = 4096;
const MAX_BONE_NAME_BYTES: usize = 4096;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn unsupported(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message.into())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bounds {
    pub origin: [f32; 3],
    pub extent: [f32; 3],
    pub sphere_radius: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bone {
    pub name: NameRef,
    pub name_text: String,
    pub flags: u32,
    /// Serialized quaternion X,Y,Z,W. No conjugation or normalization applied.
    pub orientation: [f32; 4],
    /// Saved reference-pose position relative to the parent.
    pub position: [f32; 3],
    pub child_count: usize,
    /// The observed root refers to itself at index zero.
    pub parent_index: usize,
    /// Preserved trailing per-bone word; meaning is not established.
    pub auxiliary: i32,
    pub source: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub material_index: usize,
    pub chunk_index: usize,
    pub first_index: usize,
    pub triangle_count: usize,
    pub auxiliary: u8,
    pub source: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexBuffer {
    /// Preserved container word; not interpreted as a boolean or format ID.
    pub auxiliary: i32,
    pub element_width: u8,
    pub indices: Vec<u32>,
    pub source: Span,
    pub data_source: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chunk {
    pub first_vertex: usize,
    /// Skeleton indices selected by the vertex's four local palette indices.
    pub bone_palette: Vec<u16>,
    pub rigid_vertex_count: usize,
    pub soft_vertex_count: usize,
    pub max_influences: usize,
    pub source: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawBulk {
    pub flags: u32,
    pub element_count: usize,
    pub stored_size: i32,
    pub stored_offset: i32,
    pub header_source: Span,
    /// Opaque inline bytes, if any. No external range is followed.
    pub inline_source: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkinVertex {
    pub tangent: [u8; 4],
    pub normal: [u8; 4],
    pub palette_indices: [u8; 4],
    pub weights: [u8; 4],
    pub position: [f32; 3],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UvData {
    /// Raw IEEE-754 binary16 pairs, vertex-major then UV channel.
    Half(Vec<[u16; 2]>),
    /// Raw IEEE-754 binary32 pairs, vertex-major then UV channel.
    FloatBits(Vec<[u32; 2]>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkinBuffer {
    pub uv_channels: usize,
    pub full_precision_uvs: bool,
    /// The inspected PC file saves true while its position storage is Float32.
    /// This declaration is preserved; it does not select a packed-position codec.
    pub declared_packed_positions: bool,
    pub mesh_extent: [f32; 3],
    pub mesh_origin: [f32; 3],
    pub stored_stride: usize,
    pub vertices: Vec<SkinVertex>,
    pub uvs: UvData,
    pub source: Span,
    pub data_source: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Influence {
    pub bone_index: usize,
    pub weight: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lod {
    pub sections: Vec<Section>,
    pub indices: IndexBuffer,
    pub active_bones: Vec<u16>,
    pub chunks: Vec<Chunk>,
    /// Preserved serialized LOD-size word, zero in the actual sample.
    pub serialized_size: i32,
    pub vertex_count: usize,
    pub required_bones: Vec<u8>,
    pub raw_bulk: RawBulk,
    pub uv_channels: usize,
    pub skin: SkinBuffer,
    /// Only the observed empty extra-influence array is supported.
    pub extra_influence_count: usize,
    pub adjacency_indices: IndexBuffer,
    pub source: Span,
}

impl Lod {
    /// Resolve nonzero weight slots through the containing chunk's bone palette.
    /// Zero-weight slots remain None; weights are not renormalized or reordered.
    pub fn influences(&self, vertex_index: usize) -> io::Result<[Option<Influence>; 4]> {
        let vertex = self
            .skin
            .vertices
            .get(vertex_index)
            .ok_or_else(|| invalid("skin vertex index is out of bounds"))?;
        let mut candidates = self.chunks.iter().filter(|chunk| {
            chunk
                .first_vertex
                .checked_add(chunk.rigid_vertex_count)
                .and_then(|end| end.checked_add(chunk.soft_vertex_count))
                .is_some_and(|end| vertex_index >= chunk.first_vertex && vertex_index < end)
        });
        let chunk = candidates
            .next()
            .ok_or_else(|| invalid("skin vertex has no chunk"))?;
        if candidates.next().is_some() {
            return Err(invalid("skin vertex belongs to overlapping chunks"));
        }
        let mut result = [None; 4];
        for (slot, output) in result.iter_mut().enumerate() {
            if vertex.weights[slot] != 0 {
                let bone = *chunk
                    .bone_palette
                    .get(vertex.palette_indices[slot] as usize)
                    .ok_or_else(|| invalid("weighted skin palette index is out of bounds"))?;
                *output = Some(Influence {
                    bone_index: bone as usize,
                    weight: vertex.weights[slot],
                });
            }
        }
        Ok(result)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkeletalMesh {
    pub export_index: usize,
    pub serial_offset: usize,
    pub net_index: i32,
    pub properties: PropertyList,
    /// Serialized property only; absent stays None. The observed absent-property
    /// profile has no color stream; explicit true is unsupported in this reader.
    pub has_vertex_colors: Option<bool>,
    pub bounds: Bounds,
    pub materials: Vec<i32>,
    pub mesh_origin: [f32; 3],
    pub rotation_origin: [i32; 3],
    pub bones: Vec<Bone>,
    pub skeletal_depth: usize,
    pub lods: Vec<Lod>,
    pub native_source: Span,
    /// First export-relative byte after the decoded LOD array.
    pub decoded_end: usize,
    pub opaque_tail: Vec<u8>,
}

impl SkeletalMesh {
    pub fn from_export(package: &Package, export_index: usize, bytes: &[u8]) -> io::Result<Self> {
        if (
            package.summary.file_version,
            package.summary.licensee_version,
        ) != (868, 3)
        {
            return Err(unsupported(
                "SkeletalMesh requires observed PC profile 868/3",
            ));
        }
        let export = package
            .exports
            .get(export_index)
            .ok_or_else(|| invalid("SkeletalMesh export index is out of bounds"))?;
        if bytes.len() != export.serial_size || bytes.len() > MAX_EXPORT_BYTES {
            return Err(invalid(
                "SkeletalMesh export size mismatch or limit exceeded",
            ));
        }
        export
            .serial_offset
            .checked_add(bytes.len())
            .ok_or_else(|| invalid("SkeletalMesh logical range overflow"))?;
        if !package
            .export_class_name(export_index)
            .is_some_and(|name| name.eq_ignore_ascii_case("SkeletalMesh"))
        {
            return Err(unsupported("selected export is not a SkeletalMesh"));
        }
        if export.object_flags & 0x0200_0000_0000_0000 != 0 {
            return Err(unsupported(
                "SkeletalMesh has an unestablished UObject preamble",
            ));
        }
        let mut input = Input { bytes, position: 0 };
        let net_index = input.i32()?;
        let properties = parse_tagged_properties(package, bytes, 4)?;
        let mut has_vertex_colors = None;
        for property in &properties.entries {
            if package
                .resolve_name(property.name)
                .is_some_and(|name| name.eq_ignore_ascii_case("bHasVertexColors"))
            {
                if property.array_index != 0 || has_vertex_colors.is_some() {
                    return Err(invalid("duplicate or indexed bHasVertexColors property"));
                }
                let Value::Boolean(value) = property.value else {
                    return Err(invalid("bHasVertexColors is not a boolean"));
                };
                has_vertex_colors = Some(value);
            }
        }
        if has_vertex_colors == Some(true) {
            return Err(unsupported(
                "skeletal vertex-color stream is not established",
            ));
        }
        input.position = properties.end_offset;
        let native_source = Span {
            offset: input.position,
            length: bytes.len() - input.position,
        };
        let bounds = Bounds {
            origin: input.floats()?,
            extent: input.floats()?,
            sphere_radius: input.float()?,
        };
        if bounds.extent.iter().any(|value| *value < 0.0) || bounds.sphere_radius < 0.0 {
            return Err(invalid("negative skeletal bounds extent or radius"));
        }
        let count = input.count(MAX_MATERIALS, 4)?;
        let mut materials = Vec::with_capacity(count);
        for _ in 0..count {
            let reference = input.i32()?;
            valid_reference(package, reference)?;
            materials.push(reference);
        }
        let mesh_origin = input.floats()?;
        let rotation_origin = [input.i32()?, input.i32()?, input.i32()?];
        let bone_count = input.count(MAX_BONES, 52)?;
        if bone_count == 0 {
            return Err(invalid("skeletal reference skeleton is empty"));
        }
        let mut bones = Vec::with_capacity(bone_count);
        for _ in 0..bone_count {
            let start = input.position;
            let name = NameRef {
                index: input.count(package.names.len().saturating_sub(1), 0)?,
                number: input.u32()?,
            };
            let name_text = package
                .resolve_name(name)
                .ok_or_else(|| invalid("bone name is unresolved"))?;
            if name_text.len() > MAX_BONE_NAME_BYTES {
                return Err(invalid("bone name exceeds limit"));
            }
            let flags = input.u32()?;
            let orientation = input.floats()?;
            let position = input.floats()?;
            let child_count = input.count(MAX_BONES, 0)?;
            let parent_index = input.count(bone_count - 1, 0)?;
            let auxiliary = input.i32()?;
            bones.push(Bone {
                name,
                name_text,
                flags,
                orientation,
                position,
                child_count,
                parent_index,
                auxiliary,
                source: input.span_from(start),
            });
        }
        validate_skeleton(&bones)?;
        let skeletal_depth = input.count(MAX_BONES, 0)?;
        let lod_count = input.count(MAX_LODS, 4)?;
        if lod_count == 0 {
            return Err(invalid("skeletal mesh has no LODs"));
        }
        let mut lods = Vec::with_capacity(lod_count);
        for _ in 0..lod_count {
            lods.push(read_lod(
                &mut input,
                export.serial_offset,
                bones.len(),
                materials.len(),
            )?);
        }
        let decoded_end = input.position;
        let opaque_tail = input.take(bytes.len() - decoded_end)?.to_vec();
        Ok(Self {
            export_index,
            serial_offset: export.serial_offset,
            net_index,
            properties,
            has_vertex_colors,
            bounds,
            materials,
            mesh_origin,
            rotation_origin,
            bones,
            skeletal_depth,
            lods,
            native_source,
            decoded_end,
            opaque_tail,
        })
    }
}

fn validate_skeleton(bones: &[Bone]) -> io::Result<()> {
    // The observed reference array is topologically ordered, with root index 0.
    if bones.first().is_none_or(|bone| bone.parent_index != 0) {
        return Err(invalid(
            "reference skeleton has no observed root at index zero",
        ));
    }
    for (index, bone) in bones.iter().enumerate().skip(1) {
        if bone.parent_index >= index {
            return Err(unsupported(
                "reference bones are not in observed parent-before-child order",
            ));
        }
    }
    Ok(())
}

fn read_lod(
    input: &mut Input<'_>,
    serial_offset: usize,
    bone_count: usize,
    material_count: usize,
) -> io::Result<Lod> {
    let start = input.position;
    let count = input.count(MAX_SECTIONS, 13)?;
    let mut sections = Vec::with_capacity(count);
    for _ in 0..count {
        let begin = input.position;
        let material_index = input.u16()? as usize;
        let chunk_index = input.u16()? as usize;
        let first_index = input.count(MAX_INDICES, 0)?;
        let triangle_count = input.count(MAX_INDICES / 3, 0)?;
        let auxiliary = input.u8()?;
        if material_index >= material_count {
            return Err(invalid("section material index is out of bounds"));
        }
        sections.push(Section {
            material_index,
            chunk_index,
            first_index,
            triangle_count,
            auxiliary,
            source: input.span_from(begin),
        });
    }
    let indices = read_indices(input)?;
    let active_bones = read_bone_indices(input, bone_count)?;
    let count = input.count(MAX_CHUNKS, 28)?;
    let mut chunks = Vec::with_capacity(count);
    for _ in 0..count {
        let begin = input.position;
        let first_vertex = input.count(MAX_VERTICES, 0)?;
        let rigid_count = input.count(MAX_VERTICES, 0)?;
        if rigid_count != 0 {
            return Err(unsupported(
                "unstripped CPU rigid vertices are not established",
            ));
        }
        let soft_count = input.count(MAX_VERTICES, 0)?;
        if soft_count != 0 {
            return Err(unsupported(
                "unstripped CPU soft vertices are not established",
            ));
        }
        let bone_palette = read_bone_indices(input, bone_count)?;
        let rigid_vertex_count = input.count(MAX_VERTICES, 0)?;
        let soft_vertex_count = input.count(MAX_VERTICES, 0)?;
        let max_influences = input.count(4, 0)?;
        if max_influences == 0 {
            return Err(invalid("skin chunk has zero influence capacity"));
        }
        chunks.push(Chunk {
            first_vertex,
            bone_palette,
            rigid_vertex_count,
            soft_vertex_count,
            max_influences,
            source: input.span_from(begin),
        });
    }
    let serialized_size = input.i32()?;
    let vertex_count = input.count(MAX_VERTICES, 0)?;
    if vertex_count == 0 {
        return Err(invalid("skeletal LOD has no vertices"));
    }
    let count = input.count(MAX_BONES, 1)?;
    let required_bones = input.take(count)?.to_vec();
    if required_bones
        .iter()
        .any(|&index| index as usize >= bone_count)
    {
        return Err(invalid("required bone index is out of bounds"));
    }
    let raw_bulk = read_bulk(input, serial_offset)?;
    let uv_channels = input.count(4, 0)?;
    if uv_channels == 0 {
        return Err(invalid("skeletal LOD has no UV channels"));
    }
    let skin = read_skin(input, vertex_count, uv_channels)?;
    let extra_influence_count = input.count(MAX_VERTICES, 0)?;
    if extra_influence_count != 0 {
        return Err(unsupported(
            "extra skeletal influence arrays are not established",
        ));
    }
    let adjacency_indices = read_indices(input)?;
    let lod = Lod {
        sections,
        indices,
        active_bones,
        chunks,
        serialized_size,
        vertex_count,
        required_bones,
        raw_bulk,
        uv_channels,
        skin,
        extra_influence_count,
        adjacency_indices,
        source: input.span_from(start),
    };
    validate_lod(&lod)?;
    Ok(lod)
}

fn read_indices(input: &mut Input<'_>) -> io::Result<IndexBuffer> {
    let start = input.position;
    let auxiliary = input.i32()?;
    let element_width = input.u8()?;
    if !matches!(element_width, 2 | 4) {
        return Err(unsupported("unknown skeletal index width"));
    }
    let stride = input.count(4, 0)?;
    if stride != element_width as usize {
        return Err(invalid("skeletal index width and stride disagree"));
    }
    let count = input.count(MAX_INDICES, stride)?;
    let begin = input.position;
    let mut indices = Vec::with_capacity(count);
    for _ in 0..count {
        indices.push(if stride == 2 {
            input.u16()? as u32
        } else {
            input.u32()?
        });
    }
    Ok(IndexBuffer {
        auxiliary,
        element_width,
        indices,
        source: input.span_from(start),
        data_source: input.span_from(begin),
    })
}
fn read_bone_indices(input: &mut Input<'_>, bone_count: usize) -> io::Result<Vec<u16>> {
    let count = input.count(MAX_BONES, 2)?;
    let mut indices = Vec::with_capacity(count);
    for _ in 0..count {
        let index = input.u16()?;
        if index as usize >= bone_count {
            return Err(invalid("skeletal bone-list index is out of bounds"));
        }
        indices.push(index);
    }
    Ok(indices)
}
fn read_bulk(input: &mut Input<'_>, serial_offset: usize) -> io::Result<RawBulk> {
    let start = input.position;
    let flags = input.u32()?;
    let element_count = input.count(MAX_EXPORT_BYTES / 4, 0)?;
    let stored_size = input.i32()?;
    let stored_offset = input.i32()?;
    let header_source = input.span_from(start);
    let begin = input.position;
    if !matches!(flags, 0 | 0x20) {
        return Err(unsupported(
            "skeletal auxiliary bulk flags are not established",
        ));
    }
    if flags == 0 && element_count != 0 {
        let expected = element_count
            .checked_mul(4)
            .ok_or_else(|| invalid("skeletal bulk size overflow"))?;
        if usize::try_from(stored_size).ok() != Some(expected)
            || usize::try_from(stored_offset).ok() != serial_offset.checked_add(begin)
        {
            return Err(unsupported(
                "skeletal auxiliary bulk is not an exact inline integer array",
            ));
        }
        input.take(expected)?;
    } else if flags == 0 && stored_size != 0 {
        return Err(invalid("empty skeletal bulk has a nonzero stored size"));
    }
    Ok(RawBulk {
        flags,
        element_count,
        stored_size,
        stored_offset,
        header_source,
        inline_source: input.span_from(begin),
    })
}
fn read_skin(
    input: &mut Input<'_>,
    vertex_count: usize,
    uv_channels: usize,
) -> io::Result<SkinBuffer> {
    let start = input.position;
    if input.count(4, 0)? != uv_channels {
        return Err(invalid("LOD and skin UV channel counts disagree"));
    }
    let full_precision_uvs = input.boolean()?;
    let declared_packed_positions = input.boolean()?;
    let mesh_extent = input.floats()?;
    let mesh_origin = input.floats()?;
    let stored_stride = input.count(128, 0)?;
    let uv_bytes = if full_precision_uvs { 8 } else { 4 };
    // Observed PC storage always contains Float32 position, even when the
    // declaration above says packed. Other strides are not reinterpreted.
    if stored_stride != 28 + uv_channels * uv_bytes {
        return Err(unsupported(
            "skeletal skin stride is not the observed float-position layout",
        ));
    }
    let count = input.count(MAX_VERTICES, stored_stride)?;
    if count != vertex_count {
        return Err(invalid("LOD and skin vertex counts disagree"));
    }
    let begin = input.position;
    let mut vertices = Vec::with_capacity(count);
    let mut half_uvs = Vec::new();
    let mut float_uvs = Vec::new();
    let uv_count = count
        .checked_mul(uv_channels)
        .ok_or_else(|| invalid("skin UV count overflow"))?;
    if full_precision_uvs {
        float_uvs.reserve(uv_count);
    } else {
        half_uvs.reserve(uv_count);
    }
    for _ in 0..count {
        vertices.push(SkinVertex {
            tangent: input.array()?,
            normal: input.array()?,
            palette_indices: input.array()?,
            weights: input.array()?,
            position: input.floats()?,
        });
        for _ in 0..uv_channels {
            if full_precision_uvs {
                float_uvs.push([input.u32()?, input.u32()?]);
            } else {
                half_uvs.push([input.u16()?, input.u16()?]);
            }
        }
    }
    let uvs = if full_precision_uvs {
        UvData::FloatBits(float_uvs)
    } else {
        UvData::Half(half_uvs)
    };
    Ok(SkinBuffer {
        uv_channels,
        full_precision_uvs,
        declared_packed_positions,
        mesh_extent,
        mesh_origin,
        stored_stride,
        vertices,
        uvs,
        source: input.span_from(start),
        data_source: input.span_from(begin),
    })
}
fn validate_lod(lod: &Lod) -> io::Result<()> {
    let mut next_vertex = 0usize;
    for chunk in &lod.chunks {
        if chunk.first_vertex != next_vertex {
            return Err(unsupported(
                "skin chunks do not form the observed ordered partition",
            ));
        }
        next_vertex = next_vertex
            .checked_add(chunk.rigid_vertex_count)
            .and_then(|end| end.checked_add(chunk.soft_vertex_count))
            .ok_or_else(|| invalid("skin chunk vertex range overflow"))?;
        if next_vertex > lod.vertex_count {
            return Err(invalid("skin chunk exceeds vertex buffer"));
        }
        for vertex in &lod.skin.vertices[chunk.first_vertex..next_vertex] {
            let mut influences = 0;
            for slot in 0..4 {
                if vertex.weights[slot] != 0 {
                    influences += 1;
                    if vertex.palette_indices[slot] as usize >= chunk.bone_palette.len() {
                        return Err(invalid("weighted palette index exceeds chunk bone list"));
                    }
                }
            }
            if influences == 0 || influences > chunk.max_influences {
                return Err(invalid("vertex influence count disagrees with its chunk"));
            }
        }
    }
    if next_vertex != lod.vertex_count {
        return Err(invalid("skin chunks do not cover the vertex buffer"));
    }
    for index in lod
        .indices
        .indices
        .iter()
        .chain(&lod.adjacency_indices.indices)
    {
        if *index as usize >= lod.vertex_count {
            return Err(invalid("skeletal vertex index is out of bounds"));
        }
    }
    for section in &lod.sections {
        let chunk = lod
            .chunks
            .get(section.chunk_index)
            .ok_or_else(|| invalid("section chunk index is out of bounds"))?;
        let end = section
            .triangle_count
            .checked_mul(3)
            .and_then(|count| section.first_index.checked_add(count))
            .ok_or_else(|| invalid("section index range overflow"))?;
        let indices = lod
            .indices
            .indices
            .get(section.first_index..end)
            .ok_or_else(|| invalid("section exceeds its index buffer"))?;
        let chunk_end = chunk.first_vertex + chunk.rigid_vertex_count + chunk.soft_vertex_count;
        if indices
            .iter()
            .any(|&index| (index as usize) < chunk.first_vertex || index as usize >= chunk_end)
        {
            return Err(invalid(
                "section index points outside its selected skin chunk",
            ));
        }
    }
    Ok(())
}
fn valid_reference(package: &Package, index: i32) -> io::Result<()> {
    if (index > 0 && index as usize > package.exports.len())
        || (index < 0 && index.unsigned_abs() as usize > package.imports.len())
    {
        return Err(invalid("skeletal material reference is out of bounds"));
    }
    Ok(())
}

struct Input<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Input<'a> {
    fn take(&mut self, count: usize) -> io::Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(count)
            .ok_or_else(|| invalid("skeletal byte range overflow"))?;
        let result = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| invalid("truncated skeletal data"))?;
        self.position = end;
        Ok(result)
    }
    fn array<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        Ok(self.take(N)?.try_into().unwrap())
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.array()?))
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.array()?))
    }
    fn float(&mut self) -> io::Result<f32> {
        let value = f32::from_bits(self.u32()?);
        if !value.is_finite() {
            return Err(invalid("nonfinite skeletal geometry value"));
        }
        Ok(value)
    }
    fn floats<const N: usize>(&mut self) -> io::Result<[f32; N]> {
        let mut values = [0.0; N];
        for value in &mut values {
            *value = self.float()?;
        }
        Ok(values)
    }
    fn boolean(&mut self) -> io::Result<bool> {
        match self.i32()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid("noncanonical skeletal boolean")),
        }
    }
    fn count(&mut self, limit: usize, minimum_width: usize) -> io::Result<usize> {
        let value = usize::try_from(self.i32()?)
            .map_err(|_| invalid("negative skeletal count or index"))?;
        if value > limit
            || (minimum_width != 0 && value > (self.bytes.len() - self.position) / minimum_width)
        {
            return Err(invalid("skeletal count exceeds its input or limit"));
        }
        Ok(value)
    }
    fn span_from(&self, start: usize) -> Span {
        Span {
            offset: start,
            length: self.position - start,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        bytes: Vec<u8>,
        index_data: usize,
        skin_data: usize,
        stride: usize,
        extra: usize,
    }
    fn word(bytes: &mut Vec<u8>, value: i32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fn index_header(bytes: &mut Vec<u8>, count: i32) {
        word(bytes, 1);
        bytes.push(2);
        word(bytes, 2);
        word(bytes, count);
    }
    fn fixture() -> Fixture {
        // Original constructed triangle; no retail asset content.
        let mut b = Vec::new();
        word(&mut b, 1);
        b.extend_from_slice(&[0; 4]);
        word(&mut b, 0);
        word(&mut b, 1);
        b.push(0);
        index_header(&mut b, 3);
        let index_data = b.len();
        for index in [0u16, 1, 2] {
            b.extend_from_slice(&index.to_le_bytes());
        }
        word(&mut b, 1);
        b.extend_from_slice(&0u16.to_le_bytes()); // active bones
        word(&mut b, 1); // chunks
        for value in [0, 0, 0, 1] {
            word(&mut b, value);
        }
        b.extend_from_slice(&0u16.to_le_bytes());
        for value in [3, 0, 1, 0, 3] {
            word(&mut b, value);
        } // rigid,soft,max,size,vertices
        word(&mut b, 1);
        b.push(0); // required bones
        for _ in 0..4 {
            word(&mut b, 0);
        } // empty auxiliary bulk
        for value in [1, 1, 0, 1] {
            word(&mut b, value);
        } // LOD UV, skin UV, fullUV, declared packed
        for value in [1.0f32, 1.0, 1.0, 0.0, 0.0, 0.0] {
            b.extend_from_slice(&value.to_bits().to_le_bytes());
        }
        let stride = b.len();
        word(&mut b, 32);
        word(&mut b, 3);
        let skin_data = b.len();
        for position in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
            b.extend_from_slice(&[128, 128, 255, 128, 128, 255, 128, 255]);
            b.extend_from_slice(&[0; 4]);
            b.extend_from_slice(&[255, 0, 0, 0]);
            for value in position {
                b.extend_from_slice(&value.to_bits().to_le_bytes());
            }
            b.extend_from_slice(&[0; 4]);
        }
        let extra = b.len();
        word(&mut b, 0);
        index_header(&mut b, 0);
        Fixture {
            bytes: b,
            index_data,
            skin_data,
            stride,
            extra,
        }
    }
    fn parse(bytes: &[u8]) -> io::Result<Lod> {
        read_lod(&mut Input { bytes, position: 0 }, 0, 1, 1)
    }

    #[test]
    fn reads_float_position_despite_preserved_packed_declaration() {
        let f = fixture();
        let lod = parse(&f.bytes).unwrap();
        assert!(lod.skin.declared_packed_positions);
        assert_eq!(lod.skin.vertices[1].position, [1.0, 0.0, 0.0]);
        assert_eq!(lod.indices.indices, [0, 1, 2]);
        assert_eq!(
            lod.influences(1).unwrap(),
            [
                Some(Influence {
                    bone_index: 0,
                    weight: 255
                }),
                None,
                None,
                None
            ]
        );
        assert_eq!(lod.source.length, f.bytes.len());
    }

    #[test]
    fn rejects_every_lod_truncation_and_excessive_counts() {
        let f = fixture();
        for end in 0..f.bytes.len() {
            assert!(parse(&f.bytes[..end]).is_err(), "accepted truncation {end}");
        }
        for value in [-1, i32::MAX] {
            let mut b = f.bytes.clone();
            b[..4].copy_from_slice(&value.to_le_bytes());
            assert!(parse(&b).is_err());
        }
        let mut b = f.bytes.clone();
        b[f.extra..f.extra + 4].copy_from_slice(&1i32.to_le_bytes());
        assert_eq!(parse(&b).unwrap_err().kind(), io::ErrorKind::Unsupported);
    }

    #[test]
    fn bounds_geometry_palette_and_stream_layout() {
        let mut wide = Vec::new();
        word(&mut wide, 7);
        wide.push(4);
        word(&mut wide, 4);
        word(&mut wide, 3);
        for value in [0u32, 65536, u32::MAX] {
            wide.extend_from_slice(&value.to_le_bytes());
        }
        let parsed = read_indices(&mut Input {
            bytes: &wide,
            position: 0,
        })
        .unwrap();
        assert_eq!(parsed.element_width, 4);
        assert_eq!(parsed.indices, [0, 65536, u32::MAX]);
        let f = fixture();
        let mut b = f.bytes.clone();
        b[f.index_data..f.index_data + 2].copy_from_slice(&3u16.to_le_bytes());
        assert!(parse(&b).is_err());
        b = f.bytes.clone();
        b[f.skin_data + 8] = 1;
        assert!(parse(&b).is_err()); // weighted palette index
        b = f.bytes.clone();
        b[f.skin_data + 12..f.skin_data + 16].fill(0);
        assert!(parse(&b).is_err());
        b = f.bytes.clone();
        b[f.stride..f.stride + 4].copy_from_slice(&20i32.to_le_bytes());
        assert!(parse(&b).is_err());
        b = f.bytes.clone();
        b[f.skin_data + 16..f.skin_data + 20].copy_from_slice(&f32::NAN.to_bits().to_le_bytes());
        assert!(parse(&b).is_err());
        // Preserve declared byte weights; this reader never renormalizes them.
        b = f.bytes;
        b[f.skin_data + 12] = 254;
        assert_eq!(
            parse(&b).unwrap().influences(0).unwrap()[0].unwrap().weight,
            254
        );
    }
}
