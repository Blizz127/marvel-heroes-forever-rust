//! Explicit Texture2D / manifest / cache-file binding for external LZO mips.

use super::{
    PayloadLimits, ResolvedMip, TextureCacheFile, TextureCacheManifest, invalid,
    validate_cache_name,
};
use crate::texture::{BulkPayload, Mip, RgbaImage, Texture2D, decode_pixels};
use crate::upk::Package;
use std::collections::HashSet;
use std::io;

const MAX_QUALIFIED_NAME_BYTES: usize = 4096;
const MAX_OUTER_DEPTH: usize = 64;

/// Derive the cooked object identity from saved outer references, without using
/// the containing filename or opening other packages. Dotted names within one
/// component are rejected because flattening them would lose path boundaries.
pub fn qualified_texture_name(package: &Package, export_index: usize) -> io::Result<String> {
    if !package
        .export_class_name(export_index)
        .is_some_and(|name| name.eq_ignore_ascii_case("Texture2D"))
    {
        return Err(invalid(
            "qualified texture identity requires a Texture2D export",
        ));
    }
    let mut reference = i32::try_from(
        export_index
            .checked_add(1)
            .ok_or_else(|| invalid("texture export index overflow"))?,
    )
    .map_err(|_| invalid("texture export index exceeds signed reference range"))?;
    let mut components = Vec::new();
    let mut visited = HashSet::new();
    let mut bytes = 0usize;
    while reference != 0 {
        if components.len() == MAX_OUTER_DEPTH || !visited.insert(reference) {
            return Err(invalid(
                "texture outer chain is cyclic or exceeds its limit",
            ));
        }
        let (name, outer) = if reference > 0 {
            let entry = package
                .exports
                .get(reference as usize - 1)
                .ok_or_else(|| invalid("texture export outer is out of bounds"))?;
            (entry.object_name, entry.outer_index)
        } else {
            let entry = package
                .imports
                .get(reference.unsigned_abs() as usize - 1)
                .ok_or_else(|| invalid("texture import outer is out of bounds"))?;
            (entry.object_name, entry.outer_index)
        };
        let name = package
            .resolve_name(name)
            .ok_or_else(|| invalid("texture outer name is unresolved"))?;
        if name.is_empty() || name.contains('.') || name.chars().any(char::is_control) {
            return Err(invalid(
                "texture outer name cannot form an unambiguous qualified name",
            ));
        }
        bytes = bytes
            .checked_add(name.len())
            .and_then(|count| count.checked_add(usize::from(!components.is_empty())))
            .ok_or_else(|| invalid("qualified texture name size overflow"))?;
        if bytes > MAX_QUALIFIED_NAME_BYTES {
            return Err(invalid("qualified texture name exceeds its byte limit"));
        }
        components.push(name);
        reference = outer;
    }
    components.reverse();
    Ok(components.join("."))
}

/// A checked binding of one explicit primary mip index. The same index is
/// queried in the manifest; no mip-tail remapping, lower-mip fallback, first-GUID
/// choice, cache search or format substitution is performed.
pub struct BoundExternalMip<'a> {
    texture: &'a Texture2D,
    mip_index: usize,
    resolved: ResolvedMip<'a>,
}

