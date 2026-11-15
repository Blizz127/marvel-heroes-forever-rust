//! Original reader for the external 0x11 bulk-data profile observed in Icons.tfc.

use super::{ResolvedMip, invalid, validate_cache_name};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

const CHUNK_TAG: u32 = 0x9e2a83c1;
const BLOCK_BYTES: usize = 0x20000;
const EXTERNAL_LZO_FLAGS: u32 = 0x11;

#[derive(Clone, Copy, Debug)]
pub struct PayloadLimits {
    pub max_stored_bytes: usize,
    pub max_decoded_bytes: usize,
    pub max_blocks: usize,
}

impl Default for PayloadLimits {
    fn default() -> Self {
        Self {
            max_stored_bytes: 64 * 1024 * 1024,
            max_decoded_bytes: 128 * 1024 * 1024,
            max_blocks: 1024,
        }
    }
}

/// An explicitly selected, read-only cache file. No manifest-derived path is
/// joined or searched. Name agreement establishes selection, not authenticity;
/// callers remain responsible for selecting their private verified asset pin.
pub struct TextureCacheFile {
    file: File,
    cache_name: String,
}

impl TextureCacheFile {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| invalid("cache path requires a Unicode file basename"))?;
        validate_cache_name(name)?;
        if !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("tfc"))
        {
            return Err(invalid("cache path must name a .tfc file"));
        }
        let file = File::open(path)?;
        if !file.metadata()?.is_file() {
            return Err(invalid("cache path is not a regular file"));
        }
        Ok(Self {
            file,
            cache_name: name.to_owned(),
        })
    }

    pub fn cache_name(&self) -> &str {
        &self.cache_name
    }

    /// Read the resolved range only, then decode the external LZO payload.
    /// `texture_cache_name`, flags and decoded size come from the matching
    /// Texture2D descriptor/mip; the manifest cannot provide those assertions.
    pub fn read_mip(
        &mut self,
        resolved: ResolvedMip<'_>,
        texture_cache_name: &str,
        bulk_flags: u32,
        decoded_size: usize,
    ) -> io::Result<Vec<u8>> {
        self.read_mip_with_limits(
            resolved,
            texture_cache_name,
            bulk_flags,
            decoded_size,
            PayloadLimits::default(),
        )
    }

    pub fn read_mip_with_limits(
        &mut self,
        resolved: ResolvedMip<'_>,
        texture_cache_name: &str,
        bulk_flags: u32,
        decoded_size: usize,
        limits: PayloadLimits,
    ) -> io::Result<Vec<u8>> {
        check_names(&self.cache_name, texture_cache_name, resolved)?;
        check_sizes(resolved.mip.stored_size, bulk_flags, decoded_size, limits)?;
        let file_bytes = self.file.metadata()?.len();
        let stored = read_range(&mut self.file, file_bytes, resolved, limits)?;
        decode_lzo_mip_with_limits(&stored, bulk_flags, decoded_size, limits)
    }
}

fn check_names(
    selected_cache_name: &str,
    texture_cache_name: &str,
    resolved: ResolvedMip<'_>,
) -> io::Result<()> {
    for name in [
        selected_cache_name,
        texture_cache_name,
        &resolved.entry.cache_name,
    ] {
        validate_cache_name(name)?;
    }
    if !selected_cache_name.eq_ignore_ascii_case(&resolved.entry.cache_name)
        || !texture_cache_name.eq_ignore_ascii_case(&resolved.entry.cache_name)
    {
        return Err(invalid(
            "texture, manifest and selected cache names disagree",
        ));
    }
    Ok(())
}

fn read_range(
    source: &mut (impl Read + Seek),
    file_bytes: u64,
    resolved: ResolvedMip<'_>,
    limits: PayloadLimits,
) -> io::Result<Vec<u8>> {
    if resolved.mip.stored_size == 0 || resolved.mip.stored_size > limits.max_stored_bytes {
        return Err(invalid("stored mip exceeds the payload byte budget"));
    }
    let range = resolved.mip.checked_range(file_bytes)?;
    let mut stored = Vec::new();
    stored
        .try_reserve_exact(resolved.mip.stored_size)
        .map_err(allocation)?;
    stored.resize(resolved.mip.stored_size, 0);
    source.seek(SeekFrom::Start(range.start))?;
    source.read_exact(&mut stored)?;
    Ok(stored)
}

