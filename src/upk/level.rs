//! Checked Engine.Level link/URL prefix for the observed UE3 868/3 profile.
//!
//! Later native containers remain opaque. No linked object, URL, actor or
//! sequence is loaded or executed. See docs/UPK_LEVEL.md for evidence and limits.

use crate::upk::Package;
use crate::upk::objects::{ObjectProperties, Span};
use std::{io, ops::Range};

pub const MAX_LEVEL_ARRAY_REFERENCES: usize = 65_536;
pub const MAX_LEVEL_URL_STRINGS: usize = 4096;
pub const MAX_LEVEL_STRING_UNITS: usize = 1_048_576;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
fn unsupported(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message.into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelLinkSlot {
    /// Native +0x70; constructor initializes this to the Level itself.
    ActorArrayOwner,
    Actor(usize),
    Model,
    ModelComponent(usize),
    /// Native +0xe8; its first element is the source of GetGameSequence.
    Sequence(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelLink {
    pub slot: LevelLinkSlot,
    /// Null=0, positive=one-based export, negative=one-based import.
    pub reference: i32,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelReferenceArray {
    pub native_offset: usize,
    pub count_span: Span,
    pub elements_span: Span,
    /// Slice of LevelObject::links, preserving order, duplicates and nulls.
    pub links: Range<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelStringEncoding {
    ByteUnits,
    Wide16LeUnits,
}

/// Exact FString framing, without Unicode, terminator or runtime normalization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelString {
    pub serialized_count: i32,
    pub encoding: LevelStringEncoding,
    /// Count plus encoded payload, relative to the complete export.
    pub span: Span,
    /// All encoded units, including any terminator actually present.
    pub payload: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelWord {
    pub span: Span,
    pub bits: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelUrl {
    pub span: Span,
    pub protocol: LevelString,
    pub host: LevelString,
    pub map: LevelString,
    pub portal: LevelString,
    /// Serialized string array at FURL +0x34; its member name is unproved.
    pub extra_strings: Vec<LevelString>,
    pub extra_strings_count_span: Span,
    pub port: LevelWord,
    /// Serialized word at FURL +0x54; no boolean meaning is imposed.
    pub word_54: LevelWord,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelObject {
    pub object: ObjectProperties,
    /// Native serialization order, including the array owner/self reference.
    pub links: Vec<LevelLink>,
    pub actors: LevelReferenceArray,
    pub url: LevelUrl,
    pub model_components: LevelReferenceArray,
    pub sequences: LevelReferenceArray,
    /// Starts before native container +0xf8; later fields are not decoded.
    /// Its completeness is not validated by this prefix reader.
    pub unparsed_native_tail: Span,
}

impl LevelObject {
    pub fn parse_export(package: &Package, export_index: usize, bytes: &[u8]) -> io::Result<Self> {
        if (
            package.summary.file_version,
            package.summary.licensee_version,
        ) != (868, 3)
        {
            return Err(unsupported(
                "Level native prefix requires UE3 profile 868/3",
            ));
        }
        let class = package
            .export_class_name(export_index)
            .ok_or_else(|| invalid("Level export class is unresolved"))?;
        if !class.eq_ignore_ascii_case("Level") {
            return Err(unsupported("native Level reader requires Engine.Level"));
        }
        // Existing reader establishes the exact Engine class identity and prefix.
        let object = ObjectProperties::parse_export(package, export_index, bytes)?;
        let mut input = Reader {
            bytes,
            position: object.native_payload.offset,
        };
        let mut links = vec![input.reference(package, LevelLinkSlot::ActorArrayOwner)?];
        let actors = input.references(package, &mut links, 0x60, LevelLinkSlot::Actor)?;
        let url = input.url()?;
        links.push(input.reference(package, LevelLinkSlot::Model)?);
        let model_components =
            input.references(package, &mut links, 0xd8, LevelLinkSlot::ModelComponent)?;
        let sequences = input.references(package, &mut links, 0xe8, LevelLinkSlot::Sequence)?;
        let unparsed_native_tail = Span {
            offset: input.position,
            length: bytes.len() - input.position,
        };
        Ok(Self {
            object,
            links,
            actors,
            url,
            model_components,
            sequences,
            unparsed_native_tail,
        })
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl Reader<'_> {
    fn take(&mut self, length: usize) -> io::Result<Span> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| invalid("Level field offset overflow"))?;
        if end > self.bytes.len() {
            return Err(invalid("truncated Level native prefix"));
        }
        let span = Span {
            offset: self.position,
            length,
        };
        self.position = end;
        Ok(span)
    }
    fn word(&mut self) -> io::Result<LevelWord> {
        let span = self.take(4)?;
        Ok(LevelWord {
            span,
            bits: u32::from_le_bytes(span.bytes(self.bytes)?.try_into().unwrap()),
        })
    }
    fn reference(&mut self, package: &Package, slot: LevelLinkSlot) -> io::Result<LevelLink> {
        let word = self.word()?;
        let reference = word.bits as i32;
        if (reference > 0 && reference as usize > package.exports.len())
            || (reference < 0 && reference.unsigned_abs() as usize > package.imports.len())
        {
            return Err(invalid("Level object reference is outside package tables"));
        }
        Ok(LevelLink {
            slot,
            reference,
            span: word.span,
        })
    }
    fn count(&mut self, maximum: usize) -> io::Result<(usize, Span)> {
        let word = self.word()?;
        let signed = word.bits as i32;
        if signed < 0 {
            return Err(invalid("negative Level array count"));
        }
        let count = signed as usize;
        if count > maximum {
            return Err(unsupported("Level array count exceeds inspection limit"));
        }
        Ok((count, word.span))
    }
    fn references(
        &mut self,
        package: &Package,
        links: &mut Vec<LevelLink>,
        native_offset: usize,
        slot: fn(usize) -> LevelLinkSlot,
    ) -> io::Result<LevelReferenceArray> {
        let (count, count_span) = self.count(MAX_LEVEL_ARRAY_REFERENCES)?;
        let byte_count = count
            .checked_mul(4)
            .ok_or_else(|| invalid("Level reference-array size overflow"))?;
        if byte_count > self.bytes.len() - self.position {
            return Err(invalid("truncated Level reference array"));
        }
        let elements_span = Span {
            offset: self.position,
            length: byte_count,
        };
        let start = links.len();
        links.reserve(count);
        for index in 0..count {
            links.push(self.reference(package, slot(index))?);
        }
        Ok(LevelReferenceArray {
            native_offset,
            count_span,
            elements_span,
            links: start..links.len(),
        })
    }
    fn string(&mut self) -> io::Result<LevelString> {
        let count = self.word()?;
        let serialized_count = count.bits as i32;
        let units = serialized_count.unsigned_abs() as usize;
        if units > MAX_LEVEL_STRING_UNITS {
            return Err(unsupported("Level string exceeds inspection limit"));
        }
        let (encoding, width) = if serialized_count < 0 {
            (LevelStringEncoding::Wide16LeUnits, 2)
        } else {
            (LevelStringEncoding::ByteUnits, 1)
        };
        let byte_count = units
            .checked_mul(width)
            .ok_or_else(|| invalid("Level string length overflow"))?;
        let payload = self.take(byte_count)?;
        Ok(LevelString {
            serialized_count,
            encoding,
            span: Span {
                offset: count.span.offset,
                length: self.position - count.span.offset,
            },
            payload,
        })
    }
    fn url(&mut self) -> io::Result<LevelUrl> {
        let start = self.position;
        let protocol = self.string()?;
        let host = self.string()?;
        let map = self.string()?;
        let portal = self.string()?;
        let (count, extra_strings_count_span) = self.count(MAX_LEVEL_URL_STRINGS)?;
        let minimum = count
            .checked_mul(4)
            .ok_or_else(|| invalid("Level URL string-array size overflow"))?;
        if minimum > self.bytes.len() - self.position {
            return Err(invalid("truncated Level URL string array"));
        }
        let mut extra_strings = Vec::with_capacity(count);
        for _ in 0..count {
            extra_strings.push(self.string()?);
        }
        let port = self.word()?;
        let word_54 = self.word()?;
        Ok(LevelUrl {
            span: Span {
                offset: start,
                length: self.position - start,
            },
            protocol,
            host,
            map,
            portal,
            extra_strings,
            extra_strings_count_span,
            port,
            word_54,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upk::{Export, Import, NameEntry, NameRef, Summary, TableLocation};
    fn name(index: usize) -> NameRef {
        NameRef { index, number: 0 }
    }
    fn package(size: usize) -> Package {
        let table = TableLocation {
            count: 0,
            offset: 0,
        };
        Package {
            summary: Summary {
                file_version: 868,
                licensee_version: 3,
                header_size: 0,
                group: String::new(),
                package_flags: 0,
                names: table,
                exports: table,
                imports: table,
                depends_offset: 0,
                import_export_guids_offset: 0,
                import_guids_count: 0,
                export_guids_count: 0,
                thumbnail_table_offset: 0,
                guid: [0; 16],
                generations: Vec::new(),
                engine_version: 0,
                cooker_version: 0,
                compression_flags: 0,
                chunks: Vec::new(),
                package_source: 0,
                parsed_bytes: 0,
            },
            names: [
                "None", "Core", "Class", "Package", "Engine", "Level", "Instance",
            ]
            .into_iter()
            .map(|value| NameEntry {
                value: value.into(),
                flags: 0,
            })
            .collect(),
            imports: vec![
                Import {
                    class_package: name(1),
                    class_name: name(2),
                    outer_index: -2,
                    object_name: name(5),
                },
                Import {
                    class_package: name(1),
                    class_name: name(3),
                    outer_index: 0,
                    object_name: name(4),
                },
            ],
            exports: vec![Export {
                class_index: -1,
                super_index: 0,
                outer_index: 0,
                object_name: name(6),
                archetype_index: 0,
                object_flags: 0,
                serial_size: size,
                serial_offset: 4096,
                export_flags: 0,
                generation_net_object_counts: Vec::new(),
                guid: [0; 16],
                package_flags: 0,
            }],
        }
    }
    fn word(bytes: &mut Vec<u8>, value: i32) {
        bytes.extend(value.to_le_bytes());
    }
    fn text(bytes: &mut Vec<u8>, count: i32, raw: &[u8]) {
        word(bytes, count);
        bytes.extend(raw);
    }
    fn fixture() -> Vec<u8> {
        let mut b = Vec::new();
        for value in [-1, 0, 0, 1, 3, 1, 0, -1] {
            word(&mut b, value);
        }
        text(&mut b, 3, &[0xff, b'X', 0]);
        text(&mut b, 0, &[]);
        text(&mut b, -2, &[0, 0xd8, 0, 0]);
        text(&mut b, 1, b"Q");
        word(&mut b, 2);
        text(&mut b, 2, b"?\0");
        text(&mut b, -1, &[0xff, 0xff]);
        for value in [-17, 0x7fc0_0001, 1, 2, -1, 1, 2, 0, 1] {
            word(&mut b, value);
        }
        b
    }
    #[test]
    fn preserves_order_raw_strings_words_and_opaque_suffix() {
        let mut bytes = fixture();
        let prefix_end = bytes.len();
        bytes.extend([39, 71, 113]);
        let level = LevelObject::parse_export(&package(bytes.len()), 0, &bytes).unwrap();
        assert_eq!(
            level
                .links
                .iter()
                .map(|link| link.reference)
                .collect::<Vec<_>>(),
            [1, 1, 0, -1, 1, -1, 1, 0, 1]
        );
        assert_eq!(level.actors.links, 1..4);
        assert_eq!(level.model_components.links, 5..7);
        assert_eq!(level.sequences.links, 7..9);
        assert_eq!(level.links[8].slot, LevelLinkSlot::Sequence(1));
        assert_eq!(
            level.url.protocol.payload.bytes(&bytes).unwrap(),
            &[0xff, b'X', 0]
        );
        assert_eq!(level.url.map.encoding, LevelStringEncoding::Wide16LeUnits);
        assert_eq!(
            level.url.map.payload.bytes(&bytes).unwrap(),
            &[0, 0xd8, 0, 0]
        );
        assert_eq!(level.url.portal.payload.bytes(&bytes).unwrap(), b"Q");
        assert_eq!(level.url.extra_strings[1].serialized_count, -1);
        assert_eq!(level.url.port.bits, (-17i32) as u32);
        assert_eq!(level.url.word_54.bits, 0x7fc0_0001);
        assert_eq!(
            level.unparsed_native_tail,
            Span {
                offset: prefix_end,
                length: 3
            }
        );
        assert_eq!(
            level.unparsed_native_tail.bytes(&bytes).unwrap(),
            &[39, 71, 113]
        );
    }
    #[test]
    fn rejects_prefix_truncation_bad_counts_and_invalid_references() {
        let bytes = fixture();
        let parsed = LevelObject::parse_export(&package(bytes.len()), 0, &bytes).unwrap();
        for length in 0..bytes.len() {
            assert!(
                LevelObject::parse_export(&package(length), 0, &bytes[..length]).is_err(),
                "accepted prefix {length}"
            );
        }
        let edits = [
            (parsed.actors.count_span.offset, -1),
            (parsed.actors.count_span.offset, 65537),
            (parsed.url.protocol.span.offset, i32::MIN),
            (parsed.url.extra_strings_count_span.offset, 4097),
            (parsed.url.extra_strings_count_span.offset, -1),
            (parsed.model_components.count_span.offset, i32::MAX),
            (parsed.sequences.count_span.offset, 3),
            (parsed.links[0].span.offset, 2),
            (parsed.links[3].span.offset, i32::MIN),
            (parsed.links[8].span.offset, -3),
        ];
        for (offset, value) in edits {
            let mut changed = bytes.clone();
            changed[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            assert!(LevelObject::parse_export(&package(changed.len()), 0, &changed).is_err());
        }
    }
    #[test]
    fn requires_native_level_identity_and_version_profile() {
        let bytes = fixture();
        for mutation in 0..6 {
            let mut p = package(bytes.len());
            match mutation {
                0 => p.names[5].value = "World".into(),
                1 => p.names[4].value = "Custom".into(),
                2 => p.exports[0].object_flags = 0x0200_0000_0000_0000,
                3 => p.summary.file_version = 867,
                4 => p.summary.licensee_version = 4,
                5 => p.imports[0].outer_index = 0,
                _ => unreachable!(),
            }
            assert!(LevelObject::parse_export(&p, 0, &bytes).is_err());
        }
        assert!(LevelObject::parse_export(&package(bytes.len()), 1, &bytes).is_err());
    }
}
