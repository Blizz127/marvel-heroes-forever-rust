//! Bounded readers for Calligraphy v11 metadata. No game data is embedded here.

use std::fmt;

pub const SUPPORTED_VERSION: u8 = 11;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    pub offset: usize,
    pub message: &'static str,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Calligraphy at byte {}: {}", self.offset, self.message)
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileKind {
    CurveDirectory,
    TypeDirectory,
    BlueprintDirectory,
    PrototypeDirectory,
    ReplacementDirectory,
    Curve,
    AssetType,
    Blueprint,
    Prototype,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Header {
    pub kind: FileKind,
    pub version: u8,
}

impl Header {
    /// Inspect the four-byte header; the body is not decoded by this method.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        Reader::new(bytes).header()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectoryKind {
    Curve,
    Type,
    Blueprint,
    Prototype,
    Replacement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenericRecord {
    pub id: u64,
    pub guid: u64,
    pub flags: u8,
    /// Archive-relative path, with Windows separators normalized to `/`.
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrototypeRecord {
    pub id: u64,
    pub guid: u64,
    pub blueprint_id: u64,
    pub flags: u8,
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplacementRecord {
    pub old_guid: u64,
    pub new_guid: u64,
    /// A display name, not a filesystem path.
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DirectoryRecord {
    Generic(GenericRecord),
    Prototype(PrototypeRecord),
    Replacement(ReplacementRecord),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Directory {
    pub kind: DirectoryKind,
    pub records: Vec<DirectoryRecord>,
}

impl Directory {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let kind = match r.header()?.kind {
            FileKind::CurveDirectory => DirectoryKind::Curve,
            FileKind::TypeDirectory => DirectoryKind::Type,
            FileKind::BlueprintDirectory => DirectoryKind::Blueprint,
            FileKind::PrototypeDirectory => DirectoryKind::Prototype,
            FileKind::ReplacementDirectory => DirectoryKind::Replacement,
            _ => return Err(r.error("expected a directory file")),
        };
        let count = usize::try_from(r.u32()?).map_err(|_| r.error("record count overflow"))?;
        let minimum = match kind {
            DirectoryKind::Prototype => 27,
            DirectoryKind::Replacement => 18,
            _ => 19,
        };
        let mut records = r.records(count, minimum)?;
        for _ in 0..count {
            let id = r.u64()?;
            let guid = r.u64()?;
            records.push(match kind {
                DirectoryKind::Replacement => DirectoryRecord::Replacement(ReplacementRecord {
                    old_guid: id,
                    new_guid: guid,
                    name: r.string()?,
                }),
                DirectoryKind::Prototype => DirectoryRecord::Prototype(PrototypeRecord {
                    id,
                    guid,
                    blueprint_id: r.u64()?,
                    flags: r.flags(3)?,
                    path: r.path()?,
                }),
                _ => DirectoryRecord::Generic(GenericRecord {
                    id,
                    guid,
                    flags: r.flags(1)?,
                    path: r.path()?,
                }),
            });
        }
        r.finish()?;
        Ok(Self { kind, records })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    pub start: i32,
    pub end: i32,
    pub values: Vec<f64>,
}

impl Curve {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        r.expect(FileKind::Curve)?;
        let start = r.i32()?;
        let end = r.i32()?;
        let span = i64::from(end) - i64::from(start) + 1;
        if span <= 0 {
            return Err(r.error("curve has an invalid position range"));
        }
        let count = usize::try_from(span).map_err(|_| r.error("curve length overflow"))?;
        let mut values = r.records(count, 8)?;
        for _ in 0..count {
            values.push(f64::from_bits(r.u64()?));
        }
        r.finish()?;
        Ok(Self { start, end, values })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetValue {
    pub id: u64,
    pub guid: u64,
    pub flags: u8,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetType {
    pub assets: Vec<AssetValue>,
}

impl AssetType {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        r.expect(FileKind::AssetType)?;
        let count = usize::from(r.u16()?);
        let mut assets = r.records(count, 19)?;
        for _ in 0..count {
            assets.push(AssetValue {
                id: r.u64()?,
                guid: r.u64()?,
                flags: r.flags(1)?,
                name: r.string()?,
            });
        }
        r.finish()?;
        Ok(Self { assets })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaseType {
    Asset,
    Boolean,
    Curve,
    Double,
    Long,
    Prototype,
    EmbeddedPrototype,
    String,
    Type,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructureType {
    Simple,
    List,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlueprintReference {
    pub id: u64,
    pub copies: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlueprintMember {
    pub id: u64,
    pub name: String,
    pub base_type: BaseType,
    pub structure: StructureType,
    pub subtype: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blueprint {
    pub runtime_binding: String,
    pub default_prototype_id: u64,
    pub parents: Vec<BlueprintReference>,
    pub contributors: Vec<BlueprintReference>,
    pub members: Vec<BlueprintMember>,
}

impl Blueprint {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        r.expect(FileKind::Blueprint)?;
        let runtime_binding = r.string()?;
        let default_prototype_id = r.u64()?;
        let parents = r.references()?;
        let contributors = r.references()?;
        let count = usize::from(r.u16()?);
        let mut members = r.records(count, 12)?;
        for _ in 0..count {
            let id = r.u64()?;
            let name = r.string()?;
            let base_type = r.base_type()?;
            let structure = match r.u8()? {
                b'S' => StructureType::Simple,
                b'L' => StructureType::List,
                _ => return Err(r.error("unknown field structure type")),
            };
            let subtype = match base_type {
                BaseType::Asset
                | BaseType::Curve
                | BaseType::Prototype
                | BaseType::EmbeddedPrototype => Some(r.u64()?),
                _ => None,
            };
            members.push(BlueprintMember {
                id,
                name,
                base_type,
                structure,
                subtype,
            });
        }
        r.finish()?;
        Ok(Self {
            runtime_binding,
            default_prototype_id,
            parents,
            contributors,
            members,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrototypeDataHeader {
    pub reference: Option<u64>,
    pub data_exists: bool,
    pub polymorphic: bool,
}

impl PrototypeDataHeader {
    /// Decode only the data header (after `PTP\x0b`, or inside a prototype).
    /// Returns the header and number of bytes consumed. Field groups remain unread.
    pub fn parse(bytes: &[u8]) -> Result<(Self, usize)> {
        let mut r = Reader::new(bytes);
        let flags = r.flags(7)?;
        let reference = if flags & 1 != 0 { Some(r.u64()?) } else { None };
        Ok((
            Self {
                reference,
                data_exists: flags & 2 != 0,
                polymorphic: flags & 4 != 0,
            },
            r.position,
        ))
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    fn error(&self, message: &'static str) -> Error {
        Error {
            offset: self.position,
            message,
        }
    }
    fn take(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| self.error("byte range overflow"))?;
        let slice = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| self.error("truncated input"))?;
        self.position = end;
        Ok(slice)
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
    fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn header(&mut self) -> Result<Header> {
        let bytes = self.take(4)?;
        let kind = match &bytes[..3] {
            b"CDR" => FileKind::CurveDirectory,
            b"TDR" => FileKind::TypeDirectory,
            b"BDR" => FileKind::BlueprintDirectory,
            b"PDR" => FileKind::PrototypeDirectory,
            b"RDR" => FileKind::ReplacementDirectory,
            b"CRV" => FileKind::Curve,
            b"TYP" => FileKind::AssetType,
            b"BPT" => FileKind::Blueprint,
            b"PTP" => FileKind::Prototype,
            _ => return Err(self.error("unrecognized file signature")),
        };
        if bytes[3] != SUPPORTED_VERSION {
            return Err(self.error("unsupported export version (expected 11)"));
        }
        Ok(Header {
            kind,
            version: bytes[3],
        })
    }

    fn expect(&mut self, expected: FileKind) -> Result<()> {
        if self.header()?.kind != expected {
            return Err(self.error("unexpected file kind"));
        }
        Ok(())
    }

    fn flags(&mut self, known: u8) -> Result<u8> {
        let flags = self.u8()?;
        if flags & !known != 0 {
            return Err(self.error("unsupported flag bits"));
        }
        Ok(flags)
    }

    fn records<T>(&self, count: usize, minimum_bytes: usize) -> Result<Vec<T>> {
        // Counts must be physically plausible before any allocation occurs.
        if count > (self.bytes.len() - self.position) / minimum_bytes {
            return Err(self.error("record count exceeds remaining input"));
        }
        let mut records = Vec::new();
        records
            .try_reserve_exact(count)
            .map_err(|_| self.error("record allocation failed"))?;
        Ok(records)
    }

    fn string(&mut self) -> Result<String> {
        let length = usize::from(self.u16()?);
        let text = std::str::from_utf8(self.take(length)?)
            .map_err(|_| self.error("invalid UTF-8 string"))?;
        if text.contains('\0') {
            return Err(self.error("NUL in string"));
        }
        Ok(text.to_owned())
    }

    fn path(&mut self) -> Result<String> {
        let path = self.string()?.replace('\\', "/");
        if path.is_empty()
            || path.starts_with('/')
            || path.contains(':')
            || path.chars().any(char::is_control)
            || path
                .split('/')
                .any(|part| part == ".." || part == "." || part.is_empty())
        {
            return Err(self.error("unsafe or empty archive-relative path"));
        }
        Ok(path)
    }

    fn references(&mut self) -> Result<Vec<BlueprintReference>> {
        let count = usize::from(self.u16()?);
        let mut entries = self.records(count, 9)?;
        for _ in 0..count {
            entries.push(BlueprintReference {
                id: self.u64()?,
                copies: self.u8()?,
            });
        }
        Ok(entries)
    }

    fn base_type(&mut self) -> Result<BaseType> {
        Ok(match self.u8()? {
            b'A' => BaseType::Asset,
            b'B' => BaseType::Boolean,
            b'C' => BaseType::Curve,
            b'D' => BaseType::Double,
            b'L' => BaseType::Long,
            b'P' => BaseType::Prototype,
            b'R' => BaseType::EmbeddedPrototype,
            b'S' => BaseType::String,
            b'T' => BaseType::Type,
            _ => return Err(self.error("unknown field base type")),
        })
    }

    fn finish(&self) -> Result<()> {
        if self.position != self.bytes.len() {
            return Err(self.error("trailing bytes after file body"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string(output: &mut Vec<u8>, text: &str) {
        output.extend_from_slice(&(text.len() as u16).to_le_bytes());
        output.extend_from_slice(text.as_bytes());
    }

    fn directory(path: &str) -> Vec<u8> {
        let mut bytes = b"TDR\x0b".to_vec();
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&101_u64.to_le_bytes());
        bytes.extend_from_slice(&202_u64.to_le_bytes());
        bytes.push(1);
        string(&mut bytes, path);
        bytes
    }

    #[test]
    fn directory_normalizes_paths_and_preserves_ids() {
        let parsed = Directory::parse(&directory("Assets\\Hero.type")).unwrap();
        assert_eq!(parsed.kind, DirectoryKind::Type);
        assert_eq!(
            parsed.records,
            vec![DirectoryRecord::Generic(GenericRecord {
                id: 101,
                guid: 202,
                flags: 1,
                path: "Assets/Hero.type".into(),
            })]
        );
    }

    #[test]
    fn truncated_directory_rejected_at_every_boundary() {
        let bytes = directory("a.type");
        for boundary in 0..bytes.len() {
            assert!(
                Directory::parse(&bytes[..boundary]).is_err(),
                "boundary {boundary}"
            );
        }
    }

    #[test]
    fn reject_impossible_count_and_trailing_data() {
        assert!(Directory::parse(b"PDR\x0b\xff\xff\xff\xff").is_err());
        let mut bytes = directory("a.type");
        bytes.push(0);
        assert!(Directory::parse(&bytes).is_err());
    }

    #[test]
    fn reject_unsafe_paths_invalid_text_flags_and_versions() {
        for path in [
            "../a",
            "a/../b",
            "/absolute",
            "C:\\file",
            "a\0b",
            "a//b",
            ".",
            "",
        ] {
            assert!(Directory::parse(&directory(path)).is_err(), "path {path:?}");
        }
        let mut bytes = directory("a.type");
        bytes[27] = 0xff;
        assert!(Directory::parse(&bytes).is_err());
        let mut bytes = directory("a.type");
        bytes[24] = 0x80;
        assert!(Directory::parse(&bytes).is_err());
        assert!(Header::parse(b"CDR\x0a").is_err());
        assert!(Header::parse(b"XYZ\x0b").is_err());
    }

    #[test]
    fn prototype_and_replacement_directory_layouts() {
        let mut bytes = b"PDR\x0b".to_vec();
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        for id in [1_u64, 2, 3] {
            bytes.extend_from_slice(&id.to_le_bytes());
        }
        bytes.push(3);
        string(&mut bytes, "Hero.prototype");
        let parsed = Directory::parse(&bytes).unwrap();
        assert!(
            matches!(&parsed.records[0], DirectoryRecord::Prototype(record) if record.blueprint_id == 3)
        );
        let mut bytes = b"RDR\x0b".to_vec();
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        for id in [1_u64, 2] {
            bytes.extend_from_slice(&id.to_le_bytes());
        }
        string(&mut bytes, "not/a/filesystem: name");
        let parsed = Directory::parse(&bytes).unwrap();
        assert!(
            matches!(&parsed.records[0], DirectoryRecord::Replacement(record) if record.new_guid == 2)
        );
    }

    #[test]
    fn curve_signed_positions_and_overflow_boundary() {
        let mut bytes = b"CRV\x0b".to_vec();
        bytes.extend_from_slice(&(-2_i32).to_le_bytes());
        bytes.extend_from_slice(&(-1_i32).to_le_bytes());
        bytes.extend_from_slice(&1.5_f64.to_le_bytes());
        bytes.extend_from_slice(&(-2.5_f64).to_le_bytes());
        assert_eq!(Curve::parse(&bytes).unwrap().values, vec![1.5, -2.5]);
        bytes[4..8].copy_from_slice(&i32::MIN.to_le_bytes());
        bytes[8..12].copy_from_slice(&i32::MAX.to_le_bytes());
        assert!(Curve::parse(&bytes).is_err());
        bytes[4..8].copy_from_slice(&2_i32.to_le_bytes());
        bytes[8..12].copy_from_slice(&1_i32.to_le_bytes());
        assert!(Curve::parse(&bytes).is_err());
    }

    #[test]
    fn blueprint_subtype_alignment_and_asset_type() {
        let mut bytes = b"BPT\x0b".to_vec();
        string(&mut bytes, "Hero");
        bytes.extend_from_slice(&3_u64.to_le_bytes());
        bytes.extend_from_slice(&[0, 0, 0, 0, 2, 0]);
        for (id, name, kind, subtype) in [
            (10_u64, "Health", b'D', None),
            (20, "Power", b'P', Some(30_u64)),
        ] {
            bytes.extend_from_slice(&id.to_le_bytes());
            string(&mut bytes, name);
            bytes.extend_from_slice(&[kind, b'S']);
            if let Some(id) = subtype {
                bytes.extend_from_slice(&id.to_le_bytes());
            }
        }
        let blueprint = Blueprint::parse(&bytes).unwrap();
        assert_eq!(blueprint.members[1].subtype, Some(30));
        bytes.pop();
        assert!(Blueprint::parse(&bytes).is_err());
        let mut bytes = b"TYP\x0b\x01\x00".to_vec();
        bytes.extend_from_slice(&[0; 16]);
        bytes.push(0);
        string(&mut bytes, "Asset");
        assert_eq!(AssetType::parse(&bytes).unwrap().assets[0].name, "Asset");
    }

    #[test]
    fn prototype_header_consumes_only_optional_reference() {
        assert_eq!(
            PrototypeDataHeader::parse(&[2, 99]).unwrap(),
            (
                PrototypeDataHeader {
                    reference: None,
                    data_exists: true,
                    polymorphic: false
                },
                1
            )
        );
        let mut bytes = vec![7];
        bytes.extend_from_slice(&42_u64.to_le_bytes());
        let (header, consumed) = PrototypeDataHeader::parse(&bytes).unwrap();
        assert_eq!(header.reference, Some(42));
        assert_eq!(consumed, 9);
        assert!(header.polymorphic);
        assert!(PrototypeDataHeader::parse(&[1]).is_err());
        assert!(PrototypeDataHeader::parse(&[8]).is_err());
    }
}

pub mod prototype;
