//! Original bounded SWF/GFX structure reader. It does not execute ActionScript.
//!
//! The observed frontend files are uncompressed `GFX` streams. See
//! `docs/GFX_FORMAT.md` for evidence, inferred extension layouts and limits.

use std::collections::BTreeMap;
use std::io;

#[path = "gfx/shapes.rs"]
pub mod shapes;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_bytes: usize,
    pub max_tags: usize,
    pub max_nesting: usize,
    pub max_string_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_bytes: 64 * 1024 * 1024,
            max_tags: 250_000,
            max_nesting: 64,
            max_string_bytes: 65_536,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub offset: usize,
    pub len: usize,
}
impl Span {
    pub fn bytes(self, source: &[u8]) -> io::Result<&[u8]> {
        let end = self
            .offset
            .checked_add(self.len)
            .ok_or_else(|| invalid("span overflow"))?;
        source
            .get(self.offset..end)
            .ok_or_else(|| invalid("span outside movie"))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Swf,
    Gfx,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x_min: i32,
    pub x_max: i32,
    pub y_min: i32,
    pub y_max: i32,
}
impl Rect {
    pub fn width_pixels(self) -> f64 {
        (i64::from(self.x_max) - i64::from(self.x_min)) as f64 / 20.0
    }
    pub fn height_pixels(self) -> f64 {
        (i64::from(self.y_max) - i64::from(self.y_min)) as f64 / 20.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub format: Format,
    pub version: u8,
    pub declared_bytes: u32,
    pub stage: Rect,
    pub frame_rate_fixed8: u16,
    pub frame_count: u16,
    pub tags_offset: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Matrix {
    /// Signed 16.16 fixed-point values, preserved without floating-point loss.
    pub scale_x: i32,
    pub scale_y: i32,
    pub rotate_skew_0: i32,
    pub rotate_skew_1: i32,
    /// Signed twips (one twentieth of a pixel).
    pub translate_x: i32,
    pub translate_y: i32,
}
impl Default for Matrix {
    fn default() -> Self {
        Self {
            scale_x: 65_536,
            scale_y: 65_536,
            rotate_skew_0: 0,
            rotate_skew_1: 0,
            translate_x: 0,
            translate_y: 0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorTransform {
    pub multiply: [i32; 4],
    pub add: [i32; 4],
}
impl Default for ColorTransform {
    fn default() -> Self {
        Self {
            multiply: [256; 4],
            add: [0; 4],
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    pub kind: u8,
    pub payload: Span,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipAction {
    pub event_flags: u32,
    pub key_code: Option<u8>,
    pub actions: Span,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    pub move_existing: bool,
    pub depth: u16,
    pub character_id: Option<u16>,
    pub class_name: Option<String>,
    pub matrix: Option<Matrix>,
    pub color: Option<ColorTransform>,
    pub ratio: Option<u16>,
    pub name: Option<String>,
    pub clip_depth: Option<u16>,
    pub filters: Option<Vec<Filter>>,
    pub blend_mode: Option<u8>,
    pub bitmap_cache: Option<u8>,
    pub visible: Option<bool>,
    pub opaque_background: Option<[u8; 4]>,
    pub clip_actions: Vec<ClipAction>,
    pub has_clip_actions: bool,
}
impl Placement {
    fn new(depth: u16) -> Self {
        Self {
            move_existing: false,
            depth,
            character_id: None,
            class_name: None,
            matrix: None,
            color: None,
            ratio: None,
            name: None,
            clip_depth: None,
            filters: None,
            blend_mode: None,
            bitmap_cache: None,
            visible: None,
            opaque_background: None,
            clip_actions: Vec::new(),
            has_clip_actions: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub id: u16,
    pub name: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalImage {
    pub id: u16,
    pub format_code: u16,
    pub width: u16,
    pub height: u16,
    pub export_name: String,
    /// Asset identifier only. Never used as a filesystem destination.
    pub file_name: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagData {
    End,
    ShowFrame,
    Background([u8; 3]),
    FileAttributes(u32),
    FrameLabel {
        name: String,
        named_anchor: bool,
    },
    Place(Placement),
    Remove {
        depth: u16,
        character_id: Option<u16>,
    },
    Sprite {
        id: u16,
        frame_count: u16,
    },
    Definition {
        id: u16,
        bounds: Option<Rect>,
    },
    ExportAssets(Vec<Symbol>),
    ImportAssets {
        url: String,
        symbols: Vec<Symbol>,
    },
    SymbolClass(Vec<Symbol>),
    ScalingGrid {
        id: u16,
        splitter: Rect,
    },
    Actions {
        sprite_id: Option<u16>,
        bytes: Span,
    },
    ExternalImage(ExternalImage),
    /// Raw range is retained in Tag::body; content has not been decoded.
    Opaque,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    pub code: u16,
    pub record_offset: usize,
    pub body: Span,
    /// Zero is the root timeline; otherwise the enclosing sprite character ID.
    pub timeline_id: u16,
    pub frame_index: u16,
    pub data: TagData,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timeline {
    pub id: u16,
    pub frame_count: u16,
    pub tags: Vec<usize>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterKind {
    Shape,
    Sprite,
    Text,
    Font,
    Bitmap,
    Button,
    Sound,
    Video,
    Binary,
    Imported,
    Other,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Character {
    pub id: u16,
    pub kind: CharacterKind,
    pub tag_index: usize,
    pub bounds: Option<Rect>,
}
#[derive(Clone, Debug)]
pub struct Movie {
    pub header: Header,
    pub tags: Vec<Tag>,
    pub dictionary: BTreeMap<u16, Character>,
    pub timelines: BTreeMap<u16, Timeline>,
    bytes: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayObject {
    pub character_id: Option<u16>,
    pub class_name: Option<String>,
    pub matrix: Matrix,
    pub color: ColorTransform,
    pub ratio: u16,
    pub name: Option<String>,
    pub clip_depth: Option<u16>,
    pub filters: Vec<Filter>,
    pub blend_mode: u8,
    pub bitmap_cache: u8,
    pub visible: bool,
    pub opaque_background: Option<[u8; 4]>,
    pub has_clip_actions: bool,
}
impl Default for DisplayObject {
    fn default() -> Self {
        Self {
            character_id: None,
            class_name: None,
            matrix: Matrix::default(),
            color: ColorTransform::default(),
            ratio: 0,
            name: None,
            clip_depth: None,
            filters: Vec::new(),
            blend_mode: 1,
            bitmap_cache: 0,
            visible: true,
            opaque_background: None,
            has_clip_actions: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticFrame {
    pub timeline_id: u16,
    pub frame_index: u16,
    pub objects: BTreeMap<u16, DisplayObject>,
    pub background: Option<[u8; 3]>,
    /// Actions are retained, never executed. This snapshot is not runtime state.
    pub ignored_action_tags: Vec<usize>,
}

impl Movie {
    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        Self::parse_with_limits(bytes, Limits::default())
    }
    pub fn parse_with_limits(bytes: &[u8], limits: Limits) -> io::Result<Self> {
        if bytes.len() > limits.max_bytes || limits.max_nesting > 256 {
            return Err(invalid("movie exceeds configured limits"));
        }
        let mut r = Reader::new(bytes, 0, limits.max_string_bytes);
        let format = match r.take(3)? {
            b"FWS" => Format::Swf,
            b"GFX" => Format::Gfx,
            b"CWS" | b"CFX" | b"ZWS" => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "compressed movie stream is not implemented",
                ));
            }
            _ => return Err(invalid("unrecognized movie signature")),
        };
        let version = r.u8()?;
        if version == 0 || version > 19 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "SWF/GFX version outside 1..=19",
            ));
        }
        let declared_bytes = r.u32()?;
        if declared_bytes as usize != bytes.len() {
            return Err(invalid("declared movie size does not match input"));
        }
        let stage = r.rect()?;
        let frame_rate_fixed8 = r.u16()?;
        let frame_count = r.u16()?;
        let header = Header {
            format,
            version,
            declared_bytes,
            stage,
            frame_rate_fixed8,
            frame_count,
            tags_offset: r.position,
        };
        let mut movie = Self {
            header,
            tags: Vec::new(),
            dictionary: BTreeMap::new(),
            timelines: BTreeMap::new(),
            bytes: bytes.to_vec(),
        };
        parse_timeline(
            &mut movie,
            bytes,
            header.tags_offset,
            bytes.len(),
            0,
            frame_count,
            0,
            limits,
        )?;
        Ok(movie)
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn tag_counts(&self) -> BTreeMap<u16, usize> {
        let mut counts = BTreeMap::new();
        for tag in &self.tags {
            *counts.entry(tag.code).or_insert(0) += 1;
        }
        counts
    }
    /// Apply recorded timeline placements/removals through an indexed ShowFrame.
    /// Does not execute actions, instantiate child timelines, or render masks/filters.
    pub fn static_frame(&self, timeline_id: u16, frame_index: u16) -> io::Result<StaticFrame> {
        let timeline = self
            .timelines
            .get(&timeline_id)
            .ok_or_else(|| invalid("unknown timeline"))?;
        if frame_index >= timeline.frame_count {
            return Err(invalid("frame outside timeline"));
        }
        let mut result = StaticFrame {
            timeline_id,
            frame_index,
            objects: BTreeMap::new(),
            background: None,
            ignored_action_tags: Vec::new(),
        };
        for &index in &timeline.tags {
            let tag = &self.tags[index];
            match &tag.data {
                TagData::ShowFrame if tag.frame_index == frame_index => return Ok(result),
                TagData::Background(color) => result.background = Some(*color),
                TagData::Actions { .. } => result.ignored_action_tags.push(index),
                TagData::Place(p) => {
                    if p.move_existing && !result.objects.contains_key(&p.depth) {
                        return Err(invalid("move has no object at depth"));
                    }
                    if !p.move_existing && p.character_id.is_none() && p.class_name.is_none() {
                        return Err(invalid("new placement has no character/class"));
                    }
                    let object = if p.move_existing {
                        result.objects.get_mut(&p.depth).unwrap()
                    } else {
                        result.objects.insert(p.depth, DisplayObject::default());
                        result.objects.get_mut(&p.depth).unwrap()
                    };
                    if let Some(value) = p.character_id {
                        object.character_id = Some(value);
                    }
                    if let Some(value) = &p.class_name {
                        object.class_name = Some(value.clone());
                    }
                    if let Some(value) = p.matrix {
                        object.matrix = value;
                    }
                    if let Some(value) = p.color {
                        object.color = value;
                    }
                    if let Some(value) = p.ratio {
                        object.ratio = value;
                    }
                    if let Some(value) = &p.name {
                        object.name = Some(value.clone());
                    }
                    if let Some(value) = p.clip_depth {
                        object.clip_depth = Some(value);
                    }
                    if let Some(value) = &p.filters {
                        object.filters = value.clone();
                    }
                    if let Some(value) = p.blend_mode {
                        object.blend_mode = value;
                    }
                    if let Some(value) = p.bitmap_cache {
                        object.bitmap_cache = value;
                    }
                    if let Some(value) = p.visible {
                        object.visible = value;
                    }
                    if let Some(value) = p.opaque_background {
                        object.opaque_background = Some(value);
                    }
                    if p.has_clip_actions {
                        object.has_clip_actions = true;
                    }
                }
                TagData::Remove {
                    depth,
                    character_id,
                } => {
                    if let Some(id) = character_id {
                        if result.objects.get(depth).and_then(|v| v.character_id) != Some(*id) {
                            return Err(invalid("remove character/depth mismatch"));
                        }
                    }
                    result.objects.remove(depth);
                }
                _ => {}
            }
        }
        Err(invalid("timeline ended before requested frame"))
    }
}

fn parse_timeline(
    movie: &mut Movie,
    bytes: &[u8],
    start: usize,
    end: usize,
    id: u16,
    declared_frames: u16,
    nesting: usize,
    limits: Limits,
) -> io::Result<()> {
    if nesting > limits.max_nesting {
        return Err(invalid("sprite nesting limit exceeded"));
    }
    let mut r = Reader::new(
        bytes
            .get(start..end)
            .ok_or_else(|| invalid("invalid timeline range"))?,
        start,
        limits.max_string_bytes,
    );
    let mut timeline = Timeline {
        id,
        frame_count: declared_frames,
        tags: Vec::new(),
    };
    let mut frames: u32 = 0;
    let mut ended = false;
    while r.remaining() > 0 {
        if movie.tags.len() >= limits.max_tags {
            return Err(invalid("tag count limit exceeded"));
        }
        let record_offset = r.absolute();
        let header = r.u16()?;
        let code = header >> 6;
        let short_length = usize::from(header & 63);
        let len = if short_length == 63 {
            r.u32()? as usize
        } else {
            short_length
        };
        let body = Span {
            offset: r.absolute(),
            len,
        };
        let payload = r.take(len)?;
        let mut data = Reader::new(payload, body.offset, limits.max_string_bytes);
        let index = movie.tags.len();
        let tag_data = parse_tag(code, &mut data, movie.header.version, movie.header.format)?;
        let sprite = match &tag_data {
            TagData::Sprite { id, frame_count } => Some((*id, *frame_count)),
            _ => None,
        };
        let definition = match &tag_data {
            TagData::Definition { id, bounds } => Some((*id, character_kind(code), *bounds)),
            TagData::Sprite { id, .. } => Some((*id, CharacterKind::Sprite, None)),
            TagData::ExternalImage(image) => Some((image.id, CharacterKind::Bitmap, None)),
            _ => None,
        };
        if let Some((id, kind, bounds)) = definition {
            insert_character(
                movie,
                Character {
                    id,
                    kind,
                    tag_index: index,
                    bounds,
                },
            )?;
        }
        if let TagData::ImportAssets { symbols, .. } = &tag_data {
            for symbol in symbols {
                insert_character(
                    movie,
                    Character {
                        id: symbol.id,
                        kind: CharacterKind::Imported,
                        tag_index: index,
                        bounds: None,
                    },
                )?;
            }
        }
        movie.tags.push(Tag {
            code,
            record_offset,
            body,
            timeline_id: id,
            frame_index: u16::try_from(frames).map_err(|_| invalid("frame count overflow"))?,
            data: tag_data,
        });
        timeline.tags.push(index);
        if let Some((sprite_id, count)) = sprite {
            parse_timeline(
                movie,
                bytes,
                body.offset + 4,
                body.offset + len,
                sprite_id,
                count,
                nesting + 1,
                limits,
            )?;
        }
        if code == 1 {
            frames += 1;
        }
        if code == 0 {
            ended = true;
            r.finish()?;
            break;
        }
    }
    if !ended {
        return Err(invalid("timeline has no End tag"));
    }
    if frames != u32::from(declared_frames) {
        return Err(invalid(format!(
            "timeline {id} ShowFrame count {frames} differs from declared {declared_frames}"
        )));
    }
    if movie.timelines.insert(id, timeline).is_some() {
        return Err(invalid("duplicate timeline ID"));
    }
    Ok(())
}
fn insert_character(movie: &mut Movie, value: Character) -> io::Result<()> {
    if value.id == 0 || movie.dictionary.insert(value.id, value).is_some() {
        return Err(invalid("zero or duplicate character ID"));
    }
    Ok(())
}
fn character_kind(code: u16) -> CharacterKind {
    match code {
        2 | 22 | 32 | 46 | 83 | 84 => CharacterKind::Shape,
        6 | 20 | 21 | 35 | 36 | 90 => CharacterKind::Bitmap,
        7 | 34 => CharacterKind::Button,
        10 | 48 | 75 | 91 => CharacterKind::Font,
        11 | 33 | 37 => CharacterKind::Text,
        14 => CharacterKind::Sound,
        60 => CharacterKind::Video,
        87 => CharacterKind::Binary,
        _ => CharacterKind::Other,
    }
}
fn parse_tag(code: u16, r: &mut Reader<'_>, version: u8, format: Format) -> io::Result<TagData> {
    let value = match code {
        0 => TagData::End,
        1 => TagData::ShowFrame,
        9 => TagData::Background(r.take(3)?.try_into().unwrap()),
        69 => TagData::FileAttributes(r.u32()?),
        4 | 26 | 70 => TagData::Place(parse_placement(code, r, version)?),
        5 => TagData::Remove {
            character_id: Some(r.u16()?),
            depth: r.u16()?,
        },
        28 => TagData::Remove {
            character_id: None,
            depth: r.u16()?,
        },
        39 => {
            let id = r.u16()?;
            let frame_count = r.u16()?;
            r.skip(r.remaining())?;
            TagData::Sprite { id, frame_count }
        }
        43 => {
            let name = r.string()?;
            let named_anchor = if r.remaining() != 0 {
                let flag = r.u8()?;
                if flag != 1 {
                    return Err(invalid("invalid frame label anchor"));
                }
                true
            } else {
                false
            };
            TagData::FrameLabel { name, named_anchor }
        }
        56 => TagData::ExportAssets(r.symbols()?),
        57 | 71 => {
            let url = r.string()?;
            if code == 71 && r.take(2)? != [1, 0] {
                return Err(invalid("invalid ImportAssets2 reserved bytes"));
            }
            TagData::ImportAssets {
                url,
                symbols: r.symbols()?,
            }
        }
        76 => TagData::SymbolClass(r.symbols()?),
        78 => TagData::ScalingGrid {
            id: r.u16()?,
            splitter: r.rect()?,
        },
        12 | 59 => {
            let sprite_id = if code == 59 { Some(r.u16()?) } else { None };
            let bytes = Span {
                offset: r.absolute(),
                len: r.remaining(),
            };
            r.skip(r.remaining())?;
            TagData::Actions { sprite_id, bytes }
        }
        1009 if format == Format::Gfx => {
            let id = u16::try_from(r.u32()?)
                .map_err(|_| invalid("external image ID exceeds observed 16-bit dictionary"))?;
            TagData::ExternalImage(ExternalImage {
                id,
                format_code: r.u16()?,
                width: r.u16()?,
                height: r.u16()?,
                export_name: r.short_string()?,
                file_name: r.short_string()?,
            })
        }
        2 | 6 | 7 | 10 | 11 | 14 | 20 | 21 | 22 | 32 | 33 | 34 | 35 | 36 | 37 | 46 | 48 | 60
        | 75 | 83 | 84 | 87 | 90 | 91 => {
            let id = r.u16()?;
            let bounds = if matches!(code, 2 | 11 | 22 | 32 | 33 | 37 | 46 | 83 | 84) {
                Some(r.rect()?)
            } else {
                None
            };
            r.skip(r.remaining())?;
            TagData::Definition { id, bounds }
        }
        _ => {
            r.skip(r.remaining())?;
            TagData::Opaque
        }
    };
    r.finish()?;
    Ok(value)
}
fn parse_placement(code: u16, r: &mut Reader<'_>, version: u8) -> io::Result<Placement> {
    if code == 4 {
        let id = r.u16()?;
        let mut result = Placement::new(r.u16()?);
        result.character_id = Some(id);
        result.matrix = Some(r.matrix()?);
        if r.remaining() > 0 {
            result.color = Some(r.color_transform(false)?);
        }
        return Ok(result);
    }
    let flags = r.u8()?;
    let extra = if code == 70 { r.u8()? } else { 0 };
    if extra & 0x80 != 0 {
        return Err(invalid("reserved PlaceObject3 flag set"));
    }
    let mut result = Placement::new(r.u16()?);
    result.move_existing = flags & 1 != 0;
    if extra & 8 != 0 || (extra & 16 != 0 && flags & 2 != 0) {
        result.class_name = Some(r.string()?);
    }
    if flags & 2 != 0 {
        result.character_id = Some(r.u16()?);
    }
    if flags & 4 != 0 {
        result.matrix = Some(r.matrix()?);
    }
    if flags & 8 != 0 {
        result.color = Some(r.color_transform(true)?);
    }
    if flags & 16 != 0 {
        result.ratio = Some(r.u16()?);
    }
    if flags & 32 != 0 {
        result.name = Some(r.string()?);
    }
    if flags & 64 != 0 {
        result.clip_depth = Some(r.u16()?);
    }
    if extra & 1 != 0 {
        result.filters = Some(r.filters()?);
    }
    if extra & 2 != 0 {
        result.blend_mode = Some(r.u8()?);
    }
    if extra & 4 != 0 {
        result.bitmap_cache = Some(r.u8()?);
    }
    if extra & 32 != 0 {
        result.visible = Some(match r.u8()? {
            0 => false,
            1 => true,
            _ => return Err(invalid("invalid visibility value")),
        });
    }
    if extra & 64 != 0 {
        result.opaque_background = Some(r.take(4)?.try_into().unwrap());
    }
    if flags & 128 != 0 {
        result.has_clip_actions = true;
        result.clip_actions = r.clip_actions(version)?;
    }
    Ok(result)
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
struct Reader<'a> {
    data: &'a [u8],
    position: usize,
    base: usize,
    max_string: usize,
}
impl<'a> Reader<'a> {
    fn new(data: &'a [u8], base: usize, max_string: usize) -> Self {
        Self {
            data,
            position: 0,
            base,
            max_string,
        }
    }
    fn remaining(&self) -> usize {
        self.data.len() - self.position
    }
    fn absolute(&self) -> usize {
        self.base + self.position
    }
    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(n)
            .ok_or_else(|| invalid("input length overflow"))?;
        let result = self
            .data
            .get(self.position..end)
            .ok_or_else(|| invalid(format!("truncated movie at {}", self.absolute())))?;
        self.position = end;
        Ok(result)
    }
    fn skip(&mut self, n: usize) -> io::Result<()> {
        self.take(n).map(|_| ())
    }
    fn finish(&self) -> io::Result<()> {
        if self.remaining() == 0 {
            Ok(())
        } else {
            Err(invalid(format!(
                "unconsumed tag bytes at {}",
                self.absolute()
            )))
        }
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn string(&mut self) -> io::Result<String> {
        let length = self.data[self.position..]
            .iter()
            .take(self.max_string.saturating_add(1))
            .position(|&b| b == 0)
            .ok_or_else(|| invalid("unterminated or over-budget string"))?;
        let text = std::str::from_utf8(self.take(length)?)
            .map_err(|_| invalid("invalid UTF-8 movie string"))?
            .to_owned();
        self.skip(1)?;
        Ok(text)
    }
    fn short_string(&mut self) -> io::Result<String> {
        let length = usize::from(self.u8()?);
        if length > self.max_string {
            return Err(invalid("string limit exceeded"));
        }
        Ok(std::str::from_utf8(self.take(length)?)
            .map_err(|_| invalid("invalid UTF-8 GFX string"))?
            .to_owned())
    }
    fn symbols(&mut self) -> io::Result<Vec<Symbol>> {
        let count = usize::from(self.u16()?);
        if count > self.remaining() / 3 {
            return Err(invalid("symbol table count exceeds tag"));
        }
        let mut symbols = Vec::with_capacity(count);
        for _ in 0..count {
            symbols.push(Symbol {
                id: self.u16()?,
                name: self.string()?,
            });
        }
        Ok(symbols)
    }
    fn rect(&mut self) -> io::Result<Rect> {
        let mut bits = Bits::new(&self.data[self.position..]);
        let n = bits.unsigned(5)? as u8;
        let result = Rect {
            x_min: bits.signed(n)?,
            x_max: bits.signed(n)?,
            y_min: bits.signed(n)?,
            y_max: bits.signed(n)?,
        };
        self.skip(bits.bytes_used())?;
        if result.x_min > result.x_max || result.y_min > result.y_max {
            return Err(invalid("inverted rectangle"));
        }
        Ok(result)
    }
    fn matrix(&mut self) -> io::Result<Matrix> {
        let mut bits = Bits::new(&self.data[self.position..]);
        let mut value = Matrix::default();
        if bits.unsigned(1)? != 0 {
            let n = bits.unsigned(5)? as u8;
            value.scale_x = bits.signed(n)?;
            value.scale_y = bits.signed(n)?;
        }
        if bits.unsigned(1)? != 0 {
            let n = bits.unsigned(5)? as u8;
            value.rotate_skew_0 = bits.signed(n)?;
            value.rotate_skew_1 = bits.signed(n)?;
        }
        let n = bits.unsigned(5)? as u8;
        value.translate_x = bits.signed(n)?;
        value.translate_y = bits.signed(n)?;
        self.skip(bits.bytes_used())?;
        Ok(value)
    }
    fn color_transform(&mut self, alpha: bool) -> io::Result<ColorTransform> {
        let mut bits = Bits::new(&self.data[self.position..]);
        let mut value = ColorTransform::default();
        let add = bits.unsigned(1)? != 0;
        let multiply = bits.unsigned(1)? != 0;
        let n = bits.unsigned(4)? as u8;
        let channels = if alpha { 4 } else { 3 };
        if multiply {
            for v in value.multiply.iter_mut().take(channels) {
                *v = bits.signed(n)?;
            }
        }
        if add {
            for v in value.add.iter_mut().take(channels) {
                *v = bits.signed(n)?;
            }
        }
        self.skip(bits.bytes_used())?;
        Ok(value)
    }
    fn filters(&mut self) -> io::Result<Vec<Filter>> {
        let count = self.u8()?;
        let mut result = Vec::with_capacity(usize::from(count));
        for _ in 0..count {
            let kind = self.u8()?;
            let offset = self.absolute();
            match kind {
                0 => self.skip(23)?,
                1 => self.skip(9)?,
                2 => self.skip(15)?,
                3 => self.skip(27)?,
                4 | 7 => {
                    let colors = usize::from(self.u8()?);
                    self.skip(colors * 5 + 19)?;
                }
                5 => {
                    let x = usize::from(self.u8()?);
                    let y = usize::from(self.u8()?);
                    self.skip(x * y * 4 + 13)?;
                }
                6 => self.skip(80)?,
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "unknown SWF filter type",
                    ));
                }
            }
            result.push(Filter {
                kind,
                payload: Span {
                    offset,
                    len: self.absolute() - offset,
                },
            });
        }
        Ok(result)
    }
    fn clip_actions(&mut self, version: u8) -> io::Result<Vec<ClipAction>> {
        if self.u16()? != 0 {
            return Err(invalid("nonzero clip-action reserved field"));
        }
        let wide = version >= 6;
        let _all_events = if wide {
            self.u32()?
        } else {
            u32::from(self.u16()?)
        };
        let mut result = Vec::new();
        loop {
            let event_flags = if wide {
                self.u32()?
            } else {
                u32::from(self.u16()?)
            };
            if event_flags == 0 {
                break;
            }
            let len = self.u32()? as usize;
            let offset = self.absolute();
            let bytes = self.take(len)?;
            let key = wide && event_flags & 0x0002_0000 != 0;
            let key_code = if key {
                Some(
                    *bytes
                        .first()
                        .ok_or_else(|| invalid("missing clip-action key code"))?,
                )
            } else {
                None
            };
            result.push(ClipAction {
                event_flags,
                key_code,
                actions: Span {
                    offset: offset + usize::from(key),
                    len: len - usize::from(key),
                },
            });
        }
        Ok(result)
    }
}
struct Bits<'a> {
    data: &'a [u8],
    position: usize,
}
impl<'a> Bits<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }
    fn bytes_used(&self) -> usize {
        self.position.div_ceil(8)
    }
    fn unsigned(&mut self, count: u8) -> io::Result<u32> {
        if count > 32 {
            return Err(invalid("bit field exceeds 32 bits"));
        }
        let mut value = 0_u32;
        for _ in 0..count {
            let byte = *self
                .data
                .get(self.position / 8)
                .ok_or_else(|| invalid("truncated bit field"))?;
            value = (value << 1) | u32::from((byte >> (7 - self.position % 8)) & 1);
            self.position += 1;
        }
        Ok(value)
    }
    fn signed(&mut self, count: u8) -> io::Result<i32> {
        let value = self.unsigned(count)?;
        if count == 0 {
            return Ok(0);
        }
        Ok(((value << (32 - count)) as i32) >> (32 - count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn record(code: u16, body: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        if body.len() < 63 {
            result.extend_from_slice(&((code << 6) | body.len() as u16).to_le_bytes());
        } else {
            result.extend_from_slice(&((code << 6) | 63).to_le_bytes());
            result.extend_from_slice(&(body.len() as u32).to_le_bytes());
        }
        result.extend_from_slice(body);
        result
    }
    fn movie(tags: &[Vec<u8>], frames: u16) -> Vec<u8> {
        let mut bytes = b"GFX\x0f\0\0\0\0".to_vec();
        bytes.extend_from_slice(&[8, 0, 0, 24]);
        bytes.extend_from_slice(&frames.to_le_bytes());
        for tag in tags {
            bytes.extend_from_slice(tag);
        }
        let size = bytes.len() as u32;
        bytes[4..8].copy_from_slice(&size.to_le_bytes());
        bytes
    }
    fn shape(id: u16) -> Vec<u8> {
        let mut bytes = id.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[8, 0]);
        record(2, &bytes)
    }
    fn place(id: u16, depth: u16) -> Vec<u8> {
        let mut b = vec![6];
        b.extend_from_slice(&depth.to_le_bytes());
        b.extend_from_slice(&id.to_le_bytes());
        b.push(0);
        record(26, &b)
    }
    #[test]
    fn timeline_move_remove_and_default_transforms() {
        let tags = [
            shape(7),
            shape(9),
            place(7, 2),
            record(1, &[]),
            record(26, &[3, 2, 0, 9, 0]),
            record(1, &[]),
            record(28, &[2, 0]),
            record(1, &[]),
            record(0, &[]),
        ];
        let m = Movie::parse(&movie(&tags, 3)).unwrap();
        let first = m.static_frame(0, 0).unwrap();
        assert_eq!(first.objects[&2].character_id, Some(7));
        assert_eq!(first.objects[&2].matrix, Matrix::default());
        assert_eq!(first.objects[&2].color, ColorTransform::default());
        assert_eq!(
            m.static_frame(0, 1).unwrap().objects[&2].character_id,
            Some(9)
        );
        assert!(m.static_frame(0, 2).unwrap().objects.is_empty());
        assert!(m.static_frame(0, 3).is_err());
    }
    #[test]
    fn signed_bits_matrix_and_color_are_lossless() {
        let mut bits = Bits::new(&[0b1010_0110]);
        assert_eq!(bits.signed(3).unwrap(), -3);
        assert_eq!(bits.signed(5).unwrap(), 6);
        let mut r = Reader::new(&[0], 0, 10);
        assert_eq!(r.matrix().unwrap(), Matrix::default());
        let mut r = Reader::new(&[0], 0, 10);
        assert_eq!(r.color_transform(true).unwrap(), ColorTransform::default());
        assert!(Bits::new(&[]).signed(1).is_err());
    }
    #[test]
    fn sprite_timeline_has_independent_indices_and_bounds() {
        let mut body = vec![3, 0, 1, 0];
        body.extend(place(7, 1));
        body.extend(record(1, &[]));
        body.extend(record(0, &[]));
        let bytes = movie(
            &[
                shape(7),
                record(39, &body),
                place(3, 5),
                record(1, &[]),
                record(0, &[]),
            ],
            1,
        );
        let parsed = Movie::parse(&bytes).unwrap();
        assert_eq!(parsed.timelines.len(), 2);
        assert_eq!(
            parsed.static_frame(0, 0).unwrap().objects[&5].character_id,
            Some(3)
        );
        assert_eq!(
            parsed.static_frame(3, 0).unwrap().objects[&1].character_id,
            Some(7)
        );
        assert!(
            Movie::parse_with_limits(
                &bytes,
                Limits {
                    max_nesting: 0,
                    ..Limits::default()
                }
            )
            .is_err()
        );
    }
    #[test]
    fn external_images_and_imports_keep_identifiers_without_file_access() {
        let mut image = 10_u32.to_le_bytes().to_vec();
        image.extend_from_slice(&[13, 0, 64, 0, 32, 0, 3]);
        image.extend(b"art");
        image.push(7);
        image.extend(b"art.tga");
        let bytes = movie(
            &[
                record(1009, &image),
                record(71, b"other.swf\0\x01\x00\x01\x00\x0b\x00entry\0"),
                record(0, &[]),
            ],
            0,
        );
        let parsed = Movie::parse(&bytes).unwrap();
        assert_eq!(parsed.dictionary[&10].kind, CharacterKind::Bitmap);
        assert_eq!(parsed.dictionary[&11].kind, CharacterKind::Imported);
        match &parsed.tags[0].data {
            TagData::ExternalImage(value) => {
                assert_eq!(value.width, 64);
                assert_eq!(value.file_name, "art.tga");
            }
            _ => panic!(),
        }
        let duplicate = movie(&[record(1009, &image), shape(10), record(0, &[])], 0);
        assert!(Movie::parse(&duplicate).is_err());
    }
    #[test]
    fn clip_actions_are_bounded_and_never_executed() {
        let mut p = vec![0x86, 1, 0, 7, 0, 0, 0, 0];
        p.extend(4_u32.to_le_bytes());
        p.extend(4_u32.to_le_bytes());
        p.extend(2_u32.to_le_bytes());
        p.extend([7, 0]);
        p.extend(0_u32.to_le_bytes());
        let bytes = movie(
            &[
                shape(7),
                record(26, &p),
                record(12, &[7, 0]),
                record(1, &[]),
                record(0, &[]),
            ],
            1,
        );
        let parsed = Movie::parse(&bytes).unwrap();
        let snapshot = parsed.static_frame(0, 0).unwrap();
        assert!(snapshot.objects[&1].has_clip_actions);
        assert_eq!(snapshot.ignored_action_tags.len(), 1);
        match &parsed.tags[1].data {
            TagData::Place(value) => {
                assert_eq!(value.clip_actions.len(), 1);
                assert_eq!(
                    value.clip_actions[0].actions.bytes(parsed.bytes()).unwrap(),
                    [7, 0]
                );
            }
            _ => panic!(),
        }
        p[16..20].copy_from_slice(&1000_u32.to_le_bytes());
        assert!(
            Movie::parse(&movie(
                &[shape(7), record(26, &p), record(1, &[]), record(0, &[])],
                1
            ))
            .is_err()
        );
    }
    #[test]
    fn place3_filters_preserve_payloads_and_reject_unknown_types() {
        let mut p = vec![6, 1, 1, 0, 7, 0, 0, 1, 1];
        p.extend([0; 9]);
        let bytes = movie(
            &[shape(7), record(70, &p), record(1, &[]), record(0, &[])],
            1,
        );
        let parsed = Movie::parse(&bytes).unwrap();
        let frame = parsed.static_frame(0, 0).unwrap();
        assert_eq!(frame.objects[&1].filters.len(), 1);
        assert_eq!(frame.objects[&1].filters[0].payload.len, 9);
        p[8] = 255;
        assert!(
            Movie::parse(&movie(
                &[shape(7), record(70, &p), record(1, &[]), record(0, &[])],
                1
            ))
            .is_err()
        );
    }
    #[test]
    fn corrupt_headers_lengths_frames_and_budget_fail() {
        let bytes = movie(&[record(1, &[]), record(0, &[])], 1);
        for end in 0..bytes.len() {
            assert!(Movie::parse(&bytes[..end]).is_err());
        }
        let mut corrupt = bytes.clone();
        corrupt[..3].copy_from_slice(b"CFX");
        assert_eq!(
            Movie::parse(&corrupt).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
        let mut corrupt = bytes.clone();
        corrupt[12..14].copy_from_slice(&2_u16.to_le_bytes());
        assert!(Movie::parse(&corrupt).is_err());
        let mut corrupt = bytes.clone();
        corrupt[14..16].copy_from_slice(&((1_u16 << 6) | 63).to_le_bytes());
        assert!(Movie::parse(&corrupt).is_err());
        assert!(
            Movie::parse_with_limits(
                &bytes,
                Limits {
                    max_tags: 1,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        assert!(
            Movie::parse_with_limits(
                &bytes,
                Limits {
                    max_bytes: 8,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        assert!(Movie::parse(&movie(&[record(0, &[]), record(0, &[])], 0)).is_err());
    }
    #[test]
    fn labels_strings_opaque_tags_and_missing_depth_are_explicit() {
        let bytes = movie(
            &[
                record(43, b"label\0\x01"),
                record(1000, &[1, 2, 3]),
                record(1, &[]),
                record(0, &[]),
            ],
            1,
        );
        let parsed = Movie::parse(&bytes).unwrap();
        assert!(matches!(parsed.tags[1].data, TagData::Opaque));
        assert_eq!(
            parsed.tags[1].body.bytes(parsed.bytes()).unwrap(),
            [1, 2, 3]
        );
        assert!(
            Movie::parse_with_limits(
                &bytes,
                Limits {
                    max_string_bytes: 2,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        let parsed = Movie::parse(&movie(
            &[record(26, &[1, 1, 0]), record(1, &[]), record(0, &[])],
            1,
        ))
        .unwrap();
        assert!(parsed.static_frame(0, 0).is_err());
    }
}
