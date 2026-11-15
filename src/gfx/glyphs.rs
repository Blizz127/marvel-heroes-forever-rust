//! Exact glyph SHAPE geometry and caller-resolved saved static text placement.
//! Glyph coordinates remain in font EM units, not shape twips or pixels.

#[allow(unused_imports)]
pub use layout::{
    layout_static, ExactPosition, FontIdentity, PlacedGlyph, ResolvedFont, StaticLayout, TextOrigin,
};

use crate::gfx::shapes::{
    BitSpan, Color, Edge, Fill, FillRule, FillStyle, Geometry, Point, Record, Shape, StyleChange,
    StyleRef, StyleSet,
};
use crate::gfx::tessellate::{self, Mesh, Options};
use crate::gfx::text::FontDefinition;
use crate::gfx::{Rect, Span};
use std::io;

#[derive(Clone, Copy, Debug)]
pub struct OutlineLimits {
    pub max_bytes: usize,
    pub max_records: usize,
}
impl Default for OutlineLimits {
    fn default() -> Self {
        Self {
            max_bytes: 16 * 1024 * 1024,
            max_records: 1_000_000,
        }
    }
}

/// One original glyph, with integer geometry and absolute source bit spans.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlyphOutline {
    pub font_id: u16,
    pub font_tag_code: u16,
    pub glyph_index: usize,
    pub code: Option<u16>,
    pub source: Span,
    pub units_per_em: u32,
    pub fill_bits: u8,
    pub line_bits: u8,
    pub records: Vec<Record>,
    pub edges: Vec<Edge>,
    /// Serialized layout bounds, if present. These may be deliberately minimal.
    pub saved_bounds: Option<Rect>,
    /// Conservative hull of anchors and controls, not an exact curve-extrema box.
    pub control_bounds: Option<Rect>,
}

/// Mesh positions remain in the original EM coordinate space.
#[derive(Clone, Debug, PartialEq)]
pub struct GlyphMesh {
    pub font_id: u16,
    pub glyph_index: usize,
    pub code: Option<u16>,
    pub units_per_em: u32,
    pub mesh: Mesh,
}

impl GlyphOutline {
    pub fn from_font(font: &FontDefinition, glyph_index: usize, movie: &[u8]) -> io::Result<Self> {
        Self::from_font_with_limits(font, glyph_index, movie, OutlineLimits::default())
    }

