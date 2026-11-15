//! Bounded reader for the observed Marvel Heroes texture-cache manifest.
//!
//! A GUID alone is not unique in the owner's manifest. Ambiguous lookups fail;
//! callers can select an exact qualified texture name instead of guessing.
//! Named cache reads validate stored byte ranges before decoding the observed
//! external LZO mip format. Other payload flag combinations are unsupported.

#[path = "texture_cache/binding.rs"]
mod binding;
pub use binding::{BoundExternalMip, qualified_texture_name};

#[path = "texture_cache/payload.rs"]
mod payload;
pub use payload::{PayloadLimits, TextureCacheFile, decode_lzo_mip, decode_lzo_mip_with_limits};

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::ops::Range;
use std::path::Path;

pub type TextureGuid = [u8; 16];

#[derive(Clone, Copy, Debug)]
pub struct ManifestLimits {
    pub max_file_bytes: usize,
    pub max_entries: usize,
    pub max_string_bytes: usize,
    pub max_mips_per_entry: usize,
    pub max_total_mips: usize,
}

impl Default for ManifestLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: 64 * 1024 * 1024,
            max_entries: 500_000,
            max_string_bytes: 4096,
            max_mips_per_entry: 64,
            max_total_mips: 2_000_000,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MipLocation {
    /// Serialized mip index, not an index into this record's mip vector.
    pub index: u32,
    pub offset: u64,
    /// Stored byte count. It is not the decoded pixel byte count.
    pub stored_size: usize,
}

impl MipLocation {
    /// Validate a declared range against the caller's actual cache-file length.
    pub fn checked_range(&self, cache_file_bytes: u64) -> io::Result<Range<u64>> {
        let length = u64::try_from(self.stored_size)
            .map_err(|_| invalid("stored mip length does not fit a file offset"))?;
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| invalid("stored mip range overflow"))?;
        if end > cache_file_bytes {
            return Err(invalid("stored mip range exceeds the selected cache file"));
        }
        Ok(self.offset..end)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheEntry {
    /// Original qualified texture identifier, independent of the cache filename.
    pub texture_name: String,
    /// Raw serialized FGuid bytes, in the same order as Texture2D's cache GUID.
    pub guid: TextureGuid,
    /// Cache basename as stored in the manifest, without the .tfc suffix.
    pub cache_name: String,
    pub mips: Vec<MipLocation>,
}

impl CacheEntry {
    /// A validated basename only; no directory or extraction operation is implied.
    pub fn cache_file_name(&self) -> String {
        format!("{}.tfc", self.cache_name)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ResolvedMip<'a> {
    pub entry: &'a CacheEntry,
    pub mip: &'a MipLocation,
}

pub struct TextureCacheManifest {
    entries: Vec<CacheEntry>,
    by_guid: HashMap<TextureGuid, Vec<usize>>,
    mip_count: usize,
}

impl TextureCacheManifest {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::open_with_limits(path, ManifestLimits::default())
    }

    pub fn open_with_limits(path: impl AsRef<Path>, limits: ManifestLimits) -> io::Result<Self> {
        let file = File::open(path)?;
        let length = usize::try_from(file.metadata()?.len())
            .map_err(|_| invalid("manifest is too large for this process"))?;
        if length > limits.max_file_bytes {
            return Err(invalid("manifest exceeds the file byte limit"));
        }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(length).map_err(allocation)?;
        let read_limit = u64::try_from(limits.max_file_bytes)
            .ok()
            .and_then(|length| length.checked_add(1))
            .ok_or_else(|| invalid("file byte limit overflow"))?;
        file.take(read_limit).read_to_end(&mut bytes)?;
        Self::parse_with_limits(&bytes, limits)
    }

    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        Self::parse_with_limits(bytes, ManifestLimits::default())
    }