/// Decode one already-selected stored mip. This lower-level API does not infer
/// a cache identity or locate its range; use TextureCacheFile for full-file reads.
pub fn decode_lzo_mip(stored: &[u8], bulk_flags: u32, decoded_size: usize) -> io::Result<Vec<u8>> {
    decode_lzo_mip_with_limits(stored, bulk_flags, decoded_size, PayloadLimits::default())
}

pub fn decode_lzo_mip_with_limits(
    stored: &[u8],
    bulk_flags: u32,
    decoded_size: usize,
    limits: PayloadLimits,
) -> io::Result<Vec<u8>> {
    check_sizes(stored.len(), bulk_flags, decoded_size, limits)?;
    if word(stored, 0)? != CHUNK_TAG {
        return Err(invalid("invalid texture-cache compression tag"));
    }
    if size(stored, 4)? != BLOCK_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "only observed 128 KiB texture-cache compression blocks are supported",
        ));
    }
    let stored_total = size(stored, 8)?;
    if size(stored, 12)? != decoded_size {
        return Err(invalid("cache envelope and texture decoded sizes disagree"));
    }
    let block_count = decoded_size.div_ceil(BLOCK_BYTES);
    if block_count > limits.max_blocks {
        return Err(invalid("texture-cache block count exceeds its budget"));
    }
    let header_bytes = block_count
        .checked_mul(8)
        .and_then(|bytes| bytes.checked_add(16))
        .ok_or_else(|| invalid("texture-cache block table size overflow"))?;
    if header_bytes > stored.len() {
        return Err(invalid("truncated texture-cache block table"));
    }

    // Check all framing before allocating output or invoking any codec.
    let mut compressed_sum = 0usize;
    let mut decoded_sum = 0usize;
    for index in 0..block_count {
        let compressed = size(stored, 16 + index * 8)?;
        let decoded = size(stored, 20 + index * 8)?;
        if compressed == 0 || decoded != BLOCK_BYTES.min(decoded_size - decoded_sum) {
            return Err(invalid("invalid texture-cache block length"));
        }
        compressed_sum = compressed_sum
            .checked_add(compressed)
            .ok_or_else(|| invalid("texture-cache compressed size overflow"))?;
        decoded_sum += decoded;
    }
    if compressed_sum != stored_total
        || decoded_sum != decoded_size
        || header_bytes.checked_add(compressed_sum) != Some(stored.len())
    {
        return Err(invalid(
            "texture-cache block totals disagree with the stored range",
        ));
    }

    let mut output = Vec::new();
    output.try_reserve_exact(decoded_size).map_err(allocation)?;
    let mut position = header_bytes;
    for index in 0..block_count {
        let compressed = size(stored, 16 + index * 8)?;
        let decoded = size(stored, 20 + index * 8)?;
        let end = position + compressed; // Complete checked sum was validated above.
        let block = crate::upk::decompress_lzo(&stored[position..end], decoded)?;
        output.extend_from_slice(&block);
        position = end;
    }
    Ok(output)
}

fn check_sizes(
    stored_size: usize,
    bulk_flags: u32,
    decoded_size: usize,
    limits: PayloadLimits,
) -> io::Result<()> {
    if bulk_flags != EXTERNAL_LZO_FLAGS {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "only observed external LZO bulk flags 0x11 are supported",
        ));
    }
    if stored_size == 0 || stored_size > limits.max_stored_bytes {
        return Err(invalid("stored mip exceeds the payload byte budget"));
    }
    if decoded_size == 0 || decoded_size > limits.max_decoded_bytes {
        return Err(invalid("decoded mip exceeds the payload byte budget"));
    }
    if decoded_size.div_ceil(BLOCK_BYTES) > limits.max_blocks {
        return Err(invalid("texture-cache block count exceeds its budget"));
    }
    Ok(())
}

fn word(bytes: &[u8], offset: usize) -> io::Result<u32> {
    let bytes = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| invalid("truncated texture-cache envelope"))?;
    Ok(u32::from_le_bytes(
        bytes.try_into().expect("fixed word length"),
    ))
}