    pub fn from_font_with_limits(
        font: &FontDefinition,
        glyph_index: usize,
        movie: &[u8],
        limits: OutlineLimits,
    ) -> io::Result<Self> {
        let expected_units = match font.tag_code {
            10 | 48 => 1024,
            75 => 20480,
            _ => return Err(unsupported("unsupported outline font tag")),
        };
        if font.units_per_em != expected_units {
            return Err(invalid("font EM unit mismatch"));
        }
        let glyph = font
            .glyphs
            .get(glyph_index)
            .ok_or_else(|| invalid("glyph index outside resolved font"))?;
        let source = glyph.shape.source;
        let end = source
            .offset
            .checked_add(source.len)
            .ok_or_else(|| invalid("glyph source overflow"))?;
        let font_end = font
            .source
            .offset
            .checked_add(font.source.len)
            .ok_or_else(|| invalid("font source overflow"))?;
        if source.offset < font.source.offset || end > font_end {
            return Err(invalid("glyph lies outside font definition"));
        }
        if source.len > limits.max_bytes {
            return Err(invalid("glyph byte limit exceeded"));
        }
        end.checked_mul(8)
            .ok_or_else(|| invalid("glyph bit span overflow"))?;
        let bytes = source.bytes(movie)?;
        let mut r = Bits {
            bytes,
            bit: 0,
            base: source.offset,
        };
        let fill_bits = r.unsigned(4)? as u8;
        let line_bits = r.unsigned(4)? as u8;
        let mut records = Vec::new();
        let mut edges = Vec::new();
        let mut cursor = Point { x: 0, y: 0 };
        let mut left = None;
        let mut right = None;
        let mut control_bounds = None;
        loop {
            if records.len() >= limits.max_records {
                return Err(invalid("glyph record limit exceeded"));
            }
            let start = r.bit;
            if r.unsigned(1)? != 0 {
                let straight = r.unsigned(1)? != 0;
                let width = r.unsigned(4)? as u8 + 2;
                let from = cursor;
                let geometry = if straight {
                    let (dx, dy) = if r.unsigned(1)? != 0 {
                        (r.signed(width)?, r.signed(width)?)
                    } else if r.unsigned(1)? != 0 {
                        (0, r.signed(width)?)
                    } else {
                        (r.signed(width)?, 0)
                    };
                    cursor = add(cursor, dx, dy)?;
                    Geometry::Line
                } else {
                    let control = add(cursor, r.signed(width)?, r.signed(width)?)?;
                    cursor = add(control, r.signed(width)?, r.signed(width)?)?;
                    include(&mut control_bounds, control);
                    Geometry::Quadratic { control }
                };
                include(&mut control_bounds, from);
                include(&mut control_bounds, cursor);
                records.push(Record::Edge(edges.len()));
                edges.push(Edge {
                    source: r.span(start),
                    from,
                    to: cursor,
                    geometry,
                    fill_left: left,
                    fill_right: right,
                    line: None,
                });
            } else {
                let flags = r.unsigned(5)? as u8;
                if flags == 0 {
                    records.push(Record::End(r.span(start)));
                    while r.bit % 8 != 0 {
                        if r.unsigned(1)? != 0 {
                            return Err(invalid("nonzero glyph end padding"));
                        }
                    }
                    if r.bit / 8 != bytes.len() {
                        return Err(invalid("bytes after glyph EndShape"));
                    }
                    break;
                }
                if flags & 16 != 0 {
                    return Err(unsupported("glyph introduces new styles"));
                }
                let mut change = StyleChange {
                    source: r.span(start),
                    flags,
                    move_to: None,
                    fill_left: None,
                    fill_right: None,
                    line: None,
                    new_style_set: None,
                };
                if flags & 1 != 0 {
                    let width = r.unsigned(5)? as u8;
                    cursor = Point {
                        x: r.signed(width)?,
                        y: r.signed(width)?,
                    };
                    change.move_to = Some(cursor);
                }
                if flags & 2 != 0 {
                    left = glyph_fill(r.unsigned(fill_bits)?)?;
                    change.fill_left = Some(left);
                }
                if flags & 4 != 0 {
                    right = glyph_fill(r.unsigned(fill_bits)?)?;
                    change.fill_right = Some(right);
                }
                if flags & 8 != 0 {
                    if r.unsigned(line_bits)? != 0 {
                        return Err(unsupported("glyph selects a stroke style"));
                    }
                    change.line = Some(None);
                }
                change.source = r.span(start);
                records.push(Record::StyleChange(change));
            }
        }
        if fill_bits != glyph.shape.fill_bits
            || line_bits != glyph.shape.line_bits
            || records.len() != glyph.shape.record_count
            || edges.len() != glyph.shape.edge_count
        {
            return Err(invalid("glyph bytes disagree with font metadata"));
        }
        Ok(Self {
            font_id: font.id,
            font_tag_code: font.tag_code,
            glyph_index,
            code: glyph.code,
            source,
            units_per_em: font.units_per_em,
            fill_bits,
            line_bits,
            records,
            edges,
            saved_bounds: font
                .layout
                .as_ref()
                .and_then(|l| l.bounds.get(glyph_index).copied()),
            control_bounds,
        })
    }

