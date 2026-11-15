//! Read-only GPAK version 1 (`.sip`) packages, with bounded raw LZ4 decoding.
//!
//! Format observations and study provenance are recorded in `docs/SIP_FORMAT.md`.
//! Archive names are identifiers, never destinations on the host filesystem.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// Resource budgets checked before allocating or decompressing archive data.
#[derive(Clone, Copy, Debug)]
pub struct SipLimits {
    pub max_entries: usize,
    pub max_name_bytes: usize,
    pub max_index_bytes: u64,
    pub max_member_bytes: usize,
}

impl Default for SipLimits {
    fn default() -> Self {
        Self {
            max_entries: 1_000_000,
            max_name_bytes: 4096,
            max_index_bytes: 128 * 1024 * 1024,
            max_member_bytes: 256 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SipEntry {
    pub file_hash: u64,
    pub name: String,
    pub mod_time: i32,
    /// Relative to the archive's data section, not the start of the file.
    pub offset: u64,
    pub compressed_size: usize,
    pub uncompressed_size: usize,
}

/// An indexed package. Payloads remain in the reader until explicitly requested.
pub struct SipArchive<R = BufReader<File>> {
    reader: R,
    entries: Vec<SipEntry>,
    by_name: HashMap<String, usize>,
    data_offset: u64,
    file_len: u64,
}

impl SipArchive<BufReader<File>> {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::open_with_limits(path, SipLimits::default())
    }

    pub fn open_with_limits(path: impl AsRef<Path>, limits: SipLimits) -> io::Result<Self> {
        Self::from_reader_with_limits(BufReader::new(File::open(path)?), limits)
    }
}

impl<R: Read + Seek> SipArchive<R> {
    /// Index a complete seekable package, starting at byte zero.
    pub fn from_reader(reader: R) -> io::Result<Self> {
        Self::from_reader_with_limits(reader, SipLimits::default())
    }

    pub fn from_reader_with_limits(mut reader: R, limits: SipLimits) -> io::Result<Self> {
        let file_len = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;
        let mut signature = [0; 4];
        reader.read_exact(&mut signature)?;
        if signature != *b"KAPG" {
            return Err(invalid("not a GPAK package (expected KAPG bytes)"));
        }
        if read_u32(&mut reader)? != 1 {
            return Err(invalid("unsupported GPAK version (expected 1)"));
        }
        let count = read_nonnegative(&mut reader, "negative entry count")?;
        // Each entry has at least 28 fixed bytes, even before its name.
        if count > limits.max_entries || count as u64 > file_len.saturating_sub(12) / 28 {
            return Err(invalid(
                "entry count exceeds package size or configured limit",
            ));
        }
        if limits.max_index_bytes < 12 || count as u64 > (limits.max_index_bytes - 12) / 28 {
            return Err(invalid("entry count exceeds configured index byte limit"));
        }
        let mut entries = Vec::new();
        entries.try_reserve(count).map_err(allocation_failed)?;
        let mut by_name = HashMap::new();
        by_name.try_reserve(count).map_err(allocation_failed)?;
        let mut index_bytes = 12_u64;
        for index in 0..count {
            let file_hash = read_u64(&mut reader)?;
            let name_len = read_nonnegative(&mut reader, "negative member name length")?;
            index_bytes = index_bytes
                .checked_add(28)
                .and_then(|n| n.checked_add(name_len as u64))
                .ok_or_else(|| invalid("index size overflow"))?;
            if name_len == 0
                || name_len > limits.max_name_bytes
                || index_bytes > limits.max_index_bytes
                || index_bytes > file_len
            {
                return Err(invalid(
                    "member name or index exceeds package size or configured limit",
                ));
            }
            let mut name_bytes = zeroed(name_len)?;
            reader.read_exact(&mut name_bytes)?;
            let name =
                String::from_utf8(name_bytes).map_err(|_| invalid("member name is not UTF-8"))?;
            validate_name(&name)?;
            if by_name.insert(name.clone(), index).is_some() {
                return Err(invalid("duplicate member name"));
            }
            let mod_time = read_u32(&mut reader)? as i32;
            let offset = read_nonnegative(&mut reader, "negative member offset")? as u64;
            let compressed_size = read_nonnegative(&mut reader, "negative compressed size")?;
            let uncompressed_size = read_nonnegative(&mut reader, "negative uncompressed size")?;
            if compressed_size > limits.max_member_bytes
                || uncompressed_size > limits.max_member_bytes
            {
                return Err(invalid("member exceeds configured byte limit"));
            }
            entries.push(SipEntry {
                file_hash,
                name,
                mod_time,
                offset,
                compressed_size,
                uncompressed_size,
            });
        }
        let data_offset = reader.stream_position()?;
        debug_assert_eq!(data_offset, index_bytes);
        for entry in &entries {
            let end = data_offset
                .checked_add(entry.offset)
                .and_then(|n| n.checked_add(entry.compressed_size as u64))
                .ok_or_else(|| invalid("member range overflow"))?;
            if end > file_len {
                return Err(invalid("member payload extends past end of package"));
            }
        }
        Ok(Self {
            reader,
            entries,
            by_name,
            data_offset,
            file_len,
        })
    }

    pub fn entries(&self) -> &[SipEntry] {
        &self.entries
    }

    pub fn data_offset(&self) -> u64 {
        self.data_offset
    }

    pub fn file_len(&self) -> u64 {
        self.file_len
    }

    pub fn entry(&self, name: &str) -> Option<&SipEntry> {
        self.by_name.get(name).map(|&index| &self.entries[index])
    }

    /// Look up an exact, case-sensitive stored member name and decompress it.
    pub fn read(&mut self, name: &str) -> io::Result<Vec<u8>> {
        let index =
            self.by_name.get(name).copied().ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "package member not found")
            })?;
        self.read_index(index)
    }

    pub fn read_index(&mut self, index: usize) -> io::Result<Vec<u8>> {
        let entry = self.entries.get(index).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "package member index not found")
        })?;
        // All operands were range-checked while indexing; metadata is immutable.
        self.reader
            .seek(SeekFrom::Start(self.data_offset + entry.offset))?;
        let mut compressed = zeroed(entry.compressed_size)?;
        self.reader.read_exact(&mut compressed)?;
        decode_lz4_block(&compressed, entry.uncompressed_size)
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn allocation_failed(error: std::collections::TryReserveError) -> io::Error {
    io::Error::new(io::ErrorKind::OutOfMemory, error)
}