    pub fn parse_with_limits(bytes: &[u8], limits: ManifestLimits) -> io::Result<Self> {
        if bytes.len() > limits.max_file_bytes {
            return Err(invalid("manifest exceeds the file byte limit"));
        }
        let mut input = Input {
            bytes,
            position: 0,
            string_limit: limits.max_string_bytes,
        };
        let count = input.nonnegative("negative manifest entry count")?;
        // Two FString lengths, two terminators, GUID and mip count are required.
        if count > limits.max_entries || count > input.remaining() / 30 {
            return Err(invalid(
                "manifest entry count exceeds its bytes or configured limit",
            ));
        }
        let mut entries = Vec::new();
        entries.try_reserve_exact(count).map_err(allocation)?;
        let mut by_guid: HashMap<TextureGuid, Vec<usize>> = HashMap::new();
        by_guid.try_reserve(count).map_err(allocation)?;
        let mut mip_count = 0usize;
        for entry_index in 0..count {
            let texture_name = input.string()?;
            if texture_name.is_empty() {
                return Err(invalid("empty qualified texture name"));
            }
            let guid: TextureGuid = input.take(16)?.try_into().expect("fixed input length");
            // The actual 1.52 manifest contains this additional FString. Older
            // reference readers omit it; do not reinterpret its bytes as a count.
            let cache_name = input.string()?;
            validate_cache_name(&cache_name)?;
            let count = input.nonnegative("negative mip count")?;
            mip_count = mip_count
                .checked_add(count)
                .ok_or_else(|| invalid("total mip count overflow"))?;
            if count > limits.max_mips_per_entry
                || count > input.remaining() / 12
                || mip_count > limits.max_total_mips
            {
                return Err(invalid("mip count exceeds its bytes or configured limit"));
            }
            let mut mips: Vec<MipLocation> = Vec::new();
            mips.try_reserve_exact(count).map_err(allocation)?;
            for _ in 0..count {
                let index = input.nonnegative("negative mip index")?;
                let offset = input.nonnegative("negative stored mip offset")?;
                let stored_size = input.nonnegative("negative stored mip size")?;
                if stored_size == 0 {
                    return Err(invalid("empty stored mip range"));
                }
                let index = u32::try_from(index).map_err(|_| invalid("mip index overflow"))?;
                if mips.iter().any(|mip| mip.index == index) {
                    return Err(invalid("duplicate mip index within one manifest entry"));
                }
                mips.push(MipLocation {
                    index,
                    offset: offset as u64,
                    stored_size,
                });
            }
            let indices = by_guid.entry(guid).or_default();
            indices.try_reserve(1).map_err(allocation)?;
            indices.push(entry_index);
            entries.push(CacheEntry {
                texture_name,
                guid,
                cache_name,
                mips,
            });
        }
        if input.remaining() != 0 {
            return Err(invalid("trailing bytes after texture-cache manifest"));
        }
        Ok(Self {
            entries,
            by_guid,
            mip_count,
        })
    }

    pub fn entries(&self) -> &[CacheEntry] {
        &self.entries
    }
    pub fn mip_count(&self) -> usize {
        self.mip_count
    }
    pub fn unique_guid_count(&self) -> usize {
        self.by_guid.len()
    }

    /// Preserve every match; repeated GUIDs are observed in the actual manifest.
    pub fn candidates(
        &self,
        guid: TextureGuid,
        mip_index: u32,
    ) -> impl Iterator<Item = ResolvedMip<'_>> + '_ {
        self.by_guid
            .get(&guid)
            .into_iter()
            .flatten()
            .flat_map(move |&index| {
                let entry = &self.entries[index];
                entry
                    .mips
                    .iter()
                    .filter(move |mip| mip.index == mip_index)
                    .map(move |mip| ResolvedMip { entry, mip })
            })
    }

    /// A GUID/mip query succeeds only when exactly one record matches.
    pub fn resolve_unique(
        &self,
        guid: TextureGuid,
        mip_index: u32,
    ) -> io::Result<Option<ResolvedMip<'_>>> {
        unique(self.candidates(guid, mip_index))
    }

    /// Select a qualified object identifier using explicit ASCII-insensitive matching.
    /// Multiple matching records remain an error, even when ranges happen to agree.
    /// Non-ASCII spelling is compared exactly except for ASCII letters.
    pub fn resolve_named(
        &self,
        guid: TextureGuid,
        mip_index: u32,
        texture_name: &str,
    ) -> io::Result<Option<ResolvedMip<'_>>> {
        unique(
            self.candidates(guid, mip_index)
                .filter(|found| found.entry.texture_name.eq_ignore_ascii_case(texture_name)),
        )
    }
}

fn unique<'a>(
    mut found: impl Iterator<Item = ResolvedMip<'a>>,
) -> io::Result<Option<ResolvedMip<'a>>> {
    let first = found.next();
    if found.next().is_some() {
        return Err(invalid(
            "ambiguous texture GUID/mip lookup; select a unique qualified texture name",
        ));
    }
    Ok(first)
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn allocation(error: std::collections::TryReserveError) -> io::Error {
    invalid(format!("manifest allocation failed: {error}"))
}