impl<'a> BoundExternalMip<'a> {
    pub fn resolve(
        texture: &'a Texture2D,
        mip_index: usize,
        manifest: &'a TextureCacheManifest,
        qualified_name: &str,
    ) -> io::Result<Self> {
        if qualified_name.is_empty()
            || qualified_name.len() > MAX_QUALIFIED_NAME_BYTES
            || qualified_name.chars().any(char::is_control)
        {
            return Err(invalid("invalid qualified texture name"));
        }
        let mip = texture
            .mips
            .get(mip_index)
            .ok_or_else(|| invalid("primary texture mip index is out of bounds"))?;
        if !matches!(mip.bulk.payload, BulkPayload::External) || mip.bulk.flags != 0x11 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "external binding requires an external mip with bulk flags 0x11",
            ));
        }
        let encoded_size = texture
            .descriptor
            .format
            .encoded_len(mip.width, mip.height)?;
        if mip.bulk.element_count != encoded_size {
            return Err(invalid(
                "external mip element count disagrees with its pixel format",
            ));
        }
        if mip.width > texture.descriptor.width || mip.height > texture.descriptor.height {
            return Err(invalid(
                "external mip dimensions exceed the texture descriptor",
            ));
        }
        let cache_name = texture
            .descriptor
            .cache_name
            .as_deref()
            .ok_or_else(|| invalid("external texture has no saved cache name"))?;
        validate_cache_name(cache_name)?;
        let serialized_index = u32::try_from(mip_index)
            .map_err(|_| invalid("primary mip index exceeds manifest index range"))?;
        let resolved = manifest
            .resolve_named(texture.cache_guid, serialized_index, qualified_name)?
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "no exact qualified texture/GUID/mip manifest record",
                )
            })?;
        if !cache_name.eq_ignore_ascii_case(&resolved.entry.cache_name) {
            return Err(invalid("texture and manifest cache names disagree"));
        }
        check_bulk_range(mip, resolved)?;
        Ok(Self {
            texture,
            mip_index,
            resolved,
        })
    }

    pub fn resolved(&self) -> ResolvedMip<'a> {
        self.resolved
    }
    pub fn mip_index(&self) -> usize {
        self.mip_index
    }
    pub fn mip(&self) -> &'a Mip {
        &self.texture.mips[self.mip_index]
    }

    /// The file is selected by the caller. TextureCacheFile checks its basename
    /// against both saved names and checks the actual file range before reading.
    pub fn read_encoded(&self, file: &mut TextureCacheFile) -> io::Result<Vec<u8>> {
        self.read_encoded_with_limits(file, PayloadLimits::default())
    }

    pub fn read_encoded_with_limits(
        &self,
        file: &mut TextureCacheFile,
        limits: PayloadLimits,
    ) -> io::Result<Vec<u8>> {
        let mip = self.mip();
        file.read_mip_with_limits(
            self.resolved,
            self.texture
                .descriptor
                .cache_name
                .as_deref()
                .expect("binding checked saved name"),
            mip.bulk.flags,
            mip.bulk.element_count,
            limits,
        )
    }

    /// Decode exact selected mip pixels; no gamma or alpha conversion is added.
    pub fn decode_rgba(&self, file: &mut TextureCacheFile) -> io::Result<RgbaImage> {
        self.decode_rgba_with_limits(file, PayloadLimits::default())
    }

    pub fn decode_rgba_with_limits(
        &self,
        file: &mut TextureCacheFile,
        limits: PayloadLimits,
    ) -> io::Result<RgbaImage> {
        let encoded = self.read_encoded_with_limits(file, limits)?;
        let mip = self.mip();
        decode_pixels(
            self.texture.descriptor.format,
            mip.width,
            mip.height,
            &encoded,
        )
    }
}