    /// Reuse the project's fill tessellator without normalizing EM coordinates.
    /// `curve_tolerance` is measured in EM units here. The caller explicitly
    /// selects fill policy; native font rasterization has not been compared.
    pub fn tessellate(
        &self,
        color: Color,
        fill_rule: FillRule,
        mut options: Options,
    ) -> io::Result<GlyphMesh> {
        if options
            .fill_rule_override
            .is_some_and(|rule| rule != fill_rule)
        {
            return Err(invalid("conflicting glyph fill policies"));
        }
        options.fill_rule_override = Some(fill_rule);
        // This is an internal geometric adapter, not a fabricated DefineShape
        // record. Font glyphs have one implicit fill, supplied by the text style.
        // Zero-length style spans explicitly mean no serialized style bytes.
        let implicit_source = Span {
            offset: self.source.offset,
            len: 0,
        };
        let shape = Shape {
            source: self.source,
            tag_code: self.font_tag_code,
            version: 0,
            id: self.font_id,
            bounds: self.control_bounds.unwrap_or(Rect {
                x_min: 0,
                x_max: 0,
                y_min: 0,
                y_max: 0,
            }),
            edge_bounds: None,
            fill_rule,
            flags: None,
            styles: vec![StyleSet {
                source: implicit_source,
                fills: vec![FillStyle {
                    source: implicit_source,
                    fill: Fill::Solid(color),
                }],
                lines: Vec::new(),
            }],
            records: self.records.clone(),
            edges: self.edges.clone(),
            unsupported: Vec::new(),
        };
        let mesh = tessellate::tessellate(&shape, options)?;
        Ok(GlyphMesh {
            font_id: self.font_id,
            glyph_index: self.glyph_index,
            code: self.code,
            units_per_em: self.units_per_em,
            mesh,
        })
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn unsupported(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}
fn add(p: Point, x: i32, y: i32) -> io::Result<Point> {
    Ok(Point {
        x: p.x
            .checked_add(x)
            .ok_or_else(|| invalid("glyph X overflow"))?,
        y: p.y
            .checked_add(y)
            .ok_or_else(|| invalid("glyph Y overflow"))?,
    })
}
fn glyph_fill(index: u32) -> io::Result<Option<StyleRef>> {
    match index {
        0 => Ok(None),
        1 => Ok(Some(StyleRef { set: 0, index: 1 })),
        _ => Err(unsupported("glyph selects an unavailable implicit fill")),
    }
}
fn include(bounds: &mut Option<Rect>, p: Point) {
    match bounds {
        Some(b) => {
            b.x_min = b.x_min.min(p.x);
            b.x_max = b.x_max.max(p.x);
            b.y_min = b.y_min.min(p.y);
            b.y_max = b.y_max.max(p.y);
        }
        None => {
            *bounds = Some(Rect {
                x_min: p.x,
                x_max: p.x,
                y_min: p.y,
                y_max: p.y,
            })
        }
    }
}
struct Bits<'a> {
    bytes: &'a [u8],
    bit: usize,
    base: usize,
}
impl Bits<'_> {
    fn unsigned(&mut self, width: u8) -> io::Result<u32> {
        if width > 32
            || usize::from(width) > self.bytes.len().saturating_mul(8).saturating_sub(self.bit)
        {
            return Err(invalid("truncated glyph bit record"));
        }
        let mut value = 0;
        for _ in 0..width {
            value = (value << 1) | u32::from((self.bytes[self.bit / 8] >> (7 - self.bit % 8)) & 1);
            self.bit += 1;
        }
        Ok(value)
    }
    fn signed(&mut self, width: u8) -> io::Result<i32> {
        let value = self.unsigned(width)?;
        Ok(match width {
            0 => 0,
            32 => value as i32,
            _ => ((value << (32 - width)) as i32) >> (32 - width),
        })
    }
    fn span(&self, start: usize) -> BitSpan {
        BitSpan {
            start: self.base * 8 + start,
            len: self.bit - start,
        }
    }
}