fn size(bytes: &[u8], offset: usize) -> io::Result<usize> {
    usize::try_from(word(bytes, offset)? as i32)
        .map_err(|_| invalid("negative texture-cache envelope size"))
}

fn allocation(error: std::collections::TryReserveError) -> io::Error {
    io::Error::new(
        io::ErrorKind::OutOfMemory,
        format!("cache payload allocation failed: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture_cache::{CacheEntry, MipLocation};
    use std::io::Cursor;

    fn envelope() -> Vec<u8> {
        // Original synthetic raw-LZO stream: four literals, then canonical end.
        let raw = [21, 1, 2, 3, 4, 17, 0, 0];
        let mut bytes = Vec::new();
        for value in [
            CHUNK_TAG,
            BLOCK_BYTES as u32,
            raw.len() as u32,
            4,
            raw.len() as u32,
            4,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&raw);
        bytes
    }

    #[test]
    fn decodes_exact_mip_and_rejects_truncation_trailing_and_invalid_framing() {
        let bytes = envelope();
        assert_eq!(decode_lzo_mip(&bytes, 0x11, 4).unwrap(), [1, 2, 3, 4]);
        for end in 0..bytes.len() {
            assert!(decode_lzo_mip(&bytes[..end], 0x11, 4).is_err(), "cut {end}");
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(decode_lzo_mip(&trailing, 0x11, 4).is_err());
        for (offset, value) in [
            (0, 0),
            (4, 0),
            (8, 7),
            (12, 5),
            (16, 0),
            (16, u32::MAX),
            (20, 5),
        ] {
            let mut bad = bytes.clone();
            bad[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            assert!(decode_lzo_mip(&bad, 0x11, 4).is_err(), "word {offset}");
        }
        let mut bad_stream = bytes;
        *bad_stream.last_mut().unwrap() = 1;
        assert!(decode_lzo_mip(&bad_stream, 0x11, 4).is_err());
    }

    #[test]
    fn enforces_bulk_profile_and_all_resource_limits() {
        let bytes = envelope();
        for flags in [0, 1, 0x10, 0x13, 0x91] {
            assert_eq!(
                decode_lzo_mip(&bytes, flags, 4).unwrap_err().kind(),
                io::ErrorKind::Unsupported
            );
        }
        assert!(decode_lzo_mip(&bytes, 0x11, 0).is_err());
        assert!(decode_lzo_mip(&bytes, 0x11, usize::MAX).is_err());
        for limits in [
            PayloadLimits {
                max_stored_bytes: bytes.len() - 1,
                ..Default::default()
            },
            PayloadLimits {
                max_decoded_bytes: 3,
                ..Default::default()
            },
            PayloadLimits {
                max_blocks: 0,
                ..Default::default()
            },
        ] {
            assert!(decode_lzo_mip_with_limits(&bytes, 0x11, 4, limits).is_err());
        }
    }

    #[test]
    fn selected_cache_names_and_actual_read_bounds_must_agree() {
        let entry = CacheEntry {
            texture_name: "P.T".into(),
            guid: [7; 16],
            cache_name: "Icons".into(),
            mips: vec![],
        };
        let mip = MipLocation {
            index: 0,
            offset: 3,
            stored_size: 4,
        };
        let resolved = ResolvedMip {
            entry: &entry,
            mip: &mip,
        };
        check_names("Icons", "icons", resolved).unwrap();
        assert!(check_names("Other", "icons", resolved).is_err());
        assert!(check_names("Icons", "Other", resolved).is_err());
        assert!(check_names("../Icons", "icons", resolved).is_err());
        let mut source = Cursor::new(vec![9, 9, 9, 1, 2, 3, 4, 8]);
        assert_eq!(
            read_range(&mut source, 8, resolved, PayloadLimits::default()).unwrap(),
            [1, 2, 3, 4]
        );
        assert!(read_range(&mut source, 6, resolved, PayloadLimits::default()).is_err());
        assert!(
            read_range(
                &mut Cursor::new(vec![0; 5]),
                8,
                resolved,
                PayloadLimits::default()
            )
            .is_err()
        );
        assert!(TextureCacheFile::open("Icons.upk").is_err());
    }
}
