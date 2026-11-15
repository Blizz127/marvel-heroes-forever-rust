//! Bounded structural font and text readers. Spans address the original movie.
//! No font substitution, text layout, ActionScript execution, or rasterization.

#[allow(unused_imports)] // Public API also used by future renderer consumers.
pub use records::{EditFlags, EditLayout, EditText, Glyph, StaticText, TextRecord};

use crate::gfx::{Rect, Span};
use std::io::{self, ErrorKind};

const MAX_FONT_BYTES: usize = 64 * 1024 * 1024;
const MAX_GLYPH_RECORDS: usize = 1_000_000;

/// The raw flags of DefineFont2/3, in wire order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FontFlags(pub u8);
impl FontFlags {
    pub fn has_layout(self) -> bool {
        self.0 & 0x80 != 0
    }
    pub fn shift_jis(self) -> bool {
        self.0 & 0x40 != 0
    }
    pub fn small_text(self) -> bool {
        self.0 & 0x20 != 0
    }
    pub fn ansi(self) -> bool {
        self.0 & 0x10 != 0
    }
    pub fn wide_offsets(self) -> bool {
        self.0 & 8 != 0
    }
    pub fn wide_codes(self) -> bool {
        self.0 & 4 != 0
    }
    pub fn italic(self) -> bool {
        self.0 & 2 != 0
    }
    pub fn bold(self) -> bool {
        self.0 & 1 != 0
    }
}

