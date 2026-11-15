//! Original bounded DefineShape/2/3/4 decoder; no tessellation or rasterization.
//! Coordinates are signed twips; transforms retain signed 16.16 values.
//! See `docs/GFX_SHAPES.md` for selected-style lifetime and unsupported semantics.

use super::{Matrix, Movie, Rect, Span};
use std::io;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_bytes: usize,
    pub max_records: usize,
    pub max_styles: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_bytes: 16 * 1024 * 1024,
            max_records: 1_000_000,
            max_styles: 131_072,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitSpan {
    /// Absolute bit offset in the containing movie, most significant bit first.
    pub start: usize,
    pub len: usize,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
impl Point {
    fn add(self, x: i32, y: i32) -> io::Result<Self> {
        Ok(Self {
            x: self
                .x
                .checked_add(x)
                .ok_or_else(|| invalid("shape X overflow"))?,
            y: self
                .y
                .checked_add(y)
                .ok_or_else(|| invalid("shape Y overflow"))?,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillRule {
    EvenOdd,
    NonZero,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape4Flags {
    pub raw: u8,
    pub uses_non_scaling_strokes: bool,
    pub uses_scaling_strokes: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientKind {
    Linear,
    Radial,
    FocalRadial,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GradientStop {
    pub ratio: u8,
    pub color: Color,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fill {
    Solid(Color),
    Gradient {
        kind: GradientKind,
        matrix: Matrix,
        /// 0 pad, 1 reflect, 2 repeat, 3 reserved (reported in `unsupported`).
        spread: u8,
        /// 0 normal RGB, 1 linear RGB, 2/3 reserved.
        interpolation: u8,
        stops: Vec<GradientStop>,
        /// Signed 8.8 fixed point, present only for focal radial gradients.
        focal_point: Option<i16>,
    },
    Bitmap {
        character_id: u16,
        matrix: Matrix,
        repeating: bool,
        smoothed: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FillStyle {
    pub source: Span,
    pub fill: Fill,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinePaint {
    Color(Color),
    Fill(FillStyle),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineFlags {
    /// The two serialized flag bytes in bit-reading (big-endian) order.
    pub raw: u16,
    pub start_cap: u8,
    pub end_cap: u8,
    pub join: u8,
    pub no_horizontal_scale: bool,
    pub no_vertical_scale: bool,
    pub pixel_hinting: bool,
    pub no_close: bool,
    /// Unsigned 8.8 fixed point, present only for miter joins.
    pub miter_limit: Option<u16>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineStyle {
    pub source: Span,
    pub width: u16,
    /// None denotes the older round cap/join, scaling stroke encoding.
    pub flags: Option<LineFlags>,
    pub paint: LinePaint,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleSet {
    pub source: Span,
    pub fills: Vec<FillStyle>,
    pub lines: Vec<LineStyle>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StyleRef {
    pub set: usize,
    /// One-based serialized index; zero is represented by None.
    pub index: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Geometry {
    Line,
    Quadratic { control: Point },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub source: BitSpan,
    pub from: Point,
    pub to: Point,
    pub geometry: Geometry,
    /// FillStyle0: the fill on the left of this directed edge.
    pub fill_left: Option<StyleRef>,
    /// FillStyle1: the fill on the right of this directed edge.
    pub fill_right: Option<StyleRef>,
    pub line: Option<StyleRef>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleChange {
    pub source: BitSpan,
    pub flags: u8,
    /// Absolute position in shape coordinates; this is not a relative move.
    pub move_to: Option<Point>,
    /// Outer None means unchanged; Some(None) selects no style.
    pub fill_left: Option<Option<StyleRef>>,
    pub fill_right: Option<Option<StyleRef>>,
    pub line: Option<Option<StyleRef>>,
    pub new_style_set: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    StyleChange(StyleChange),
    Edge(usize),
    End(BitSpan),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedFeature {
    pub source: BitSpan,
    pub description: &'static str,
    pub value: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shape {
    pub source: Span,
    pub tag_code: u16,
    pub version: u8,
    pub id: u16,
    pub bounds: Rect,
    pub edge_bounds: Option<Rect>,
    /// Shape4's winding bit is mapped to NonZero by inference; raw flags remain
    /// available. No winding-dependent rasterization has been validated here.
    pub fill_rule: FillRule,
    pub flags: Option<Shape4Flags>,
    /// Immutable banks preserve resolved selections across StateNewStyles.
    pub styles: Vec<StyleSet>,
    pub records: Vec<Record>,
    pub edges: Vec<Edge>,
    /// Recognized layout but reserved/unsupported semantic values; never normalized.
    pub unsupported: Vec<UnsupportedFeature>,
}
impl Shape {
    pub fn parse(tag_code: u16, body: &[u8], body_offset: usize) -> io::Result<Self> {
        Self::parse_with_limits(tag_code, body, body_offset, Limits::default())
    }
    pub fn parse_with_limits(
        tag_code: u16,
        body: &[u8],
        body_offset: usize,
        limits: Limits,
    ) -> io::Result<Self> {
        let version = match tag_code {
            2 => 1,
            22 => 2,
            32 => 3,
            83 => 4,
            _ => return Err(unsupported("not a DefineShape/2/3/4 tag")),
        };
        if body.len() > limits.max_bytes {
            return Err(invalid("shape byte limit"));
        }
        let end = body_offset
            .checked_add(body.len())
            .ok_or_else(|| invalid("shape offset overflow"))?;
        end.checked_mul(8)
            .ok_or_else(|| invalid("shape bit offset overflow"))?;
        let mut r = Reader {
            bytes: body,
            bit: 0,
            base: body_offset,
            version,
            limits,
            style_count: 0,
            unsupported: Vec::new(),
        };
        let id = r.u16()?;
        if id == 0 {
            return Err(invalid("zero shape ID"));
        }
        let bounds = r.rect()?;
        let mut edge_bounds = None;
        let mut flags = None;
        let mut fill_rule = FillRule::EvenOdd;
        if version == 4 {
            edge_bounds = Some(r.rect()?);
            let at = r.bit;
            let raw = r.u8()?;
            if raw & 0xf8 != 0 {
                r.note(at, 5, "reserved DefineShape4 flags", u32::from(raw >> 3));
            }
            if raw & 4 != 0 {
                fill_rule = FillRule::NonZero;
            }
            flags = Some(Shape4Flags {
                raw,
                uses_non_scaling_strokes: raw & 2 != 0,
                uses_scaling_strokes: raw & 1 != 0,
            });
        }
        let mut styles = vec![r.styles()?];
        let mut fill_bits = r.unsigned(4)? as u8;
        let mut line_bits = r.unsigned(4)? as u8;
        let mut set = 0;
        let mut left = None;
        let mut right = None;
        let mut line = None;
        let mut cursor = Point::default();
        let mut records = Vec::new();
        let mut edges = Vec::new();
        loop {
            if records.len() >= limits.max_records {
                return Err(invalid("shape record limit"));
            }
            let start = r.bit;
            if r.unsigned(1)? != 0 {
                let straight = r.unsigned(1)? != 0;
                let bits = r.unsigned(4)? as u8 + 2;
                let from = cursor;
                let geometry;
                if straight {
                    let (dx, dy) = if r.unsigned(1)? != 0 {
                        (r.signed(bits)?, r.signed(bits)?)
                    } else if r.unsigned(1)? != 0 {
                        (0, r.signed(bits)?)
                    } else {
                        (r.signed(bits)?, 0)
                    };
                    cursor = cursor.add(dx, dy)?;
                    geometry = Geometry::Line;
                } else {
                    let control = cursor.add(r.signed(bits)?, r.signed(bits)?)?;
                    cursor = control.add(r.signed(bits)?, r.signed(bits)?)?;
                    geometry = Geometry::Quadratic { control };
                }
                records.push(Record::Edge(edges.len()));
                edges.push(Edge {
                    source: r.span(start),
                    from,
                    to: cursor,
                    geometry,
                    fill_left: left,
                    fill_right: right,
                    line,
                });
            } else {
                let change_flags = r.unsigned(5)? as u8;
                if change_flags == 0 {
                    records.push(Record::End(r.span(start)));
                    r.align();
                    if r.bit / 8 != body.len() {
                        return Err(invalid("bytes after EndShape"));
                    }
                    break;
                }
                let mut change = StyleChange {
                    source: r.span(start),
                    flags: change_flags,
                    move_to: None,
                    fill_left: None,
                    fill_right: None,
                    line: None,
                    new_style_set: None,
                };
                if change_flags & 1 != 0 {
                    let bits = r.unsigned(5)? as u8;
                    cursor = Point {
                        x: r.signed(bits)?,
                        y: r.signed(bits)?,
                    };
                    change.move_to = Some(cursor);
                }
                if change_flags & 2 != 0 {
                    left = style_ref(set, r.unsigned(fill_bits)?, styles[set].fills.len())?;
                    change.fill_left = Some(left);
                }
                if change_flags & 4 != 0 {
                    right = style_ref(set, r.unsigned(fill_bits)?, styles[set].fills.len())?;
                    change.fill_right = Some(right);
                }
                if change_flags & 8 != 0 {
                    line = style_ref(set, r.unsigned(line_bits)?, styles[set].lines.len())?;
                    change.line = Some(line);
                }
                if change_flags & 16 != 0 {
                    if version == 1 {
                        return Err(invalid("DefineShape cannot contain StateNewStyles"));
                    }
                    r.align();
                    set = styles.len();
                    styles.push(r.styles()?);
                    change.new_style_set = Some(set);
                    fill_bits = r.unsigned(4)? as u8;
                    line_bits = r.unsigned(4)? as u8;
                }
                change.source = r.span(start);
                records.push(Record::StyleChange(change));
            }
        }
        Ok(Self {
            source: Span {
                offset: body_offset,
                len: body.len(),
            },
            tag_code,
            version,
            id,
            bounds,
            edge_bounds,
            fill_rule,
            flags,
            styles,
            records,
            edges,
            unsupported: r.unsupported,
        })
    }
    pub fn fill(&self, reference: StyleRef) -> Option<&FillStyle> {
        self.styles
            .get(reference.set)?
            .fills
            .get(usize::from(reference.index.checked_sub(1)?))
    }
    pub fn line(&self, reference: StyleRef) -> Option<&LineStyle> {
        self.styles
            .get(reference.set)?
            .lines
            .get(usize::from(reference.index.checked_sub(1)?))
    }
}
pub fn parse_movie_tag(movie: &Movie, tag_index: usize) -> io::Result<Shape> {
    let tag = movie
        .tags
        .get(tag_index)
        .ok_or_else(|| invalid("shape tag index outside movie"))?;
    Shape::parse(tag.code, tag.body.bytes(movie.bytes())?, tag.body.offset)
}
fn style_ref(set: usize, index: u32, count: usize) -> io::Result<Option<StyleRef>> {
    if index == 0 {
        return Ok(None);
    }
    if index as usize > count {
        return Err(invalid("shape style index outside current bank"));
    }
    Ok(Some(StyleRef {
        set,
        index: index as u16,
    }))
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn unsupported(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

struct Reader<'a> {
    bytes: &'a [u8],
    bit: usize,
    base: usize,
    version: u8,
    limits: Limits,
    style_count: usize,
    unsupported: Vec<UnsupportedFeature>,
}
impl Reader<'_> {
    fn unsigned(&mut self, n: u8) -> io::Result<u32> {
        if n > 32 {
            return Err(invalid("shape bit field wider than 32"));
        }
        let end = self
            .bit
            .checked_add(usize::from(n))
            .ok_or_else(|| invalid("shape bit overflow"))?;
        if end > self.bytes.len() * 8 {
            return Err(invalid("truncated shape bit field"));
        }
        let mut value = 0;
        while self.bit < end {
            value = (value << 1) | u32::from((self.bytes[self.bit / 8] >> (7 - self.bit % 8)) & 1);
            self.bit += 1;
        }
        Ok(value)
    }
    fn signed(&mut self, n: u8) -> io::Result<i32> {
        let value = self.unsigned(n)?;
        Ok(if n == 0 {
            0
        } else {
            ((value << (32 - n)) as i32) >> (32 - n)
        })
    }
    fn align(&mut self) {
        self.bit = (self.bit + 7) & !7;
    }
    fn u8(&mut self) -> io::Result<u8> {
        if self.bit % 8 != 0 {
            return Err(invalid("unaligned shape byte field"));
        }
        Ok(self.unsigned(8)? as u8)
    }
    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes([self.u8()?, self.u8()?]))
    }
    fn span(&self, start: usize) -> BitSpan {
        BitSpan {
            start: self.base * 8 + start,
            len: self.bit - start,
        }
    }
    fn byte_span(&self, start: usize) -> Span {
        Span {
            offset: self.base + start / 8,
            len: (self.bit - start) / 8,
        }
    }
    fn note(&mut self, bit: usize, len: usize, description: &'static str, value: u32) {
        self.unsupported.push(UnsupportedFeature {
            source: BitSpan {
                start: self.base * 8 + bit,
                len,
            },
            description,
            value,
        });
    }
    fn rect(&mut self) -> io::Result<Rect> {
        let bits = self.unsigned(5)? as u8;
        let rect = Rect {
            x_min: self.signed(bits)?,
            x_max: self.signed(bits)?,
            y_min: self.signed(bits)?,
            y_max: self.signed(bits)?,
        };
        self.align();
        if rect.x_min > rect.x_max || rect.y_min > rect.y_max {
            return Err(invalid("reversed shape bounds"));
        }
        Ok(rect)
    }
    fn matrix(&mut self) -> io::Result<Matrix> {
        let mut value = Matrix::default();
        if self.unsigned(1)? != 0 {
            let bits = self.unsigned(5)? as u8;
            value.scale_x = self.signed(bits)?;
            value.scale_y = self.signed(bits)?;
        }
        if self.unsigned(1)? != 0 {
            let bits = self.unsigned(5)? as u8;
            value.rotate_skew_0 = self.signed(bits)?;
            value.rotate_skew_1 = self.signed(bits)?;
        }
        let bits = self.unsigned(5)? as u8;
        value.translate_x = self.signed(bits)?;
        value.translate_y = self.signed(bits)?;
        self.align();
        Ok(value)
    }
    fn color(&mut self) -> io::Result<Color> {
        Ok(Color {
            red: self.u8()?,
            green: self.u8()?,
            blue: self.u8()?,
            alpha: if self.version >= 3 { self.u8()? } else { 255 },
        })
    }
    fn count(&mut self, extended: bool) -> io::Result<usize> {
        let short = self.u8()?;
        let count = if short == 255 && extended {
            usize::from(self.u16()?)
        } else {
            usize::from(short)
        };
        self.style_count = self
            .style_count
            .checked_add(count)
            .ok_or_else(|| invalid("shape style count overflow"))?;
        if self.style_count > self.limits.max_styles {
            return Err(invalid("shape style limit"));
        }
        Ok(count)
    }
    fn styles(&mut self) -> io::Result<StyleSet> {
        let start = self.bit;
        let count = self.count(self.version >= 2)?;
        let mut fills = Vec::new();
        for _ in 0..count {
            fills.push(self.fill()?);
        }
        let count = self.count(true)?;
        let mut lines = Vec::new();
        for _ in 0..count {
            lines.push(self.line_style()?);
        }
        Ok(StyleSet {
            source: self.byte_span(start),
            fills,
            lines,
        })
    }
    fn fill(&mut self) -> io::Result<FillStyle> {
        let start = self.bit;
        let kind = self.u8()?;
        let fill = match kind {
            0 => Fill::Solid(self.color()?),
            0x10 | 0x12 | 0x13 => {
                let matrix = self.matrix()?;
                let at = self.bit;
                let modes = self.u8()?;
                let spread = modes >> 6;
                let interpolation = (modes >> 4) & 3;
                let count = modes & 15;
                if spread == 3 {
                    self.note(at, 2, "reserved gradient spread mode", u32::from(spread));
                }
                if interpolation >= 2 {
                    self.note(
                        at + 2,
                        2,
                        "reserved gradient interpolation mode",
                        u32::from(interpolation),
                    );
                }
                if count == 0 {
                    self.note(at + 4, 4, "empty gradient", 0);
                }
                if self.version < 4 {
                    if spread != 0 || interpolation != 0 {
                        self.note(
                            at,
                            4,
                            "gradient modes require DefineShape4",
                            u32::from(modes >> 4),
                        );
                    }
                    if count > 8 {
                        self.note(
                            at + 4,
                            4,
                            "more than eight gradient stops require DefineShape4",
                            u32::from(count),
                        );
                    }
                    if kind == 0x13 {
                        self.note(
                            start,
                            8,
                            "focal gradient requires DefineShape4",
                            u32::from(kind),
                        );
                    }
                }
                let mut stops = Vec::new();
                for _ in 0..count {
                    let at = self.bit;
                    let ratio = self.u8()?;
                    if stops
                        .last()
                        .is_some_and(|stop: &GradientStop| stop.ratio > ratio)
                    {
                        self.note(at, 8, "gradient stops out of ratio order", u32::from(ratio));
                    }
                    stops.push(GradientStop {
                        ratio,
                        color: self.color()?,
                    });
                }
                let focal_point = if kind == 0x13 {
                    Some(self.u16()? as i16)
                } else {
                    None
                };
                if let Some(value) = focal_point {
                    if !(-256..=256).contains(&value) {
                        self.note(
                            self.bit - 16,
                            16,
                            "focal point outside -1 to 1",
                            value as u16 as u32,
                        );
                    }
                }
                Fill::Gradient {
                    kind: match kind {
                        0x10 => GradientKind::Linear,
                        0x12 => GradientKind::Radial,
                        _ => GradientKind::FocalRadial,
                    },
                    matrix,
                    spread,
                    interpolation,
                    stops,
                    focal_point,
                }
            }
            0x40..=0x43 => Fill::Bitmap {
                character_id: self.u16()?,
                matrix: self.matrix()?,
                repeating: kind & 1 == 0,
                smoothed: kind < 0x42,
            },
            _ => {
                return Err(unsupported(&format!(
                    "unknown fill type 0x{kind:02x} at byte {}",
                    self.base + start / 8
                )));
            }
        };
        Ok(FillStyle {
            source: self.byte_span(start),
            fill,
        })
    }
    fn line_style(&mut self) -> io::Result<LineStyle> {
        let start = self.bit;
        let width = self.u16()?;
        let (flags, paint) = if self.version < 4 {
            (None, LinePaint::Color(self.color()?))
        } else {
            let at = self.bit;
            let raw = self.unsigned(16)? as u16;
            let start_cap = (raw >> 14) as u8;
            let join = ((raw >> 12) & 3) as u8;
            let has_fill = raw & 0x0800 != 0;
            let end_cap = (raw & 3) as u8;
            if start_cap == 3 {
                self.note(at, 2, "reserved line start cap", 3);
            }
            if join == 3 {
                self.note(at + 2, 2, "reserved line join", 3);
            }
            if raw & 0x00f8 != 0 {
                self.note(
                    at + 8,
                    5,
                    "reserved LineStyle2 flags",
                    u32::from((raw >> 3) & 31),
                );
            }
            if end_cap == 3 {
                self.note(at + 14, 2, "reserved line end cap", 3);
            }
            let miter_limit = if join == 2 { Some(self.u16()?) } else { None };
            let flags = LineFlags {
                raw,
                start_cap,
                end_cap,
                join,
                no_horizontal_scale: raw & 0x0400 != 0,
                no_vertical_scale: raw & 0x0200 != 0,
                pixel_hinting: raw & 0x0100 != 0,
                no_close: raw & 4 != 0,
                miter_limit,
            };
            let paint = if has_fill {
                LinePaint::Fill(self.fill()?)
            } else {
                LinePaint::Color(self.color()?)
            };
            (Some(flags), paint)
        };
        Ok(LineStyle {
            source: self.byte_span(start),
            width,
            flags,
            paint,
        })
    }
}

#[cfg(test)]
mod tests {
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
        fn bits(&mut self, value: u32, n: u8) {
            for shift in (0..n).rev() {
                if self.bit % 8 == 0 {
                    self.bytes.push(0);
                }
                self.bytes[self.bit / 8] |= (((value >> shift) & 1) as u8) << (7 - self.bit % 8);
                self.bit += 1;
            }
        }
        fn signed(&mut self, value: i32, n: u8) {
            self.bits(value as u32, n);
        }
        fn align(&mut self) {
            while self.bit % 8 != 0 {
                self.bits(0, 1);
            }
        }
        fn byte(&mut self, value: u8) {
            assert_eq!(self.bit % 8, 0);
            self.bits(u32::from(value), 8);
        }
        fn u16(&mut self, value: u16) {
            for b in value.to_le_bytes() {
                self.byte(b);
            }
        }
        fn rect(&mut self) {
            self.bits(0, 5);
            self.align();
        }
        fn matrix(&mut self) {
            self.bits(0, 7);
            self.align();
        }
        fn transformed_matrix(&mut self) {
            self.bits(1, 1);
            self.bits(19, 5);
            self.signed(131_072, 19);
            self.signed(-65_536, 19);
            self.bits(1, 1);
            self.bits(16, 5);
            self.signed(16_384, 16);
            self.signed(-8_192, 16);
            self.bits(8, 5);
            self.signed(-60, 8);
            self.signed(80, 8);
            self.align();
        }
        fn header(&mut self, version: u8, flags: u8) {
            self.u16(7);
            self.rect();
            if version == 4 {
                self.rect();
                self.byte(flags);
            }
        }
        fn rgb(&mut self, red: u8, alpha: bool) {
            self.byte(red);
            self.byte(2);
            self.byte(3);
            if alpha {
                self.byte(129);
            }
        }
        fn solid(&mut self, red: u8, alpha: bool) {
            self.byte(0);
            self.rgb(red, alpha);
        }
        fn line(&mut self, dx: i32, dy: i32) {
            self.bits(1, 1);
            self.bits(1, 1);
            self.bits(5, 4);
            self.bits(1, 1);
            self.signed(dx, 7);
            self.signed(dy, 7);
        }
        fn change(&mut self, flags: u8) {
            self.bits(0, 1);
            self.bits(u32::from(flags), 5);
        }
        fn end(&mut self) {
            self.bits(0, 6);
            self.align();
        }
    }
    fn simple() -> Vec<u8> {
        let mut w = Writer::new();
        w.header(3, 0);
        w.byte(2);
        w.solid(10, true);
        w.solid(20, true);
        w.byte(1);
        w.u16(25);
        w.rgb(30, true);
        w.bits(2, 4);
        w.bits(1, 4);
        w.change(1 | 2 | 4 | 8);
        w.bits(7, 5);
        w.signed(-40, 7);
        w.signed(20, 7);
        w.bits(1, 2);
        w.bits(2, 2);
        w.bits(1, 1);
        w.line(30, -10);
        w.bits(1, 1);
        w.bits(0, 1);
        w.bits(4, 4);
        for delta in [-5, 8, 10, -4] {
            w.signed(delta, 6);
        }
        w.end();
        w.bytes
    }
    #[test]
    fn directed_edges_keep_side_assignments_and_signed_curves() {
        let bytes = simple();
        let shape = Shape::parse(32, &bytes, 100).unwrap();
        assert_eq!(shape.edges.len(), 2);
        let first = &shape.edges[0];
        assert_eq!(first.from, Point { x: -40, y: 20 });
        assert_eq!(first.to, Point { x: -10, y: 10 });
        assert_eq!(first.fill_left, Some(StyleRef { set: 0, index: 1 }));
        assert_eq!(first.fill_right, Some(StyleRef { set: 0, index: 2 }));
        assert_eq!(shape.line(first.line.unwrap()).unwrap().width, 25);
        let curve = &shape.edges[1];
        assert_eq!(
            curve.geometry,
            Geometry::Quadratic {
                control: Point { x: -15, y: 18 }
            }
        );
        assert_eq!(curve.to, Point { x: -5, y: 14 });
        assert!(first.source.start >= 800);
        assert_eq!(
            shape.source,
            Span {
                offset: 100,
                len: bytes.len()
            }
        );
        assert!(matches!(shape.records.last(), Some(Record::End(_))));
        assert_eq!(
            shape.fill(first.fill_left.unwrap()).unwrap().fill,
            Fill::Solid(Color {
                red: 10,
                green: 2,
                blue: 3,
                alpha: 129
            })
        );
    }
    #[test]
    fn new_style_bank_resolves_prior_selectors_before_replacement() {
        let mut w = Writer::new();
        w.header(2, 0);
        w.byte(1);
        w.solid(10, false);
        w.byte(0);
        w.byte(0x10);
        w.change(2);
        w.bits(1, 1);
        w.line(1, 0);
        w.change(16 | 4);
        w.bits(1, 1);
        w.align();
        w.byte(1);
        w.solid(20, false);
        w.byte(0);
        w.byte(0x10);
        w.line(1, 0);
        w.change(4);
        w.bits(1, 1);
        w.line(1, 0);
        w.end();
        let s = Shape::parse(22, &w.bytes, 0).unwrap();
        assert_eq!(s.styles.len(), 2);
        assert_eq!(s.edges[1].fill_right, Some(StyleRef { set: 0, index: 1 }));
        assert_eq!(s.edges[2].fill_left, Some(StyleRef { set: 0, index: 1 }));
        assert_eq!(s.edges[2].fill_right, Some(StyleRef { set: 1, index: 1 }));
        assert_eq!(
            s.fill(s.edges[2].fill_right.unwrap()).unwrap().fill,
            Fill::Solid(Color {
                red: 20,
                green: 2,
                blue: 3,
                alpha: 255
            })
        );
    }
    #[test]
    fn bitmap_and_gradient_matrices_and_line_style2_remain_exact() {
        let mut w = Writer::new();
        w.header(4, 7);
        w.byte(4);
        w.byte(0x43);
        w.u16(17);
        w.transformed_matrix();
        for kind in [0x10, 0x12, 0x13] {
            w.byte(kind);
            w.matrix();
            w.byte(0x92);
            w.byte(0);
            w.rgb(10, true);
            w.byte(255);
            w.rgb(20, true);
            if kind == 0x13 {
                w.u16((-128i16) as u16);
            }
        }
        w.byte(1);
        w.u16(30);
        w.byte(0xaf);
        w.byte(5);
        w.u16(512);
        w.solid(90, true);
        w.byte(0);
        w.end();
        let s = Shape::parse(83, &w.bytes, 9).unwrap();
        assert_eq!(s.fill_rule, FillRule::NonZero);
        assert_eq!(s.edge_bounds, Some(s.bounds));
        let Fill::Bitmap {
            character_id,
            matrix,
            repeating,
            smoothed,
        } = &s.styles[0].fills[0].fill
        else {
            panic!()
        };
        assert_eq!(*character_id, 17);
        assert!(!repeating);
        assert!(!smoothed);
        assert_eq!(
            *matrix,
            Matrix {
                scale_x: 131072,
                scale_y: -65536,
                rotate_skew_0: 16384,
                rotate_skew_1: -8192,
                translate_x: -60,
                translate_y: 80
            }
        );
        let Fill::Gradient {
            kind,
            spread,
            interpolation,
            stops,
            focal_point,
            ..
        } = &s.styles[0].fills[3].fill
        else {
            panic!()
        };
        assert_eq!(*kind, GradientKind::FocalRadial);
        assert_eq!((*spread, *interpolation), (2, 1));
        assert_eq!(stops.len(), 2);
        assert_eq!(*focal_point, Some(-128));
        let line = &s.styles[0].lines[0];
        let flags = line.flags.unwrap();
        assert_eq!((flags.start_cap, flags.end_cap, flags.join), (2, 1, 2));
        assert!(
            flags.no_horizontal_scale
                && flags.no_vertical_scale
                && flags.pixel_hinting
                && flags.no_close
        );
        assert_eq!(flags.miter_limit, Some(512));
        assert!(matches!(line.paint, LinePaint::Fill(_)));
        assert!(s.unsupported.is_empty());
    }
    #[test]
    fn horizontal_vertical_edges_and_absolute_moves() {
        let mut w = Writer::new();
        w.header(1, 0);
        w.byte(0);
        w.byte(0);
        w.byte(0);
        w.change(1);
        w.bits(5, 5);
        w.signed(10, 5);
        w.signed(5, 5);
        w.bits(1, 1);
        w.bits(1, 1);
        w.bits(2, 4);
        w.bits(0, 1);
        w.bits(0, 1);
        w.signed(-3, 4);
        w.bits(1, 1);
        w.bits(1, 1);
        w.bits(2, 4);
        w.bits(0, 1);
        w.bits(1, 1);
        w.signed(7, 4);
        w.change(1);
        w.bits(2, 5);
        w.signed(1, 2);
        w.signed(-1, 2);
        w.line(1, 0);
        w.end();
        let s = Shape::parse(2, &w.bytes, 0).unwrap();
        assert_eq!(s.edges[0].to, Point { x: 7, y: 5 });
        assert_eq!(s.edges[1].to, Point { x: 7, y: 12 });
        assert_eq!(s.edges[2].from, Point { x: 1, y: -1 });
    }
    #[test]
    fn reserved_flags_are_preserved_with_source_ranges() {
        let mut w = Writer::new();
        w.header(4, 0x80);
        w.byte(1);
        w.byte(0x10);
        w.matrix();
        w.byte(0xe1);
        w.byte(0);
        w.rgb(1, true);
        w.byte(1);
        w.u16(1);
        w.byte(0xf0);
        w.byte(0xfb);
        w.rgb(1, true);
        w.byte(0);
        w.end();
        let s = Shape::parse(83, &w.bytes, 500).unwrap();
        assert_eq!(s.unsupported.len(), 7);
        assert!(
            s.unsupported
                .iter()
                .all(|f| f.source.start >= 4000 && f.source.len > 0)
        );
        assert_eq!(s.flags.unwrap().raw, 0x80);
        assert_eq!(s.styles[0].lines[0].flags.unwrap().raw, 0xf0fb);
    }
    #[test]
    fn truncation_trailing_bytes_and_budgets_fail_without_partial_shapes() {
        let bytes = simple();
        for len in 0..bytes.len() {
            assert!(Shape::parse(32, &bytes[..len], 0).is_err(), "prefix {len}");
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(Shape::parse(32, &trailing, 0).is_err());
        for limits in [
            Limits {
                max_bytes: 1,
                ..Limits::default()
            },
            Limits {
                max_records: 1,
                ..Limits::default()
            },
            Limits {
                max_styles: 1,
                ..Limits::default()
            },
        ] {
            assert!(Shape::parse_with_limits(32, &bytes, 0, limits).is_err());
        }
        assert!(Shape::parse(32, &bytes, usize::MAX).is_err());
        assert_eq!(
            Shape::parse(46, &bytes, 0).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
    #[test]
    fn missing_style_unknown_fill_and_forbidden_new_styles_are_errors() {
        let mut w = Writer::new();
        w.header(1, 0);
        w.byte(0);
        w.byte(0);
        w.byte(0x10);
        w.change(2);
        w.bits(1, 1);
        w.end();
        assert!(
            Shape::parse(2, &w.bytes, 0)
                .unwrap_err()
                .to_string()
                .contains("style index")
        );
        let mut w = Writer::new();
        w.header(3, 0);
        w.byte(1);
        w.byte(0x7f);
        assert_eq!(
            Shape::parse(32, &w.bytes, 0).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
        let mut w = Writer::new();
        w.header(1, 0);
        w.byte(0);
        w.byte(0);
        w.byte(0);
        w.change(16);
        w.align();
        w.byte(0);
        w.byte(0);
        w.byte(0);
        w.end();
        assert!(
            Shape::parse(2, &w.bytes, 0)
                .unwrap_err()
                .to_string()
                .contains("StateNewStyles")
        );
    }
    #[test]
    fn extended_counts_and_zero_style_selections() {
        let mut w = Writer::new();
        w.header(2, 0);
        w.byte(255);
        w.u16(256);
        for _ in 0..256 {
            w.solid(1, false);
        }
        w.byte(0);
        w.byte(0x90);
        w.change(2);
        w.bits(256, 9);
        w.line(1, 1);
        w.change(2);
        w.bits(0, 9);
        w.line(1, 1);
        w.end();
        let s = Shape::parse(22, &w.bytes, 0).unwrap();
        assert_eq!(s.edges[0].fill_left, Some(StyleRef { set: 0, index: 256 }));
        assert_eq!(s.edges[1].fill_left, None);
        assert!(s.fill(StyleRef { set: 0, index: 0 }).is_none());
    }

    #[test]
    fn legacy_line_count_extension_and_gradient_semantics() {
        // Unlike legacy fill counts, LINESTYLEARRAY permits the extension.
        let mut w = Writer::new();
        w.header(1, 0);
        w.byte(0);
        w.byte(255);
        w.u16(1);
        w.u16(20);
        w.rgb(1, false);
        w.byte(1);
        w.change(8);
        w.bits(1, 1);
        w.line(1, 0);
        w.end();
        let shape = Shape::parse(2, &w.bytes, 0).unwrap();
        assert_eq!(shape.line(shape.edges[0].line.unwrap()).unwrap().width, 20);

        let mut w = Writer::new();
        w.header(3, 0);
        w.byte(1);
        w.byte(0x13);
        w.matrix();
        w.byte(0x49);
        for ratio in (0..9).rev() {
            w.byte(ratio);
            w.rgb(1, true);
        }
        w.u16(300);
        w.byte(0);
        w.byte(0);
        w.end();
        let shape = Shape::parse(32, &w.bytes, 0).unwrap();
        // Modes, stop count, focal-kind, eight ordering violations, focal range.
        assert_eq!(shape.unsupported.len(), 12);
    }
}
