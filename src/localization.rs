//! Bounded LOC2 metadata and STR2 localized-string readers.
//!
//! The layouts were checked against private owner-supplied English files. Strings,
//! flags, and alternate forms are preserved without formatting or selecting them.
//! No game data is embedded here. Study sources are recorded in project provenance.

use std::collections::HashMap;
use std::fmt;

pub const SUPPORTED_VERSION: u8 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    pub offset: usize,
    pub message: &'static str,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "localization at byte {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocaleLimits {
    pub max_file_bytes: usize,
    pub max_entries: usize,
    pub max_flags: usize,
    /// Alternate forms per entry, excluding its default string.
    pub max_variants_per_entry: usize,
    pub max_total_variants: usize,
    /// UTF-8 bytes in any individual string, excluding its terminator or prefix.
    pub max_string_bytes: usize,
    /// Sum of copied UTF-8 bytes, including repeated references to the same span.
    pub max_decoded_bytes: usize,
}

impl Default for LocaleLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: 64 * 1024 * 1024,
            max_entries: u16::MAX as usize,
            max_flags: u8::MAX as usize,
            max_variants_per_entry: 256,
            max_total_variants: 65_536,
            max_string_bytes: 1024 * 1024,
            max_decoded_bytes: 64 * 1024 * 1024,
        }
    }
}