/// A validated SHAPE stream, retained as source bytes rather than interpreted paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlyphShape {
    pub source: Span,
    pub fill_bits: u8,
    pub line_bits: u8,
    pub record_count: usize,
    pub edge_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontGlyph {
    pub shape: GlyphShape,
    /// Raw character code, not a Rust Unicode scalar or a resolved alias.
    pub code: Option<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KerningPair {
    pub source: Span,
    pub left_code: u16,
    pub right_code: u16,
    pub adjustment: i16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontLayout {
    pub source: Span,
    pub ascent: u16,
    pub descent: u16,
    pub leading: i16,
    pub advances: Vec<i16>,
    /// Raw font EM coordinates; Rect pixel helpers assume twips and do not apply.
    pub bounds: Vec<Rect>,
    pub kerning: Vec<KerningPair>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontDefinition {
    pub source: Span,
    pub tag_code: u16,
    pub id: u16,
    pub flags: Option<FontFlags>,
    pub language: Option<u8>,
    /// Exact name bytes, including any serialized trailing NUL. No codepage guess.
    pub name: Option<Span>,
    pub glyphs: Vec<FontGlyph>,
    pub layout: Option<FontLayout>,
    /// DefineFont3 uses 20 times the 1024-unit EM square of earlier tags.
    pub units_per_em: u32,
}

impl FontDefinition {
    /// Read DefineFont (10), DefineFont2 (48), or DefineFont3 (75).
    /// `body_offset` is the tag body's absolute offset in the original movie.
    pub fn parse(tag_code: u16, body: &[u8], body_offset: usize) -> io::Result<Self> {
        if !matches!(tag_code, 10 | 48 | 75) {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "unsupported font definition tag",
            ));
        }
        if body.len() > MAX_FONT_BYTES {
            return Err(invalid("font body exceeds reader limit"));
        }
        body_offset
            .checked_add(body.len())
            .ok_or_else(|| invalid("font source offset overflow"))?;
        let source = Span {
            offset: body_offset,
            len: body.len(),
        };
        let mut r = FontReader {
            bytes: body,
            pos: 0,
            base: body_offset,
        };
        let id = r.u16()?;
        if tag_code == 10 {
            let table_start = r.pos;
            let first = usize::from(r.u16()?);
            if first < 2 || first % 2 != 0 {
                return Err(invalid("invalid DefineFont first glyph offset"));
            }
            let count = first / 2;
            r.require((count - 1) * 2)?;
            let mut offsets = Vec::with_capacity(count + 1);
            offsets.push(first);
            for _ in 1..count {
                offsets.push(usize::from(r.u16()?));
            }
            offsets.push(body.len() - table_start);
            let glyphs = read_glyphs(&r, table_start, &offsets, first)?;
            return Ok(Self {
                source,
                tag_code,
                id,
                flags: None,
                language: None,
                name: None,
                glyphs,
                layout: None,
                units_per_em: 1024,
            });
        }
        let flags = FontFlags(r.u8()?);
        if tag_code == 75 && !flags.wide_codes() {
            return Err(invalid("DefineFont3 requires wide character codes"));
        }
        let language = r.u8()?;
        let name_len = usize::from(r.u8()?);
        let name = r.take_span(name_len)?;
        let count = usize::from(r.u16()?);
        let table_start = r.pos;
        let mut glyphs = if count == 0 {
            // Observed in the original frontend: zero glyphs omit all offset tables,
            // including CodeTableOffset. Do not consume fictitious bytes here.
            Vec::new()
        } else {
            let width = if flags.wide_offsets() { 4 } else { 2 };
            let table_len = (count + 1) * width;
            r.require(table_len)?;
            let mut offsets = Vec::with_capacity(count + 1);
            for _ in 0..=count {
                offsets.push(r.offset(flags.wide_offsets())?);
            }
            let glyphs = read_glyphs(&r, table_start, &offsets, table_len)?;
            r.pos = table_start
                .checked_add(offsets[count])
                .ok_or_else(|| invalid("font code offset overflow"))?;
            glyphs
        };
        r.require(count * if flags.wide_codes() { 2 } else { 1 })?;
        for glyph in &mut glyphs {
            glyph.code = Some(r.code(flags.wide_codes())?);
        }
        let layout = if flags.has_layout() {
            let start = r.pos;
            let ascent = r.u16()?;
            let descent = r.u16()?;
            let leading = r.i16()?;
            r.require(count * 2)?;
            let mut advances = Vec::with_capacity(count);
            for _ in 0..count {
                advances.push(r.i16()?);
            }
            let mut bounds = Vec::with_capacity(count);
            for _ in 0..count {
                bounds.push(r.rect()?);
            }
            let kerning_count = usize::from(r.u16()?);
            r.require(kerning_count * if flags.wide_codes() { 6 } else { 4 })?;
            let mut kerning = Vec::with_capacity(kerning_count);
            for _ in 0..kerning_count {
                let begin = r.pos;
                let left_code = r.code(flags.wide_codes())?;
                let right_code = r.code(flags.wide_codes())?;
                let adjustment = r.i16()?;
                kerning.push(KerningPair {
                    source: r.span(begin, r.pos - begin),
                    left_code,
                    right_code,
                    adjustment,
                });
            }
            Some(FontLayout {
                source: r.span(start, r.pos - start),
                ascent,
                descent,
                leading,
                advances,
                bounds,
                kerning,
            })
        } else {
            None
        };
        if r.pos != body.len() {
            return Err(invalid("trailing bytes in font definition"));
        }
        Ok(Self {
            source,
            tag_code,
            id,
            flags: Some(flags),
            language: Some(language),
            name: Some(name),
            glyphs,
            layout,
            units_per_em: if tag_code == 75 { 20480 } else { 1024 },
        })
    }
}

fn read_glyphs(
    r: &FontReader<'_>,
    start: usize,
    offsets: &[usize],
    table_len: usize,
) -> io::Result<Vec<FontGlyph>> {
    if offsets.first() != Some(&table_len) {
        return Err(invalid("font first shape does not follow offset table"));
    }
    let mut glyphs = Vec::with_capacity(offsets.len() - 1);
    for pair in offsets.windows(2) {
        let begin = start
            .checked_add(pair[0])
            .ok_or_else(|| invalid("glyph offset overflow"))?;
        let end = start
            .checked_add(pair[1])
            .ok_or_else(|| invalid("glyph end overflow"))?;
        if begin >= end || end > r.bytes.len() {
            return Err(invalid("invalid glyph shape range"));
        }
        let source = r.span(begin, end - begin);
        let shape = read_shape(&r.bytes[begin..end], source)?;
        glyphs.push(FontGlyph { shape, code: None });
    }
    Ok(glyphs)
}

fn read_shape(bytes: &[u8], source: Span) -> io::Result<GlyphShape> {
    let mut r = FontBits { bytes, bit: 0 };
    let fill_bits = r.bits(4)? as u8;
    let line_bits = r.bits(4)? as u8;
    let mut record_count = 0;
    let mut edge_count = 0;
    loop {
        record_count += 1;
        if record_count > MAX_GLYPH_RECORDS {
            return Err(invalid("glyph record limit exceeded"));
        }
        if r.bits(1)? != 0 {
            edge_count += 1;
            let straight = r.bits(1)? != 0;
            let width = r.bits(4)? as u8 + 2;
            if straight {
                if r.bits(1)? != 0 {
                    r.bits(width)?;
                    r.bits(width)?;
                } else {
                    r.bits(1)?;
                    r.bits(width)?;
                }
            } else {
                for _ in 0..4 {
                    r.bits(width)?;
                }
            }
        } else {
            let flags = r.bits(5)?;
            if flags == 0 {
                break;
            }
            if flags & 0x10 != 0 {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "glyph SHAPE cannot introduce style arrays",
                ));
            }
            if flags & 1 != 0 {
                let width = r.bits(5)? as u8;
                r.bits(width)?;
                r.bits(width)?;
            }
            if flags & 2 != 0 {
                r.bits(fill_bits)?;
            }
            if flags & 4 != 0 {
                r.bits(fill_bits)?;
            }
            if flags & 8 != 0 {
                r.bits(line_bits)?;
            }
        }
    }
    while r.bit % 8 != 0 {
        if r.bits(1)? != 0 {
            return Err(invalid("nonzero glyph end padding"));
        }
    }
    if r.bit / 8 != bytes.len() {
        return Err(invalid("trailing bytes in glyph shape"));
    }
    Ok(GlyphShape {
        source,
        fill_bits,
        line_bits,
        record_count,
        edge_count,
    })
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}

