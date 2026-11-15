//! Bounded StaticMesh geometry reads for the observed UE3 868/3, internal-v18 profile.
//! Collision/material behavior and the post-LOD tail are not interpreted.

use std::io;

use crate::upk::{
    Package,
    objects::{PropertyList, Span, parse_tagged_properties},
};

const MAX_BYTES: usize = 64 * 1024 * 1024;
const MAX_LODS: usize = 32;
const MAX_VERTICES: usize = 1_000_000;
const MAX_INDICES: usize = 12_000_000;
const MAX_SECTIONS: usize = 4096;
const MAX_COLLISION_RECORDS: usize = 4_000_000;
const MAX_UV_CHANNELS: usize = 4;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn unsupported(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message.into())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub origin: [f32; 3],
    pub box_extent: [f32; 3],
    pub sphere_radius: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackedNormal(pub [u8; 4]);

impl PackedNormal {
    /// Decode the stored biased components without renormalizing or changing handedness.
    pub fn xyz(self) -> [f32; 3] {
        [self.0[0], self.0[1], self.0[2]].map(|byte| f32::from(byte) / 127.5 - 1.0)
    }
    pub fn w(self) -> f32 {
        f32::from(self.0[3]) / 127.5 - 1.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UvPrecision {
    Half,
    Float,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VertexAttributes {
    pub precision: UvPrecision,
    pub uv_channels: usize,
    /// Tangent X and normal/tangent Z, respectively. Tangent Y is not serialized.
    pub tangents: Vec<[PackedNormal; 2]>,
    /// Vertex-major/channel-minor raw components. Half components occupy low 16 bits.
    pub uv_bits: Vec<[u32; 2]>,
}
impl VertexAttributes {
    pub fn uv(&self, vertex: usize, channel: usize) -> Option<[f32; 2]> {
        if vertex >= self.tangents.len() || channel >= self.uv_channels {
            return None;
        }
        let raw = *self
            .uv_bits
            .get(vertex.checked_mul(self.uv_channels)?.checked_add(channel)?)?;
        Some(match self.precision {
            UvPrecision::Half => raw.map(|bits| half_to_float(bits as u16)),
            UvPrecision::Float => raw.map(f32::from_bits),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Section {
    pub material: i32,
    /// Two serialized words whose gameplay/collision semantics are not established here.
    pub unknown_flags: [i32; 2],
    pub shadow_casting: i32,
    pub first_index: usize,
    pub triangle_count: usize,
    pub min_vertex: usize,
    pub max_vertex: usize,
    pub section_index: i32,
    pub auxiliary_pairs: Vec<[i32; 2]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawBulk {
    pub flags: u32,
    pub element_count: usize,
    pub stored_size: usize,
    pub stored_offset: i32,
    /// Inline editor/source bytes, if present. Their internal record layout is opaque.
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CollisionData {
    /// Serialized quantization bounds, without assigning node/triangle collision behavior.
    pub bounds_vectors: [[f32; 3]; 2],
    pub quantized_nodes: Vec<[u8; 6]>,
    pub triangle_records: Vec<[u16; 4]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lod {
    pub source_bulk: RawBulk,
    pub sections: Vec<Section>,
    pub positions: Vec<[f32; 3]>,
    pub attributes: VertexAttributes,
    /// Serialized BGRA8 values; no transfer conversion or premultiplication.
    pub colors_bgra: Vec<[u8; 4]>,
    pub color_stride: usize,
    pub triangle_indices: Vec<u16>,
    pub wireframe_indices: Vec<u16>,
    /// A third index stream present in this profile; its rendering role is not inferred.
    pub additional_indices: Vec<u16>,
    /// All spans are relative to the native payload supplied to parse_native.
    pub serialized: Span,
    pub position_bytes: Span,
    pub attribute_bytes: Span,
    pub color_bytes: Option<Span>,
    pub index_bytes: [Span; 3],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectMetadata {
    pub export_index: usize,
    pub net_index: i32,
    pub properties: PropertyList,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticMesh {
    pub object: Option<ObjectMetadata>,
    pub native_logical_offset: usize,
    pub bounds: Bounds,
    pub body_setup: i32,
    pub collision: CollisionData,
    pub internal_version: i32,
    pub extra_lod: Option<Lod>,
    /// Seven-byte records before the LOD array; meaning remains unknown.
    pub auxiliary_records: Vec<(i32, [u8; 3])>,
    pub auxiliary_words: [i32; 2],
    pub lods: Vec<Lod>,
    pub parsed_bytes: usize,
    /// Preserved post-LOD bytes; not counted as understood fields.
    pub trailing_bytes: Vec<u8>,
}

impl StaticMesh {
    pub fn from_export(package: &Package, index: usize, bytes: &[u8]) -> io::Result<Self> {
        if (
            package.summary.file_version,
            package.summary.licensee_version,
        ) != (868, 3)
        {
            return Err(unsupported(
                "StaticMesh requires the observed UE3 868/3 profile",
            ));
        }
        let export = package
            .exports
            .get(index)
            .ok_or_else(|| invalid("StaticMesh export index out of bounds"))?;
        if bytes.len() != export.serial_size || bytes.len() > MAX_BYTES {
            return Err(invalid("StaticMesh export size mismatch or limit exceeded"));
        }
        if !package
            .export_class_name(index)
            .is_some_and(|name| name.eq_ignore_ascii_case("StaticMesh"))
        {
            return Err(unsupported("selected export is not a StaticMesh"));
        }
        let net_index = Input::new(bytes, 0).i32()?;
        let properties = parse_tagged_properties(package, bytes, 4)?;
        let offset = export
            .serial_offset
            .checked_add(properties.end_offset)
            .ok_or_else(|| invalid("native logical offset overflow"))?;
        let mut mesh = Self::parse_native(&bytes[properties.end_offset..], offset)?;
        valid_reference(package, mesh.body_setup)?;
        for lod in mesh.extra_lod.iter().chain(&mesh.lods) {
            for section in &lod.sections {
                valid_reference(package, section.material)?;
            }
        }
        mesh.object = Some(ObjectMetadata {
            export_index: index,
            net_index,
            properties,
        });
        Ok(mesh)
    }

    /// Read an explicitly selected 868/3 native tail at its logical package offset.
    /// Geometry is decoded; collision/editor auxiliaries and post-LOD bytes remain opaque.
    pub fn parse_native(bytes: &[u8], logical_offset: usize) -> io::Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err(invalid("StaticMesh native bytes exceed limit"));
        }
        logical_offset
            .checked_add(bytes.len())
            .ok_or_else(|| invalid("logical mesh range overflow"))?;
        let mut input = Input::new(bytes, logical_offset);
        let bounds = Bounds {
            origin: input.vector()?,
            box_extent: input.vector()?,
            sphere_radius: input.float()?,
        };
        if bounds.box_extent.iter().any(|v| *v < 0.0) || bounds.sphere_radius < 0.0 {
            return Err(invalid("negative mesh bounds extent or radius"));
        }
        let body_setup = input.i32()?;
        let bounds_vectors = [input.vector()?, input.vector()?];
        let (nodes, count) = input.bulk(6, MAX_COLLISION_RECORDS)?;
        let quantized_nodes = copy_records::<6>(nodes, count)?;
        let (triangles, count) = input.bulk(8, MAX_COLLISION_RECORDS)?;
        let mut triangle_records = reserve(count)?;
        for row in triangles.chunks_exact(8) {
            triangle_records.push(std::array::from_fn(|i| {
                u16::from_le_bytes([row[i * 2], row[i * 2 + 1]])
            }));
        }
        let internal_version = input.i32()?;
        if internal_version != 18 {
            return Err(unsupported(format!(
                "unobserved StaticMesh internal version {internal_version}"
            )));
        }
        let extra_lod = match input.i32()? {
            0 => None,
            1 => Some(input.lod()?),
            _ => return Err(invalid("noncanonical extra-LOD flag")),
        };
        let count = input.count(MAX_COLLISION_RECORDS, 7, "auxiliary record")?;
        let mut auxiliary_records = reserve(count)?;
        for _ in 0..count {
            auxiliary_records.push((input.i32()?, input.array()?));
        }
        let auxiliary_words = [input.i32()?, input.i32()?];
        let count = input.count(MAX_LODS, 16, "LOD")?;
        let mut lods = reserve(count)?;
        for _ in 0..count {
            lods.push(input.lod()?);
        }
        let parsed_bytes = input.position;
        let trailing_bytes = input.take(bytes.len() - parsed_bytes)?.to_vec();
        Ok(Self {
            object: None,
            native_logical_offset: logical_offset,
            bounds,
            body_setup,
            collision: CollisionData {
                bounds_vectors,
                quantized_nodes,
                triangle_records,
            },
            internal_version,
            extra_lod,
            auxiliary_records,
            auxiliary_words,
            lods,
            parsed_bytes,
            trailing_bytes,
        })
    }
}

fn valid_reference(package: &Package, index: i32) -> io::Result<()> {
    let valid = if index > 0 {
        index as usize <= package.exports.len()
    } else if index < 0 {
        index.unsigned_abs() as usize <= package.imports.len()
    } else {
        true
    };
    if valid {
        Ok(())
    } else {
        Err(invalid("mesh object reference outside package tables"))
    }
}

fn reserve<T>(count: usize) -> io::Result<Vec<T>> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| invalid("mesh allocation failed"))?;
    Ok(result)
}
fn copy_records<const N: usize>(bytes: &[u8], count: usize) -> io::Result<Vec<[u8; N]>> {
    let mut records = reserve(count)?;
    for chunk in bytes.chunks_exact(N) {
        records.push(chunk.try_into().unwrap());
    }
    Ok(records)
}

struct Input<'a> {
    bytes: &'a [u8],
    position: usize,
    logical_offset: usize,
}
impl<'a> Input<'a> {
    fn new(bytes: &'a [u8], logical_offset: usize) -> Self {
        Self {
            bytes,
            position: 0,
            logical_offset,
        }
    }
    fn take(&mut self, length: usize) -> io::Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| invalid("mesh range overflow"))?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| invalid(format!("truncated mesh at native byte {}", self.position)))?;
        self.position = end;
        Ok(bytes)
    }
    fn array<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        Ok(self.take(N)?.try_into().unwrap())
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.array()?))
    }
    fn positive(&mut self, label: &str) -> io::Result<usize> {
        usize::try_from(self.i32()?).map_err(|_| invalid(format!("negative {label}")))
    }
    fn float(&mut self) -> io::Result<f32> {
        let value = f32::from_bits(self.u32()?);
        if !value.is_finite() {
            return Err(invalid("nonfinite mesh float"));
        }
        Ok(value)
    }
    fn vector(&mut self) -> io::Result<[f32; 3]> {
        Ok([self.float()?, self.float()?, self.float()?])
    }
    fn count(&mut self, max: usize, stride: usize, label: &str) -> io::Result<usize> {
        let count = self.positive(label)?;
        if count > max || count > (self.bytes.len() - self.position) / stride {
            return Err(invalid(format!(
                "{label} count exceeds limit or remaining bytes"
            )));
        }
        Ok(count)
    }
    fn bulk(&mut self, stride: usize, max: usize) -> io::Result<(&'a [u8], usize)> {
        let observed = self.positive("bulk stride")?;
        if observed != stride {
            return Err(unsupported(format!(
                "bulk stride {observed}, expected {stride}"
            )));
        }
        let count = self.count(max, stride, "bulk element")?;
        let bytes = self.take(
            count
                .checked_mul(stride)
                .ok_or_else(|| invalid("bulk byte overflow"))?,
        )?;
        Ok((bytes, count))
    }
    fn raw_bulk(&mut self) -> io::Result<RawBulk> {
        let flags = self.u32()?;
        if !matches!(flags, 0 | 0x20) {
            return Err(unsupported(format!(
                "unestablished mesh source-bulk flags {flags:#x}"
            )));
        }
        let element_count = self.positive("source-bulk element count")?;
        let stored_size = self.positive("source-bulk size")?;
        let stored_offset = self.i32()?;
        if element_count > MAX_BYTES || stored_size > MAX_BYTES {
            return Err(invalid("mesh source-bulk budget exceeded"));
        }
        let payload = if flags & 0x20 != 0 || stored_size == 0 {
            Vec::new()
        } else {
            let current = self
                .logical_offset
                .checked_add(self.position)
                .ok_or_else(|| invalid("source bulk offset overflow"))?;
            if usize::try_from(stored_offset).ok() != Some(current) {
                return Err(unsupported("non-inline mesh source bulk is not loaded"));
            }
            self.take(stored_size)?.to_vec()
        };
        Ok(RawBulk {
            flags,
            element_count,
            stored_size,
            stored_offset,
            payload,
        })
    }
    fn section(&mut self) -> io::Result<Section> {
        let material = self.i32()?;
        let unknown_flags = [self.i32()?, self.i32()?];
        let shadow_casting = self.i32()?;
        let first_index = self.positive("section first index")?;
        let triangle_count = self.positive("section triangle count")?;
        let min_vertex = self.positive("section minimum vertex")?;
        let max_vertex = self.positive("section maximum vertex")?;
        let section_index = self.i32()?;
        let count = self.count(MAX_INDICES, 8, "section auxiliary pair")?;
        let mut auxiliary_pairs = reserve(count)?;
        for _ in 0..count {
            auxiliary_pairs.push([self.i32()?, self.i32()?]);
        }
        if self.u8()? != 0 {
            return Err(unsupported("PS3 mesh section payload is unsupported"));
        }
        Ok(Section {
            material,
            unknown_flags,
            shadow_casting,
            first_index,
            triangle_count,
            min_vertex,
            max_vertex,
            section_index,
            auxiliary_pairs,
        })
    }
    fn index_buffer(&mut self) -> io::Result<(Vec<u16>, Span)> {
        let (bytes, count) = self.bulk(2, MAX_INDICES)?;
        let span = Span {
            offset: self.position - bytes.len(),
            length: bytes.len(),
        };
        let mut indices = reserve(count)?;
        for pair in bytes.chunks_exact(2) {
            indices.push(u16::from_le_bytes([pair[0], pair[1]]));
        }
        Ok((indices, span))
    }
    fn lod(&mut self) -> io::Result<Lod> {
        let start = self.position;
        let source_bulk = self.raw_bulk()?;
        let count = self.count(MAX_SECTIONS, 41, "section")?;
        let mut sections = reserve(count)?;
        for _ in 0..count {
            sections.push(self.section()?);
        }
        let position_stride = self.positive("position stride")?;
        if position_stride != 12 {
            return Err(unsupported("mesh positions must be float3"));
        }
        let vertex_count = self.positive("position vertex count")?;
        let (bytes, count) = self.bulk(12, MAX_VERTICES)?;
        if count != vertex_count {
            return Err(invalid("position vertex counts disagree"));
        }
        let position_bytes = Span {
            offset: self.position - bytes.len(),
            length: bytes.len(),
        };
        let mut positions = reserve(count)?;
        for row in bytes.chunks_exact(12) {
            let p = std::array::from_fn(|i| {
                f32::from_le_bytes(row[i * 4..i * 4 + 4].try_into().unwrap())
            });
            if p.iter().any(|v| !v.is_finite()) {
                return Err(invalid("nonfinite vertex position"));
            }
            positions.push(p);
        }
        let uv_channels = self.positive("UV channel count")?;
        if uv_channels > MAX_UV_CHANNELS || (uv_channels == 0 && vertex_count != 0) {
            return Err(unsupported("unestablished UV channel count"));
        }
        let attribute_stride = self.positive("attribute stride")?;
        if self.positive("attribute vertex count")? != vertex_count {
            return Err(invalid("attribute vertex count disagrees with positions"));
        }
        let precision = match self.i32()? {
            0 => UvPrecision::Half,
            1 => UvPrecision::Float,
            _ => return Err(invalid("noncanonical UV precision flag")),
        };
        let uv_stride = if precision == UvPrecision::Half { 4 } else { 8 };
        let expected_stride = 8 + uv_channels * uv_stride;
        if attribute_stride != expected_stride {
            return Err(unsupported(
                "attribute stride disagrees with packed tangents and UVs",
            ));
        }
        let (bytes, count) = self.bulk(expected_stride, MAX_VERTICES)?;
        if count != vertex_count {
            return Err(invalid("attribute bulk count disagrees with vertex count"));
        }
        let attribute_bytes = Span {
            offset: self.position - bytes.len(),
            length: bytes.len(),
        };
        let mut tangents = reserve(count)?;
        let mut uv_bits = reserve(
            count
                .checked_mul(uv_channels)
                .ok_or_else(|| invalid("UV count overflow"))?,
        )?;
        for row in bytes.chunks_exact(expected_stride) {
            tangents.push([
                PackedNormal(row[0..4].try_into().unwrap()),
                PackedNormal(row[4..8].try_into().unwrap()),
            ]);
            for pair in row[8..].chunks_exact(uv_stride) {
                let raw = if precision == UvPrecision::Half {
                    [
                        u32::from(u16::from_le_bytes(pair[0..2].try_into().unwrap())),
                        u32::from(u16::from_le_bytes(pair[2..4].try_into().unwrap())),
                    ]
                } else {
                    [
                        u32::from_le_bytes(pair[0..4].try_into().unwrap()),
                        u32::from_le_bytes(pair[4..8].try_into().unwrap()),
                    ]
                };
                let decoded = if precision == UvPrecision::Half {
                    raw.map(|bits| half_to_float(bits as u16))
                } else {
                    raw.map(f32::from_bits)
                };
                if decoded.iter().any(|v| !v.is_finite()) {
                    return Err(invalid("nonfinite mesh UV"));
                }
                uv_bits.push(raw);
            }
        }
        let attributes = VertexAttributes {
            precision,
            uv_channels,
            tangents,
            uv_bits,
        };
        let color_stride = self.positive("color stride")?;
        let color_count = self.positive("color vertex count")?;
        let (colors_bgra, color_bytes) = if color_count == 0 {
            if !matches!(color_stride, 0 | 4) {
                return Err(unsupported("unestablished empty color stride"));
            }
            (Vec::new(), None)
        } else {
            if color_stride != 4 || color_count != vertex_count {
                return Err(invalid("color stream count or stride mismatch"));
            }
            let (bytes, count) = self.bulk(4, MAX_VERTICES)?;
            if count != color_count {
                return Err(invalid("color bulk count mismatch"));
            }
            (
                copy_records::<4>(bytes, count)?,
                Some(Span {
                    offset: self.position - bytes.len(),
                    length: bytes.len(),
                }),
            )
        };
        if self.positive("LOD vertex count")? != vertex_count {
            return Err(invalid("LOD vertex count disagrees with streams"));
        }
        let (triangle_indices, triangle_span) = self.index_buffer()?;
        let (wireframe_indices, wireframe_span) = self.index_buffer()?;
        let (additional_indices, additional_span) = self.index_buffer()?;
        if triangle_indices.len() % 3 != 0 || wireframe_indices.len() % 2 != 0 {
            return Err(invalid(
                "triangle or wireframe index topology count invalid",
            ));
        }
        if triangle_indices
            .iter()
            .chain(&wireframe_indices)
            .chain(&additional_indices)
            .any(|&index| usize::from(index) >= vertex_count)
        {
            return Err(invalid("mesh index outside vertex buffer"));
        }
        for section in &sections {
            let end = section
                .triangle_count
                .checked_mul(3)
                .and_then(|size| section.first_index.checked_add(size))
                .ok_or_else(|| invalid("section index range overflow"))?;
            if end > triangle_indices.len() {
                return Err(invalid("section exceeds triangle index buffer"));
            }
            if section.triangle_count != 0
                && (section.min_vertex > section.max_vertex || section.max_vertex >= vertex_count)
            {
                return Err(invalid("section vertex range invalid"));
            }
            if triangle_indices[section.first_index..end]
                .iter()
                .any(|&index| {
                    usize::from(index) < section.min_vertex
                        || usize::from(index) > section.max_vertex
                })
            {
                return Err(invalid("section index outside declared vertex range"));
            }
        }
        Ok(Lod {
            source_bulk,
            sections,
            positions,
            attributes,
            colors_bgra,
            color_stride,
            triangle_indices,
            wireframe_indices,
            additional_indices,
            serialized: Span {
                offset: start,
                length: self.position - start,
            },
            position_bytes,
            attribute_bytes,
            color_bytes,
            index_bytes: [triangle_span, wireframe_span, additional_span],
        })
    }
}

/// IEEE binary16 conversion, preserving signed zero and subnormal values.
fn half_to_float(value: u16) -> f32 {
    let sign = u32::from(value & 0x8000) << 16;
    let exponent = u32::from((value >> 10) & 31);
    let fraction = u32::from(value & 1023);
    let bits = match exponent {
        0 if fraction == 0 => sign,
        0 => {
            let shift = fraction.leading_zeros() - 21;
            let normalized = (fraction << shift) & 1023;
            sign | ((113 - shift) << 23) | (normalized << 13)
        }
        31 => sign | 0x7f80_0000 | (fraction << 13),
        _ => sign | ((exponent + 112) << 23) | (fraction << 13),
    };
    f32::from_bits(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        bytes: Vec<u8>,
        version: usize,
        bulk: usize,
        section: usize,
        position: usize,
        attributes: usize,
        indices: usize,
        end: usize,
    }

    fn words(bytes: &mut Vec<u8>, values: &[i32]) {
        for value in values {
            bytes.extend(value.to_le_bytes());
        }
    }
    fn patch(bytes: &mut [u8], offset: usize, value: i32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    // Original hand-built data, unrelated to any game asset. One triangle with
    // deliberately distinct normals/UVs, so component swaps cannot pass.
    fn fixture(full_uv: bool, colors: bool) -> Fixture {
        let mut bytes = Vec::new();
        for value in [0.0_f32, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0] {
            bytes.extend(value.to_le_bytes());
        }
        words(&mut bytes, &[0]);
        for value in [-1.0_f32, -1.0, -1.0, 1.0, 1.0, 1.0] {
            bytes.extend(value.to_le_bytes());
        }
        words(&mut bytes, &[6, 0, 8, 0]);
        let version = bytes.len();
        words(&mut bytes, &[18, 0, 0, 0, 0, 1]);
        let bulk = bytes.len();
        words(&mut bytes, &[0, 0, 0, 0, 1]);
        let section = bytes.len();
        words(&mut bytes, &[0, 7, 9, 1, 0, 1, 0, 2, 0, 1, 11, 13]);
        bytes.push(0);
        words(&mut bytes, &[12, 3, 12, 3]);
        let position = bytes.len();
        for vertex in [[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
            for component in vertex {
                bytes.extend(component.to_le_bytes());
            }
        }
        let stride = if full_uv { 16 } else { 12 };
        words(&mut bytes, &[1, stride, 3, i32::from(full_uv), stride, 3]);
        let attributes = bytes.len();
        for _ in 0..3 {
            bytes.extend([0, 127, 255, 255, 128, 0, 255, 0]);
            if full_uv {
                bytes.extend(0.5_f32.to_le_bytes());
                bytes.extend((-2.0_f32).to_le_bytes());
            } else {
                bytes.extend(0x3800_u16.to_le_bytes());
                bytes.extend(0xc000_u16.to_le_bytes());
            }
        }
        if colors {
            words(&mut bytes, &[4, 3, 4, 3]);
            bytes.extend([2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]);
        } else {
            words(&mut bytes, &[0, 0]);
        }
        words(&mut bytes, &[3, 2, 3]);
        let indices = bytes.len();
        bytes.extend([0, 0, 1, 0, 2, 0]);
        words(&mut bytes, &[2, 0, 2, 3]);
        bytes.extend([2, 0, 1, 0, 0, 0]);
        let end = bytes.len();
        bytes.extend([0xde, 0xad, 0xbe, 0xef]);
        Fixture {
            bytes,
            version,
            bulk,
            section,
            position,
            attributes,
            indices,
            end,
        }
    }

    #[test]
    fn preserves_geometry_components_and_opaque_tail() {
        let f = fixture(false, true);
        let mesh = StaticMesh::parse_native(&f.bytes, 1000).unwrap();
        let lod = &mesh.lods[0];
        assert_eq!(mesh.internal_version, 18);
        assert_eq!(mesh.parsed_bytes, f.end);
        assert_eq!(mesh.trailing_bytes, [0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(
            lod.positions,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
        );
        assert_eq!(lod.attributes.tangents[0][0].0, [0, 127, 255, 255]);
        assert_eq!(lod.attributes.tangents[0][1].0, [128, 0, 255, 0]);
        assert_eq!(lod.attributes.uv_bits[0], [0x3800, 0xc000]);
        assert_eq!(lod.attributes.uv(1, 0), Some([0.5, -2.0]));
        assert_eq!(lod.attributes.uv(3, 0), None);
        assert_eq!(lod.attributes.uv(0, 1), None);
        assert_eq!(lod.colors_bgra[1], [11, 13, 17, 19]);
        assert_eq!(lod.triangle_indices, [0, 1, 2]);
        assert_eq!(lod.additional_indices, [2, 1, 0]);
        assert_eq!(lod.sections[0].unknown_flags, [7, 9]);
        assert_eq!(lod.sections[0].auxiliary_pairs, [[11, 13]]);
        assert_eq!(lod.position_bytes.offset, f.position);
        assert_eq!(lod.attribute_bytes.offset, f.attributes);
        assert_eq!(lod.index_bytes[0].offset, f.indices);
    }

    #[test]
    fn full_precision_uv_branch_preserves_ieee_bits() {
        let f = fixture(true, false);
        let mesh = StaticMesh::parse_native(&f.bytes, 0).unwrap();
        let attributes = &mesh.lods[0].attributes;
        assert_eq!(attributes.precision, UvPrecision::Float);
        assert_eq!(
            attributes.uv_bits[2],
            [0.5_f32.to_bits(), (-2.0_f32).to_bits()]
        );
        assert_eq!(attributes.uv(2, 0), Some([0.5, -2.0]));
        assert!(mesh.lods[0].colors_bgra.is_empty());
    }

    #[test]
    fn optional_and_multiple_lods_remain_separate() {
        let f = fixture(false, false);
        let lod = &f.bytes[f.bulk..f.end];
        let mut bytes = f.bytes[..f.bulk].to_vec();
        patch(&mut bytes, f.bulk - 4, 2);
        bytes.extend(lod);
        bytes.extend(lod);
        bytes.extend(&f.bytes[f.end..]);
        let mesh = StaticMesh::parse_native(&bytes, 0).unwrap();
        assert_eq!(mesh.lods.len(), 2);
        assert!(mesh.extra_lod.is_none());
        assert_eq!(mesh.lods[1].serialized.offset, f.end);

        let mut bytes = f.bytes[..f.version + 8].to_vec();
        patch(&mut bytes, f.version + 4, 1);
        bytes.extend(lod);
        bytes.extend(&f.bytes[f.version + 8..]);
        let mesh = StaticMesh::parse_native(&bytes, 0).unwrap();
        assert_eq!(
            mesh.extra_lod.as_ref().unwrap().serialized.offset,
            f.version + 8
        );
        assert_eq!(mesh.lods.len(), 1);
        assert_eq!(mesh.parsed_bytes, f.end + lod.len());
    }

    #[test]
    fn every_geometry_prefix_is_rejected() {
        let f = fixture(false, false);
        for end in 0..f.end {
            assert!(
                StaticMesh::parse_native(&f.bytes[..end], 0).is_err(),
                "accepted {end}"
            );
        }
        let mesh = StaticMesh::parse_native(&f.bytes[..f.end], 0).unwrap();
        assert!(mesh.trailing_bytes.is_empty());
    }

    #[test]
    fn excessive_counts_and_unestablished_profiles_are_rejected() {
        let f = fixture(false, false);
        for (offset, value) in [
            (60, i32::MAX),
            (60, -1),
            (f.version, 19),
            (f.bulk, 2),
            (f.attributes - 20, 8),
        ] {
            let mut bytes = f.bytes.clone();
            patch(&mut bytes, offset, value);
            assert!(
                StaticMesh::parse_native(&bytes, 0).is_err(),
                "accepted {offset}:{value}"
            );
        }
        assert!(StaticMesh::parse_native(&f.bytes, usize::MAX).is_err());
    }

    #[test]
    fn out_of_range_indices_and_sections_are_rejected() {
        let f = fixture(false, false);
        let mut bytes = f.bytes.clone();
        bytes[f.indices + 4..f.indices + 6].copy_from_slice(&3_u16.to_le_bytes());
        assert!(StaticMesh::parse_native(&bytes, 0).is_err());
        for (offset, value) in [
            (f.section + 16, 1),
            (f.section + 20, 2),
            (f.section + 24, 1),
            (f.section + 28, 3),
        ] {
            let mut bytes = f.bytes.clone();
            patch(&mut bytes, offset, value);
            assert!(
                StaticMesh::parse_native(&bytes, 0).is_err(),
                "accepted section {offset}:{value}"
            );
        }
        let mut bytes = f.bytes.clone();
        bytes[f.section + 48] = 1;
        assert_eq!(
            StaticMesh::parse_native(&bytes, 0).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }

    #[test]
    fn mismatched_streams_and_nonfinite_values_are_rejected() {
        let f = fixture(false, false);
        for (offset, value) in [
            (f.position - 4, 2),
            (f.attributes - 4, 2),
            (f.attributes - 16, 4),
            (f.position, 0x7fc0_0000),
        ] {
            let mut bytes = f.bytes.clone();
            patch(&mut bytes, offset, value);
            assert!(
                StaticMesh::parse_native(&bytes, 0).is_err(),
                "accepted stream {offset}:{value}"
            );
        }
        let mut bytes = f.bytes.clone();
        bytes[f.attributes + 8..f.attributes + 10].copy_from_slice(&0x7c00_u16.to_le_bytes());
        assert!(StaticMesh::parse_native(&bytes, 0).is_err());
    }

    #[test]
    fn inline_source_bulk_is_preserved_and_external_bulk_rejected() {
        let f = fixture(false, false);
        let mut bytes = f.bytes.clone();
        patch(&mut bytes, f.bulk + 4, 3);
        patch(&mut bytes, f.bulk + 8, 3);
        patch(&mut bytes, f.bulk + 12, 1000 + f.bulk as i32 + 16);
        bytes.splice(f.bulk + 16..f.bulk + 16, [5, 7, 11]);
        let mesh = StaticMesh::parse_native(&bytes, 1000).unwrap();
        assert_eq!(mesh.lods[0].source_bulk.payload, [5, 7, 11]);
        assert_eq!(mesh.parsed_bytes, f.end + 3);
        assert_eq!(
            StaticMesh::parse_native(&bytes, 0).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }

    #[test]
    fn half_values_and_normal_extrema_are_exact() {
        assert_eq!(half_to_float(0).to_bits(), 0);
        assert_eq!(half_to_float(0x8000).to_bits(), 0x8000_0000);
        assert_eq!(half_to_float(1), 2.0_f32.powi(-24));
        assert_eq!(half_to_float(0x03ff), 1023.0 * 2.0_f32.powi(-24));
        assert_eq!(half_to_float(0x0400), 2.0_f32.powi(-14));
        assert_eq!(half_to_float(0x7bff), 65504.0);
        assert!(half_to_float(0x7c00).is_infinite());
        assert!(half_to_float(0x7e00).is_nan());
        assert_eq!(PackedNormal([0, 255, 0, 255]).xyz(), [-1.0, 1.0, -1.0]);
        assert_eq!(PackedNormal([0, 0, 0, 255]).w(), 1.0);
    }
}
