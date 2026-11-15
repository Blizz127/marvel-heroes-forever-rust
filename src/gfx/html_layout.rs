//! Source-preserving metric layout for the observed P/FONT initial-text subset.
//! This is a documented metric model, not a claim of native GFx pixel parity.

use crate::gfx::{
    Span,
    edit_layout::{
        EditGlyph, EditLine, EditTextLayout, GUTTER_TWIPS, TextContentKind, TextSource, Twips,
    },
    glyphs::ResolvedFont,
    html_text::{
        self, Alignment, Atom, AtomValue, Element, FieldDefaults, FontSize, HtmlText, Issue,
        IssueKind, Run, Source, Tag, Token,
    },
    text::EditText,
};
use std::{collections::BTreeMap, fmt, ops::Range};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlLayoutError {
    Parse(html_text::HtmlError),
    Unsupported(&'static str),
    Invalid(&'static str),
    UnresolvedFont(u16),
    FaceMismatch {
        declared: String,
        source_import: String,
        target_export: String,
    },
    MissingGlyph {
        codepoint: u32,
        source_utf16: usize,
    },
    NeedsWrapping {
        paragraph: usize,
        required: Twips,
        available: Twips,
    },
}
impl fmt::Display for HtmlLayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for HtmlLayoutError {}
type Result<T> = std::result::Result<T, HtmlLayoutError>;

/// The caller must establish both symbols through the same movie-library route
/// that produced `resolved`. Matching text alone is not a font resolution route.
#[derive(Clone, Copy, Debug)]
pub struct HtmlFont<'a> {
    pub resolved: ResolvedFont<'a>,
    pub source_import_symbol: &'a str,
    pub target_export_symbol: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlParagraph {
    pub element_index: usize,
    pub font_element_index: usize,
    pub atom_range: Range<usize>,
    pub line_index: usize,
    pub alignment: Alignment,
    pub height_twips: u16,
    pub kerning: bool,
    /// Signed font-space adjustments after each glyph; zero after the last.
    pub kerning_after: Vec<i16>,
}

/// Owned source declarations remain alongside geometry. Glyph source offsets
/// address the original markup; paragraph structure is never converted to an
/// invented text newline or a synthetic movie span.
#[derive(Clone, Debug)]
pub struct HtmlLayout {
    pub geometry: EditTextLayout,
    pub markup: String,
    pub defaults: FieldDefaults,
    pub tokens: Vec<Token>,
    pub elements: Vec<Element>,
    pub atoms: Vec<Atom>,
    pub runs: Vec<Run>,
    pub issues: Vec<Issue>,
    pub paragraphs: Vec<HtmlParagraph>,
    pub source_import_symbol: String,
    pub target_export_symbol: String,
}

fn source(s: &Source) -> TextSource {
    TextSource {
        bytes: s.bytes.clone(),
        utf16: s.utf16.clone(),
        movie_bytes: s.movie_bytes,
    }
}
fn joined(a: &Source, b: &Source) -> Result<TextSource> {
    let movie_bytes = match (a.movie_bytes, b.movie_bytes) {
        (Some(x), Some(y)) => Some(Span {
            offset: x.offset,
            len: y
                .offset
                .checked_add(y.len)
                .and_then(|end| end.checked_sub(x.offset))
                .ok_or(HtmlLayoutError::Invalid("source span overflow"))?,
        }),
        (None, None) => None,
        _ => return Err(HtmlLayoutError::Invalid("inconsistent source spans")),
    };
    Ok(TextSource {
        bytes: a.bytes.start..b.bytes.end,
        utf16: a.utf16.start..b.utf16.end,
        movie_bytes,
    })
}
fn exact_height(size: FontSize) -> Result<u16> {
    let FontSize::AbsolutePixels(n) = size else {
        return Err(HtmlLayoutError::Unsupported("relative font size"));
    };
    let scale = 10_i64
        .checked_pow(u32::from(n.decimal_places))
        .ok_or(HtmlLayoutError::Invalid("font size scale"))?;
    if n.coefficient % scale != 0 {
        return Err(HtmlLayoutError::Unsupported("fractional HTML pixel size"));
    }
    let twips = n
        .coefficient
        .checked_mul(20)
        .ok_or(HtmlLayoutError::Invalid("font size overflow"))?;
    if twips <= 0 || twips % scale != 0 {
        return Err(HtmlLayoutError::Unsupported(
            "font size is not positive integral twips",
        ));
    }
    u16::try_from(twips / scale)
        .map_err(|_| HtmlLayoutError::Unsupported("font size exceeds bounded twip range"))
}
fn simple_ltr(ch: char) -> bool {
    let n = ch as u32;
    matches!(n, 0x20..=0x7e | 0xa0..=0x2ff | 0x370..=0x482 | 0x48a..=0x52f | 0x2010..=0x2027 | 0x2030..=0x205e | 0x20a0..=0x20cf)
        && n != 0xad
}

/// Lay out saved initial HTML only. Supports exact embedded-font binding,
/// explicit P/FONT declarations, pair kerning, and no automatic line wrapping.
/// Unsupported declarations and ambiguous defaults fail before returning geometry.
pub fn layout_initial<'a, F>(field: &EditText, mut resolve: F) -> Result<HtmlLayout>
where
    F: FnMut(u16) -> Result<HtmlFont<'a>>,
{
    if field.flags.password() {
        return Err(HtmlLayoutError::Unsupported("password"));
    }
    if field.flags.auto_size() {
        return Err(HtmlLayoutError::Unsupported("automatic sizing"));
    }
    if field.flags.has_font_class() || field.font_class.is_some() {
        return Err(HtmlLayoutError::Unsupported("FontClass"));
    }
    if !field.flags.use_outlines() {
        return Err(HtmlLayoutError::Unsupported("device font"));
    }
    if field.bounds.x_min > field.bounds.x_max || field.bounds.y_min > field.bounds.y_max {
        return Err(HtmlLayoutError::Invalid("inverted bounds"));
    }
    let doc = HtmlText::parse_initial(field).map_err(HtmlLayoutError::Parse)?;
    if doc
        .issues
        .iter()
        .any(|v| v.kind != IssueKind::ObservedFontExtension)
    {
        return Err(HtmlLayoutError::Unsupported("unresolved HTML declaration"));
    }
    let local_id = field
        .font_id
        .ok_or(HtmlLayoutError::Invalid("missing FontID"))?;
    let saved_height = field
        .font_height
        .ok_or(HtmlLayoutError::Invalid("missing saved font height"))?;
    let layout = field.layout.ok_or(HtmlLayoutError::Unsupported(
        "implicit field layout defaults",
    ))?;
    if layout.alignment > 2 {
        return Err(HtmlLayoutError::Unsupported(
            "unknown or justified alignment",
        ));
    }
    let rgba = field
        .color
        .ok_or(HtmlLayoutError::Unsupported("missing saved color"))?;
    if rgba[3] != 255 {
        return Err(HtmlLayoutError::Unsupported(
            "HTML RGB and nonopaque field alpha composition",
        ));
    }
    let binding = resolve(local_id)?;
    let resolved = binding.resolved;
    let font = resolved.definition;
    let units = match font.tag_code {
        48 => 1024,
        75 => 20480,
        _ => return Err(HtmlLayoutError::Unsupported("font definition profile")),
    };
    if font.units_per_em != units
        || font.id != resolved.identity.character_id
        || font.glyphs.len() > 65535
    {
        return Err(HtmlLayoutError::Invalid("font units, identity or count"));
    }
    let flags = font
        .flags
        .ok_or(HtmlLayoutError::Invalid("missing font flags"))?;
    if flags.shift_jis() || flags.ansi() || !flags.wide_codes() {
        return Err(HtmlLayoutError::Unsupported("font encoding"));
    }
    let metrics = font
        .layout
        .as_ref()
        .ok_or(HtmlLayoutError::Invalid("missing font metrics"))?;
    if metrics.advances.len() != font.glyphs.len() || metrics.kerning.len() > 65535 {
        return Err(HtmlLayoutError::Invalid("font metric counts"));
    }
    let mut codes = BTreeMap::new();
    for (i, g) in font.glyphs.iter().enumerate() {
        let code = g
            .code
            .ok_or(HtmlLayoutError::Invalid("missing glyph code"))?;
        if codes.insert(code, i).is_some() {
            return Err(HtmlLayoutError::Invalid("duplicate glyph code"));
        }
    }
    let mut kern = BTreeMap::new();
    for p in &metrics.kerning {
        if kern
            .insert((p.left_code, p.right_code), p.adjustment)
            .is_some()
        {
            return Err(HtmlLayoutError::Invalid("duplicate kerning pair"));
        }
    }
    let denominator = units * 2;
    let d = i128::from(denominator);
    let tw = |numerator| Twips {
        numerator,
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
        return Err(HtmlLayoutError::Invalid("negative available width"));
    }
    let available = right - left;
    let mut top = (i128::from(field.bounds.y_min) + i128::from(GUTTER_TWIPS)) * d;
    let bottom = (i128::from(field.bounds.y_max) - i128::from(GUTTER_TWIPS)) * d;
    let mut out = EditTextLayout {
        field_id: field.id,
        field_source: field.body,
        bounds: field.bounds,
        flags: field.flags,
        content_kind: TextContentKind::SavedInitial,
        font: resolved.identity,
        local_font_id: local_id,
        height_twips: saved_height,
        units_per_em: units,
        denominator,
        utf16_len: doc.source.encode_utf16().count(),
        glyphs: Vec::new(),
        lines: Vec::new(),
        overflow_x: false,
        overflow_y: false,
        max_input_length: field.max_length,
    };
    let mut roots = Vec::new();
    let mut children = vec![Vec::new(); doc.elements.len()];
    let mut paragraph_runs = vec![Vec::new(); doc.elements.len()];
    for (i, e) in doc.elements.iter().enumerate() {
        if let Some(parent) = e.parent {
            children[parent].push(i);
        } else {
            roots.push(i);
        }
    }
    for run in &doc.runs {
        let root = run
            .paragraph_element
            .ok_or(HtmlLayoutError::Unsupported("content outside paragraph"))?;
        paragraph_runs[root].push(run);
    }
    if roots.is_empty() {
        return Err(HtmlLayoutError::Unsupported("empty or implicit paragraph"));
    }
    if roots.len() > 1 && !field.flags.multiline() {
        return Err(HtmlLayoutError::Unsupported(
            "multiple paragraphs in single-line field",
        ));
    }
    let mut paragraphs = Vec::with_capacity(roots.len());
    let mut consumed_atoms = 0;
    for root in roots {
        let p = &doc.elements[root];
        if p.tag != Tag::Paragraph
            || p.self_closing
            || p.closing.is_none()
            || p.attributes.len() != 1
        {
            return Err(HtmlLayoutError::Unsupported("paragraph grammar"));
        }
        let align = p
            .paragraph
            .alignment
            .as_ref()
            .ok_or(HtmlLayoutError::Unsupported("implicit paragraph alignment"))?
            .value;
        if align == Alignment::Justify {
            return Err(HtmlLayoutError::Unsupported("justified paragraph"));
        }
        let alignment_code = match align {
            Alignment::Left => 0,
            Alignment::Right => 1,
            Alignment::Center => 2,
            Alignment::Justify => unreachable!(),
        };
        if !field.flags.multiline() && alignment_code != layout.alignment {
            return Err(HtmlLayoutError::Unsupported(
                "single-line paragraph alignment differs from field",
            ));
        }
        if children[root].len() != 1 {
            return Err(HtmlLayoutError::Unsupported(
                "paragraph requires one explicit FONT",
            ));
        }
        let fi = children[root][0];
        let f = &doc.elements[fi];
        if f.tag != Tag::Font
            || f.self_closing
            || f.closing.is_none()
            || f.attributes.len() != 5
            || !children[fi].is_empty()
        {
            return Err(HtmlLayoutError::Unsupported("mixed or nested font grammar"));
        }
        let c = &f.character;
        let face = &c
            .face
            .as_ref()
            .ok_or(HtmlLayoutError::Unsupported("missing explicit face"))?
            .value;
        if face != binding.source_import_symbol || face != binding.target_export_symbol {
            return Err(HtmlLayoutError::FaceMismatch {
                declared: face.clone(),
                source_import: binding.source_import_symbol.to_owned(),
                target_export: binding.target_export_symbol.to_owned(),
            });
        }
        let height = exact_height(
            c.size
                .as_ref()
                .ok_or(HtmlLayoutError::Unsupported("missing explicit size"))?
                .value,
        )?;
        let rgb = c
            .color
            .as_ref()
            .ok_or(HtmlLayoutError::Unsupported("missing explicit RGB"))?
            .value;
        let spacing = c
            .letter_spacing
            .as_ref()
            .ok_or(HtmlLayoutError::Unsupported("implicit letter spacing"))?
            .value;
        if spacing.coefficient != 0 {
            return Err(HtmlLayoutError::Unsupported("nonzero HTML letter spacing"));
        }
        let kerning = c
            .kerning
            .as_ref()
            .ok_or(HtmlLayoutError::Unsupported("implicit kerning"))?
            .value;
        if !kerning {
            return Err(HtmlLayoutError::Unsupported(
                "HTML kerning disable inheritance",
            ));
        }
        let start = consumed_atoms;
        for run in &paragraph_runs[root] {
            if run.scope != Some(fi) || run.atoms.start != consumed_atoms {
                return Err(HtmlLayoutError::Unsupported(
                    "non-FONT or noncontiguous content",
                ));
            }
            consumed_atoms = run.atoms.end;
        }
        if start == consumed_atoms {
            return Err(HtmlLayoutError::Unsupported("empty paragraph metrics"));
        }
        let atoms = &doc.atoms[start..consumed_atoms];
        let mut glyph_indices = Vec::with_capacity(atoms.len());
        for a in atoms {
            let AtomValue::Scalar(ch) = a.value else {
                return Err(HtmlLayoutError::Unsupported("unresolved entity"));
            };
            if !simple_ltr(ch) {
                return Err(HtmlLayoutError::Unsupported(
                    "control or complex text shaping",
                ));
            }
            let code = u16::try_from(ch as u32)
                .map_err(|_| HtmlLayoutError::Unsupported("non-BMP font code"))?;
            let gi = codes
                .get(&code)
                .copied()
                .ok_or(HtmlLayoutError::MissingGlyph {
                    codepoint: ch as u32,
                    source_utf16: a.source.utf16.start,
                })?;
            glyph_indices.push((ch, code, gi));
        }
        let adjustments: Vec<i16> = glyph_indices
            .iter()
            .enumerate()
            .map(|(i, (_, code, _))| {
                if kerning {
                    glyph_indices
                        .get(i + 1)
                        .and_then(|(_, next, _)| kern.get(&(*code, *next)))
                        .copied()
                        .unwrap_or(0)
                } else {
                    0
                }
            })
            .collect();
        let h = i128::from(height) * 2;
        let width: i128 = glyph_indices
            .iter()
            .zip(&adjustments)
            .map(|((_, _, gi), k)| (i128::from(metrics.advances[*gi]) + i128::from(*k)) * h)
            .sum();
        if field.flags.word_wrap() && width > available {
            return Err(HtmlLayoutError::NeedsWrapping {
                paragraph: paragraphs.len(),
                required: tw(width),
                available: tw(available),
            });
        }
        let shift = match align {
            Alignment::Left => 0,
            Alignment::Right => available - width,
            Alignment::Center => (available - width) / 2,
            Alignment::Justify => unreachable!(),
        };
        let line_height = (i128::from(metrics.ascent) + i128::from(metrics.descent)) * h;
        let baseline = top + i128::from(metrics.ascent) * h;
        let mut x = left + shift;
        let begin = out.glyphs.len();
        for ((a, (ch, code, gi)), k) in atoms.iter().zip(&glyph_indices).zip(&adjustments) {
            let advance = (i128::from(metrics.advances[*gi]) + i128::from(*k)) * h;
            out.glyphs.push(EditGlyph {
                source: source(&a.source),
                unicode: *ch,
                code: *code,
                glyph_index: *gi,
                shape_source: font.glyphs[*gi].shape.source,
                font: resolved.identity,
                local_font_id: local_id,
                line_index: out.lines.len(),
                x: tw(x),
                baseline: tw(baseline),
                advance: tw(advance),
                height_twips: height,
                units_per_em: units,
                color: Some([rgb[0], rgb[1], rgb[2], 255]),
            });
            x += advance;
        }
        out.overflow_x |= width > available;
        out.overflow_y |= top + line_height > bottom;
        out.lines.push(EditLine {
            glyphs: begin..out.glyphs.len(),
            source: joined(&atoms[0].source, &atoms[atoms.len() - 1].source)?,
            break_source: None,
            x: tw(left + shift),
            top: tw(top),
            width: tw(width),
            height: tw(line_height),
            baseline: tw(baseline),
            available_width: tw(available),
        });
        paragraphs.push(HtmlParagraph {
            element_index: root,
            font_element_index: fi,
            atom_range: start..consumed_atoms,
            line_index: out.lines.len() - 1,
            alignment: align,
            height_twips: height,
            kerning,
            kerning_after: adjustments,
        });
        top += line_height + i128::from(layout.leading) * d;
    }
    if consumed_atoms != doc.atoms.len() || doc.elements.len() != paragraphs.len() * 2 {
        return Err(HtmlLayoutError::Unsupported(
            "content outside supported paragraphs",
        ));
    }
    Ok(HtmlLayout {
        geometry: out,
        markup: doc.source.to_owned(),
        defaults: doc
            .field_defaults
            .ok_or(HtmlLayoutError::Invalid("missing source defaults"))?,
        tokens: doc.tokens,
        elements: doc.elements,
        atoms: doc.atoms,
        runs: doc.runs,
        issues: doc.issues,
        paragraphs,
        source_import_symbol: binding.source_import_symbol.to_owned(),
        target_export_symbol: binding.target_export_symbol.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::{
        Rect,
        glyphs::FontIdentity,
        text::{
            EditFlags, EditLayout, FontDefinition, FontFlags, FontGlyph, FontLayout, GlyphShape,
            KerningPair,
        },
    };
    fn font() -> FontDefinition {
        FontDefinition {
            source: Span { offset: 0, len: 1 },
            tag_code: 48,
            id: 9,
            flags: Some(FontFlags(0x84)),
            language: Some(0),
            name: None,
            glyphs: [65, 66, 38, 0x2022]
                .map(|code| FontGlyph {
                    code: Some(code),
                    shape: GlyphShape {
                        source: Span {
                            offset: code as usize,
                            len: 1,
                        },
                        fill_bits: 0,
                        line_bits: 0,
                        record_count: 0,
                        edge_count: 0,
                    },
                })
                .to_vec(),
            layout: Some(FontLayout {
                source: Span { offset: 0, len: 0 },
                ascent: 800,
                descent: 200,
                leading: 999,
                advances: vec![600, 500, 400, 400],
                bounds: vec![],
                kerning: vec![KerningPair {
                    source: Span { offset: 0, len: 6 },
                    left_code: 65,
                    right_code: 66,
                    adjustment: -100,
                }],
            }),
            units_per_em: 1024,
        }
    }
    fn markup(text: &str) -> String {
        format!(
            "<p align=\"left\"><font face=\"$Exact\" size=\"20\" color=\"#010203\" letterSpacing=\"0.000000\" kerning=\"1\">{text}</font></p>"
        )
    }
    fn field(value: String) -> EditText {
        let len = value.len() + 1;
        EditText {
            id: 2,
            bounds: Rect {
                x_min: 0,
                x_max: 2000,
                y_min: 0,
                y_max: 2000,
            },
            flags: EditFlags { bits: 0xad23 },
            font_id: Some(1),
            font_class: None,
            font_class_span: None,
            font_height: Some(400),
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
            initial_text: Some(value),
            initial_text_span: Some(Span { offset: 100, len }),
            body: Span {
                offset: 0,
                len: 4000,
            },
        }
    }
    fn bind(f: &FontDefinition) -> HtmlFont<'_> {
        HtmlFont {
            resolved: ResolvedFont {
                identity: FontIdentity {
                    movie_id: 3,
                    character_id: 9,
                },
                definition: f,
            },
            source_import_symbol: "$Exact",
            target_export_symbol: "$Exact",
        }
    }
    #[test]
    fn pair_adjustment_alignment_and_no_cross_paragraph_kerning() {
        let f = font();
        let t = field(markup("AB") + &markup("B"));
        let result = layout_initial(&t, |_| Ok(bind(&f))).unwrap();
        let g = &result.geometry;
        assert_eq!(g.glyphs[0].x.as_f64(), 40.0);
        assert_eq!(g.glyphs[1].x.as_f64(), 40.0 + 500.0 * 400.0 / 1024.0);
        assert_eq!(g.lines[0].width.as_f64(), 1000.0 * 400.0 / 1024.0);
        assert_eq!(result.paragraphs[0].kerning_after, [-100, 0]);
        assert_eq!(
            g.lines[1].top.as_f64() - g.lines[0].top.as_f64(),
            1000.0 * 400.0 / 1024.0 + 20.0
        );
        assert!(g.lines.iter().all(|l| l.break_source.is_none()));
        assert_eq!(g.position(0, [1024, 0]).unwrap()[0].as_f64(), 440.0);
    }
    #[test]
    fn entities_unicode_and_movie_spans_are_source_coordinates() {
        let f = font();
        let t = field(markup("A&amp;•"));
        let r = layout_initial(&t, |_| Ok(bind(&f))).unwrap();
        let amp = &r.geometry.glyphs[1];
        let bullet = &r.geometry.glyphs[2];
        assert_eq!(&r.markup[amp.source.bytes.clone()], "&amp;");
        assert_eq!(amp.unicode, '&');
        assert_eq!(amp.source.bytes.len(), 5);
        assert_eq!(
            amp.source.movie_bytes.unwrap().offset,
            100 + amp.source.bytes.start
        );
        assert_eq!(bullet.source.bytes.len(), 3);
        assert_eq!(bullet.source.utf16.len(), 1);
        assert_eq!(r.atoms[2].decoded_utf16, Some(2..3));
    }
    #[test]
    fn exact_symbol_includes_dollar_prefix() {
        let f = font();
        let t = field(markup("AB"));
        assert!(matches!(
            layout_initial(&t, |_| Ok(HtmlFont {
                target_export_symbol: "Exact",
                ..bind(&f)
            })),
            Err(HtmlLayoutError::FaceMismatch { .. })
        ));
    }
    #[test]
    fn saved_margins_indent_and_paragraph_alignment_use_exact_units() {
        let f = font();
        for (name, code, ratio) in [("left", 0, 0.0), ("center", 2, 0.5), ("right", 1, 1.0)] {
            let mut t = field(markup("AB").replace("align=\"left\"", &format!("align=\"{name}\"")));
            t.layout = Some(EditLayout {
                alignment: code,
                left_margin: 10,
                right_margin: 30,
                indent: 20,
                leading: -10,
            });
            let r = layout_initial(&t, |_| Ok(bind(&f))).unwrap();
            let available = 2000.0 - 80.0 - 10.0 - 30.0 - 20.0;
            let width = 1000.0 * 400.0 / 1024.0;
            assert_eq!(r.geometry.lines[0].available_width.as_f64(), available);
            assert_eq!(
                r.geometry.glyphs[0].x.as_f64(),
                70.0 + (available - width) * ratio
            );
        }
    }
    #[test]
    fn single_line_paragraphs_and_nonopaque_alpha_are_unresolved() {
        let f = font();
        let mut t = field(markup("A") + &markup("B"));
        t.flags.bits &= !0x2000;
        assert!(matches!(
            layout_initial(&t, |_| Ok(bind(&f))),
            Err(HtmlLayoutError::Unsupported(
                "multiple paragraphs in single-line field"
            ))
        ));
        t = field(markup("A"));
        t.color.as_mut().unwrap()[3] = 127;
        assert!(matches!(
            layout_initial(&t, |_| Ok(bind(&f))),
            Err(HtmlLayoutError::Unsupported(
                "HTML RGB and nonopaque field alpha composition"
            ))
        ));
    }
    #[test]
    fn wrapping_rejected_and_fixed_width_overflow_retained() {
        let f = font();
        let mut t = field(markup("AB"));
        t.bounds.x_max = 100;
        t.flags.bits |= 0x4000;
        assert!(matches!(
            layout_initial(&t, |_| Ok(bind(&f))),
            Err(HtmlLayoutError::NeedsWrapping { .. })
        ));
        t.flags.bits &= !0x4000;
        assert!(
            layout_initial(&t, |_| Ok(bind(&f)))
                .unwrap()
                .geometry
                .overflow_x
        );
    }
    #[test]
    fn invalid_or_unestablished_formats_fail_closed() {
        let f = font();
        for text in [
            markup("A").replace("0.000000", "1"),
            markup("A").replace("size=\"20\"", "size=\"20.001\""),
            markup("A").replace("size=\"20\"", "size=\"20.05\""),
            markup("A").replace("kerning=\"1\"", "kerning=\"0\""),
            markup("<b>A</b>"),
            markup("A&unknown;"),
            markup("A").replace("color=", "unknown="),
        ] {
            assert!(layout_initial(&field(text), |_| Ok(bind(&f))).is_err());
        }
        let mut bad = f.clone();
        let pair = bad.layout.as_ref().unwrap().kerning[0].clone();
        bad.layout.as_mut().unwrap().kerning.push(pair);
        assert!(matches!(
            layout_initial(&field(markup("AB")), |_| Ok(bind(&bad))),
            Err(HtmlLayoutError::Invalid("duplicate kerning pair"))
        ));
    }
}