struct FontReader<'a> {
    bytes: &'a [u8],
    pos: usize,
    base: usize,
}
impl FontReader<'_> {
    fn require(&self, n: usize) -> io::Result<()> {
        if n > self.bytes.len().saturating_sub(self.pos) {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "truncated font definition",
            ));
        }
        Ok(())
    }
    fn span(&self, start: usize, len: usize) -> Span {
        Span {
            offset: self.base + start,
            len,
        }
    }
    fn take_span(&mut self, len: usize) -> io::Result<Span> {
        self.require(len)?;
        let span = self.span(self.pos, len);
        self.pos += len;
        Ok(span)
    }
    fn u8(&mut self) -> io::Result<u8> {
        self.require(1)?;
        let v = self.bytes[self.pos];
        self.pos += 1;
        Ok(v)
    }
    fn u16(&mut self) -> io::Result<u16> {
        self.require(2)?;
        let v = u16::from_le_bytes(self.bytes[self.pos..self.pos + 2].try_into().unwrap());
        self.pos += 2;
        Ok(v)
    }
    fn i16(&mut self) -> io::Result<i16> {
        Ok(self.u16()? as i16)
    }
    fn offset(&mut self, wide: bool) -> io::Result<usize> {
        if !wide {
            return Ok(usize::from(self.u16()?));
        }
        self.require(4)?;
        let v = u32::from_le_bytes(self.bytes[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        usize::try_from(v).map_err(|_| invalid("font offset exceeds address space"))
    }
    fn code(&mut self, wide: bool) -> io::Result<u16> {
        if wide {
            self.u16()
        } else {
            Ok(u16::from(self.u8()?))
        }
    }
    fn rect(&mut self) -> io::Result<Rect> {
        let mut bits = FontBits {
            bytes: &self.bytes[self.pos..],
            bit: 0,
        };
        let width = bits.bits(5)? as u8;
        let rect = Rect {
            x_min: bits.signed(width)?,
            x_max: bits.signed(width)?,
            y_min: bits.signed(width)?,
            y_max: bits.signed(width)?,
        };
        self.pos += bits.bit.div_ceil(8);
        Ok(rect)
    }
}

struct FontBits<'a> {
    bytes: &'a [u8],
    bit: usize,
}
impl FontBits<'_> {
    fn bits(&mut self, width: u8) -> io::Result<u32> {
        if width > 32
            || usize::from(width) > self.bytes.len().saturating_mul(8).saturating_sub(self.bit)
        {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "truncated font bit record",
            ));
        }
        let mut value = 0;
        for _ in 0..width {
            value = (value << 1) | u32::from((self.bytes[self.bit / 8] >> (7 - self.bit % 8)) & 1);
            self.bit += 1;
        }
        Ok(value)
    }
    fn signed(&mut self, width: u8) -> io::Result<i32> {
        let value = self.bits(width)?;
        if width == 0 {
            return Ok(0);
        }
        Ok(((value << (32 - width)) as i32) >> (32 - width))
    }
}

#[cfg(test)]
mod font_tests {
    use super::*;

    fn one_glyph() -> Vec<u8> {
        // Font3, wide codes and layout; one empty glyph mapped to U+0041.
        vec![
            1, 0, 0x84, 0, 1, b'A', 1, 0, 4, 0, 6, 0, 0, 0, 65, 0, 0, 4, 0, 1, 0xff, 0xff, 0, 2, 0,
            1, 0, 65, 0, 66, 0, 0xfe, 0xff,
        ]
    }