/// An unchanged UTF-8 string and its byte span in the source file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocaleText {
    pub text: String,
    /// First UTF-8 byte, after any length prefix.
    pub source_offset: usize,
    /// Exclusive end of UTF-8 bytes; a STR2 NUL terminator is not included.
    pub source_end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocaleFlag {
    pub source_offset: usize,
    pub value: u16,
    pub mask: u16,
    pub text: LocaleText,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocaleMetadata {
    pub name: LocaleText,
    pub language_display_name: LocaleText,
    pub region_display_name: LocaleText,
    /// A stored identifier only; parsing does not use it as a filesystem path.
    pub directory: LocaleText,
    pub flags: Vec<LocaleFlag>,
}

impl LocaleMetadata {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        Self::parse_with_limits(bytes, LocaleLimits::default())
    }

    pub fn parse_with_limits(bytes: &[u8], limits: LocaleLimits) -> Result<Self> {
        let mut r = Reader::new(bytes, limits)?;
        r.header(b"LOC")?;
        let mut budget = TextBudget::new(limits);
        let name = r.prefixed_text(&mut budget)?;
        let language_display_name = r.prefixed_text(&mut budget)?;
        let region_display_name = r.prefixed_text(&mut budget)?;
        let directory = r.prefixed_text(&mut budget)?;
        let count = usize::from(r.u8()?);
        if count > limits.max_flags {
            return Err(error(r.pos - 1, "locale flag count exceeds limit"));
        }
        r.require_records(count, 6)?;
        let mut flags = reserved_vec(count, r.pos)?;
        for _ in 0..count {
            flags.push(LocaleFlag {
                source_offset: r.pos,
                value: r.u16()?,
                mask: r.u16()?,
                text: r.prefixed_text(&mut budget)?,
            });
        }
        if r.pos != bytes.len() {
            return Err(error(r.pos, "trailing bytes after locale metadata"));
        }
        Ok(Self {
            name,
            language_display_name,
            region_display_name,
            directory,
            flags,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringVariant {
    /// Beginning of this alternate form's 14-byte table record.
    pub source_offset: usize,
    pub flags_consumed: u64,
    pub flags_produced: u16,
    pub text: LocaleText,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringEntry {
    pub id: u64,
    /// Beginning of this entry's 16-byte table record.
    pub source_offset: usize,
    /// Stored count. Both 0 and 1 mean no alternate forms.
    pub num_variants: u16,
    /// Flags belonging to the default string.
    pub flags_produced: u16,
    pub default: LocaleText,
    pub variants: Vec<StringVariant>,
}

#[derive(Clone, Debug)]
pub struct StringTable {
    entries: Vec<StringEntry>,
    by_id: HashMap<u64, usize>,
    /// End of all variable-length table records and beginning of the string page.
    pub string_page_offset: usize,
}

impl StringTable {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        Self::parse_with_limits(bytes, LocaleLimits::default())
    }

    pub fn parse_with_limits(bytes: &[u8], limits: LocaleLimits) -> Result<Self> {
        let mut r = Reader::new(bytes, limits)?;
        r.header(b"STR")?;
        let count = usize::from(r.u16()?);
        if count > limits.max_entries {
            return Err(error(4, "string entry count exceeds limit"));
        }
        r.require_records(count, 16)?;
        let mut raw_entries = reserved_vec(count, r.pos)?;
        let mut by_id = HashMap::new();
        by_id
            .try_reserve(count)
            .map_err(|_| error(r.pos, "ID index allocation failed"))?;
        let mut total_variants = 0usize;
        for index in 0..count {
            let source_offset = r.pos;
            let id = r.u64()?;
            if by_id.insert(id, index).is_some() {
                return Err(error(source_offset, "duplicate locale string ID"));
            }
            let num_variants = r.u16()?;
            let flags_produced = r.u16()?;
            let text_offset = r.offset()?;
            let variant_count = usize::from(num_variants.saturating_sub(1));
            if variant_count > limits.max_variants_per_entry {
                return Err(error(
                    source_offset + 8,
                    "entry variant count exceeds limit",
                ));
            }
            total_variants = total_variants
                .checked_add(variant_count)
                .ok_or_else(|| error(source_offset, "total variant count overflow"))?;
            if total_variants > limits.max_total_variants {
                return Err(error(source_offset, "total variant count exceeds limit"));
            }
            r.require_records(variant_count, 14)?;
            let mut variants = reserved_vec(variant_count, r.pos)?;
            for _ in 0..variant_count {
                variants.push(RawVariant {
                    source_offset: r.pos,
                    flags_consumed: r.u64()?,
                    flags_produced: r.u16()?,
                    text_offset: r.offset()?,
                });
            }
            raw_entries.push(RawEntry {
                source_offset,
                id,
                num_variants,
                flags_produced,
                text_offset,
                variants,
            });
        }

        let string_page_offset = r.pos;
        let string_count = count
            .checked_add(total_variants)
            .ok_or_else(|| error(r.pos, "total string count overflow"))?;
        let mut spans = reserved_vec(string_count, r.pos)?;
        let mut budget = TextBudget::new(limits);
        let mut entries = reserved_vec(count, r.pos)?;
        for raw in raw_entries {
            let default = budget.c_string(bytes, raw.text_offset, string_page_offset)?;
            spans.push((default.source_offset, default.source_end + 1));
            let mut variants = reserved_vec(raw.variants.len(), raw.source_offset)?;
            for variant in raw.variants {
                let text = budget.c_string(bytes, variant.text_offset, string_page_offset)?;
                spans.push((text.source_offset, text.source_end + 1));
                variants.push(StringVariant {
                    source_offset: variant.source_offset,
                    flags_consumed: variant.flags_consumed,
                    flags_produced: variant.flags_produced,
                    text,
                });
            }
            entries.push(StringEntry {
                id: raw.id,
                source_offset: raw.source_offset,
                num_variants: raw.num_variants,
                flags_produced: raw.flags_produced,
                default,
                variants,
            });
        }

        // Reusing one exact C-string span is allowed. Partial overlap, gaps, and
        // unreferenced trailing bytes are outside the observed packed-page format.
        spans.sort_unstable();
        spans.dedup();
        let mut covered = string_page_offset;
        for (start, end) in spans {
            if start < covered {
                return Err(error(start, "localized string spans overlap"));
            }
            if start > covered {
                return Err(error(covered, "gap in localized string page"));
            }
            covered = end;
        }
        if covered != bytes.len() {
            return Err(error(covered, "unreferenced trailing string-page bytes"));
        }
        Ok(Self {
            entries,
            by_id,
            string_page_offset,
        })
    }

    /// Entries retain file order; an ID's value is not assumed to encode its key.
    pub fn entries(&self) -> &[StringEntry] {
        &self.entries
    }

    /// A present empty string remains `Some`; a missing ID returns `None`.
    pub fn get(&self, id: u64) -> Option<&StringEntry> {
        self.by_id.get(&id).map(|&index| &self.entries[index])
    }
}

struct RawVariant {
    source_offset: usize,
    flags_consumed: u64,
    flags_produced: u16,
    text_offset: usize,
}

struct RawEntry {
    source_offset: usize,
    id: u64,
    num_variants: u16,
    flags_produced: u16,
    text_offset: usize,
    variants: Vec<RawVariant>,
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], limits: LocaleLimits) -> Result<Self> {
        if bytes.len() > limits.max_file_bytes {
            return Err(error(0, "file exceeds byte limit"));
        }
        Ok(Self { bytes, pos: 0 })
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(count)
            .ok_or_else(|| error(self.pos, "byte offset overflow"))?;
        let slice = self
            .bytes
            .get(self.pos..end)
            .ok_or_else(|| error(self.pos, "truncated localization data"))?;
        self.pos = end;
        Ok(slice)
    }

    fn header(&mut self, expected: &[u8; 3]) -> Result<()> {
        if self.take(3)? != expected {
            return Err(error(0, "unexpected localization signature"));
        }
        if self.u8()? != SUPPORTED_VERSION {
            return Err(error(3, "unsupported localization version"));
        }
        Ok(())
    }

    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn offset(&mut self) -> Result<usize> {
        usize::try_from(self.u32()?).map_err(|_| error(self.pos - 4, "string offset overflow"))
    }

    fn require_records(&self, count: usize, width: usize) -> Result<()> {
        let needed = count
            .checked_mul(width)
            .ok_or_else(|| error(self.pos, "record size overflow"))?;
        if needed > self.bytes.len() - self.pos {
            return Err(error(self.pos, "record count exceeds remaining bytes"));
        }
        Ok(())
    }

    fn prefixed_text(&mut self, budget: &mut TextBudget) -> Result<LocaleText> {
        let length = usize::from(self.u16()?);
        let start = self.pos;
        self.take(length)?;
        budget.copy_text(self.bytes, start, self.pos)
    }
}

struct TextBudget {
    limits: LocaleLimits,
    decoded: usize,
}

impl TextBudget {
    fn new(limits: LocaleLimits) -> Self {
        Self { limits, decoded: 0 }
    }

    fn copy_text(&mut self, bytes: &[u8], start: usize, end: usize) -> Result<LocaleText> {
        let slice = bytes
            .get(start..end)
            .ok_or_else(|| error(start, "string span outside file"))?;
        if slice.len() > self.limits.max_string_bytes {
            return Err(error(start, "localized string exceeds byte limit"));
        }
        let decoded = self
            .decoded
            .checked_add(slice.len())
            .ok_or_else(|| error(start, "decoded string byte count overflow"))?;
        if decoded > self.limits.max_decoded_bytes {
            return Err(error(start, "decoded strings exceed cumulative byte limit"));
        }
        let value = std::str::from_utf8(slice)
            .map_err(|_| error(start, "invalid UTF-8 in localized string"))?;
        let mut text = String::new();
        text.try_reserve_exact(value.len())
            .map_err(|_| error(start, "string allocation failed"))?;
        text.push_str(value);
        self.decoded = decoded;
        Ok(LocaleText {
            text,
            source_offset: start,
            source_end: end,
        })
    }

    fn c_string(&mut self, bytes: &[u8], start: usize, page: usize) -> Result<LocaleText> {
        if start < page {
            return Err(error(start, "string offset points into table metadata"));
        }
        if start >= bytes.len() {
            return Err(error(start, "string offset outside file"));
        }
        let available = bytes.len() - start;
        let scan_len = available.min(self.limits.max_string_bytes.saturating_add(1));
        let length = bytes[start..start + scan_len]
            .iter()
            .position(|&byte| byte == 0)
            .ok_or_else(|| {
                error(
                    start,
                    if available > self.limits.max_string_bytes {
                        "localized string exceeds byte limit"
                    } else {
                        "localized string lacks NUL terminator"
                    },
                )
            })?;
        self.copy_text(bytes, start, start + length)
    }
}

fn reserved_vec<T>(count: usize, offset: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| error(offset, "record allocation failed"))?;
    Ok(values)
}

fn error(offset: usize, message: &'static str) -> Error {
    Error { offset, message }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata() -> Vec<u8> {
        let mut bytes = b"LOC\x02".to_vec();
        for text in ["English", "français", "Everywhere", "eng.all"] {
            bytes.extend_from_slice(&(text.len() as u16).to_le_bytes());
            bytes.extend_from_slice(text.as_bytes());
        }
        bytes.push(1);
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(&15u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.push(b'P');
        bytes
    }

    type TestEntry<'a> = (u64, u16, &'a str, Vec<(u64, u16, &'a str)>);

    fn table(records: &[TestEntry<'_>]) -> Vec<u8> {
        let mut bytes = b"STR\x02".to_vec();
        bytes.extend_from_slice(&(records.len() as u16).to_le_bytes());
        let mut pending = Vec::new();
        for (id, flags, text, variants) in records {
            bytes.extend_from_slice(&id.to_le_bytes());
            bytes.extend_from_slice(&(1 + variants.len() as u16).to_le_bytes());
            bytes.extend_from_slice(&flags.to_le_bytes());
            pending.push((bytes.len(), *text));
            bytes.extend_from_slice(&0u32.to_le_bytes());
            for (consumed, produced, text) in variants {
                bytes.extend_from_slice(&consumed.to_le_bytes());
                bytes.extend_from_slice(&produced.to_le_bytes());
                pending.push((bytes.len(), *text));
                bytes.extend_from_slice(&0u32.to_le_bytes());
            }
        }
        for (field, text) in pending {
            let offset = bytes.len() as u32;
            bytes[field..field + 4].copy_from_slice(&offset.to_le_bytes());
            bytes.extend_from_slice(text.as_bytes());
            bytes.push(0);
        }
        bytes
    }

    #[test]
    fn locale_metadata_preserves_unicode_flags_and_spans() {
        let bytes = metadata();
        let locale = LocaleMetadata::parse(&bytes).unwrap();
        assert_eq!(locale.name.text, "English");
        assert_eq!(locale.language_display_name.text, "français");
        assert_eq!(locale.directory.text, "eng.all");
        let text = &locale.language_display_name;
        assert_eq!(
            &bytes[text.source_offset..text.source_end],
            "français".as_bytes()
        );
        assert_eq!(locale.flags[0].value, 4);
        assert_eq!(locale.flags[0].mask, 15);
        assert_eq!(locale.flags[0].text.text, "P");
        let mut trailing = bytes;
        trailing.push(0);
        assert!(LocaleMetadata::parse(&trailing).is_err());
    }

    #[test]
    fn strings_preserve_variants_unicode_and_empty_lookup() {
        let mut bytes = table(&[
            (
                9,
                7,
                "café $Name$",
                vec![(0xffff_ffff_ffff_fff4, 2, "中文")],
            ),
            (0, 2, "", vec![]),
        ]);
        let strings = StringTable::parse(&bytes).unwrap();
        assert_eq!(strings.entries().len(), 2);
        let entry = strings.get(9).unwrap();
        assert_eq!(entry.default.text, "café $Name$");
        assert_eq!(entry.num_variants, 2);
        assert_eq!(entry.flags_produced, 7);
        assert_eq!(entry.variants[0].flags_consumed, 0xffff_ffff_ffff_fff4);
        assert_eq!(entry.variants[0].flags_produced, 2);
        assert_eq!(entry.variants[0].text.text, "中文");
        assert_eq!(strings.get(0).unwrap().default.text, "");
        assert!(strings.get(1).is_none());
        // The zero-count representation also retains its one default string.
        bytes[44..46].copy_from_slice(&0u16.to_le_bytes());
        assert_eq!(
            StringTable::parse(&bytes)
                .unwrap()
                .get(0)
                .unwrap()
                .num_variants,
            0
        );
        assert!(
            StringTable::parse(b"STR\x02\0\0")
                .unwrap()
                .entries()
                .is_empty()
        );
    }

    #[test]
    fn duplicate_ids_headers_and_truncated_counts_are_rejected() {
        assert!(StringTable::parse(&table(&[(1, 0, "a", vec![]), (1, 0, "b", vec![])])).is_err());
        assert!(StringTable::parse(b"STR\x01\0\0").is_err());
        assert!(StringTable::parse(b"LOC\x02\0\0").is_err());
        assert!(StringTable::parse(b"STR\x02\x01\0").is_err());
        let mut truncated = table(&[(1, 0, "a", vec![])]);
        truncated[14..16].copy_from_slice(&2u16.to_le_bytes());
        assert!(StringTable::parse(&truncated).is_err());
    }

    #[test]
    fn offsets_require_complete_nonoverlapping_pages_but_allow_exact_reuse() {
        let bytes = table(&[(1, 0, "a", vec![])]);
        for offset in [0u32, 6, bytes.len() as u32, u32::MAX, 23] {
            let mut damaged = bytes.clone();
            damaged[18..22].copy_from_slice(&offset.to_le_bytes());
            assert!(StringTable::parse(&damaged).is_err(), "offset {offset}");
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert!(StringTable::parse(&trailing).is_err());
        let mut overlap = table(&[(1, 0, "abcd", vec![]), (2, 0, "x", vec![])]);
        overlap[34..38].copy_from_slice(&39u32.to_le_bytes());
        overlap.truncate(43);
        assert!(StringTable::parse(&overlap).is_err());
        let mut shared = table(&[(1, 0, "a", vec![]), (2, 0, "a", vec![])]);
        shared[34..38].copy_from_slice(&38u32.to_le_bytes());
        shared.truncate(40);
        assert_eq!(
            StringTable::parse(&shared)
                .unwrap()
                .get(2)
                .unwrap()
                .default
                .text,
            "a"
        );
        let limits = LocaleLimits {
            max_decoded_bytes: 1,
            ..LocaleLimits::default()
        };
        assert!(StringTable::parse_with_limits(&shared, limits).is_err());
    }

    #[test]
    fn malformed_utf8_and_unterminated_strings_are_rejected() {
        let mut bytes = table(&[(1, 0, "a", vec![])]);
        bytes[22] = 0xff;
        assert!(StringTable::parse(&bytes).is_err());
        bytes[22] = b'a';
        bytes[23] = b'x';
        assert!(StringTable::parse(&bytes).is_err());
        let mut locale = metadata();
        locale[6] = 0xff;
        assert!(LocaleMetadata::parse(&locale).is_err());
    }

    #[test]
    fn configured_budgets_bound_file_records_and_decoded_strings() {
        let bytes = table(&[(1, 0, "é", vec![(4, 2, "中")])]);
        for limits in [
            LocaleLimits {
                max_file_bytes: bytes.len() - 1,
                ..LocaleLimits::default()
            },
            LocaleLimits {
                max_entries: 0,
                ..LocaleLimits::default()
            },
            LocaleLimits {
                max_variants_per_entry: 0,
                ..LocaleLimits::default()
            },
            LocaleLimits {
                max_total_variants: 0,
                ..LocaleLimits::default()
            },
            LocaleLimits {
                max_string_bytes: 1,
                ..LocaleLimits::default()
            },
            LocaleLimits {
                max_decoded_bytes: 4,
                ..LocaleLimits::default()
            },
        ] {
            assert!(StringTable::parse_with_limits(&bytes, limits).is_err());
        }
        let limits = LocaleLimits {
            max_flags: 0,
            ..LocaleLimits::default()
        };
        assert!(LocaleMetadata::parse_with_limits(&metadata(), limits).is_err());
    }
}

/// An explicitly selected locale and its string partitions. Unknown IDs remain
/// None; a stored empty string remains Some with an empty text value.
pub struct LocaleCatalog {
    pub metadata: LocaleMetadata,
    tables: Vec<StringTable>,
    by_id: std::collections::HashMap<u64, (usize, usize)>,
}

impl LocaleCatalog {
    pub fn new(metadata: LocaleMetadata, tables: Vec<StringTable>) -> std::io::Result<Self> {
        if tables.len() > 64 {
            return Err(locale_error("locale partition limit exceeded"));
        }
        let mut count = 0usize;
        for table in &tables {
            count = count
                .checked_add(table.entries().len())
                .ok_or_else(|| locale_error("locale entry count overflow"))?;
        }
        if count > 1_000_000 {
            return Err(locale_error("combined locale entry limit exceeded"));
        }
        let mut by_id = std::collections::HashMap::new();
        by_id
            .try_reserve(count)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::OutOfMemory, e))?;
        for (table_index, table) in tables.iter().enumerate() {
            for (entry_index, entry) in table.entries().iter().enumerate() {
                if by_id.insert(entry.id, (table_index, entry_index)).is_some() {
                    return Err(locale_error("duplicate string ID across locale partitions"));
                }
            }
        }
        Ok(Self {
            metadata,
            tables,
            by_id,
        })
    }

    /// Paths are supplied by the caller. Serialized locale directory text is
    /// metadata only and is never joined onto a filesystem path here.
    pub fn open<P: AsRef<std::path::Path>>(
        metadata_path: impl AsRef<std::path::Path>,
        string_paths: impl IntoIterator<Item = P>,
    ) -> std::io::Result<Self> {
        let bytes = read_locale_file(metadata_path.as_ref())?;
        let mut total_bytes = bytes.len();
        let metadata = LocaleMetadata::parse(&bytes)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        let mut tables = Vec::new();
        for path in string_paths {
            if tables.len() >= 64 {
                return Err(locale_error("locale partition limit exceeded"));
            }
            let bytes = read_locale_file(path.as_ref())?;
            total_bytes = total_bytes
                .checked_add(bytes.len())
                .ok_or_else(|| locale_error("locale byte sum overflow"))?;
            if total_bytes > 256 * 1024 * 1024 {
                return Err(locale_error("combined locale file byte limit exceeded"));
            }
            tables
                .push(StringTable::parse(&bytes).map_err(|error| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
                })?);
        }
        Self::new(metadata, tables)
    }

    pub fn get(&self, id: u64) -> Option<&StringEntry> {
        let &(table, entry) = self.by_id.get(&id)?;
        self.tables.get(table)?.entries().get(entry)
    }
    pub fn len(&self) -> usize {
        self.by_id.len()
    }
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
    pub fn tables(&self) -> &[StringTable] {
        &self.tables
    }
}

/// Directly serialized UI localization fields, resolved through existing
/// schema-aware prototype decoding. Absent fields are not inherited or guessed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiKeyBinding {
    pub key: String,
    pub member: String,
    pub serialized_parent_reference: Option<u64>,
    pub display_text_id: Option<u64>,
    pub tooltip_text_id: Option<u64>,
}

impl UiKeyBinding {
    /// Resolves the exact archive identifier UI/LocalizedInfo/{key}.prototype.
    /// None means no such key; Some with an absent field means its value was not
    /// serialized directly. A present ID may still be missing from the locale.
    pub fn read(
        reader: &mut crate::prototype_catalog::PrototypeReader<'_>,
        key: &str,
    ) -> std::io::Result<Option<Self>> {
        validate_ui_key(key)?;
        let member = format!("Calligraphy/UI/LocalizedInfo/{key}.prototype");
        let prototype = match reader.read_member(&member) {
            Ok(prototype) => prototype,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let mut result = Self {
            key: key.to_owned(),
            member,
            serialized_parent_reference: prototype.header.reference,
            display_text_id: None,
            tooltip_text_id: None,
        };
        for group in &prototype.groups {
            for field in &group.simple_fields {
                let schema = reader
                    .blueprint(field.schema_blueprint_id)
                    .ok_or_else(|| locale_error("localized UI field schema is unavailable"))?;
                let declaration = schema
                    .members
                    .iter()
                    .find(|item| item.id == field.id)
                    .ok_or_else(|| locale_error("localized UI field declaration is unavailable"))?;
                let target = match declaration.name.as_str() {
                    "DisplayText" => &mut result.display_text_id,
                    "TooltipText" => &mut result.tooltip_text_id,
                    _ => continue,
                };
                if group.copy_number != 0 || schema.runtime_binding != "UILocalizedInfoPrototype" {
                    return Err(locale_error("unsupported localized UI field ownership"));
                }
                let crate::calligraphy::prototype::FieldData::Simple(
                    crate::calligraphy::prototype::Value::String(id),
                ) = field.data
                else {
                    return Err(locale_error("localized UI text field is not a string ID"));
                };
                if target.replace(id).is_some() {
                    return Err(locale_error("ambiguous duplicate localized UI text field"));
                }
            }
        }
        Ok(Some(result))
    }
}

fn validate_ui_key(key: &str) -> std::io::Result<()> {
    if key.is_empty()
        || key.len() > 4096
        || key.chars().any(|c| c.is_control() || "\\:".contains(c))
        || key
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(locale_error("invalid localized UI key"));
    }
    Ok(())
}

fn read_locale_file(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    const MAX: usize = 64 * 1024 * 1024;
    let file = std::fs::File::open(path)?;
    let len = usize::try_from(file.metadata()?.len())
        .map_err(|_| locale_error("locale file size overflow"))?;
    if len > MAX {
        return Err(locale_error("locale file byte limit exceeded"));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(len)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::OutOfMemory, e))?;
    file.take(MAX as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > MAX {
        return Err(locale_error("locale file grew past byte limit"));
    }
    Ok(bytes)
}

fn locale_error(message: &'static str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod catalog_tests {
    use super::*;
    fn metadata() -> LocaleMetadata {
        LocaleMetadata::parse(&[b'L', b'O', b'C', 2, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap()
    }
    fn table(id: u64, text: &str) -> StringTable {
        let mut bytes = b"STR\x02\x01\x00".to_vec();
        bytes.extend_from_slice(&id.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&22u32.to_le_bytes());
        bytes.extend_from_slice(text.as_bytes());
        bytes.push(0);
        StringTable::parse(&bytes).unwrap()
    }
    #[test]
    fn catalog_keeps_empty_and_missing_distinct_and_rejects_cross_partition_collisions() {
        let catalog =
            LocaleCatalog::new(metadata(), vec![table(0, ""), table(7, "example")]).unwrap();
        assert_eq!(catalog.len(), 2);
        assert!(!catalog.is_empty());
        assert_eq!(catalog.get(0).unwrap().default.text, "");
        assert_eq!(catalog.get(7).unwrap().default.text, "example");
        assert!(catalog.get(8).is_none());
        assert!(LocaleCatalog::new(metadata(), vec![table(7, "a"), table(7, "b")]).is_err());
        let empty = StringTable::parse(b"STR\x02\x00\x00").unwrap();
        assert!(LocaleCatalog::new(metadata(), vec![empty; 65]).is_err());
    }
    #[test]
    fn ui_key_is_an_exact_bounded_archive_identifier() {
        for key in ["Frontend/Example", "Nested/Example_Label"] {
            validate_ui_key(key).unwrap();
        }
        for key in [
            "",
            "/Example",
            "../Example",
            "Frontend/./Example",
            "Frontend//Example",
            "C:Example",
            "Frontend\\Example",
            "Example\0",
        ] {
            assert!(validate_ui_key(key).is_err(), "{key:?}");
        }
        assert!(validate_ui_key(&"x".repeat(4097)).is_err());
    }
}