#[cfg(test)]
mod outline_tests {
    use super::*;
    struct Writer {
        bytes: Vec<u8>,
        bit: usize,
    }
    impl Writer {
        fn new() -> Self {
            Self {
                bytes: Vec::new(),
                bit: 0,
            }
        }
        fn bits(&mut self, value: u32, count: u8) {
            for shift in (0..count).rev() {
                if self.bit % 8 == 0 {
                    self.bytes.push(0);
                }
                self.bytes[self.bit / 8] |= (((value >> shift) & 1) as u8) << (7 - self.bit % 8);
                self.bit += 1;
            }
        }
        fn signed(&mut self, value: i32, count: u8) {
            self.bits(value as u32, count);
        }
        fn line(&mut self, dx: i32, dy: i32) {
            self.bits(1, 1);
            self.bits(1, 1);
            self.bits(6, 4);
            self.bits(1, 1);
            self.signed(dx, 8);
            self.signed(dy, 8);
        }
        fn finish(mut self) -> Vec<u8> {
            self.bits(0, 6);
            while self.bit % 8 != 0 {
                self.bits(0, 1);
            }
            self.bytes
        }
    }
    fn square() -> Vec<u8> {
        let mut w = Writer::new();
        w.bits(1, 4);
        w.bits(0, 4);
        w.bits(0, 1);
        w.bits(3, 5);
        w.bits(7, 5);
        w.signed(-10, 7);
        w.signed(20, 7);
        w.bits(1, 1);
        w.line(100, 0);
        w.line(0, 100);
        w.line(-100, 0);
        w.line(0, -100);
        w.finish()
    }
    fn font(shape: &[u8]) -> (FontDefinition, Vec<u8>) {
        let mut body = vec![7, 0, 4, 0, 0, 1, 0, 4, 0];
        body.extend_from_slice(&((4 + shape.len()) as u16).to_le_bytes());
        body.extend_from_slice(shape);
        body.extend_from_slice(&65u16.to_le_bytes());
        let definition = FontDefinition::parse(75, &body, 30).unwrap();
        let mut movie = vec![0; 30];
        movie.extend(body);
        (definition, movie)
    }
    #[test]
    fn exact_signed_lines_and_implicit_fill_mesh() {
        let (font, movie) = font(&square());
        let glyph = GlyphOutline::from_font(&font, 0, &movie).unwrap();
        assert_eq!(glyph.edges[0].from, Point { x: -10, y: 20 });
        assert_eq!(glyph.edges[0].to, Point { x: 90, y: 20 });
        assert_eq!(glyph.edges[3].to, Point { x: -10, y: 20 });
        assert_eq!(glyph.edges[0].source.start / 8, 45);
        assert_eq!(glyph.code, Some(65));
        assert_eq!(glyph.units_per_em, 20480);
        let result = glyph
            .tessellate(
                Color {
                    red: 4,
                    green: 5,
                    blue: 6,
                    alpha: 255,
                },
                FillRule::EvenOdd,
                Options::default(),
            )
            .unwrap();
        let fill = &result.mesh.fills[0];
        assert_eq!(fill.indices.len(), 6);
        let area: f64 = fill
            .indices
            .chunks_exact(3)
            .map(|t| {
                let a = fill.positions[t[0] as usize];
                let b = fill.positions[t[1] as usize];
                let c = fill.positions[t[2] as usize];
                ((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)) / 2.0
            })
            .sum();
        assert_eq!(area, 10000.0);
        assert_eq!(fill.paint.source.len, 0);
    }
    #[test]
    fn curves_retain_both_relative_deltas() {
        let mut w = Writer::new();
        w.bits(1, 4);
        w.bits(0, 4);
        w.bits(0, 1);
        w.bits(2, 5);
        w.bits(1, 1);
        w.bits(1, 1);
        w.bits(0, 1);
        w.bits(6, 4);
        w.signed(30, 8);
        w.signed(-20, 8);
        w.signed(40, 8);
        w.signed(50, 8);
        let (font, movie) = font(&w.finish());
        let glyph = GlyphOutline::from_font(&font, 0, &movie).unwrap();
        assert_eq!(
            glyph.edges[0].geometry,
            Geometry::Quadratic {
                control: Point { x: 30, y: -20 }
            }
        );
        assert_eq!(glyph.edges[0].to, Point { x: 70, y: 30 });
        assert_eq!(
            glyph.control_bounds,
            Some(Rect {
                x_min: 0,
                x_max: 70,
                y_min: -20,
                y_max: 30
            })
        );
    }
    #[test]
    fn rejects_stale_metadata_bounds_and_limits() {
        let (mut font, movie) = font(&square());
        assert!(GlyphOutline::from_font(&font, 1, &movie).is_err());
        assert!(GlyphOutline::from_font_with_limits(
            &font,
            0,
            &movie,
            OutlineLimits {
                max_bytes: 1,
                max_records: 10
            }
        )
        .is_err());
        assert!(GlyphOutline::from_font_with_limits(
            &font,
            0,
            &movie,
            OutlineLimits {
                max_bytes: 100,
                max_records: 1
            }
        )
        .is_err());
        for end in 0..font.glyphs[0].shape.source.offset + font.glyphs[0].shape.source.len {
            assert!(GlyphOutline::from_font(&font, 0, &movie[..end]).is_err());
        }
        font.glyphs[0].shape.edge_count += 1;
        assert!(GlyphOutline::from_font(&font, 0, &movie).is_err());
        font.glyphs[0].shape.edge_count -= 1;
        font.units_per_em = 1024;
        assert!(GlyphOutline::from_font(&font, 0, &movie).is_err());
    }
    #[test]
    fn empty_glyph_and_coordinate_overflow_are_explicit() {
        let (font, movie) = font(&[0, 0]);
        let glyph = GlyphOutline::from_font(&font, 0, &movie).unwrap();
        assert!(glyph.edges.is_empty());
        assert_eq!(glyph.control_bounds, None);
        assert!(add(Point { x: i32::MAX, y: 0 }, 1, 0).is_err());
        assert!(glyph_fill(2).is_err());
    }
}