fn validate_cache_name(name: &str) -> io::Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.ends_with(['.', ' '])
        || name
            .chars()
            .any(|c| c.is_control() || "/\\:<>\"|?*".contains(c))
    {
        return Err(invalid("cache name is not a safe file basename"));
    }
    Ok(())
}

struct Input<'a> {
    bytes: &'a [u8],
    position: usize,
    string_limit: usize,
}
impl<'a> Input<'a> {
    fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }
    fn take(&mut self, count: usize) -> io::Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(count)
            .ok_or_else(|| invalid("manifest cursor overflow"))?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| invalid(format!("truncated manifest at byte {}", self.position)))?;
        self.position = end;
        Ok(bytes)
    }
    fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(
            self.take(4)?.try_into().expect("fixed input length"),
        ))
    }
    fn nonnegative(&mut self, message: &'static str) -> io::Result<usize> {
        usize::try_from(self.i32()?).map_err(|_| invalid(message))
    }
    fn string(&mut self) -> io::Result<String> {
        let encoded_length = self.i32()?;
        if encoded_length == 0 {
            return Ok(String::new());
        }
        let units = usize::try_from(encoded_length.unsigned_abs())
            .map_err(|_| invalid("string length overflow"))?;
        let byte_count = units
            .checked_mul(if encoded_length < 0 { 2 } else { 1 })
            .ok_or_else(|| invalid("string byte length overflow"))?;
        if byte_count > self.string_limit {
            return Err(invalid("manifest string exceeds configured byte limit"));
        }
        let bytes = self.take(byte_count)?;
        let value = if encoded_length < 0 {
            if bytes[bytes.len() - 2..] != [0, 0] {
                return Err(invalid("unterminated UTF-16 manifest string"));
            }
            let units: Vec<_> = bytes[..bytes.len() - 2]
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect();
            String::from_utf16(&units).map_err(|_| invalid("invalid UTF-16 manifest string"))?
        } else {
            if bytes.last() != Some(&0) {
                return Err(invalid("unterminated narrow manifest string"));
            }
            let value = &bytes[..bytes.len() - 1];
            if !value.is_ascii() {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "non-ASCII narrow manifest string requires an established code page",
                ));
            }
            String::from_utf8(value.to_vec()).expect("ASCII is UTF-8")
        };
        if value.chars().any(char::is_control) {
            return Err(invalid("control character inside manifest string"));
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&i32::try_from(value.len() + 1).unwrap().to_le_bytes());
        output.extend_from_slice(value.as_bytes());
        output.push(0);
    }
    fn record(
        output: &mut Vec<u8>,
        name: &str,
        guid: TextureGuid,
        cache: &str,
        mips: &[(i32, i32, i32)],
    ) {
        string(output, name);
        output.extend_from_slice(&guid);
        string(output, cache);
        output.extend_from_slice(&i32::try_from(mips.len()).unwrap().to_le_bytes());
        for (index, offset, size) in mips {
            for value in [index, offset, size] {
                output.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
    fn one() -> Vec<u8> {
        let mut bytes = 1i32.to_le_bytes().to_vec();
        record(
            &mut bytes,
            "Package.Texture",
            [7; 16],
            "Cache",
            &[(3, 40, 9), (5, 0, 4)],
        );
        bytes
    }

    #[test]
    fn named_lookup_keeps_serialized_mip_indices_and_checks_ranges() {
        let manifest = TextureCacheManifest::parse(&one()).unwrap();
        assert_eq!(manifest.entries().len(), 1);
        assert_eq!(manifest.mip_count(), 2);
        let found = manifest.resolve_unique([7; 16], 3).unwrap().unwrap();
        assert_eq!(found.entry.cache_file_name(), "Cache.tfc");
        assert_eq!(found.mip.checked_range(49).unwrap(), 40..49);
        assert!(found.mip.checked_range(48).is_err());
        assert!(manifest.resolve_unique([7; 16], 0).unwrap().is_none());
        assert!(manifest.resolve_unique([1; 16], 3).unwrap().is_none());
        assert!(
            manifest
                .resolve_named([7; 16], 3, "package.texture")
                .unwrap()
                .is_some()
        );
        assert!(
            manifest
                .resolve_named([7; 16], 3, "Package.Other")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn duplicate_guids_require_a_unique_qualified_name() {
        let mut bytes = 2i32.to_le_bytes().to_vec();
        record(&mut bytes, "One.Texture", [3; 16], "Cache", &[(1, 0, 10)]);
        record(&mut bytes, "Two.Texture", [3; 16], "Cache", &[(1, 10, 10)]);
        let manifest = TextureCacheManifest::parse(&bytes).unwrap();
        assert_eq!(manifest.unique_guid_count(), 1);
        assert_eq!(manifest.candidates([3; 16], 1).count(), 2);
        assert!(manifest.resolve_unique([3; 16], 1).is_err());
        assert_eq!(
            manifest
                .resolve_named([3; 16], 1, "two.texture")
                .unwrap()
                .unwrap()
                .mip
                .offset,
            10
        );
        let mut duplicate = 2i32.to_le_bytes().to_vec();
        record(
            &mut duplicate,
            "One.Texture",
            [3; 16],
            "Cache",
            &[(1, 0, 10)],
        );
        record(
            &mut duplicate,
            "one.texture",
            [3; 16],
            "Cache",
            &[(1, 0, 10)],
        );
        assert!(
            TextureCacheManifest::parse(&duplicate)
                .unwrap()
                .resolve_named([3; 16], 1, "One.Texture")
                .is_err()
        );
    }

    #[test]
    fn rejects_all_truncation_points_and_trailing_data() {
        let bytes = one();
        for cut in 0..bytes.len() {
            assert!(
                TextureCacheManifest::parse(&bytes[..cut]).is_err(),
                "cut {cut}"
            );
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert!(TextureCacheManifest::parse(&trailing).is_err());
    }

    #[test]
    fn rejects_bad_names_negative_fields_duplicates_and_budgets() {
        for cache in [
            "",
            "../Cache",
            "dir/Cache",
            "dir\\Cache",
            "C:Cache",
            "Cache.",
            "Cache ",
        ] {
            let mut bytes = 1i32.to_le_bytes().to_vec();
            record(&mut bytes, "P.T", [0; 16], cache, &[(0, 0, 1)]);
            assert!(TextureCacheManifest::parse(&bytes).is_err(), "{cache}");
        }
        for mip in [(-1, 0, 1), (0, -1, 1), (0, 0, -1), (0, 0, 0)] {
            let mut bytes = 1i32.to_le_bytes().to_vec();
            record(&mut bytes, "P.T", [0; 16], "C", &[mip]);
            assert!(TextureCacheManifest::parse(&bytes).is_err());
        }
        let mut duplicate = 1i32.to_le_bytes().to_vec();
        record(&mut duplicate, "P.T", [0; 16], "C", &[(0, 0, 1), (0, 1, 1)]);
        assert!(TextureCacheManifest::parse(&duplicate).is_err());
        let bytes = one();
        for limits in [
            ManifestLimits {
                max_file_bytes: bytes.len() - 1,
                ..Default::default()
            },
            ManifestLimits {
                max_entries: 0,
                ..Default::default()
            },
            ManifestLimits {
                max_string_bytes: 2,
                ..Default::default()
            },
            ManifestLimits {
                max_mips_per_entry: 1,
                ..Default::default()
            },
            ManifestLimits {
                max_total_mips: 1,
                ..Default::default()
            },
        ] {
            assert!(TextureCacheManifest::parse_with_limits(&bytes, limits).is_err());
        }
        assert!(TextureCacheManifest::parse(&(-1i32).to_le_bytes()).is_err());
        assert!(TextureCacheManifest::parse(&i32::MAX.to_le_bytes()).is_err());
    }

    #[test]
    fn decodes_utf16_and_rejects_invalid_string_encoding() {
        let mut input = Input {
            bytes: &[0xff, 0xff, 0xff, 0xff, 0, 0],
            position: 0,
            string_limit: 4,
        };
        assert_eq!(input.string().unwrap(), "");
        let bytes = [0xfd, 0xff, 0xff, 0xff, 0xa9, 0x03, b'X', 0, 0, 0];
        let mut input = Input {
            bytes: &bytes,
            position: 0,
            string_limit: 10,
        };
        assert_eq!(input.string().unwrap(), "ΩX");
        for bytes in [
            vec![2, 0, 0, 0, 0x80, 0],
            vec![2, 0, 0, 0, b'X', 1],
            vec![3, 0, 0, 0, 0, b'X', 0],
            vec![0xfe, 0xff, 0xff, 0xff, 0, 0xd8, 0, 0],
            i32::MIN.to_le_bytes().to_vec(),
        ] {
            let mut input = Input {
                bytes: &bytes,
                position: 0,
                string_limit: 32,
            };
            assert!(input.string().is_err());
        }
    }
}
