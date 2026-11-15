//! Original metric layout for a bounded plain-text DefineEditText subset.
//! No HTML, device-font substitution, complex shaping, automatic sizing or VM.

use crate::gfx::glyphs::{FontIdentity, ResolvedFont};
use crate::gfx::text::{EditFlags, EditLayout, EditText};
use crate::gfx::{Rect, Span};
use std::collections::BTreeMap;
use std::fmt;
use std::ops::Range;

pub const MAX_TEXT_BYTES: usize = 1_048_576;
pub const MAX_TEXT_SCALARS: usize = 262_144;
pub const MAX_LINES: usize = 65_536;
/// Adobe TextLineMetrics documents a two-pixel gutter on each side.
pub const GUTTER_TWIPS: i32 = 40;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnsupportedFeature {
    Html,
    DeviceFont,
    FontClass,
    AutoSize,
    Password,
    Justify,
    SingleLineBreak,
    Tab,
    ControlCharacter,
    CharacterProfile,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EditLayoutError {
    Unsupported {
        feature: UnsupportedFeature,
        utf16_index: Option<usize>,
    },
    MissingFont,
    MissingHeight,
    UnresolvedFont {
        local_id: u16,
    },
    InvalidFont(&'static str),
    MissingGlyph {
        codepoint: u32,
        utf16_index: usize,
    },
    AmbiguousGlyph {
        code: u16,
    },
    NeedsWrapping {
        utf16_range: Range<usize>,
        required: Twips,
        available: Twips,
    },
    InvalidField(&'static str),
    Limit(&'static str),
}
impl fmt::Display for EditLayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EditLayoutError {}
type Result<T> = std::result::Result<T, EditLayoutError>;

/// Exact field-local twips. All values in one result share a denominator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Twips {
    pub numerator: i128,
    pub denominator: u32,
}
impl Twips {
    pub fn as_f64(self) -> f64 {
        self.numerator as f64 / f64::from(self.denominator)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextSource {
    /// Indices into the supplied UTF-8 string, excluding a source terminator.
    pub bytes: Range<usize>,
    /// Indices into that string encoded as UTF-16 code units.
    pub utf16: Range<usize>,
    /// Present only for a saved initial string with an original source span.
    pub movie_bytes: Option<Span>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditGlyph {
    pub source: TextSource,
    pub unicode: char,
    pub code: u16,
    pub glyph_index: usize,
    pub shape_source: Span,
    pub font: FontIdentity,
    pub local_font_id: u16,
    pub line_index: usize,
    pub x: Twips,
    pub baseline: Twips,
    pub advance: Twips,
    pub height_twips: u16,
    pub units_per_em: u32,
    /// None preserves a missing saved color instead of manufacturing one.
    pub color: Option<[u8; 4]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditLine {
    pub glyphs: Range<usize>,
    pub source: TextSource,
    /// CR, LF, or CRLF separator consumed after this line, if present.
    pub break_source: Option<TextSource>,
    pub x: Twips,
    pub top: Twips,
    pub width: Twips,
    pub height: Twips,
    pub baseline: Twips,
    pub available_width: Twips,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextContentKind {
    SavedInitial,
    PlainUpdate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditTextLayout {
    pub field_id: u16,
    pub field_source: Span,
    pub bounds: Rect,
    pub flags: EditFlags,
    pub content_kind: TextContentKind,
    pub font: FontIdentity,
    pub local_font_id: u16,
    pub height_twips: u16,
    pub units_per_em: u32,
    pub denominator: u32,
    pub utf16_len: usize,
    pub glyphs: Vec<EditGlyph>,
    pub lines: Vec<EditLine>,
    pub overflow_x: bool,
    pub overflow_y: bool,
    /// Retained for later input handling; saved/script text is not truncated.
    pub max_input_length: Option<u16>,
}
impl EditTextLayout {
    /// Scale an original integer EM point and add the field-local baseline.
    /// Scene/display transforms and clipping belong to the caller.
    pub fn position(&self, glyph_index: usize, point: [i32; 2]) -> Result<[Twips; 2]> {
        let g = self
            .glyphs
            .get(glyph_index)
            .ok_or(EditLayoutError::InvalidField("glyph index outside layout"))?;
        if !matches!(g.units_per_em, 1024 | 20480) || self.denominator != g.units_per_em * 2 {
            return Err(EditLayoutError::InvalidFont("inconsistent output units"));
        }
        Ok([
            Twips {
                numerator: g
                    .x
                    .numerator
                    .checked_add(i128::from(point[0]) * i128::from(g.height_twips) * 2)
                    .ok_or(EditLayoutError::InvalidField("glyph position overflow"))?,
                denominator: self.denominator,
            },
            Twips {
                numerator: g
                    .baseline
                    .numerator
                    .checked_add(i128::from(point[1]) * i128::from(g.height_twips) * 2)
                    .ok_or(EditLayoutError::InvalidField("glyph position overflow"))?,
                denominator: self.denominator,
            },
        ])
    }
}

/// Lay out the original initial string. HTML is kept unsupported even if its
/// serialized bytes happen not to include a tag delimiter.
pub fn layout_initial<'a, F>(field: &EditText, resolve: F) -> Result<EditTextLayout>
where
    F: FnMut(u16) -> Result<ResolvedFont<'a>>,
{
    if field.flags.password() {
        return Err(unsupported(UnsupportedFeature::Password, None));
    }
    if field.flags.html() {
        return Err(unsupported(UnsupportedFeature::Html, None));
    }
    let value = field.initial_text.as_deref().unwrap_or("");
    layout_impl(
        field,
        value,
        field.initial_text_span,
        TextContentKind::SavedInitial,
        resolve,
    )
}

/// An explicit plain-text update. This supplies new unformatted content; it is
/// not an HTML-to-text conversion and does not apply a runtime style sheet.
pub fn layout_plain<'a, F>(field: &EditText, value: &str, resolve: F) -> Result<EditTextLayout>
where
    F: FnMut(u16) -> Result<ResolvedFont<'a>>,
{
    layout_impl(field, value, None, TextContentKind::PlainUpdate, resolve)
}

fn unsupported(feature: UnsupportedFeature, utf16_index: Option<usize>) -> EditLayoutError {
    EditLayoutError::Unsupported {
        feature,
        utf16_index,
    }
}

struct Token {
    ch: char,
    source: TextSource,
    glyph_index: Option<usize>,
    advance: i128,
}

fn source(bytes: Range<usize>, utf16: Range<usize>, saved: Option<Span>) -> Result<TextSource> {
    let movie_bytes = if let Some(saved) = saved {
        let end = saved
            .offset
            .checked_add(bytes.end)
            .ok_or(EditLayoutError::InvalidField("source offset overflow"))?;
        let start = saved
            .offset
            .checked_add(bytes.start)
            .ok_or(EditLayoutError::InvalidField("source offset overflow"))?;
        if bytes.end > saved.len {
            return Err(EditLayoutError::InvalidField(
                "text source exceeds saved span",
            ));
        }
        Some(Span {
            offset: start,
            len: end - start,
        })
    } else {
        None
    };
    Ok(TextSource {
        bytes,
        utf16,
        movie_bytes,
    })
}

fn layout_impl<'a, F>(
    field: &EditText,
    value: &str,
    saved: Option<Span>,
    content_kind: TextContentKind,
    mut resolve: F,
) -> Result<EditTextLayout>
where
    F: FnMut(u16) -> Result<ResolvedFont<'a>>,
{
    if value.len() > MAX_TEXT_BYTES {
        return Err(EditLayoutError::Limit("text byte limit"));
    }
    let scalar_count = value.chars().count();
    if scalar_count > MAX_TEXT_SCALARS {
        return Err(EditLayoutError::Limit("text scalar limit"));
    }
    if field.bounds.x_min > field.bounds.x_max || field.bounds.y_min > field.bounds.y_max {
        return Err(EditLayoutError::InvalidField("inverted field bounds"));
    }
    for (flag, reason) in [
        (field.flags.has_font_class(), UnsupportedFeature::FontClass),
        (!field.flags.use_outlines(), UnsupportedFeature::DeviceFont),
        (field.flags.auto_size(), UnsupportedFeature::AutoSize),
        (field.flags.password(), UnsupportedFeature::Password),
    ] {
        if flag {
            return Err(unsupported(reason, None));
        }
    }
    let local_id = field.font_id.ok_or(EditLayoutError::MissingFont)?;
    let height = field.font_height.ok_or(EditLayoutError::MissingHeight)?;
    let resolved = resolve(local_id)?;
    let font = resolved.definition;
    if font.glyphs.len() > usize::from(u16::MAX) {
        return Err(EditLayoutError::InvalidFont(
            "font glyph count exceeds serialized limit",
        ));
    }
    let expected_units = match font.tag_code {
        48 => 1024,
        75 => 20480,
        _ => {
            return Err(EditLayoutError::InvalidFont(
                "dynamic font requires DefineFont2/3",
            ))
        }
    };
    if font.id != resolved.identity.character_id || font.units_per_em != expected_units {
        return Err(EditLayoutError::InvalidFont(
            "resolved font identity or units mismatch",
        ));
    }
    let font_flags = font
        .flags
        .ok_or(EditLayoutError::InvalidFont("missing font encoding flags"))?;
    if font_flags.shift_jis() || font_flags.ansi() || !font_flags.wide_codes() {
        return Err(EditLayoutError::InvalidFont(
            "legacy font codepage is unsupported",
        ));
    }
    let metrics = font
        .layout
        .as_ref()
        .ok_or(EditLayoutError::InvalidFont("missing font layout metrics"))?;
    if metrics.advances.len() != font.glyphs.len() {
        return Err(EditLayoutError::InvalidFont("advance count mismatch"));
    }
    let layout = field.layout.unwrap_or(EditLayout {
        alignment: 0,
        left_margin: 0,
        right_margin: 0,
        indent: 0,
        leading: 0,
    });
    if layout.alignment == 3 {
        return Err(unsupported(UnsupportedFeature::Justify, None));
    }
    if layout.alignment > 3 {
        return Err(EditLayoutError::InvalidField("unknown alignment"));
    }
    let denominator = font.units_per_em * 2;
    let d = i128::from(denominator);
    let h = i128::from(height) * 2;
    let twips = |n| Twips {
        numerator: n,
        denominator,
    };
    let left = (i128::from(field.bounds.x_min)
        + i128::from(GUTTER_TWIPS)
        + i128::from(layout.left_margin)
        + i128::from(layout.indent))
        * d;
    let right = (i128::from(field.bounds.x_max)
        - i128::from(GUTTER_TWIPS)
        - i128::from(layout.right_margin))
        * d;
    if right < left {
        return Err(EditLayoutError::InvalidField(
            "margins and gutter exceed field width",
        ));
    }
    let available = right - left;
    let top = (i128::from(field.bounds.y_min) + i128::from(GUTTER_TWIPS)) * d;
    let ascent = i128::from(metrics.ascent) * h;
    let descent = i128::from(metrics.descent) * h;
    let line_height = ascent + descent;
    // DefineEditText.Leading is already in twips, not font EM units. It is
    // the chosen paragraph spacing; do not add FontLeading a second time.
    let line_step = line_height + i128::from(layout.leading) * d;
    let mut codes = BTreeMap::new();
    for (index, glyph) in font.glyphs.iter().enumerate() {
        let code = glyph
            .code
            .ok_or(EditLayoutError::InvalidFont("missing character code"))?;
        if codes.insert(code, index).is_some() {
            return Err(EditLayoutError::AmbiguousGlyph { code });
        }
    }
    let mut tokens = Vec::with_capacity(scalar_count);
    let mut chars = value.char_indices().peekable();
    let mut utf16 = 0usize;
    while let Some((byte, ch)) = chars.next() {
        let start = utf16;
        utf16 += ch.len_utf16();
        let mut end = byte + ch.len_utf8();
        if ch == '\r' || ch == '\n' {
            if !field.flags.multiline() {
                return Err(unsupported(
                    UnsupportedFeature::SingleLineBreak,
                    Some(start),
                ));
            }
            if ch == '\r' && chars.peek().is_some_and(|(_, next)| *next == '\n') {
                let (at, _) = chars.next().unwrap();
                end = at + 1;
                utf16 += 1;
            }
            tokens.push(Token {
                ch: '\r',
                source: source(byte..end, start..utf16, saved)?,
                glyph_index: None,
                advance: 0,
            });
            continue;
        }
        if ch == '\t' {
            return Err(unsupported(UnsupportedFeature::Tab, Some(start)));
        }
        if ch.is_control() {
            return Err(unsupported(
                UnsupportedFeature::ControlCharacter,
                Some(start),
            ));
        }
        let cp = ch as u32;
        let code = u16::try_from(cp).map_err(|_| EditLayoutError::MissingGlyph {
            codepoint: cp,
            utf16_index: start,
        })?;
        // Explicit simple left-to-right subset. Combining/RTL/Indic shaping,
        // discretionary breaks and bidi controls need a later text engine.
        if !simple_ltr(cp) {
            return Err(unsupported(
                UnsupportedFeature::CharacterProfile,
                Some(start),
            ));
        }
        let index = *codes.get(&code).ok_or(EditLayoutError::MissingGlyph {
            codepoint: cp,
            utf16_index: start,
        })?;
        let advance = i128::from(metrics.advances[index]) * h;
        tokens.push(Token {
            ch,
            source: source(byte..end, start..utf16, saved)?,
            glyph_index: Some(index),
            advance,
        });
    }
    let line_count = tokens.iter().filter(|t| t.glyph_index.is_none()).count() + 1;
    if line_count > MAX_LINES {
        return Err(EditLayoutError::Limit("line count limit"));
    }
    if line_count > 1 && line_step <= 0 {
        return Err(EditLayoutError::InvalidField(
            "nonpositive multiline baseline step",
        ));
    }
    let mut output = EditTextLayout {
        field_id: field.id,
        field_source: field.body,
        bounds: field.bounds,
        flags: field.flags,
        content_kind,
        font: resolved.identity,
        local_font_id: local_id,
        height_twips: height,
        units_per_em: font.units_per_em,
        denominator,
        utf16_len: utf16,
        glyphs: Vec::with_capacity(tokens.len()),
        lines: Vec::with_capacity(line_count),
        overflow_x: false,
        overflow_y: false,
        max_input_length: field.max_length,
    };
    let mut begin = 0usize;
    let mut line_byte = 0usize;
    let mut line_utf16 = 0usize;
    loop {
        let end = tokens[begin..]
            .iter()
            .position(|t| t.glyph_index.is_none())
            .map(|at| begin + at)
            .unwrap_or(tokens.len());
        let width = tokens[begin..end].iter().map(|t| t.advance).sum::<i128>();
        let end_byte = tokens
            .get(end)
            .map(|t| t.source.bytes.start)
            .unwrap_or(value.len());
        let end_utf16 = tokens
            .get(end)
            .map(|t| t.source.utf16.start)
            .unwrap_or(utf16);
        if field.flags.word_wrap() && width > available {
            return Err(EditLayoutError::NeedsWrapping {
                utf16_range: line_utf16..end_utf16,
                required: twips(width),
                available: twips(available),
            });
        }
        let offset = match layout.alignment {
            0 => 0,
            1 => available - width,
            2 => (available - width) / 2,
            _ => unreachable!(),
        };
        let mut x = left + offset;
        let line_index = output.lines.len();
        let y = top + line_index as i128 * line_step;
        let baseline = y + ascent;
        let first = output.glyphs.len();
        for token in &tokens[begin..end] {
            let index = token.glyph_index.unwrap();
            output.glyphs.push(EditGlyph {
                source: token.source.clone(),
                unicode: token.ch,
                code: token.ch as u16,
                glyph_index: index,
                shape_source: font.glyphs[index].shape.source,
                font: resolved.identity,
                local_font_id: local_id,
                line_index,
                x: twips(x),
                baseline: twips(baseline),
                advance: twips(token.advance),
                height_twips: height,
                units_per_em: font.units_per_em,
                color: field.color,
            });
            x += token.advance;
        }
        output.overflow_x |= width > available;
        output.overflow_y |=
            y + line_height > (i128::from(field.bounds.y_max) - i128::from(GUTTER_TWIPS)) * d;
        output.lines.push(EditLine {
            glyphs: first..output.glyphs.len(),
            source: source(line_byte..end_byte, line_utf16..end_utf16, saved)?,
            break_source: tokens.get(end).map(|t| t.source.clone()),
            x: twips(left + offset),
            top: twips(y),
            width: twips(width),
            height: twips(line_height),
            baseline: twips(baseline),
            available_width: twips(available),
        });
        if end == tokens.len() {
            break;
        }
        line_byte = tokens[end].source.bytes.end;
        line_utf16 = tokens[end].source.utf16.end;
        begin = end + 1;
    }
    Ok(output)
}

fn simple_ltr(code: u32) -> bool {
    matches!(code, 0x20..=0x7e | 0xa0..=0x2ff | 0x370..=0x482 | 0x48a..=0x52f | 0x2010..=0x2027 | 0x2030..=0x205e | 0x20a0..=0x20cf)
        && code != 0xad
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::text::{FontDefinition, FontFlags, FontGlyph, FontLayout, GlyphShape};
    fn font() -> FontDefinition {
        FontDefinition {
            source: Span { offset: 0, len: 10 },
            tag_code: 75,
            id: 9,
            flags: Some(FontFlags(0x84)),
            language: Some(0),
            name: None,
            glyphs: [32u16, 65, 66, 0xe9]
                .iter()
                .map(|&code| FontGlyph {
                    shape: GlyphShape {
                        source: Span { offset: 2, len: 2 },
                        fill_bits: 0,
                        line_bits: 0,
                        record_count: 1,
                        edge_count: 0,
                    },
                    code: Some(code),
                })
                .collect(),
            layout: Some(FontLayout {
                source: Span { offset: 0, len: 0 },
                ascent: 16000,
                descent: 4000,
                leading: 900,
                advances: vec![4000, 10000, 12000, 8000],
                bounds: vec![],
                kerning: vec![],
            }),
            units_per_em: 20480,
        }
    }
    fn field() -> EditText {
        EditText {
            id: 4,
            bounds: Rect {
                x_min: 0,
                x_max: 2000,
                y_min: 0,
                y_max: 2000,
            },
            flags: EditFlags { bits: 0x8521 },
            font_id: Some(1),
            font_class: None,
            font_class_span: None,
            font_height: Some(1024),
            color: Some([1, 2, 3, 255]),
            max_length: None,
            layout: Some(EditLayout {
                alignment: 0,
                left_margin: 0,
                right_margin: 0,
                indent: 0,
                leading: 20,
            }),
            variable_name: String::new(),
            variable_name_span: Span { offset: 0, len: 1 },
            initial_text: Some("AB".into()),
            initial_text_span: Some(Span {
                offset: 100,
                len: 3,
            }),
            body: Span {
                offset: 0,
                len: 200,
            },
        }
    }
    fn resolved(f: &FontDefinition) -> ResolvedFont<'_> {
        ResolvedFont {
            identity: FontIdentity {
                movie_id: 7,
                character_id: 9,
            },
            definition: f,
        }
    }
    #[test]
    fn saved_metrics_gutter_spans_and_positions_are_exact() {
        let f = font();
        let t = field();
        let out = layout_initial(&t, |_| Ok(resolved(&f))).unwrap();
        assert_eq!(out.glyphs.len(), 2);
        assert_eq!(out.lines[0].width.as_f64(), 1100.0);
        assert_eq!(out.glyphs[0].x.as_f64(), 40.0);
        assert_eq!(out.glyphs[0].baseline.as_f64(), 840.0);
        assert_eq!(out.glyphs[1].x.as_f64(), 540.0);
        assert_eq!(out.glyphs[1].source.utf16, 1..2);
        assert_eq!(
            out.glyphs[1].source.movie_bytes,
            Some(Span {
                offset: 101,
                len: 1
            })
        );
        assert_eq!(
            out.position(0, [20480, -20480]).unwrap().map(Twips::as_f64),
            [1064.0, -184.0]
        );
    }
    #[test]
    fn alignment_margins_and_crlf_preserve_source_indices() {
        let f = font();
        let mut t = field();
        t.flags.bits |= 0x2000;
        t.layout.as_mut().unwrap().alignment = 2;
        let out = layout_plain(&t, "A\r\né\nB", |_| Ok(resolved(&f))).unwrap();
        assert_eq!(out.lines.len(), 3);
        assert_eq!(out.lines[0].break_source.as_ref().unwrap().utf16, 1..3);
        assert_eq!(out.glyphs[1].source.bytes, 3..5);
        assert_eq!(out.glyphs[1].source.utf16, 3..4);
        assert_eq!(out.glyphs[0].x.as_f64(), 750.0);
        assert_eq!(out.glyphs[1].baseline.as_f64(), 1860.0);
        assert!(out.overflow_y);
        assert!(out.glyphs[0].source.movie_bytes.is_none());
    }
    #[test]
    fn unsupported_content_never_substitutes_or_strips() {
        let f = font();
        let mut t = field();
        t.flags.bits |= 2;
        assert!(matches!(
            layout_initial(&t, |_| Ok(resolved(&f))),
            Err(EditLayoutError::Unsupported {
                feature: UnsupportedFeature::Html,
                ..
            })
        ));
        assert!(layout_plain(&t, "A", |_| Ok(resolved(&f))).is_ok());
        assert!(matches!(
            layout_plain(&t, "A😀", |_| Ok(resolved(&f))),
            Err(EditLayoutError::MissingGlyph { utf16_index: 1, .. })
        ));
        assert!(matches!(
            layout_plain(&t, "A\u{301}", |_| Ok(resolved(&f))),
            Err(EditLayoutError::Unsupported {
                feature: UnsupportedFeature::CharacterProfile,
                ..
            })
        ));
        assert!(matches!(
            layout_plain(&t, "C", |_| Ok(resolved(&f))),
            Err(EditLayoutError::MissingGlyph { codepoint: 67, .. })
        ));
        t.flags.bits |= 0x1000;
        assert!(matches!(
            layout_initial(&t, |_| Ok(resolved(&f))),
            Err(EditLayoutError::Unsupported {
                feature: UnsupportedFeature::Password,
                ..
            })
        ));
        assert!(matches!(
            layout_plain(&t, "AB", |_| Ok(resolved(&f))),
            Err(EditLayoutError::Unsupported {
                feature: UnsupportedFeature::Password,
                ..
            })
        ));
    }
    #[test]
    fn wrapping_and_bad_font_metrics_fail_with_typed_reasons() {
        let mut f = font();
        let mut t = field();
        t.flags.bits |= 0x4000;
        assert!(matches!(
            layout_plain(&t, "ABAB", |_| Ok(resolved(&f))),
            Err(EditLayoutError::NeedsWrapping { .. })
        ));
        f.layout.as_mut().unwrap().advances.pop();
        assert!(matches!(
            layout_initial(&t, |_| Ok(resolved(&f))),
            Err(EditLayoutError::InvalidFont(_))
        ));
        assert!(matches!(
            layout_initial(&t, |_| Err(EditLayoutError::UnresolvedFont { local_id: 1 })),
            Err(EditLayoutError::UnresolvedFont { local_id: 1 })
        ));
    }
    #[test]
    fn empty_and_zero_height_keep_identity_without_truncating_saved_text() {
        let f = font();
        let mut t = field();
        t.font_height = Some(0);
        t.max_length = Some(1);
        let out = layout_initial(&t, |_| Ok(resolved(&f))).unwrap();
        assert_eq!(out.glyphs.len(), 2);
        assert_eq!(out.lines[0].width.as_f64(), 0.0);
        let empty = layout_plain(&t, "", |_| Ok(resolved(&f))).unwrap();
        assert!(empty.glyphs.is_empty());
        assert_eq!(empty.lines.len(), 1);
        assert_eq!(
            empty.font,
            FontIdentity {
                movie_id: 7,
                character_id: 9
            }
        );
    }
}