    #[test]
    fn preserves_codes_metrics_and_absolute_shape_spans() {
        let bytes = one_glyph();
        let font = FontDefinition::parse(75, &bytes, 100).unwrap();
        assert_eq!(
            font.glyphs[0].shape.source,
            Span {
                offset: 112,
                len: 2
            }
        );
        assert_eq!(font.glyphs[0].code, Some(65));
        assert_eq!(font.units_per_em, 20480);
        let layout = font.layout.unwrap();
        assert_eq!(
            (layout.ascent, layout.descent, layout.leading),
            (1024, 256, -1)
        );
        assert_eq!(layout.advances, [512]);
        assert_eq!(layout.kerning[0].adjustment, -2);
    }

    #[test]
    fn truncations_offsets_and_unterminated_shapes_fail() {
        let bytes = one_glyph();
        for end in 0..bytes.len() {
            assert!(
                FontDefinition::parse(75, &bytes[..end], 0).is_err(),
                "prefix {end}"
            );
        }
        for (index, value) in [(8, 2), (10, 3), (10, 255), (13, 1)] {
            let mut broken = bytes.clone();
            broken[index] = value;
            assert!(FontDefinition::parse(75, &broken, 0).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(FontDefinition::parse(75, &trailing, 0).is_err());
        assert!(FontDefinition::parse(75, &bytes, usize::MAX).is_err());
    }

    #[test]
    fn observed_empty_font_and_legacy_offset_table() {
        let empty = [1, 0, 4, 0, 1, 0, 0, 0];
        assert!(FontDefinition::parse(75, &empty, 0)
            .unwrap()
            .glyphs
            .is_empty());
        let old = FontDefinition::parse(10, &[1, 0, 2, 0, 0, 0], 30).unwrap();
        assert_eq!(old.glyphs[0].shape.source.offset, 34);
        assert_eq!(old.glyphs[0].code, None);
        assert!(FontDefinition::parse(1005, &[], 0).is_err());
    }

    #[test]
    fn narrow_font2_codes_and_unsupported_glyph_styles() {
        // DefineFont2 with no layout, narrow offsets and narrow character codes.
        let narrow = [2, 0, 0, 0, 0, 1, 0, 4, 0, 6, 0, 0, 0, 65];
        let font = FontDefinition::parse(48, &narrow, 0).unwrap();
        assert_eq!(font.glyphs[0].code, Some(65));
        assert_eq!(font.units_per_em, 1024);
        assert!(FontDefinition::parse(75, &narrow, 0).is_err());
        // SHAPE flags introducing style arrays are forbidden in glyph streams.
        let mut forbidden = narrow;
        forbidden[12] = 0x40;
        assert_eq!(
            FontDefinition::parse(48, &forbidden, 0).unwrap_err().kind(),
            ErrorKind::Unsupported
        );
    }
}

/// Structural text records from SWF DefineText, DefineText2 and DefineEditText.
///
/// Readers preserve glyph indices, signed advances, explicit style changes and
/// editable-text flags. They do not resolve fonts, execute HTML/ActionScript or
/// assert that the text will render like the native client.
pub mod records {
    use crate::gfx::{Matrix, Rect, Span};
    use std::io;

    pub const MAX_TEXT_BODY_BYTES: usize = 8 * 1024 * 1024;
    pub const MAX_TEXT_RECORDS: usize = 65_536;
    pub const MAX_TEXT_GLYPHS: usize = 1_000_000;
    pub const MAX_TEXT_STRING_BYTES: usize = 65_536;

    fn invalid(message: &'static str) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, message)
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Glyph {
        pub index: u32,
        /// Signed horizontal advance in twips, not a Unicode code point.
        pub advance: i32,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct TextRecord {
        pub span: Span,
        /// Original TEXTRECORD flag byte, including its required type bit.
        pub flags: u8,
        pub font_id: Option<u16>,
        /// DefineText RGB is expanded with alpha 255; DefineText2 retains RGBA.
        pub color: Option<[u8; 4]>,
        pub x_offset: Option<i16>,
        pub y_offset: Option<i16>,
        pub text_height: Option<u16>,
        /// Packed glyph entries, including final byte padding.
        pub glyph_data: Span,
        pub glyphs: Vec<Glyph>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct StaticText {
        pub tag_code: u16,
        pub id: u16,
        pub bounds: Rect,
        pub matrix: Matrix,
        pub glyph_bits: u8,
        pub advance_bits: u8,
        /// Absent style values inherit prior state; this parser leaves them absent.
        pub records: Vec<TextRecord>,
        pub body: Span,
        pub terminator: Span,
    }

    impl StaticText {
        pub fn parse(tag_code: u16, body: &[u8], body_offset: usize) -> io::Result<Self> {
            if tag_code != 11 && tag_code != 33 {
                return Err(invalid("static text requires DefineText or DefineText2"));
            }
            let mut input = Reader::new(body, body_offset)?;
            let id = input.u16()?;
            let bounds = input.rect()?;
            let matrix = input.matrix()?;
            let glyph_bits = input.u8()?;
            let advance_bits = input.u8()?;
            if glyph_bits > 32 || advance_bits > 32 {
                return Err(invalid("text glyph bit width exceeds 32"));
            }
            let mut records = Vec::new();
            let mut total_glyphs = 0usize;
            let terminator;
            loop {
                let start = input.byte_position();
                let flags = input.u8()?;
                if flags == 0 {
                    terminator = input.span(start, input.byte_position())?;
                    break;
                }
                if flags & 0x80 == 0 || flags & 0x70 != 0 {
                    return Err(invalid("invalid TEXTRECORD type or reserved flags"));
                }
                if records.len() == MAX_TEXT_RECORDS {
                    return Err(invalid("text record count exceeds inspection limit"));
                }
                let font_id = if flags & 8 != 0 {
                    Some(input.u16()?)
                } else {
                    None
                };
                let color = if flags & 4 != 0 {
                    Some(input.color(tag_code == 33)?)
                } else {
                    None
                };
                let x_offset = if flags & 1 != 0 {
                    Some(input.i16()?)
                } else {
                    None
                };
                let y_offset = if flags & 2 != 0 {
                    Some(input.i16()?)
                } else {
                    None
                };
                let text_height = if flags & 8 != 0 {
                    Some(input.u16()?)
                } else {
                    None
                };
                let count = usize::from(input.u8()?);
                total_glyphs = total_glyphs
                    .checked_add(count)
                    .ok_or_else(|| invalid("text glyph count overflow"))?;
                if total_glyphs > MAX_TEXT_GLYPHS {
                    return Err(invalid("text glyph count exceeds inspection limit"));
                }
                let glyph_start = input.byte_position();
                input.ensure_bits(count * (usize::from(glyph_bits) + usize::from(advance_bits)))?;
                let mut glyphs = Vec::with_capacity(count);
                for _ in 0..count {
                    glyphs.push(Glyph {
                        index: input.unsigned(glyph_bits)?,
                        advance: input.signed(advance_bits)?,
                    });
                }
                input.align();
                let end = input.byte_position();
                records.push(TextRecord {
                    span: input.span(start, end)?,
                    flags,
                    font_id,
                    color,
                    x_offset,
                    y_offset,
                    text_height,
                    glyph_data: input.span(glyph_start, end)?,
                    glyphs,
                });
            }
            input.finish()?;
            Ok(Self {
                tag_code,
                id,
                bounds,
                matrix,
                glyph_bits,
                advance_bits,
                records,
                body: Span {
                    offset: body_offset,
                    len: body.len(),
                },
                terminator,
            })
        }
    }

    /// The two bit-packed flag bytes in wire order: HasText is bit 15 and
    /// UseOutlines is bit 0. This is not a little-endian UI16 field.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct EditFlags {
        pub bits: u16,
    }

    impl EditFlags {
        pub fn has_text(self) -> bool {
            self.bits & 0x8000 != 0
        }
        pub fn word_wrap(self) -> bool {
            self.bits & 0x4000 != 0
        }
        pub fn multiline(self) -> bool {
            self.bits & 0x2000 != 0
        }
        pub fn password(self) -> bool {
            self.bits & 0x1000 != 0
        }
        pub fn read_only(self) -> bool {
            self.bits & 0x0800 != 0
        }
        pub fn has_text_color(self) -> bool {
            self.bits & 0x0400 != 0
        }
        pub fn has_max_length(self) -> bool {
            self.bits & 0x0200 != 0
        }
        pub fn has_font(self) -> bool {
            self.bits & 0x0100 != 0
        }
        pub fn has_font_class(self) -> bool {
            self.bits & 0x0080 != 0
        }
        pub fn auto_size(self) -> bool {
            self.bits & 0x0040 != 0
        }
        pub fn has_layout(self) -> bool {
            self.bits & 0x0020 != 0
        }
        pub fn no_select(self) -> bool {
            self.bits & 0x0010 != 0
        }
        pub fn border(self) -> bool {
            self.bits & 0x0008 != 0
        }
        pub fn was_static(self) -> bool {
            self.bits & 0x0004 != 0
        }
        pub fn html(self) -> bool {
            self.bits & 0x0002 != 0
        }
        pub fn use_outlines(self) -> bool {
            self.bits & 0x0001 != 0
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct EditLayout {
        /// Wire values: 0 left, 1 right, 2 center, 3 justified.
        pub alignment: u8,
        pub left_margin: u16,
        pub right_margin: u16,
        pub indent: u16,
        pub leading: i16,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct EditText {
        pub id: u16,
        pub bounds: Rect,
        pub flags: EditFlags,
        pub font_id: Option<u16>,
        pub font_class: Option<String>,
        pub font_class_span: Option<Span>,
        /// Present when HasFont is set, per the SWF19 field-layout table.
        pub font_height: Option<u16>,
        pub color: Option<[u8; 4]>,
        pub max_length: Option<u16>,
        pub layout: Option<EditLayout>,
        pub variable_name: String,
        pub variable_name_span: Span,
        pub initial_text: Option<String>,
        pub initial_text_span: Option<Span>,
        pub body: Span,
    }

    impl EditText {
        pub fn parse(body: &[u8], body_offset: usize) -> io::Result<Self> {
            let mut input = Reader::new(body, body_offset)?;
            let id = input.u16()?;
            let bounds = input.rect()?;
            let flags = EditFlags {
                bits: u16::from_be_bytes(input.array()?),
            };
            if flags.has_font() && flags.has_font_class() {
                return Err(invalid("edit text selects both FontID and FontClass"));
            }
            let font_id = if flags.has_font() {
                Some(input.u16()?)
            } else {
                None
            };
            let (font_class, font_class_span) = if flags.has_font_class() {
                let (value, span) = input.string()?;
                (Some(value), Some(span))
            } else {
                (None, None)
            };
            let font_height = if flags.has_font() {
                Some(input.u16()?)
            } else {
                None
            };
            let color = if flags.has_text_color() {
                Some(input.color(true)?)
            } else {
                None
            };
            let max_length = if flags.has_max_length() {
                Some(input.u16()?)
            } else {
                None
            };
            let layout = if flags.has_layout() {
                let alignment = input.u8()?;
                if alignment > 3 {
                    return Err(invalid("invalid edit-text alignment"));
                }
                Some(EditLayout {
                    alignment,
                    left_margin: input.u16()?,
                    right_margin: input.u16()?,
                    indent: input.u16()?,
                    leading: input.i16()?,
                })
            } else {
                None
            };
            let (variable_name, variable_name_span) = input.string()?;
            let (initial_text, initial_text_span) = if flags.has_text() {
                let (value, span) = input.string()?;
                (Some(value), Some(span))
            } else {
                (None, None)
            };
            input.finish()?;
            Ok(Self {
                id,
                bounds,
                flags,
                font_id,
                font_class,
                font_class_span,
                font_height,
                color,
                max_length,
                layout,
                variable_name,
                variable_name_span,
                initial_text,
                initial_text_span,
                body: Span {
                    offset: body_offset,
                    len: body.len(),
                },
            })
        }
    }

    struct Reader<'a> {
        bytes: &'a [u8],
        base: usize,
        bit: usize,
    }

    impl<'a> Reader<'a> {
        fn new(bytes: &'a [u8], base: usize) -> io::Result<Self> {
            if bytes.len() > MAX_TEXT_BODY_BYTES {
                return Err(invalid("text tag exceeds the 8 MiB inspection limit"));
            }
            base.checked_add(bytes.len())
                .ok_or_else(|| invalid("text body span overflow"))?;
            Ok(Self {
                bytes,
                base,
                bit: 0,
            })
        }

        fn byte_position(&self) -> usize {
            self.bit / 8
        }

        fn ensure_bits(&self, count: usize) -> io::Result<()> {
            if count > self.bytes.len() * 8 - self.bit {
                return Err(invalid("truncated text bit fields"));
            }
            Ok(())
        }

        fn unsigned(&mut self, count: u8) -> io::Result<u32> {
            if count > 32 {
                return Err(invalid("text bit width exceeds 32"));
            }
            self.ensure_bits(usize::from(count))?;
            let mut value = 0u32;
            for _ in 0..count {
                value =
                    (value << 1) | u32::from((self.bytes[self.bit / 8] >> (7 - self.bit % 8)) & 1);
                self.bit += 1;
            }
            Ok(value)
        }

        fn signed(&mut self, count: u8) -> io::Result<i32> {
            let value = self.unsigned(count)?;
            Ok(match count {
                0 => 0,
                32 => value as i32,
                _ => ((value << (32 - count)) as i32) >> (32 - count),
            })
        }

        fn align(&mut self) {
            self.bit = self.bit.div_ceil(8) * 8;
        }

        fn take(&mut self, len: usize) -> io::Result<&'a [u8]> {
            if self.bit % 8 != 0 {
                return Err(invalid("unaligned text byte field"));
            }
            let start = self.byte_position();
            let end = start
                .checked_add(len)
                .ok_or_else(|| invalid("text field size overflow"))?;
            let value = self
                .bytes
                .get(start..end)
                .ok_or_else(|| invalid("truncated text byte field"))?;
            self.bit = end * 8;
            Ok(value)
        }

        fn array<const N: usize>(&mut self) -> io::Result<[u8; N]> {
            self.take(N)?
                .try_into()
                .map_err(|_| invalid("truncated text fixed field"))
        }

        fn u8(&mut self) -> io::Result<u8> {
            Ok(self.array::<1>()?[0])
        }
        fn u16(&mut self) -> io::Result<u16> {
            Ok(u16::from_le_bytes(self.array()?))
        }
        fn i16(&mut self) -> io::Result<i16> {
            Ok(i16::from_le_bytes(self.array()?))
        }

        fn color(&mut self, alpha: bool) -> io::Result<[u8; 4]> {
            let [red, green, blue] = self.array()?;
            Ok([red, green, blue, if alpha { self.u8()? } else { 255 }])
        }

        fn rect(&mut self) -> io::Result<Rect> {
            let bits = self.unsigned(5)? as u8;
            let result = Rect {
                x_min: self.signed(bits)?,
                x_max: self.signed(bits)?,
                y_min: self.signed(bits)?,
                y_max: self.signed(bits)?,
            };
            self.align();
            if result.x_min > result.x_max || result.y_min > result.y_max {
                return Err(invalid("inverted text bounds"));
            }
            Ok(result)
        }

        fn matrix(&mut self) -> io::Result<Matrix> {
            let mut result = Matrix::default();
            if self.unsigned(1)? != 0 {
                let bits = self.unsigned(5)? as u8;
                result.scale_x = self.signed(bits)?;
                result.scale_y = self.signed(bits)?;
            }
            if self.unsigned(1)? != 0 {
                let bits = self.unsigned(5)? as u8;
                result.rotate_skew_0 = self.signed(bits)?;
                result.rotate_skew_1 = self.signed(bits)?;
            }
            let bits = self.unsigned(5)? as u8;
            result.translate_x = self.signed(bits)?;
            result.translate_y = self.signed(bits)?;
            self.align();
            Ok(result)
        }

        fn span(&self, start: usize, end: usize) -> io::Result<Span> {
            if start > end || end > self.bytes.len() {
                return Err(invalid("invalid text subspan"));
            }
            Ok(Span {
                offset: self
                    .base
                    .checked_add(start)
                    .ok_or_else(|| invalid("text span overflow"))?,
                len: end - start,
            })
        }

        fn string(&mut self) -> io::Result<(String, Span)> {
            let start = self.byte_position();
            let available = self.bytes.len() - start;
            let scan = available.min(MAX_TEXT_STRING_BYTES + 1);
            let length = self.bytes[start..start + scan]
                .iter()
                .position(|&b| b == 0)
                .ok_or_else(|| invalid("unterminated or overlong text string"))?;
            let raw = self.take(length + 1)?;
            let text = std::str::from_utf8(&raw[..length])
                .map_err(|_| invalid("text string is not UTF-8"))?
                .to_owned();
            Ok((text, self.span(start, self.byte_position())?))
        }

        fn finish(&self) -> io::Result<()> {
            if self.bit != self.bytes.len() * 8 {
                return Err(invalid("trailing bytes after text record"));
            }
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn static_prefix() -> Vec<u8> {
            // ID 7; zero-width RECT; identity MATRIX; 3-bit glyph/4-bit advance.
            vec![7, 0, 0, 0, 3, 4]
        }

        #[test]
        fn static_records_keep_optional_style_and_signed_glyphs() {
            let mut bytes = static_prefix();
            bytes.extend([0x8f, 2, 0, 10, 20, 30, 40, 0xfe, 0xff, 4, 0, 0x90, 1, 2]);
            // (glyph1,advance-2),(glyph5,advance7), padded to a byte boundary.
            bytes.extend([0x3d, 0x5c]);
            bytes.extend([0x80, 1, 0x42, 0]);
            let text = StaticText::parse(33, &bytes, 100).unwrap();
            assert_eq!(text.id, 7);
            assert_eq!(text.matrix, Matrix::default());
            assert_eq!(text.records[0].font_id, Some(2));
            assert_eq!(text.records[0].color, Some([10, 20, 30, 40]));
            assert_eq!(text.records[0].x_offset, Some(-2));
            assert_eq!(text.records[0].y_offset, Some(4));
            assert_eq!(text.records[0].text_height, Some(400));
            assert_eq!(
                text.records[0].glyphs,
                [
                    Glyph {
                        index: 1,
                        advance: -2
                    },
                    Glyph {
                        index: 5,
                        advance: 7
                    }
                ]
            );
            assert_eq!(text.records[1].font_id, None);
            assert_eq!(
                text.records[1].glyphs,
                [Glyph {
                    index: 2,
                    advance: 1
                }]
            );
            assert_eq!(text.records[0].span.offset, 106);
            assert_eq!(
                text.terminator,
                Span {
                    offset: 100 + bytes.len() - 1,
                    len: 1
                }
            );
        }

        #[test]
        fn define_text_expands_rgb_and_accepts_empty_style_run() {
            let mut bytes = static_prefix();
            bytes.extend([0x84, 1, 2, 3, 0, 0]);
            let text = StaticText::parse(11, &bytes, 0).unwrap();
            assert_eq!(text.records[0].color, Some([1, 2, 3, 255]));
            assert!(text.records[0].glyphs.is_empty());
        }

        #[test]
        fn rejects_static_truncation_flags_widths_and_trailing_data() {
            let mut good = static_prefix();
            good.extend([0x80, 1, 0x24, 0]);
            for n in 0..good.len() {
                assert!(StaticText::parse(33, &good[..n], 0).is_err());
            }
            let mut bad = good.clone();
            bad[6] = 0x90;
            assert!(StaticText::parse(33, &bad, 0).is_err());
            bad = good.clone();
            bad[4] = 33;
            assert!(StaticText::parse(33, &bad, 0).is_err());
            bad = good.clone();
            bad.push(0);
            assert!(StaticText::parse(33, &bad, 0).is_err());
            assert!(StaticText::parse(10, &good, 0).is_err());
            assert!(StaticText::parse(33, &good, usize::MAX).is_err());
        }

        fn edit_with_layout() -> Vec<u8> {
            // HasText,HasColor,HasMaxLength,HasFont; HasLayout,HTML,UseOutlines.
            let mut bytes = vec![9, 0, 0, 0x87, 0x23, 4, 0, 0xe0, 1, 1, 2, 3, 4, 30, 0];
            bytes.extend([2, 10, 0, 20, 0, 3, 0, 0xfc, 0xff]);
            bytes.extend(b"account.name\0<p>Text</p>\0");
            bytes
        }

        #[test]
        fn edit_text_preserves_flags_layout_strings_and_absolute_spans() {
            let bytes = edit_with_layout();
            let text = EditText::parse(&bytes, 40).unwrap();
            assert!(text.flags.has_text() && text.flags.html() && text.flags.use_outlines());
            assert!(!text.flags.password());
            assert_eq!(text.font_id, Some(4));
            assert_eq!(text.font_height, Some(480));
            assert_eq!(text.color, Some([1, 2, 3, 4]));
            assert_eq!(text.max_length, Some(30));
            assert_eq!(
                text.layout.unwrap(),
                EditLayout {
                    alignment: 2,
                    left_margin: 10,
                    right_margin: 20,
                    indent: 3,
                    leading: -4
                }
            );
            assert_eq!(text.variable_name, "account.name");
            assert_eq!(text.initial_text.as_deref(), Some("<p>Text</p>"));
            assert_eq!(
                text.variable_name_span,
                Span {
                    offset: 64,
                    len: 13
                }
            );
        }

        #[test]
        fn font_class_uses_formal_field_table_without_font_id_height() {
            let mut bytes = vec![1, 0, 0, 0, 0x80];
            bytes.extend(b"ExampleFont\0field\0");
            let text = EditText::parse(&bytes, 0).unwrap();
            assert_eq!(text.font_class.as_deref(), Some("ExampleFont"));
            assert_eq!(text.font_id, None);
            assert_eq!(text.font_height, None);
            assert_eq!(text.initial_text, None);
        }

        #[test]
        fn rejects_edit_truncation_conflicting_fonts_and_invalid_strings() {
            let good = edit_with_layout();
            for n in 0..good.len() {
                assert!(EditText::parse(&good[..n], 0).is_err());
            }
            let mut bad = good.clone();
            bad[4] |= 0x80;
            assert!(EditText::parse(&bad, 0).is_err());
            bad = good.clone();
            bad[15] = 4;
            assert!(EditText::parse(&bad, 0).is_err());
            bad = good.clone();
            bad.push(0);
            assert!(EditText::parse(&bad, 0).is_err());
            assert!(EditText::parse(&[1, 0, 0, 0, 0, 0xff, 0], 0).is_err());
            assert!(EditText::parse(&good, usize::MAX).is_err());
        }
    }
}