/// Saved static-text placement with explicit font resolution and coordinate origin.
/// This module does not shape strings, resolve aliases, or lay out editable text.
pub mod layout {
    use crate::gfx::text::{FontDefinition, StaticText};
    use crate::gfx::{Matrix, Rect, Span};
    use std::collections::BTreeMap;
    use std::io;

    pub const MAX_STATIC_RECORDS: usize = 65_536;
    pub const MAX_STATIC_GLYPHS: usize = 1_000_000;

    fn invalid(message: &'static str) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, message)
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct FontIdentity {
        pub movie_id: usize,
        pub character_id: u16,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct ResolvedFont<'a> {
        pub identity: FontIdentity,
        pub definition: &'a FontDefinition,
    }

    /// The saved bounds and Adobe's offset prose do not establish the native
    /// origin convention for every movie. Callers must select it explicitly.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum TextOrigin {
        TextLocal,
        BoundsRelative,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct PlacedGlyph {
        pub record_index: usize,
        pub index_in_record: usize,
        pub record_source: Span,
        /// The ID used by this text's movie, which may be an imported ID.
        pub local_font_id: u16,
        /// The actual caller-selected font definition's movie and dictionary ID.
        pub font: FontIdentity,
        pub glyph_index: u32,
        /// Raw code-table value; neither Unicode conversion nor character lookup.
        pub code: Option<u16>,
        pub shape_source: Span,
        /// Untransformed twips. Advances accumulate without rounding or kerning.
        pub pen_x: i64,
        pub pen_y: i64,
        pub advance: i32,
        pub text_height: u16,
        /// None means no color has been supplied; no default color is invented.
        pub color: Option<[u8; 4]>,
        pub units_per_em: u32,
    }

    /// Exact transformed twips, with a shared positive denominator. This keeps
    /// the 16.16 matrix and EM scaling exact until a consumer requests floats.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ExactPosition {
        pub x_numerator: i128,
        pub y_numerator: i128,
        pub denominator: i128,
    }

