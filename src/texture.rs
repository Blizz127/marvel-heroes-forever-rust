//! Bounded Texture2D metadata and inline mip reads for the observed UE3 868/3 profile.
//!
//! Texture cache names do not imply external storage: the bulk flags decide.
//! No external files are opened by this module. Color transfer and sampling stay
//! explicit metadata; decoding does not apply gamma or alpha premultiplication.

use crate::upk::{
    Package,
    objects::{ObjectProperties, Property, PropertyList, TagDetail, Value},
};
use std::io;

#[path = "texture/bc.rs"]
pub mod bc;

pub const MAX_NATIVE_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_RGBA_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_MIPS: usize = 32;
pub const MAX_DIMENSION: u32 = 16_384;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn unsupported(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    Bgra8,
    Bc1,
    Bc2,
    Bc3,
}

impl PixelFormat {
    pub fn from_ue3_name(name: &str) -> io::Result<Self> {
        match name.to_ascii_lowercase().as_str() {
            "pf_a8r8g8b8" => Ok(Self::Bgra8),
            "pf_dxt1" => Ok(Self::Bc1),
            "pf_dxt3" => Ok(Self::Bc2),
            "pf_dxt5" => Ok(Self::Bc3),
            _ => Err(unsupported(format!(
                "unsupported Texture2D pixel format {name}"
            ))),
        }
    }
    pub fn encoded_len(self, width: u32, height: u32) -> io::Result<usize> {
        let rgba = rgba_len(width, height)?;
        if self == Self::Bgra8 {
            return Ok(rgba);
        }
        let blocks = ((width as usize + 3) / 4)
            .checked_mul((height as usize + 3) / 4)
            .ok_or_else(|| invalid("texture block count overflow"))?;
        blocks
            .checked_mul(if self == Self::Bc1 { 8 } else { 16 })
            .ok_or_else(|| invalid("texture byte count overflow"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Descriptor {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    /// Serialized property only. None means absent, not an assumed default.
    pub srgb: Option<bool>,
    pub never_stream: Option<bool>,
    pub cache_name: Option<String>,
    pub mip_tail_base_index: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BulkPayload {
    /// No bytes are serialized for an unused/zero-element bulk.
    Absent,
    /// Bytes exactly as stored. Compression, when present, is declared in flags.
    Inline(Vec<u8>),
    /// References an external TFC; -1 offset/size placeholders remain in BulkData.
    External,
    /// References a different logical range in the same package.
    Separate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BulkData {
    pub flags: u32,
    pub element_count: usize,
    pub stored_size: i32,
    pub stored_offset: i32,
    pub header_logical_offset: usize,
    pub payload: BulkPayload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mip {
    pub width: u32,
    pub height: u32,
    pub bulk: BulkData,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Texture2D {
    pub descriptor: Descriptor,
    pub source_art: BulkData,
    pub mips: Vec<Mip>,
    pub cache_guid: [u8; 16],
    pub cached_pvrtc_mips: Vec<Mip>,
    pub cached_flash_max_resolution: i32,
    pub cached_atitc_mips: Vec<Mip>,
    pub cached_flash_mips: BulkData,
    pub cached_etc_mips: Vec<Mip>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    /// Tightly packed row-major RGBA8. No transfer or premultiplication applied.
    pub pixels: Vec<u8>,
}

impl Texture2D {
    pub fn from_export(package: &Package, index: usize, bytes: &[u8]) -> io::Result<Self> {
        let object = ObjectProperties::parse_export(package, index, bytes)?;
        if !object.class_name.eq_ignore_ascii_case("texture2d") {
            return Err(unsupported("export is not a Texture2D"));
        }
        let props = &object.properties;
        let width = dimension(required_int(package, props, "SizeX")?)?;
        let height = dimension(required_int(package, props, "SizeY")?)?;
        let format_property = required(package, props, "Format")?;
        let TagDetail::Enumeration(enum_name) = format_property.detail else {
            return Err(invalid("Texture2D Format is not an enum property"));
        };
        if !package
            .resolve_name(enum_name)
            .is_some_and(|n| n.eq_ignore_ascii_case("EPixelFormat"))
        {
            return Err(invalid("Texture2D Format has the wrong enum type"));
        }
        let format_name = property_name(package, format_property)?;
        let descriptor = Descriptor {
            width,
            height,
            format: PixelFormat::from_ue3_name(&format_name)?,
            srgb: optional_bool(package, props, "SRGB")?,
            never_stream: optional_bool(package, props, "NeverStream")?,
            cache_name: unique(package, props, "TextureFileCacheName")?
                .map(|p| property_name(package, p))
                .transpose()?,
            mip_tail_base_index: unique(package, props, "MipTailBaseIdx")?
                .map(property_int)
                .transpose()?,
        };
        let logical_offset = object.native_payload.logical_offset(object.serial_offset)?;
        Self::parse_native(
            descriptor,
            object.native_payload.bytes(bytes)?,
            logical_offset,
        )
    }

    /// Parse the established native tail. Caller supplies property metadata and
    /// the tail's absolute logical offset, not a stored/compressed file offset.
    pub fn parse_native(
        descriptor: Descriptor,
        bytes: &[u8],
        logical_offset: usize,
    ) -> io::Result<Self> {
        rgba_len(descriptor.width, descriptor.height)?;
        if bytes.len() > MAX_NATIVE_BYTES {
            return Err(invalid("texture native payload exceeds limit"));
        }
        logical_offset
            .checked_add(bytes.len())
            .ok_or_else(|| invalid("native logical range overflow"))?;
        let mut reader = Reader {
            bytes,
            position: 0,
            logical_offset,
        };
        let source_art = reader.bulk()?;
        let mips = reader.mips()?;
        if mips.is_empty() {
            return Err(invalid("Texture2D has no primary mips"));
        }
        for mip in &mips {
            if mip.width > descriptor.width || mip.height > descriptor.height {
                return Err(invalid("mip dimensions exceed Texture2D dimensions"));
            }
            let expected = descriptor.format.encoded_len(mip.width, mip.height)?;
            if !matches!(mip.bulk.payload, BulkPayload::Absent)
                && mip.bulk.element_count != expected
            {
                return Err(invalid(
                    "mip byte count disagrees with dimensions/pixel format",
                ));
            }
        }
        let cache_guid = reader.take(16)?.try_into().unwrap();
        let cached_pvrtc_mips = reader.mips()?;
        let cached_flash_max_resolution = reader.i32()?;
        if cached_flash_max_resolution < 0 {
            return Err(invalid("negative cached Flash resolution"));
        }
        let cached_atitc_mips = reader.mips()?;
        let cached_flash_mips = reader.bulk()?;
        let cached_etc_mips = reader.mips()?;
        if reader.position != bytes.len() {
            return Err(unsupported(
                "unrecognized bytes after Texture2D native tail",
            ));
        }
        Ok(Self {
            descriptor,
            source_art,
            mips,
            cache_guid,
            cached_pvrtc_mips,
            cached_flash_max_resolution,
            cached_atitc_mips,
            cached_flash_mips,
            cached_etc_mips,
        })
    }

    pub fn decode_mip(&self, index: usize) -> io::Result<RgbaImage> {
        let mip = self
            .mips
            .get(index)
            .ok_or_else(|| invalid("mip index is out of bounds"))?;
        let bytes = match &mip.bulk.payload {
            BulkPayload::Inline(bytes) => bytes,
            BulkPayload::External => {
                return Err(unsupported(
                    "mip requires external TFC data and manifest resolution",
                ));
            }
            BulkPayload::Separate => {
                return Err(unsupported("mip requires a separate logical package range"));
            }
            BulkPayload::Absent => return Err(unsupported("mip has no serialized pixel payload")),
        };
        if mip.bulk.flags & (0x02 | 0x10 | 0x80) != 0 {
            return Err(unsupported(
                "individually compressed texture bulk is not decoded yet",
            ));
        }
        decode_pixels(self.descriptor.format, mip.width, mip.height, bytes)
    }
}

/// Decode already resolved and decompressed mip bytes. Callers obtaining bytes
/// from a texture cache must independently validate the cache identity and range.
pub fn decode_pixels(
    format: PixelFormat,
    width: u32,
    height: u32,
    bytes: &[u8],
) -> io::Result<RgbaImage> {
    let expected = format.encoded_len(width, height)?;
    if bytes.len() != expected {
        return Err(invalid("mip payload length does not match pixel format"));
    }
    let pixels = match format {
        PixelFormat::Bgra8 => {
            let mut pixels = Vec::new();
            pixels
                .try_reserve_exact(bytes.len())
                .map_err(|_| invalid("RGBA allocation failed"))?;
            for bgra in bytes.chunks_exact(4) {
                pixels.extend_from_slice(&[bgra[2], bgra[1], bgra[0], bgra[3]]);
            }
            pixels
        }
        PixelFormat::Bc1 => bc::decode(bc::Format::Bc1, width, height, bytes)?,
        PixelFormat::Bc2 => bc::decode(bc::Format::Bc2, width, height, bytes)?,
        PixelFormat::Bc3 => bc::decode(bc::Format::Bc3, width, height, bytes)?,
    };
    Ok(RgbaImage {
        width,
        height,
        pixels,
    })
}

fn rgba_len(width: u32, height: u32) -> io::Result<usize> {
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(invalid("texture dimensions are zero or exceed limit"));
    }
    let bytes = (width as usize)
        .checked_mul(height as usize)
        .and_then(|v| v.checked_mul(4))
        .ok_or_else(|| invalid("RGBA byte size overflow"))?;
    if bytes > MAX_RGBA_BYTES {
        return Err(invalid("RGBA image exceeds allocation limit"));
    }
    Ok(bytes)
}
fn dimension(value: i32) -> io::Result<u32> {
    let value = u32::try_from(value).map_err(|_| invalid("negative texture dimension"))?;
    if value == 0 || value > MAX_DIMENSION {
        return Err(invalid("texture dimension is zero or exceeds limit"));
    }
    Ok(value)
}
fn unique<'a>(
    package: &Package,
    props: &'a PropertyList,
    name: &str,
) -> io::Result<Option<&'a Property>> {
    let mut matching = props.entries.iter().filter(|p| {
        package
            .resolve_name(p.name)
            .is_some_and(|n| n.eq_ignore_ascii_case(name))
    });
    let first = matching.next();
    if matching.next().is_some() {
        return Err(invalid(format!("duplicate Texture2D property {name}")));
    }
    if first.is_some_and(|p| p.array_index != 0) {
        return Err(invalid("indexed scalar Texture2D property"));
    }
    Ok(first)
}
fn required<'a>(
    package: &Package,
    props: &'a PropertyList,
    name: &str,
) -> io::Result<&'a Property> {
    unique(package, props, name)?
        .ok_or_else(|| invalid(format!("missing Texture2D property {name}")))
}
fn property_int(p: &Property) -> io::Result<i32> {
    if let Value::Integer(value) = p.value {
        Ok(value)
    } else {
        Err(invalid("expected integer Texture2D property"))
    }
}
fn required_int(package: &Package, props: &PropertyList, name: &str) -> io::Result<i32> {
    property_int(required(package, props, name)?)
}
fn property_name(package: &Package, p: &Property) -> io::Result<String> {
    if let Value::Name(value) = p.value {
        package
            .resolve_name(value)
            .ok_or_else(|| invalid("unresolved Texture2D property name"))
    } else {
        Err(invalid("expected named Texture2D property value"))
    }
}
fn optional_bool(package: &Package, props: &PropertyList, name: &str) -> io::Result<Option<bool>> {
    unique(package, props, name)?
        .map(|p| match p.value {
            Value::Boolean(value) => Ok(value),
            _ => Err(invalid("expected boolean Texture2D property")),
        })
        .transpose()
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
    logical_offset: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, size: usize) -> io::Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(size)
            .ok_or_else(|| invalid("native read range overflow"))?;
        let data = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| invalid("truncated Texture2D native payload"))?;
        self.position = end;
        Ok(data)
    }
    fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn absolute_position(&self) -> io::Result<usize> {
        self.logical_offset
            .checked_add(self.position)
            .ok_or_else(|| invalid("logical offset overflow"))
    }
    fn bulk(&mut self) -> io::Result<BulkData> {
        let header_logical_offset = self.absolute_position()?;
        let flags = self.u32()?;
        if flags & !(0x01 | 0x02 | 0x10 | 0x20 | 0x40 | 0x80) != 0 {
            return Err(unsupported("unrecognized UE3 texture bulk flags"));
        }
        if (flags & (0x02 | 0x10 | 0x80)).count_ones() > 1 {
            return Err(invalid("multiple texture bulk compression codecs"));
        }
        if flags & 0x41 == 0x41 {
            return Err(invalid("bulk is both external and separately located"));
        }
        let element_count =
            usize::try_from(self.i32()?).map_err(|_| invalid("negative bulk element count"))?;
        if element_count > MAX_NATIVE_BYTES {
            return Err(invalid("bulk element count exceeds limit"));
        }
        let stored_size = self.i32()?;
        let stored_offset = self.i32()?;
        if stored_size < -1 || stored_offset < -1 {
            return Err(invalid("invalid negative bulk placeholder"));
        }
        let payload = if flags & 0x20 != 0 || element_count == 0 {
            BulkPayload::Absent
        } else if flags & 0x01 != 0 {
            BulkPayload::External
        } else if flags & 0x40 != 0 {
            if stored_offset < 0 || stored_size < 0 {
                return Err(invalid("invalid separate package bulk range"));
            }
            BulkPayload::Separate
        } else {
            let size =
                usize::try_from(stored_size).map_err(|_| invalid("negative inline bulk size"))?;
            if size > MAX_NATIVE_BYTES {
                return Err(invalid("inline bulk exceeds limit"));
            }
            if flags & (0x02 | 0x10 | 0x80) == 0 && size != element_count {
                return Err(invalid(
                    "uncompressed bulk byte count differs from stored size",
                ));
            }
            if usize::try_from(stored_offset).ok() != Some(self.absolute_position()?) {
                return Err(invalid(
                    "inline bulk offset does not match logical position",
                ));
            }
            let input = self.take(size)?;
            let mut data = Vec::new();
            data.try_reserve_exact(size)
                .map_err(|_| invalid("bulk allocation failed"))?;
            data.extend_from_slice(input);
            BulkPayload::Inline(data)
        };
        Ok(BulkData {
            flags,
            element_count,
            stored_size,
            stored_offset,
            header_logical_offset,
            payload,
        })
    }
    fn mips(&mut self) -> io::Result<Vec<Mip>> {
        let count = usize::try_from(self.i32()?).map_err(|_| invalid("negative mip count"))?;
        if count > MAX_MIPS || count > (self.bytes.len() - self.position) / 24 {
            return Err(invalid("mip count exceeds available bytes or limit"));
        }
        let mut mips = Vec::with_capacity(count);
        for _ in 0..count {
            let bulk = self.bulk()?;
            let width = dimension(self.i32()?)?;
            let height = dimension(self.i32()?)?;
            rgba_len(width, height)?;
            mips.push(Mip {
                width,
                height,
                bulk,
            });
        }
        Ok(mips)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn descriptor() -> Descriptor {
        Descriptor {
            width: 2,
            height: 1,
            format: PixelFormat::Bgra8,
            srgb: Some(false),
            never_stream: None,
            cache_name: None,
            mip_tail_base_index: None,
        }
    }
    fn put(out: &mut Vec<u8>, value: i32) {
        out.extend_from_slice(&value.to_le_bytes());
    }
    fn header(out: &mut Vec<u8>, flags: i32, count: i32, size: i32, offset: i32) {
        for v in [flags, count, size, offset] {
            put(out, v);
        }
    }
    fn native(external: bool) -> Vec<u8> {
        let mut out = Vec::new();
        header(&mut out, 0, 0, 0, 16);
        put(&mut out, 1);
        if external {
            header(&mut out, 0x11, 8, -1, -1);
        } else {
            header(&mut out, 0, 8, 8, 36);
            out.extend_from_slice(&[3, 2, 1, 4, 7, 6, 5, 8]);
        }
        put(&mut out, 2);
        put(&mut out, 1);
        out.extend_from_slice(&[0; 16]);
        put(&mut out, 0);
        put(&mut out, 0);
        put(&mut out, 0);
        header(&mut out, 0x21, 0, -1, -1);
        put(&mut out, 0);
        out
    }
    #[test]
    fn bgra_decodes_without_transfer_or_premultiplication() {
        let tex = Texture2D::parse_native(descriptor(), &native(false), 0).unwrap();
        assert_eq!(tex.decode_mip(0).unwrap().pixels, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(tex.descriptor.srgb, Some(false));
        assert!(tex.decode_mip(1).is_err());
    }
    #[test]
    fn external_placeholders_remain_explicit() {
        let tex = Texture2D::parse_native(descriptor(), &native(true), 0).unwrap();
        assert_eq!(tex.mips[0].bulk.stored_offset, -1);
        assert_eq!(tex.mips[0].bulk.payload, BulkPayload::External);
        assert_eq!(
            tex.decode_mip(0).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
    #[test]
    fn truncation_and_trailing_data_fail() {
        let bytes = native(false);
        for n in 0..bytes.len() {
            assert!(
                Texture2D::parse_native(descriptor(), &bytes[..n], 0).is_err(),
                "prefix {n}"
            );
        }
        let mut tail = bytes;
        tail.push(0);
        assert!(Texture2D::parse_native(descriptor(), &tail, 0).is_err());
    }
    #[test]
    fn malformed_bulk_and_dimensions_fail() {
        for (offset, value) in [(20, 4), (24, -1), (28, 7), (32, 37), (44, -1)] {
            let mut bytes = native(false);
            bytes[offset..offset + 4].copy_from_slice(&i32::to_le_bytes(value));
            assert!(
                Texture2D::parse_native(descriptor(), &bytes, 0).is_err(),
                "offset {offset}"
            );
        }
        let mut huge = descriptor();
        huge.width = 16_384;
        huge.height = 16_384;
        assert!(Texture2D::parse_native(huge, &native(false), 0).is_err());
        assert!(Texture2D::parse_native(descriptor(), &native(false), usize::MAX).is_err());
        assert!(PixelFormat::from_ue3_name("pf_unknown").is_err());
    }
}