fn check_bulk_range(mip: &Mip, resolved: ResolvedMip<'_>) -> io::Result<()> {
    match (mip.bulk.stored_offset, mip.bulk.stored_size) {
        (-1, -1) => Ok(()), // Actual saved placeholder pair; manifest supplies the range.
        (offset, size) if offset >= 0 && size > 0 => {
            if offset as u64 != resolved.mip.offset || size as usize != resolved.mip.stored_size {
                return Err(invalid(
                    "explicit external bulk range disagrees with the manifest",
                ));
            }
            Ok(())
        }
        _ => Err(invalid(
            "external bulk range has an unsupported placeholder combination",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture::{BulkData, Descriptor, PixelFormat};

    fn texture() -> Texture2D {
        let absent = BulkData {
            flags: 0,
            element_count: 0,
            stored_size: 0,
            stored_offset: 0,
            header_logical_offset: 0,
            payload: BulkPayload::Absent,
        };
        Texture2D {
            descriptor: Descriptor {
                width: 4,
                height: 4,
                format: PixelFormat::Bgra8,
                srgb: None,
                never_stream: None,
                cache_name: Some("Icons".into()),
                mip_tail_base_index: None,
            },
            source_art: absent.clone(),
            mips: vec![Mip {
                width: 4,
                height: 4,
                bulk: BulkData {
                    flags: 0x11,
                    element_count: 64,
                    stored_size: -1,
                    stored_offset: -1,
                    header_logical_offset: 0,
                    payload: BulkPayload::External,
                },
            }],
            cache_guid: [5; 16],
            cached_pvrtc_mips: Vec::new(),
            cached_flash_max_resolution: 0,
            cached_atitc_mips: Vec::new(),
            cached_flash_mips: absent,
            cached_etc_mips: Vec::new(),
        }
    }
    fn string(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend_from_slice(&((value.len() + 1) as i32).to_le_bytes());
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(0);
    }
    fn manifest(names: &[&str], mip_index: i32) -> TextureCacheManifest {
        let mut bytes = (names.len() as i32).to_le_bytes().to_vec();
        for name in names {
            string(&mut bytes, name);
            bytes.extend_from_slice(&[5; 16]);
            string(&mut bytes, "Icons");
            for word in [1, mip_index, 100, 40] {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        TextureCacheManifest::parse(&bytes).unwrap()
    }

    #[test]
    fn requires_exact_name_guid_index_and_cache_without_fallback() {
        let mut texture = texture();
        let manifest = manifest(&["Group.One", "Group.Two"], 0);
        assert!(manifest.resolve_unique([5; 16], 0).is_err());
        let bound = BoundExternalMip::resolve(&texture, 0, &manifest, "group.one").unwrap();
        assert_eq!(bound.resolved().entry.texture_name, "Group.One");
        assert_eq!(bound.mip_index(), 0);
        assert_eq!(
            BoundExternalMip::resolve(&texture, 0, &manifest, "Group.Missing")
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::NotFound
        );
        assert!(BoundExternalMip::resolve(&texture, 1, &manifest, "Group.One").is_err());
        texture.cache_guid = [6; 16];
        assert_eq!(
            BoundExternalMip::resolve(&texture, 0, &manifest, "Group.One")
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::NotFound
        );
        texture.cache_guid = [5; 16];
        texture.descriptor.cache_name = Some("Other".into());
        assert!(BoundExternalMip::resolve(&texture, 0, &manifest, "Group.One").is_err());
        texture.descriptor.cache_name = Some("../Icons".into());
        assert!(BoundExternalMip::resolve(&texture, 0, &manifest, "Group.One").is_err());
        texture.descriptor.cache_name = None;
        assert!(BoundExternalMip::resolve(&texture, 0, &manifest, "Group.One").is_err());
    }

    #[test]
    fn rejects_ambiguous_named_records_and_missing_explicit_mip() {
        let mut texture = texture();
        let duplicates = manifest(&["Group.One", "group.one"], 0);
        assert!(BoundExternalMip::resolve(&texture, 0, &duplicates, "Group.One").is_err());
        let sparse = manifest(&["Group.One"], 2);
        texture.mips = vec![texture.mips[0].clone(); 3];
        assert_eq!(
            BoundExternalMip::resolve(&texture, 0, &sparse, "Group.One")
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(
            BoundExternalMip::resolve(&texture, 2, &sparse, "Group.One")
                .unwrap()
                .resolved()
                .mip
                .index,
            2
        );
    }

    #[test]
    fn checks_bulk_kind_size_and_explicit_range_assertions() {
        let original = texture();
        let manifest = manifest(&["Group.One"], 0);
        for (offset, size) in [
            (-1, 40),
            (100, -1),
            (-2, -2),
            (101, 40),
            (100, 39),
            (100, 0),
        ] {
            let mut texture = original.clone();
            texture.mips[0].bulk.stored_offset = offset;
            texture.mips[0].bulk.stored_size = size;
            assert!(BoundExternalMip::resolve(&texture, 0, &manifest, "Group.One").is_err());
        }
        let mut texture = original.clone();
        texture.mips[0].bulk.stored_offset = 100;
        texture.mips[0].bulk.stored_size = 40;
        assert!(BoundExternalMip::resolve(&texture, 0, &manifest, "Group.One").is_ok());
        texture = original.clone();
        texture.mips[0].bulk.element_count = 63;
        assert!(BoundExternalMip::resolve(&texture, 0, &manifest, "Group.One").is_err());
        texture = original.clone();
        texture.mips[0].bulk.flags = 0x01;
        assert_eq!(
            BoundExternalMip::resolve(&texture, 0, &manifest, "Group.One")
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::Unsupported
        );
        texture = original.clone();
        texture.mips[0].bulk.payload = BulkPayload::Inline(vec![0; 64]);
        assert!(BoundExternalMip::resolve(&texture, 0, &manifest, "Group.One").is_err());
        texture = original.clone();
        texture.mips[0].width = 0;
        assert!(BoundExternalMip::resolve(&texture, 0, &manifest, "Group.One").is_err());
        texture = original;
        texture.descriptor.width = 2;
        assert!(BoundExternalMip::resolve(&texture, 0, &manifest, "Group.One").is_err());
    }
}