    impl ExactPosition {
        pub fn twips(self) -> [f64; 2] {
            [
                self.x_numerator as f64 / self.denominator as f64,
                self.y_numerator as f64 / self.denominator as f64,
            ]
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct StaticLayout {
        pub text_id: u16,
        pub source: Span,
        pub bounds: Rect,
        pub matrix: Matrix,
        pub glyphs: Vec<PlacedGlyph>,
        pub final_pen: [i64; 2],
    }

    impl StaticLayout {
        /// Scale a glyph's raw EM point by text_height / units_per_em, add the
        /// saved pen (and bounds minimum only when explicitly selected), then
        /// apply TextMatrix. Display-list parent matrices are not applied here.
        pub fn position(
            &self,
            placement_index: usize,
            em_point: [i32; 2],
            origin: TextOrigin,
        ) -> io::Result<ExactPosition> {
            let glyph = self
                .glyphs
                .get(placement_index)
                .ok_or_else(|| invalid("static glyph index outside layout"))?;
            if !matches!(glyph.units_per_em, 1024 | 20480) {
                return Err(invalid("unsupported font EM size"));
            }
            let units = i128::from(glyph.units_per_em);
            let height = i128::from(glyph.text_height);
            let (origin_x, origin_y) = match origin {
                TextOrigin::TextLocal => (0, 0),
                TextOrigin::BoundsRelative => {
                    (i128::from(self.bounds.x_min), i128::from(self.bounds.y_min))
                }
            };
            // Pen values originate in i64, EM size is at most 20480, and matrix
            // coefficients are i32. Even two maximal terms fit in signed i128.
            let x = (i128::from(glyph.pen_x) + origin_x) * units + i128::from(em_point[0]) * height;
            let y = (i128::from(glyph.pen_y) + origin_y) * units + i128::from(em_point[1]) * height;
            let denominator = units * 65_536;
            Ok(ExactPosition {
                x_numerator: x * i128::from(self.matrix.scale_x)
                    + y * i128::from(self.matrix.rotate_skew_1)
                    + i128::from(self.matrix.translate_x) * denominator,
                y_numerator: x * i128::from(self.matrix.rotate_skew_0)
                    + y * i128::from(self.matrix.scale_y)
                    + i128::from(self.matrix.translate_y) * denominator,
                denominator,
            })
        }
    }

    /// Replay only saved TEXTRECORD changes and GLYPHENTRY advances. A local
    /// font ID is resolved once per call and cached; resolver failures propagate.
    /// No code-point substitution, font metrics or kerning alter saved advances.
    /// Aliases sharing a resolved identity must return the same definition
    /// instance; conflicting definitions for one identity are rejected.
    pub fn layout_static<'a, F>(text: &StaticText, mut resolve: F) -> io::Result<StaticLayout>
    where
        F: FnMut(u16) -> io::Result<ResolvedFont<'a>>,
    {
        if !matches!(text.tag_code, 11 | 33) {
            return Err(invalid("static layout requires DefineText or DefineText2"));
        }
        if text.records.len() > MAX_STATIC_RECORDS {
            return Err(invalid("static text record limit exceeded"));
        }
        let mut count = 0usize;
        for record in &text.records {
            count = count
                .checked_add(record.glyphs.len())
                .ok_or_else(|| invalid("static text glyph count overflow"))?;
            if count > MAX_STATIC_GLYPHS {
                return Err(invalid("static text glyph limit exceeded"));
            }
            if record.font_id.is_some() != record.text_height.is_some() {
                return Err(invalid("static font change requires a paired text height"));
            }
        }
        let mut fonts = BTreeMap::<u16, ResolvedFont<'a>>::new();
        let mut identities = BTreeMap::<FontIdentity, &'a FontDefinition>::new();
        let mut font_id = None;
        let mut height = None;
        let mut color = None;
        let mut pen_x = 0i64;
        let mut pen_y = 0i64;
        let mut glyphs = Vec::with_capacity(count);
        for (record_index, record) in text.records.iter().enumerate() {
            if let Some(value) = record.font_id {
                font_id = Some(value);
                height = record.text_height;
            }
            if let Some(value) = record.color {
                color = Some(value);
            }
            if let Some(value) = record.x_offset {
                pen_x = i64::from(value);
            }
            if let Some(value) = record.y_offset {
                pen_y = i64::from(value);
            }
            if record.glyphs.is_empty() {
                continue;
            }
            let local_font_id = font_id.ok_or_else(|| invalid("glyph has no selected font"))?;
            let text_height = height.ok_or_else(|| invalid("glyph has no selected height"))?;
            if let std::collections::btree_map::Entry::Vacant(entry) = fonts.entry(local_font_id) {
                let font = resolve(local_font_id)?;
                if font.identity.character_id != font.definition.id {
                    return Err(invalid("resolved identity does not match font definition"));
                }
                let expected_units = match font.definition.tag_code {
                    10 | 48 => 1024,
                    75 => 20480,
                    _ => return Err(invalid("unsupported resolved font definition")),
                };
                if font.definition.units_per_em != expected_units {
                    return Err(invalid("resolved font has inconsistent EM size"));
                }
                if let Some(previous) = identities.get(&font.identity) {
                    if !std::ptr::eq(*previous, font.definition) {
                        return Err(invalid(
                            "resolved identity refers to conflicting definitions",
                        ));
                    }
                } else {
                    identities.insert(font.identity, font.definition);
                }
                entry.insert(font);
            }
            let resolved = fonts
                .get(&local_font_id)
                .ok_or_else(|| invalid("resolved font was not retained"))?;
            for (index_in_record, glyph) in record.glyphs.iter().enumerate() {
                let index = usize::try_from(glyph.index)
                    .map_err(|_| invalid("glyph index exceeds host address space"))?;
                let source_glyph = resolved
                    .definition
                    .glyphs
                    .get(index)
                    .ok_or_else(|| invalid("saved glyph index outside resolved font"))?;
                glyphs.push(PlacedGlyph {
                    record_index,
                    index_in_record,
                    record_source: record.span,
                    local_font_id,
                    font: resolved.identity,
                    glyph_index: glyph.index,
                    code: source_glyph.code,
                    shape_source: source_glyph.shape.source,
                    pen_x,
                    pen_y,
                    advance: glyph.advance,
                    text_height,
                    color,
                    units_per_em: resolved.definition.units_per_em,
                });
                pen_x = pen_x
                    .checked_add(i64::from(glyph.advance))
                    .ok_or_else(|| invalid("static text pen overflow"))?;
            }
        }
        Ok(StaticLayout {
            text_id: text.id,
            source: text.body,
            bounds: text.bounds,
            matrix: text.matrix,
            glyphs,
            final_pen: [pen_x, pen_y],
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::gfx::text::{FontGlyph, Glyph, GlyphShape, TextRecord};

        fn span() -> Span {
            Span { offset: 20, len: 4 }
        }

        fn font() -> FontDefinition {
            FontDefinition {
                source: span(),
                tag_code: 75,
                id: 9,
                flags: None,
                language: None,
                name: None,
                glyphs: vec![FontGlyph {
                    shape: GlyphShape {
                        source: span(),
                        fill_bits: 1,
                        line_bits: 0,
                        record_count: 1,
                        edge_count: 0,
                    },
                    code: Some(65),
                }],
                layout: None,
                units_per_em: 20480,
            }
        }

        fn record() -> TextRecord {
            TextRecord {
                span: span(),
                flags: 0x8f,
                font_id: Some(4),
                color: Some([1, 2, 3, 4]),
                x_offset: Some(-5),
                y_offset: Some(8),
                text_height: Some(512),
                glyph_data: span(),
                glyphs: vec![Glyph {
                    index: 0,
                    advance: 11,
                }],
            }
        }

        fn text(records: Vec<TextRecord>) -> StaticText {
            StaticText {
                tag_code: 33,
                id: 6,
                bounds: Rect {
                    x_min: 100,
                    x_max: 200,
                    y_min: 40,
                    y_max: 80,
                },
                matrix: Matrix::default(),
                glyph_bits: 1,
                advance_bits: 5,
                records,
                body: span(),
                terminator: span(),
            }
        }

        fn resolved(font: &FontDefinition) -> ResolvedFont<'_> {
            ResolvedFont {
                identity: FontIdentity {
                    movie_id: 7,
                    character_id: 9,
                },
                definition: font,
            }
        }

        #[test]
        fn inherits_style_and_pen_but_offsets_replace_coordinates() {
            let font = font();
            let first = record();
            let mut second = first.clone();
            second.font_id = None;
            second.text_height = None;
            second.color = None;
            second.x_offset = None;
            second.y_offset = None;
            second.glyphs[0].advance = -3;
            let mut third = second.clone();
            third.x_offset = Some(2);
            third.y_offset = Some(-4);
            let mut calls = 0;
            let layout = layout_static(&text(vec![first, second, third]), |id| {
                assert_eq!(id, 4);
                calls += 1;
                Ok(resolved(&font))
            })
            .unwrap();
            assert_eq!(calls, 1);
            assert_eq!(
                layout
                    .glyphs
                    .iter()
                    .map(|g| (g.pen_x, g.pen_y))
                    .collect::<Vec<_>>(),
                [(-5, 8), (6, 8), (2, -4)]
            );
            assert_eq!(layout.final_pen, [-1, -4]);
            assert_eq!(layout.glyphs[1].local_font_id, 4);
            assert_eq!(layout.glyphs[1].font, resolved(&font).identity);
            assert_eq!(layout.glyphs[1].code, Some(65));
            assert_eq!(layout.glyphs[1].color, Some([1, 2, 3, 4]));
            assert_eq!(layout.glyphs[1].text_height, 512);
        }

        #[test]
        fn exact_scaling_matrix_order_and_origin_are_explicit() {
            let font = font();
            let mut text = text(vec![record()]);
            text.matrix = Matrix {
                scale_x: 131072,
                scale_y: 65536,
                rotate_skew_0: 65536,
                rotate_skew_1: -65536,
                translate_x: 3,
                translate_y: -7,
            };
            let layout = layout_static(&text, |_| Ok(resolved(&font))).unwrap();
            // EM [20480,-10240] -> [512,-256] -> pen [507,-248].
            assert_eq!(
                layout
                    .position(0, [20480, -10240], TextOrigin::TextLocal)
                    .unwrap()
                    .twips(),
                [1265.0, 252.0]
            );
            assert_eq!(
                layout
                    .position(0, [20480, -10240], TextOrigin::BoundsRelative)
                    .unwrap()
                    .twips(),
                [1425.0, 392.0]
            );
            assert!(layout.position(1, [0, 0], TextOrigin::TextLocal).is_err());
        }

        #[test]
        fn missing_font_bad_index_and_resolver_failure_do_not_substitute() {
            let font = font();
            let mut r = record();
            r.font_id = None;
            r.text_height = None;
            assert!(layout_static(&text(vec![r]), |_| Ok(resolved(&font))).is_err());
            let mut r = record();
            r.glyphs[0].index = 1;
            assert!(layout_static(&text(vec![r]), |_| Ok(resolved(&font))).is_err());
            let error = layout_static(&text(vec![record()]), |_| {
                Err(io::Error::new(io::ErrorKind::NotFound, "unresolved font"))
            })
            .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::NotFound);
        }

        #[test]
        fn rejects_inconsistent_fonts_and_preserves_unknown_color() {
            let mut font = font();
            let text = text(vec![record()]);
            font.id = 8;
            assert!(layout_static(&text, |_| Ok(resolved(&font))).is_err());
            font.id = 9;
            font.units_per_em = 0;
            assert!(layout_static(&text, |_| Ok(resolved(&font))).is_err());
            font.units_per_em = 20480;
            let mut text = text;
            text.records[0].color = None;
            assert_eq!(
                layout_static(&text, |_| Ok(resolved(&font)))
                    .unwrap()
                    .glyphs[0]
                    .color,
                None
            );
            text.records[0].text_height = None;
            assert!(layout_static(&text, |_| Ok(resolved(&font))).is_err());
        }

        #[test]
        fn enforces_glyph_budget_before_resolving_and_handles_empty_text() {
            let font = font();
            let mut r = record();
            r.glyphs = vec![
                Glyph {
                    index: 0,
                    advance: 0
                };
                MAX_STATIC_GLYPHS + 1
            ];
            assert!(
                layout_static(&text(vec![r]), |_| panic!("must fail before resolution")).is_err()
            );
            let empty = layout_static(&text(vec![]), |_| Ok(resolved(&font))).unwrap();
            assert!(empty.glyphs.is_empty());
            assert_eq!(empty.final_pen, [0, 0]);
        }

        #[test]
        fn aliases_require_consistent_identity_and_zero_height_is_retained() {
            let font = font();
            let duplicate = font.clone();
            let first = record();
            let mut second = first.clone();
            second.font_id = Some(5);
            second.text_height = Some(0);
            let text = text(vec![first, second]);
            let layout = layout_static(&text, |_| Ok(resolved(&font))).unwrap();
            assert_eq!(layout.glyphs[1].text_height, 0);
            assert_eq!(
                layout
                    .position(1, [20480, 20480], TextOrigin::TextLocal)
                    .unwrap()
                    .twips(),
                [-5.0, 8.0]
            );
            assert!(layout_static(&text, |id| Ok(resolved(if id == 4 {
                &font
            } else {
                &duplicate
            })))
            .is_err());
        }
    }
}