fn zeroed(len: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(len).map_err(allocation_failed)?;
    bytes.resize(len, 0);
    Ok(bytes)
}

fn read_u32(reader: &mut impl Read) -> io::Result<u32> {
    let mut bytes = [0; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(reader: &mut impl Read) -> io::Result<u64> {
    let mut bytes = [0; 8];
    reader.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

fn read_nonnegative(reader: &mut impl Read, message: &'static str) -> io::Result<usize> {
    let value = read_u32(reader)? as i32;
    usize::try_from(value).map_err(|_| invalid(message))
}

fn validate_name(name: &str) -> io::Result<()> {
    if name
        .chars()
        .any(|c| c.is_control() || c == '\\' || c == ':')
        || name
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(invalid("member name is not a safe relative package path"));
    }
    Ok(())
}

/// Decode one raw LZ4 block, bounded by the already-validated declared output.
/// Overlapping copies intentionally read bytes produced earlier in the match.
fn decode_lz4_block(input: &[u8], expected: usize) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(expected)
        .map_err(allocation_failed)?;
    let mut cursor = 0;
    while cursor < input.len() {
        let token = input[cursor];
        cursor += 1;
        let literals = lz4_length(input, &mut cursor, usize::from(token >> 4))?;
        let literal_end = cursor
            .checked_add(literals)
            .filter(|&end| end <= input.len())
            .ok_or_else(|| invalid("truncated LZ4 literals"))?;
        if literals > expected.saturating_sub(output.len()) {
            return Err(invalid("LZ4 literals exceed declared output size"));
        }
        output.extend_from_slice(&input[cursor..literal_end]);
        cursor = literal_end;
        if cursor == input.len() {
            break;
        }
        let offset_end = cursor
            .checked_add(2)
            .filter(|&end| end <= input.len())
            .ok_or_else(|| invalid("truncated LZ4 match offset"))?;
        let distance = u16::from_le_bytes([input[cursor], input[cursor + 1]]) as usize;
        cursor = offset_end;
        if distance == 0 || distance > output.len() {
            return Err(invalid("invalid LZ4 match distance"));
        }
        let length = lz4_length(input, &mut cursor, usize::from(token & 15))?
            .checked_add(4)
            .ok_or_else(|| invalid("LZ4 match length overflow"))?;
        if length > expected.saturating_sub(output.len()) {
            return Err(invalid("LZ4 match exceeds declared output size"));
        }
        for _ in 0..length {
            let value = output[output.len() - distance];
            output.push(value);
        }
    }
    if output.len() != expected {
        return Err(invalid("LZ4 output size differs from declared size"));
    }
    Ok(output)
}

fn lz4_length(input: &[u8], cursor: &mut usize, initial: usize) -> io::Result<usize> {
    let mut length = initial;
    if initial == 15 {
        loop {
            let byte = *input
                .get(*cursor)
                .ok_or_else(|| invalid("truncated LZ4 length"))?;
            *cursor += 1;
            length = length
                .checked_add(usize::from(byte))
                .ok_or_else(|| invalid("LZ4 length overflow"))?;
            if byte != 255 {
                break;
            }
        }
    }
    Ok(length)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn archive(name: &[u8], offset: i32, compressed: &[u8], decoded: i32) -> Vec<u8> {
        let mut bytes = b"KAPG".to_vec();
        bytes.extend(1_u32.to_le_bytes());
        bytes.extend(1_i32.to_le_bytes());
        bytes.extend(123_u64.to_le_bytes());
        bytes.extend((name.len() as i32).to_le_bytes());
        bytes.extend(name);
        bytes.extend(0_i32.to_le_bytes());
        bytes.extend(offset.to_le_bytes());
        bytes.extend((compressed.len() as i32).to_le_bytes());
        bytes.extend(decoded.to_le_bytes());
        bytes.extend(compressed);
        bytes
    }

    #[test]
    fn reads_member_and_rejects_invalid_metadata() {
        let bytes = archive(b"Calligraphy/test", 0, b"\x30abc", 3);
        let mut pak = SipArchive::from_reader(Cursor::new(&bytes)).unwrap();
        assert_eq!(pak.read("Calligraphy/test").unwrap(), b"abc");
        assert_eq!(
            pak.read("calligraphy/test").unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        assert!(SipArchive::from_reader(Cursor::new(archive(b"ok", -1, b"\0", 0))).is_err());
        assert!(SipArchive::from_reader(Cursor::new(archive(b"ok", 1, b"\0", 0))).is_err());
        assert!(SipArchive::from_reader(Cursor::new(archive(b"ok", 0, b"\0", -1))).is_err());
        for name in [
            b"../x".as_slice(),
            b"/x",
            b"C:/x",
            b"a\\b",
            b"a//b",
            b"a/./b",
            b"a\0b",
            b"\xff",
        ] {
            assert!(SipArchive::from_reader(Cursor::new(archive(name, 0, b"\0", 0))).is_err());
        }
        for end in 0..bytes.len() {
            assert!(SipArchive::from_reader(Cursor::new(&bytes[..end])).is_err());
        }
    }

    #[test]
    fn rejects_duplicate_names_and_budget_excess() {
        let one = archive(b"same", 0, b"\0", 0);
        let mut two = one[..12].to_vec();
        two[8..12].copy_from_slice(&2_i32.to_le_bytes());
        two.extend_from_slice(&one[12..one.len() - 1]);
        two.extend_from_slice(&one[12..]);
        assert!(SipArchive::from_reader(Cursor::new(two)).is_err());
        let limits = SipLimits {
            max_member_bytes: 2,
            ..SipLimits::default()
        };
        assert!(
            SipArchive::from_reader_with_limits(
                Cursor::new(archive(b"a", 0, b"\x30abc", 3)),
                limits
            )
            .is_err()
        );
        let limits = SipLimits {
            max_entries: 0,
            ..SipLimits::default()
        };
        assert!(SipArchive::from_reader_with_limits(Cursor::new(one), limits).is_err());
    }

    #[test]
    fn follows_offsets_independently_of_table_order() {
        let first = archive(b"a", 2, b"\x10a", 1);
        let second = archive(b"b", 0, b"\x10b", 1);
        let mut bytes = first[..12].to_vec();
        bytes[8..12].copy_from_slice(&2_i32.to_le_bytes());
        bytes.extend_from_slice(&first[12..first.len() - 2]);
        bytes.extend_from_slice(&second[12..second.len() - 2]);
        bytes.extend_from_slice(b"\x10b\x10a");
        let mut pak = SipArchive::from_reader(Cursor::new(bytes)).unwrap();
        assert_eq!(pak.read("a").unwrap(), b"a");
        assert_eq!(pak.read("b").unwrap(), b"b");
    }

    #[test]
    fn lz4_overlap_and_extended_lengths() {
        // One literal, a distance-one overlapping match, then five final literals.
        assert_eq!(
            decode_lz4_block(b"\x14a\x01\x00\x50bcdef", 14).unwrap(),
            b"aaaaaaaaabcdef"
        );
        let mut literals = vec![0xf0, 255, 1];
        literals.extend(vec![b'x'; 271]);
        assert_eq!(decode_lz4_block(&literals, 271).unwrap(), vec![b'x'; 271]);
        assert_eq!(
            decode_lz4_block(b"\x1fa\x01\x00\x00\x50bcdef", 25).unwrap(),
            [vec![b'a'; 20], b"bcdef".to_vec()].concat()
        );
    }

    #[test]
    fn lz4_rejects_corruption_and_wrong_output_length() {
        for (input, expected) in [
            (b"\xf0".as_slice(), 20),
            (b"\x20a", 2),
            (b"\x10a\x01", 5),
            (b"\x10a\0\0", 5),
            (b"\x10a\x02\0", 5),
            (b"\x1fa\x01\0", 20),
            (b"\x10a", 0),
            (b"\x10a", 2),
            (b"\x10a\x01\0", 4),
        ] {
            assert!(
                decode_lz4_block(input, expected).is_err(),
                "accepted {input:?}"
            );
        }
    }
}
