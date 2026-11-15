//! Bounded metadata reader for the observed Marvel Heroes PC UE3 profile.
//!
//! This does not deserialize objects, textures, meshes, or executable scripts.
//! Format evidence and current coverage are recorded in `docs/UPK_FORMAT.md`.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[path = "upk/lzo.rs"]
mod lzo;
pub(crate) use lzo::decompress as decompress_lzo;
#[path = "upk/objects.rs"]
pub mod objects;
#[path = "upk/world.rs"]
pub mod world;
#[path = "upk/level.rs"]
pub mod level;
#[path = "upk/model.rs"]
pub mod model;
#[path = "upk/model_component.rs"]
pub mod model_component;
#[path = "upk/polys.rs"]
pub mod polys;
#[path = "upk/class.rs"]
pub mod class;

pub const PACKAGE_TAG: u32 = 0x9e2a_83c1;
pub const FILE_VERSION: u16 = 868;
pub const LICENSEE_VERSION: u16 = 3;
pub const MAX_PACKAGE_BYTES: usize = 512 * 1024 * 1024;
pub const MAX_EXPORT_BYTES: usize = 64 * 1024 * 1024;
const MAX_TABLE_ENTRIES: usize = 1_000_000;
const MAX_STRING_BYTES: usize = 1024 * 1024;
const MAX_GENERATIONS: usize = 4096;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn unsupported(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableLocation {
    pub count: usize,
    /// Offset in the logical, decompressed package address space.
    pub offset: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Generation {
    pub export_count: u32,
    pub name_count: u32,
    pub net_object_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompressedChunk {
    pub logical_offset: usize,
    pub logical_size: usize,
    pub stored_offset: usize,
    pub stored_size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    pub file_version: u16,
    pub licensee_version: u16,
    pub header_size: usize,
    pub group: String,
    pub package_flags: u32,
    pub names: TableLocation,
    pub exports: TableLocation,
    pub imports: TableLocation,
    pub depends_offset: usize,
    pub import_export_guids_offset: usize,
    pub import_guids_count: usize,
    pub export_guids_count: usize,
    pub thumbnail_table_offset: usize,
    pub guid: [u8; 16],
    pub generations: Vec<Generation>,
    pub engine_version: u32,
    pub cooker_version: u32,
    pub compression_flags: u32,
    pub chunks: Vec<CompressedChunk>,
    pub package_source: u32,
    /// End of the fields decoded here; later summary fields remain opaque.
    pub parsed_bytes: usize,
}

impl Summary {
    /// Parse the observed 868/3 summary from a complete stored package.
    ///
    /// Compressed files are supported for summary inspection. Chunk storage
    /// bounds are checked against the supplied bytes; this does not decompress.
    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() > MAX_PACKAGE_BYTES {
            return Err(invalid("package exceeds the 512 MiB inspection limit"));
        }
        let mut input = Cursor::new(bytes);
        if input.u32()? != PACKAGE_TAG {
            return Err(invalid("not a little-endian UE3 package"));
        }
        let file_version = input.u16()?;
        let licensee_version = input.u16()?;
        if (file_version, licensee_version) != (FILE_VERSION, LICENSEE_VERSION) {
            return Err(unsupported(format!(
                "unsupported UE3 profile {file_version}/{licensee_version}; expected {FILE_VERSION}/{LICENSEE_VERSION}"
            )));
        }
        let header_size = input.nonnegative("header size")?;
        let group = input.string()?;
        let package_flags = input.u32()?;
        let names = input.table()?;
        let exports = input.table()?;
        let imports = input.table()?;
        let depends_offset = input.nonnegative("dependency table offset")?;
        let import_export_guids_offset = input.nonnegative("GUID table offset")?;
        let import_guids_count = input.count("import GUID count", MAX_TABLE_ENTRIES)?;
        let export_guids_count = input.count("export GUID count", MAX_TABLE_ENTRIES)?;
        let thumbnail_table_offset = input.nonnegative("thumbnail table offset")?;
        let guid = input.array()?;
        let generation_count = input.count("generation count", MAX_GENERATIONS)?;
        input.ensure_items(generation_count, 12)?;
        let mut generations = Vec::with_capacity(generation_count);
        for _ in 0..generation_count {
            generations.push(Generation {
                export_count: input.u32()?,
                name_count: input.u32()?,
                net_object_count: input.u32()?,
            });
        }
        let engine_version = input.u32()?;
        let cooker_version = input.u32()?;
        let compression_flags = input.u32()?;
        let chunk_count = input.count("compressed chunk count", MAX_TABLE_ENTRIES)?;
        input.ensure_items(chunk_count, 16)?;
        let mut chunks: Vec<CompressedChunk> = Vec::with_capacity(chunk_count);
        for _ in 0..chunk_count {
            let chunk = CompressedChunk {
                logical_offset: input.nonnegative("logical chunk offset")?,
                logical_size: input.nonnegative("logical chunk size")?,
                stored_offset: input.nonnegative("stored chunk offset")?,
                stored_size: input.nonnegative("stored chunk size")?,
            };
            if chunk.logical_size == 0 || chunk.stored_size == 0 {
                return Err(invalid("empty compressed chunk"));
            }
            checked_end(chunk.stored_offset, chunk.stored_size, bytes.len())?;
            checked_end(chunk.logical_offset, chunk.logical_size, MAX_PACKAGE_BYTES)?;
            if let Some(previous) = chunks.last() {
                if chunk.logical_offset < previous.logical_offset + previous.logical_size
                    || chunk.stored_offset < previous.stored_offset + previous.stored_size
                {
                    return Err(invalid("overlapping or unordered compressed chunks"));
                }
            }
            chunks.push(chunk);
        }
        let package_source = input.u32()?;
        if compression_flags == 0 && !chunks.is_empty() {
            return Err(invalid("compressed chunks have no compression flags"));
        }
        if let Some(first) = chunks.first() {
            if first.stored_offset < input.position {
                return Err(invalid("compressed data overlaps decoded summary fields"));
            }
        }
        let logical_size = chunks.last().map_or(bytes.len(), |chunk| {
            chunk.logical_offset + chunk.logical_size
        });
        if header_size > logical_size {
            return Err(invalid("header exceeds logical package size"));
        }
        for table in [names, imports, exports] {
            if table.count != 0 && (table.offset == 0 || table.offset >= header_size) {
                return Err(invalid("nonempty metadata table is outside the header"));
            }
        }
        let tables = [names, imports, exports];
        for (index, table) in tables.iter().enumerate() {
            if table.count != 0 {
                if chunks.is_empty() && table.offset < input.position {
                    return Err(invalid("metadata table overlaps summary fields"));
                }
                if tables[index + 1..]
                    .iter()
                    .any(|other| other.count != 0 && other.offset == table.offset)
                {
                    return Err(invalid("metadata tables share the same offset"));
                }
            }
        }
        Ok(Self {
            file_version,
            licensee_version,
            header_size,
            group,
            package_flags,
            names,
            exports,
            imports,
            depends_offset,
            import_export_guids_offset,
            import_guids_count,
            export_guids_count,
            thumbnail_table_offset,
            guid,
            generations,
            engine_version,
            cooker_version,
            compression_flags,
            chunks,
            package_source,
            parsed_bytes: input.position,
        })
    }

    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::parse(&read_package(path)?)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameEntry {
    pub value: String,
    pub flags: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NameRef {
    pub index: usize,
    /// Zero denotes the base name; positive values encode suffix `number - 1`.
    pub number: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    pub class_package: NameRef,
    pub class_name: NameRef,
    pub outer_index: i32,
    pub object_name: NameRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Export {
    pub class_index: i32,
    pub super_index: i32,
    pub outer_index: i32,
    pub object_name: NameRef,
    pub archetype_index: i32,
    pub object_flags: u64,
    pub serial_size: usize,
    pub serial_offset: usize,
    pub export_flags: u32,
    pub generation_net_object_counts: Vec<u32>,
    pub guid: [u8; 16],
    pub package_flags: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Package {
    pub summary: Summary,
    pub names: Vec<NameEntry>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
}

/// Binds parsed metadata to its original stored bytes for checked payload reads.
/// Reading exports does not deserialize their class-specific native payloads.
pub struct PackageArchive {
    package: Package,
    stored: Vec<u8>,
}

impl PackageArchive {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::parse(read_package(path)?)
    }

    pub fn parse(stored: Vec<u8>) -> io::Result<Self> {
        let package = Package::parse(&stored)?;
        Ok(Self { package, stored })
    }

    pub fn package(&self) -> &Package {
        &self.package
    }

    pub fn read_export(&self, index: usize) -> io::Result<Vec<u8>> {
        let export = self
            .package
            .exports
            .get(index)
            .ok_or_else(|| invalid("export index is out of bounds"))?;
        self.read_range(export.serial_offset, export.serial_size)
    }

    /// Read at most 64 MiB in the logical package address space.
    ///
    /// For compressed packages, only ranges covered by compression-map chunks
    /// are accepted. The stored summary is longer than its logical counterpart;
    /// this API deliberately does not invent a reconstructed summary prefix.
    pub fn read_range(&self, offset: usize, length: usize) -> io::Result<Vec<u8>> {
        if length > MAX_EXPORT_BYTES {
            return Err(invalid("logical range exceeds the 64 MiB read limit"));
        }
        let summary = &self.package.summary;
        let logical_size = summary.chunks.last().map_or(self.stored.len(), |chunk| {
            chunk.logical_offset + chunk.logical_size
        });
        let end = checked_end(offset, length, logical_size)?;
        if length == 0 {
            return Ok(Vec::new());
        }
        if summary.compression_flags == 0 && summary.chunks.is_empty() {
            return Ok(self.stored[offset..end].to_vec());
        }
        if summary.compression_flags != 2 {
            return Err(unsupported("logical range codec is unsupported"));
        }
        let mut output = Vec::with_capacity(length);
        let mut position = offset;
        for chunk in &summary.chunks {
            let chunk_end = chunk.logical_offset + chunk.logical_size;
            if chunk_end <= position {
                continue;
            }
            if position == end {
                break;
            }
            if chunk.logical_offset > position {
                return Err(unsupported(
                    "logical range is not covered by compressed chunks",
                ));
            }
            let take = (end - position).min(chunk_end - position);
            let stored_end =
                checked_end(chunk.stored_offset, chunk.stored_size, self.stored.len())?;
            output.extend_from_slice(&decode_chunk_range(
                &self.stored[chunk.stored_offset..stored_end],
                chunk.logical_size,
                position - chunk.logical_offset,
                take,
            )?);
            position += take;
        }
        if position != end {
            return Err(invalid("incomplete logical range coverage"));
        }
        Ok(output)
    }
}

impl Package {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::parse(&read_package(path)?)
    }

    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        let summary = Summary::parse(bytes)?;
        if summary.compression_flags == 0 && summary.chunks.is_empty() {
            return Self::parse_tables(summary, bytes, bytes.len());
        }
        if summary.compression_flags != 2 || summary.chunks.is_empty() {
            return Err(unsupported(
                "only the observed LZO compression flag 2 is decoded",
            ));
        }
        let logical_size = summary
            .chunks
            .last()
            .map(|chunk| chunk.logical_offset + chunk.logical_size)
            .ok_or_else(|| invalid("compressed package has no chunks"))?;
        let header = decode_metadata_header(&summary, bytes)?;
        Self::parse_tables(summary, &header, logical_size)
    }

    fn parse_tables(summary: Summary, bytes: &[u8], logical_size: usize) -> io::Result<Self> {
        let mut input = table_cursor(&summary, summary.names, bytes, 12)?;
        let mut names = Vec::with_capacity(summary.names.count);
        for _ in 0..summary.names.count {
            names.push(NameEntry {
                value: input.string()?,
                flags: input.u64()?,
            });
        }
        let mut input = table_cursor(&summary, summary.imports, bytes, 28)?;
        let mut imports = Vec::with_capacity(summary.imports.count);
        for _ in 0..summary.imports.count {
            imports.push(Import {
                class_package: input.name_ref(names.len())?,
                class_name: input.name_ref(names.len())?,
                outer_index: input.object_index(&summary)?,
                object_name: input.name_ref(names.len())?,
            });
        }
        let mut input = table_cursor(&summary, summary.exports, bytes, 68)?;
        let mut exports = Vec::with_capacity(summary.exports.count);
        for _ in 0..summary.exports.count {
            let class_index = input.object_index(&summary)?;
            let super_index = input.object_index(&summary)?;
            let outer_index = input.object_index(&summary)?;
            let object_name = input.name_ref(names.len())?;
            let archetype_index = input.object_index(&summary)?;
            let object_flags = input.u64()?;
            let serial_size = input.nonnegative("export serial size")?;
            let serial_offset = input.nonnegative("export serial offset")?;
            checked_end(serial_offset, serial_size, logical_size)?;
            if serial_size != 0 && serial_offset < summary.header_size {
                return Err(invalid("export data overlaps package header"));
            }
            let export_flags = input.u32()?;
            let count = input.count("export generation count", MAX_GENERATIONS)?;
            input.ensure_items(count, 4)?;
            let mut generation_net_object_counts = Vec::with_capacity(count);
            for _ in 0..count {
                generation_net_object_counts.push(input.u32()?);
            }
            let guid = input.array()?;
            let package_flags = input.u32()?;
            exports.push(Export {
                class_index,
                super_index,
                outer_index,
                object_name,
                archetype_index,
                object_flags,
                serial_size,
                serial_offset,
                export_flags,
                generation_net_object_counts,
                guid,
                package_flags,
            });
        }
        Ok(Self {
            summary,
            names,
            imports,
            exports,
        })
    }

    pub fn resolve_name(&self, name: NameRef) -> Option<String> {
        let base = &self.names.get(name.index)?.value;
        Some(if name.number == 0 {
            base.clone()
        } else {
            format!("{base}_{}", name.number - 1)
        })
    }

    /// Resolve the local name of an import/export reference, without loading
    /// another package. Zero is the null object reference.
    pub fn resolve_object_name(&self, index: i32) -> Option<String> {
        let name = if index > 0 {
            self.exports.get(index as usize - 1)?.object_name
        } else if index < 0 {
            self.imports
                .get(index.unsigned_abs() as usize - 1)?
                .object_name
        } else {
            return None;
        };
        self.resolve_name(name)
    }

    pub fn export_class_name(&self, index: usize) -> Option<String> {
        let class_index = self.exports.get(index)?.class_index;
        if class_index == 0 {
            Some("Class".to_owned())
        } else {
            self.resolve_object_name(class_index)
        }
    }
}

fn decode_metadata_header(summary: &Summary, bytes: &[u8]) -> io::Result<Vec<u8>> {
    let first = summary
        .chunks
        .first()
        .ok_or_else(|| invalid("missing compression map"))?;
    let prefix = first.logical_offset.min(summary.header_size);
    checked_end(0, prefix, bytes.len())?;
    let mut header = Vec::with_capacity(summary.header_size);
    header.extend_from_slice(&bytes[..prefix]);
    for chunk in &summary.chunks {
        if header.len() == summary.header_size {
            break;
        }
        if chunk.logical_offset != header.len() {
            return Err(invalid("gap in compressed metadata coverage"));
        }
        let end = checked_end(chunk.stored_offset, chunk.stored_size, bytes.len())?;
        let needed = chunk.logical_size.min(summary.header_size - header.len());
        header.extend_from_slice(&decode_chunk_prefix(
            &bytes[chunk.stored_offset..end],
            chunk.logical_size,
            needed,
        )?);
    }
    if header.len() != summary.header_size {
        return Err(invalid("compressed chunks do not cover metadata header"));
    }
    Ok(header)
}

/// Validate the complete chunk envelope, then decode only blocks needed for
/// metadata. Export payload blocks remain unexamined; this is not file integrity
/// validation and is deliberately separate from the private pin's SHA-256.
fn decode_chunk_prefix(bytes: &[u8], logical_size: usize, needed: usize) -> io::Result<Vec<u8>> {
    decode_chunk_range(bytes, logical_size, 0, needed)
}

fn decode_chunk_range(
    bytes: &[u8],
    logical_size: usize,
    offset: usize,
    needed: usize,
) -> io::Result<Vec<u8>> {
    let requested_end = checked_end(offset, needed, logical_size)?;
    let mut input = Cursor::new(bytes);
    if input.u32()? != PACKAGE_TAG {
        return Err(invalid("invalid compressed chunk tag"));
    }
    let block_size = input.nonnegative("compression block size")?;
    if block_size != 0x20000 {
        return Err(unsupported(
            "only observed 128 KiB UE3 compression blocks are supported",
        ));
    }
    let stored_total = input.nonnegative("compressed payload size")?;
    let logical_total = input.nonnegative("decompressed chunk size")?;
    if logical_total != logical_size {
        return Err(invalid("compression envelope disagrees with chunk map"));
    }
    let count = logical_total.div_ceil(block_size);
    input.ensure_items(count, 8)?;
    let mut blocks = Vec::with_capacity(count);
    let mut stored_sum = 0usize;
    let mut logical_sum = 0usize;
    for index in 0..count {
        let stored = input.nonnegative("stored block size")?;
        let logical = input.nonnegative("logical block size")?;
        let expected = block_size.min(logical_total - logical_sum);
        if stored == 0 || logical != expected || index + 1 < count && logical != block_size {
            return Err(invalid("invalid compression block size"));
        }
        stored_sum = stored_sum
            .checked_add(stored)
            .ok_or_else(|| invalid("stored block sum overflow"))?;
        logical_sum += logical;
        blocks.push((stored, logical));
    }
    if logical_sum != logical_total
        || stored_sum != stored_total
        || checked_end(input.position, stored_sum, bytes.len())? != bytes.len()
    {
        return Err(invalid("compression block totals disagree with envelope"));
    }
    let mut output = Vec::with_capacity(needed);
    let mut logical_position = 0usize;
    for (stored, logical) in blocks {
        if output.len() == needed {
            break;
        }
        let payload = input.take(stored)?;
        let block_end = logical_position + logical;
        if block_end > offset && logical_position < requested_end {
            let decoded = decompress_lzo(payload, logical)?;
            let start = offset.saturating_sub(logical_position);
            let end = logical.min(requested_end - logical_position);
            output.extend_from_slice(&decoded[start..end]);
        }
        logical_position = block_end;
    }
    if output.len() != needed {
        return Err(invalid("compression blocks do not cover logical read"));
    }
    Ok(output)
}

fn read_package(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    if file.metadata()?.len() > MAX_PACKAGE_BYTES as u64 {
        return Err(invalid("package exceeds the 512 MiB inspection limit"));
    }
    let mut bytes = Vec::new();
    file.take(MAX_PACKAGE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_PACKAGE_BYTES {
        return Err(invalid("package grew beyond the inspection limit"));
    }
    Ok(bytes)
}

fn checked_end(offset: usize, size: usize, limit: usize) -> io::Result<usize> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| invalid("range overflow"))?;
    if end > limit {
        return Err(invalid("package range is out of bounds"));
    }
    Ok(end)
}

fn table_cursor<'a>(
    summary: &Summary,
    table: TableLocation,
    bytes: &'a [u8],
    minimum_entry_bytes: usize,
) -> io::Result<Cursor<'a>> {
    if table.count == 0 {
        return Ok(Cursor::new(&[]));
    }
    let mut end = summary.header_size;
    for offset in [
        summary.names.offset,
        summary.imports.offset,
        summary.exports.offset,
        summary.depends_offset,
        summary.import_export_guids_offset,
        summary.thumbnail_table_offset,
    ] {
        if offset > table.offset && offset < end {
            end = offset;
        }
    }
    let data = bytes
        .get(table.offset..end)
        .ok_or_else(|| invalid("table bounds are invalid"))?;
    let input = Cursor::new(data);
    input.ensure_items(table.count, minimum_entry_bytes)?;
    Ok(input)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(&mut self, size: usize) -> io::Result<&'a [u8]> {
        let end = checked_end(self.position, size, self.bytes.len())?;
        let result = &self.bytes[self.position..end];
        self.position = end;
        Ok(result)
    }

    fn array<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        self.take(N)?
            .try_into()
            .map_err(|_| invalid("truncated fixed field"))
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
    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    fn nonnegative(&mut self, label: &str) -> io::Result<usize> {
        usize::try_from(self.i32()?).map_err(|_| invalid(format!("negative {label}")))
    }

    fn count(&mut self, label: &str, limit: usize) -> io::Result<usize> {
        let count = self.nonnegative(label)?;
        if count > limit {
            return Err(invalid(format!("{label} exceeds limit")));
        }
        Ok(count)
    }

    fn table(&mut self) -> io::Result<TableLocation> {
        Ok(TableLocation {
            count: self.count("table entry count", MAX_TABLE_ENTRIES)?,
            offset: self.nonnegative("table offset")?,
        })
    }

    fn ensure_items(&self, count: usize, size: usize) -> io::Result<()> {
        let needed = count
            .checked_mul(size)
            .ok_or_else(|| invalid("table size overflow"))?;
        checked_end(self.position, needed, self.bytes.len())?;
        Ok(())
    }

    fn string(&mut self) -> io::Result<String> {
        let count = self.i32()?;
        if count == 0 {
            return Ok(String::new());
        }
        let units =
            usize::try_from(count.unsigned_abs()).map_err(|_| invalid("string length overflow"))?;
        let size = units
            .checked_mul(if count < 0 { 2 } else { 1 })
            .ok_or_else(|| invalid("string length overflow"))?;
        if size > MAX_STRING_BYTES {
            return Err(invalid("string exceeds inspection limit"));
        }
        let bytes = self.take(size)?;
        if count > 0 {
            if bytes.last() != Some(&0) || bytes[..bytes.len() - 1].contains(&0) {
                return Err(invalid("narrow string has an invalid terminator"));
            }
            if !bytes.is_ascii() {
                return Err(unsupported(
                    "non-ASCII narrow FString code page is not established",
                ));
            }
            String::from_utf8(bytes[..bytes.len() - 1].to_vec())
                .map_err(|_| invalid("invalid ASCII string"))
        } else {
            let mut text: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .collect();
            if text.pop() != Some(0) || text.contains(&0) {
                return Err(invalid("wide string has an invalid terminator"));
            }
            String::from_utf16(&text).map_err(|_| invalid("invalid UTF-16 string"))
        }
    }

    fn name_ref(&mut self, count: usize) -> io::Result<NameRef> {
        let index = self.nonnegative("name index")?;
        let number = self.u32()?;
        if index >= count {
            return Err(invalid("name reference is outside name table"));
        }
        Ok(NameRef { index, number })
    }

    fn object_index(&mut self, summary: &Summary) -> io::Result<i32> {
        let index = self.i32()?;
        if index > 0 && index as usize > summary.exports.count
            || index < 0 && index.unsigned_abs() as usize > summary.imports.count
        {
            return Err(invalid("object reference is outside import/export tables"));
        }
        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fn patch(bytes: &mut [u8], offset: usize, value: usize) {
        bytes[offset..offset + 4].copy_from_slice(&(value as u32).to_le_bytes());
    }
    fn text(bytes: &mut Vec<u8>, value: &str) {
        word(bytes, value.len() as u32 + 1);
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(0);
    }
    fn name(bytes: &mut Vec<u8>, index: u32) {
        word(bytes, index);
        word(bytes, 0);
    }

    // Constructed format fixture; no retail asset bytes are embedded in tests.
    pub(super) fn fixture() -> Vec<u8> {
        let mut bytes = Vec::new();
        word(&mut bytes, PACKAGE_TAG);
        word(
            &mut bytes,
            u32::from(FILE_VERSION) | (u32::from(LICENSEE_VERSION) << 16),
        );
        word(&mut bytes, 0); // header size, patched below
        text(&mut bytes, "None");
        word(&mut bytes, 0); // package flags
        word(&mut bytes, 2);
        let names_offset_field = bytes.len();
        word(&mut bytes, 0);
        word(&mut bytes, 1);
        let exports_offset_field = bytes.len();
        word(&mut bytes, 0);
        word(&mut bytes, 1);
        let imports_offset_field = bytes.len();
        word(&mut bytes, 0);
        let depends_offset_field = bytes.len();
        word(&mut bytes, 0);
        for _ in 0..4 {
            word(&mut bytes, 0);
        }
        bytes.extend_from_slice(&[0; 16]);
        word(&mut bytes, 1); // generation count
        for value in [1, 2, 0, 10897, 136, 0, 0, 0, 0, 0] {
            word(&mut bytes, value);
        }
        let names_offset = bytes.len();
        for value in ["Core", "Object"] {
            text(&mut bytes, value);
            bytes.extend_from_slice(&0u64.to_le_bytes());
        }
        let imports_offset = bytes.len();
        name(&mut bytes, 0);
        name(&mut bytes, 1);
        word(&mut bytes, 0);
        name(&mut bytes, 1);
        let exports_offset = bytes.len();
        word(&mut bytes, u32::MAX); // class = import 1
        word(&mut bytes, 0);
        word(&mut bytes, 0);
        name(&mut bytes, 1);
        word(&mut bytes, 0);
        bytes.extend_from_slice(&0u64.to_le_bytes());
        word(&mut bytes, 3);
        let serial_offset_field = bytes.len();
        word(&mut bytes, 0);
        word(&mut bytes, 0);
        word(&mut bytes, 0);
        bytes.extend_from_slice(&[0; 16]);
        word(&mut bytes, 0);
        let depends_offset = bytes.len();
        word(&mut bytes, 0);
        let header_size = bytes.len();
        patch(&mut bytes, 8, header_size);
        patch(&mut bytes, names_offset_field, names_offset);
        patch(&mut bytes, imports_offset_field, imports_offset);
        patch(&mut bytes, exports_offset_field, exports_offset);
        patch(&mut bytes, depends_offset_field, depends_offset);
        patch(&mut bytes, serial_offset_field, header_size);
        bytes.extend_from_slice(&[7, 8, 9]);
        bytes
    }

    #[test]
    fn reads_metadata_and_resolves_numbered_names() {
        let package = Package::parse(&fixture()).unwrap();
        assert_eq!(package.names.len(), 2);
        assert_eq!(package.imports.len(), 1);
        assert_eq!(package.exports[0].class_index, -1);
        assert_eq!(package.exports[0].serial_size, 3);
        assert_eq!(
            package
                .resolve_name(NameRef {
                    index: 1,
                    number: 3
                })
                .as_deref(),
            Some("Object_2")
        );
        assert_eq!(
            package.resolve_name(NameRef {
                index: 3,
                number: 0
            }),
            None
        );
    }

    #[test]
    fn rejects_unknown_profile_and_byte_order() {
        let mut bytes = fixture();
        bytes[4] ^= 1;
        assert_eq!(
            Package::parse(&bytes).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
        bytes = fixture();
        bytes[..4].reverse();
        assert_eq!(
            Package::parse(&bytes).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn every_truncation_of_required_fixture_data_is_rejected() {
        let bytes = fixture();
        for length in 0..bytes.len() {
            assert!(
                Package::parse(&bytes[..length]).is_err(),
                "accepted length {length}"
            );
        }
    }

    #[test]
    fn rejects_corrupt_table_counts_references_and_export_ranges() {
        let original = fixture();
        let package = Package::parse(&original).unwrap();
        for (offset, value) in [
            (25, u32::MAX as usize),                               // negative name count
            (29, 8),                                               // names overlap summary
            (45, package.summary.names.offset),                    // table offsets collide
            (package.summary.imports.offset, 2),                   // name index out of range
            (package.summary.exports.offset, 2), // class export index out of range
            (package.summary.exports.offset + 32, usize::MAX), // negative serial size
            (package.summary.exports.offset + 36, original.len()), // data runs past EOF
        ] {
            let mut bytes = original.clone();
            patch(&mut bytes, offset, value);
            assert!(
                Package::parse(&bytes).is_err(),
                "accepted mutation at {offset}"
            );
        }
    }

    #[test]
    fn string_reader_accepts_unicode_and_rejects_ambiguous_or_broken_strings() {
        let mut bytes = (-3i32).to_le_bytes().to_vec();
        for code in [0x03b1u16, 0x03b2, 0] {
            bytes.extend_from_slice(&code.to_le_bytes());
        }
        assert_eq!(Cursor::new(&bytes).string().unwrap(), "αβ");
        for bytes in [
            vec![2, 0, 0, 0, b'a', b'b'],
            vec![2, 0, 0, 0, 255, 0],
            vec![3, 0, 0, 0, 0, b'a', 0],
            i32::MIN.to_le_bytes().to_vec(),
            vec![254, 255, 255, 255, 0, 216, 0, 0],
        ] {
            assert!(Cursor::new(&bytes).string().is_err());
        }
    }

    #[test]
    fn compressed_envelope_checks_totals_before_decoding() {
        let mut bytes = Vec::new();
        for value in [PACKAGE_TAG, 0x20000, 3, 4, 3, 4] {
            word(&mut bytes, value);
        }
        bytes.extend_from_slice(&[0x11, 0, 0]); // empty LZO stream cannot yield 4 bytes
        assert!(decode_chunk_prefix(&bytes, 4, 4).is_err());
        let mut mismatched = bytes.clone();
        patch(&mut mismatched, 8, 5);
        assert!(decode_chunk_prefix(&mismatched, 4, 4).is_err());
        assert!(decode_chunk_prefix(&bytes, 5, 4).is_err());
        bytes.push(0);
        assert!(decode_chunk_prefix(&bytes, 4, 4).is_err());
    }

    #[test]
    fn archive_payload_access_checks_bounds_and_binding() {
        let bytes = fixture();
        let archive = PackageArchive::parse(bytes.clone()).unwrap();
        let export = &archive.package().exports[0];
        assert_eq!(archive.read_export(0).unwrap(), &[7, 8, 9]);
        assert_eq!(
            archive.read_range(export.serial_offset + 1, 2).unwrap(),
            &[8, 9]
        );
        assert_eq!(
            archive.package().export_class_name(0).as_deref(),
            Some("Object")
        );
        assert_eq!(
            archive.package().resolve_object_name(-1).as_deref(),
            Some("Object")
        );
        assert!(archive.read_export(1).is_err());
        assert!(archive.read_range(bytes.len(), 1).is_err());
        assert!(archive.read_range(usize::MAX, 2).is_err());
        assert!(archive.read_range(0, MAX_EXPORT_BYTES + 1).is_err());
    }

    #[test]
    fn logical_ranges_cross_compressed_block_boundaries() {
        fn literals(data: &[u8]) -> Vec<u8> {
            let mut output = Vec::new();
            if data.len() <= 238 {
                output.push(data.len() as u8 + 17);
            } else {
                output.push(0);
                let mut extension = data.len() - 18;
                while extension > 255 {
                    output.push(0);
                    extension -= 255;
                }
                output.push(extension as u8);
            }
            output.extend_from_slice(data);
            output.extend_from_slice(&[17, 0, 0]);
            output
        }
        let first = literals(&vec![b'a'; 0x20000]);
        let second = literals(&[b'b'; 17]);
        let mut bytes = Vec::new();
        for value in [
            PACKAGE_TAG,
            0x20000,
            (first.len() + second.len()) as u32,
            0x20011,
            first.len() as u32,
            0x20000,
            second.len() as u32,
            17,
        ] {
            word(&mut bytes, value);
        }
        bytes.extend_from_slice(&first);
        bytes.extend_from_slice(&second);
        assert_eq!(
            decode_chunk_range(&bytes, 0x20011, 0x1fffd, 9).unwrap(),
            b"aaabbbbbb"
        );
        assert_eq!(
            decode_chunk_range(&bytes, 0x20011, 0x2000f, 2).unwrap(),
            b"bb"
        );
        assert!(decode_chunk_range(&bytes, 0x20011, 0x20010, 2).is_err());
        assert!(decode_chunk_range(&bytes[..bytes.len() - 1], 0x20011, 0, 1).is_err());
    }
}
