//! Bounded source-preserving HTML text declarations, not a browser or text layout.
//! Runtime defaults, whitespace, paragraph separators and font resolution remain explicit.

use crate::gfx::{
    Span,
    text::{EditFlags, EditLayout, EditText},
};
use std::{fmt, ops::Range};

pub const MAX_BYTES: usize = 1_048_576;
pub const MAX_SCALARS: usize = 262_144;
pub const MAX_TOKENS: usize = 65_536;
pub const MAX_ELEMENTS: usize = 16_384;
pub const MAX_ATTRIBUTES: usize = 65_536;
pub const MAX_DEPTH: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Syntax(&'static str),
    Limit(&'static str),
    InvalidField(&'static str),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlError {
    pub byte_offset: usize,
    pub kind: ErrorKind,
}
impl fmt::Display for HtmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTML byte {}: {:?}", self.byte_offset, self.kind)
    }
}
impl std::error::Error for HtmlError {}
type Result<T> = std::result::Result<T, HtmlError>;
fn error(byte_offset: usize, kind: ErrorKind) -> HtmlError {
    HtmlError { byte_offset, kind }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Source {
    pub bytes: Range<usize>,
    /// UTF-16 indices in the original markup, not a stripped/rendered string.
    pub utf16: Range<usize>,
    pub movie_bytes: Option<Span>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Evidence {
    Documented,
    ObservedExtension,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration<T> {
    pub value: T,
    pub source: Source,
    pub evidence: Evidence,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Decimal {
    pub coefficient: i64,
    pub decimal_places: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontSize {
    AbsolutePixels(Decimal),
    Relative(Decimal),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Alignment {
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CharacterDeclarations {
    pub face: Option<Declaration<String>>,
    pub size: Option<Declaration<FontSize>>,
    /// RGB only. It does not replace the saved field's alpha channel here.
    pub color: Option<Declaration<[u8; 3]>>,
    pub bold: Option<Declaration<bool>>,
    pub italic: Option<Declaration<bool>>,
    pub underline: Option<Declaration<bool>>,
    pub letter_spacing: Option<Declaration<Decimal>>,
    pub kerning: Option<Declaration<bool>>,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParagraphDeclarations {
    pub alignment: Option<Declaration<Alignment>>,
    pub block_indent: Option<Declaration<Decimal>>,
    pub indent: Option<Declaration<Decimal>>,
    pub leading: Option<Declaration<Decimal>>,
    pub left_margin: Option<Declaration<Decimal>>,
    pub right_margin: Option<Declaration<Decimal>>,
    pub tab_stops: Option<Declaration<Vec<u32>>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldDefaults {
    pub field_id: u16,
    pub flags: EditFlags,
    pub font_id: Option<u16>,
    pub font_class: Option<String>,
    pub height_twips: Option<u16>,
    pub color_rgba: Option<[u8; 4]>,
    pub layout_twips: Option<EditLayout>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tag {
    Paragraph,
    Font,
    Bold,
    Italic,
    Underline,
    TextFormat,
    Break,
    Unknown(String),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub name_source: Source,
    pub value_source: Source,
    /// None means an entity in this value was not interpreted; raw bytes remain available.
    pub decoded_value: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Element {
    pub tag: Tag,
    pub parent: Option<usize>,
    pub opening: Source,
    pub closing: Option<Source>,
    pub self_closing: bool,
    pub attributes: Vec<Attribute>,
    pub character: CharacterDeclarations,
    pub paragraph: ParagraphDeclarations,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Open { element: usize },
    Close { element: usize },
    Text { atoms: Range<usize> },
    Entity { atom: usize },
    Break { element: usize },
    OpaqueMarkup,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub source: Source,
    pub kind: TokenKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AtomValue {
    Scalar(char),
    UnresolvedEntity,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Atom {
    pub value: AtomValue,
    pub source: Source,
    /// Entity-decoded content index, excluding structural P/BR boundaries.
    /// Unknown entities make this index unknown for themselves and later atoms.
    pub decoded_utf16: Option<Range<usize>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Run {
    pub atoms: Range<usize>,
    pub scope: Option<usize>,
    pub paragraph_element: Option<usize>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IssueKind {
    UnknownTag,
    UnknownAttribute,
    InvalidAttributeValue,
    ObservedFontExtension,
    UnresolvedEntity,
    OpaqueMarkup,
    NestedParagraph,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Issue {
    pub source: Source,
    pub kind: IssueKind,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnresolvedBehavior {
    RuntimeWhitespaceAndStyleSheet,
    ParagraphAndBreakToTextMapping,
    FontAliasAndTypefaceResolution,
    RelativeSizeAndFieldDefaultApplication,
    AttributeNumericUnitsAndRuntimeRounding,
}

#[derive(Clone, Debug)]
pub struct HtmlText<'a> {
    pub source: &'a str,
    pub field_defaults: Option<FieldDefaults>,
    pub tokens: Vec<Token>,
    pub elements: Vec<Element>,
    pub atoms: Vec<Atom>,
    pub runs: Vec<Run>,
    pub issues: Vec<Issue>,
    pub unresolved_behavior: [UnresolvedBehavior; 5],
}
impl<'a> HtmlText<'a> {
    pub fn parse(source: &'a str) -> Result<Self> {
        Parser::new(source, None)?.parse()
    }
    pub fn parse_initial(field: &'a EditText) -> Result<Self> {
        if !field.flags.html() {
            return Err(error(
                0,
                ErrorKind::InvalidField("field is not marked HTML"),
            ));
        }
        let text = field.initial_text.as_deref().unwrap_or("");
        if let Some(span) = field.initial_text_span {
            if span.len != text.len() + 1 {
                return Err(error(
                    0,
                    ErrorKind::InvalidField("saved HTML span must include exactly one NUL"),
                ));
            }
            span.offset
                .checked_add(span.len)
                .ok_or_else(|| error(0, ErrorKind::InvalidField("saved HTML span overflow")))?;
        } else if field.initial_text.is_some() {
            return Err(error(
                0,
                ErrorKind::InvalidField("saved HTML text lacks its span"),
            ));
        }
        let mut result = Parser::new(text, field.initial_text_span)?.parse()?;
        result.field_defaults = Some(FieldDefaults {
            field_id: field.id,
            flags: field.flags,
            font_id: field.font_id,
            font_class: field.font_class.clone(),
            height_twips: field.font_height,
            color_rgba: field.color,
            layout_twips: field.layout,
        });
        Ok(result)
    }

    /// Inner-to-outer declaration scopes. No CSS, defaults or relative sizes are applied.
    pub fn scopes(&self, run: usize) -> Option<impl Iterator<Item = &Element>> {
        let mut next = self.runs.get(run)?.scope;
        Some(
            std::iter::from_fn(move || {
                let index = next?;
                let element = self.elements.get(index)?;
                next = element.parent;
                Some(element)
            })
            .take(MAX_DEPTH),
        )
    }
}

struct Parser<'a> {
    out: HtmlText<'a>,
    saved: Option<Span>,
    utf16: Vec<usize>,
    position: usize,
    stack: Vec<usize>,
    attribute_count: usize,
    decoded_utf16: Option<usize>,
}
impl<'a> Parser<'a> {
    fn new(source: &'a str, saved: Option<Span>) -> Result<Self> {
        if source.len() > MAX_BYTES {
            return Err(error(0, ErrorKind::Limit("HTML bytes")));
        }
        let mut utf16 = vec![0; source.len() + 1];
        let mut units = 0;
        let mut scalars = 0;
        for (index, ch) in source.char_indices() {
            utf16[index] = units;
            units += ch.len_utf16();
            scalars += 1;
        }
        utf16[source.len()] = units;
        if scalars > MAX_SCALARS {
            return Err(error(0, ErrorKind::Limit("HTML source scalars")));
        }
        Ok(Self {
            out: HtmlText {
                source,
                field_defaults: None,
                tokens: Vec::new(),
                elements: Vec::new(),
                atoms: Vec::new(),
                runs: Vec::new(),
                issues: Vec::new(),
                unresolved_behavior: [
                    UnresolvedBehavior::RuntimeWhitespaceAndStyleSheet,
                    UnresolvedBehavior::ParagraphAndBreakToTextMapping,
                    UnresolvedBehavior::FontAliasAndTypefaceResolution,
                    UnresolvedBehavior::RelativeSizeAndFieldDefaultApplication,
                    UnresolvedBehavior::AttributeNumericUnitsAndRuntimeRounding,
                ],
            },
            saved,
            utf16,
            position: 0,
            stack: Vec::new(),
            attribute_count: 0,
            decoded_utf16: Some(0),
        })
    }
    fn source(&self, start: usize, end: usize) -> Source {
        Source {
            bytes: start..end,
            utf16: self.utf16[start]..self.utf16[end],
            movie_bytes: self.saved.map(|s| Span {
                offset: s.offset + start,
                len: end - start,
            }),
        }
    }
    fn issue(&mut self, start: usize, end: usize, kind: IssueKind) {
        self.out.issues.push(Issue {
            source: self.source(start, end),
            kind,
        });
    }
    fn token(&mut self, start: usize, end: usize, kind: TokenKind) -> Result<()> {
        if self.out.tokens.len() == MAX_TOKENS {
            return Err(error(start, ErrorKind::Limit("HTML tokens")));
        }
        self.out.tokens.push(Token {
            source: self.source(start, end),
            kind,
        });
        Ok(())
    }
    fn atom(&mut self, start: usize, end: usize, value: Option<char>) -> Result<usize> {
        if self.out.atoms.len() == MAX_SCALARS {
            return Err(error(start, ErrorKind::Limit("decoded HTML atoms")));
        }
        let decoded_utf16 = match (self.decoded_utf16, value) {
            (Some(begin), Some(ch)) => {
                let end = begin + ch.len_utf16();
                self.decoded_utf16 = Some(end);
                Some(begin..end)
            }
            _ => {
                self.decoded_utf16 = None;
                None
            }
        };
        let index = self.out.atoms.len();
        self.out.atoms.push(Atom {
            value: value.map_or(AtomValue::UnresolvedEntity, AtomValue::Scalar),
            source: self.source(start, end),
            decoded_utf16,
        });
        Ok(index)
    }
    fn run(&mut self, atoms: Range<usize>) {
        let scope = self.stack.last().copied();
        let paragraph_element = self
            .stack
            .iter()
            .rev()
            .find(|&&i| self.out.elements[i].tag == Tag::Paragraph)
            .copied();
        self.out.runs.push(Run {
            atoms,
            scope,
            paragraph_element,
        });
    }
    fn parse(mut self) -> Result<HtmlText<'a>> {
        while self.position < self.out.source.len() {
            let start = self.position;
            match self.out.source.as_bytes()[start] {
                b'<' => self.tag()?,
                b'&' => {
                    let end = entity_end(self.out.source, start);
                    let value = decode_entity(&self.out.source[start..end]);
                    if value.is_none() {
                        self.issue(start, end, IssueKind::UnresolvedEntity);
                    }
                    let atom = self.atom(start, end, value)?;
                    self.run(atom..atom + 1);
                    self.token(start, end, TokenKind::Entity { atom })?;
                    self.position = end;
                }
                _ => {
                    let end = self.out.source[start..]
                        .find(['<', '&'])
                        .map_or(self.out.source.len(), |n| start + n);
                    let first = self.out.atoms.len();
                    for (relative, ch) in self.out.source[start..end].char_indices() {
                        self.atom(start + relative, start + relative + ch.len_utf8(), Some(ch))?;
                    }
                    let atoms = first..self.out.atoms.len();
                    self.run(atoms.clone());
                    self.token(start, end, TokenKind::Text { atoms })?;
                    self.position = end;
                }
            }
        }
        if !self.stack.is_empty() {
            return Err(error(
                self.position,
                ErrorKind::Syntax("unclosed tag; implicit repair is unsupported"),
            ));
        }
        Ok(self.out)
    }
    fn skip_space(&mut self) {
        while self
            .out
            .source
            .as_bytes()
            .get(self.position)
            .is_some_and(|b| b.is_ascii_whitespace())
        {
            self.position += 1;
        }
    }
    fn name(&mut self) -> Result<(String, Range<usize>)> {
        let start = self.position;
        while self
            .out
            .source
            .as_bytes()
            .get(self.position)
            .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b':'))
        {
            self.position += 1;
        }
        if start == self.position {
            return Err(error(
                start,
                ErrorKind::Syntax("expected ASCII tag/attribute name"),
            ));
        }
        if self.position - start > 128 {
            return Err(error(start, ErrorKind::Limit("tag/attribute name")));
        }
        Ok((
            self.out.source[start..self.position].to_ascii_lowercase(),
            start..self.position,
        ))
    }
    fn tag(&mut self) -> Result<()> {
        let start = self.position;
        if self.out.source[start..].starts_with("<!--") {
            let end = self.out.source[start + 4..]
                .find("-->")
                .map(|n| start + 4 + n + 3)
                .ok_or_else(|| error(start, ErrorKind::Syntax("unclosed comment")))?;
            self.issue(start, end, IssueKind::OpaqueMarkup);
            self.token(start, end, TokenKind::OpaqueMarkup)?;
            self.position = end;
            return Ok(());
        }
        self.position += 1;
        let closing = self.out.source.as_bytes().get(self.position) == Some(&b'/');
        if closing {
            self.position += 1;
        }
        let (name, _) = self.name()?;
        self.skip_space();
        if closing {
            if self.out.source.as_bytes().get(self.position) != Some(&b'>') {
                return Err(error(
                    self.position,
                    ErrorKind::Syntax("closing tag has extra content"),
                ));
            }
            self.position += 1;
            let Some(&index) = self.stack.last() else {
                return Err(error(
                    start,
                    ErrorKind::Syntax("closing tag without opening"),
                ));
            };
            if tag_name(&self.out.elements[index].tag) != name {
                return Err(error(
                    start,
                    ErrorKind::Syntax("mismatched tag; implicit repair is unsupported"),
                ));
            }
            self.stack.pop();
            self.out.elements[index].closing = Some(self.source(start, self.position));
            return self.token(start, self.position, TokenKind::Close { element: index });
        }
        let mut attributes = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        while !matches!(
            self.out.source.as_bytes().get(self.position),
            Some(b'>' | b'/')
        ) {
            if attributes.len() == 64 || self.attribute_count == MAX_ATTRIBUTES {
                return Err(error(self.position, ErrorKind::Limit("HTML attributes")));
            }
            let (name, name_span) = self.name()?;
            if !seen.insert(name.clone()) {
                return Err(error(
                    name_span.start,
                    ErrorKind::Syntax("duplicate attribute; precedence is unsupported"),
                ));
            }
            self.skip_space();
            if self.out.source.as_bytes().get(self.position) != Some(&b'=') {
                return Err(error(
                    self.position,
                    ErrorKind::Syntax("attribute requires a value"),
                ));
            }
            self.position += 1;
            self.skip_space();
            let quote = *self
                .out
                .source
                .as_bytes()
                .get(self.position)
                .ok_or_else(|| {
                    error(self.position, ErrorKind::Syntax("missing attribute quote"))
                })?;
            if !matches!(quote, b'\'' | b'"') {
                return Err(error(
                    self.position,
                    ErrorKind::Syntax("unquoted attribute is unsupported"),
                ));
            }
            self.position += 1;
            let begin = self.position;
            let size = self.out.source[begin..]
                .find(char::from(quote))
                .ok_or_else(|| error(begin, ErrorKind::Syntax("unclosed attribute quote")))?;
            let end = begin + size;
            self.position = end + 1;
            let decoded_value = decode_attribute(&self.out.source[begin..end]);
            if decoded_value.is_none() {
                self.issue(begin, end, IssueKind::UnresolvedEntity);
            }
            attributes.push(Attribute {
                name,
                name_source: self.source(name_span.start, name_span.end),
                value_source: self.source(begin, end),
                decoded_value,
            });
            self.attribute_count += 1;
            if self
                .out
                .source
                .as_bytes()
                .get(self.position)
                .is_some_and(|b| !b.is_ascii_whitespace() && !matches!(b, b'>' | b'/'))
            {
                return Err(error(
                    self.position,
                    ErrorKind::Syntax("attributes must be separated"),
                ));
            }
            self.skip_space();
        }
        let self_closing = self.out.source.as_bytes().get(self.position) == Some(&b'/');
        if self_closing {
            self.position += 1;
        }
        if self.out.source.as_bytes().get(self.position) != Some(&b'>') {
            return Err(error(
                self.position,
                ErrorKind::Syntax("tag lacks terminator"),
            ));
        }
        self.position += 1;
        if self.out.elements.len() == MAX_ELEMENTS {
            return Err(error(start, ErrorKind::Limit("HTML elements")));
        }
        let tag = match name.as_str() {
            "p" => Tag::Paragraph,
            "font" => Tag::Font,
            "b" => Tag::Bold,
            "i" => Tag::Italic,
            "u" => Tag::Underline,
            "textformat" => Tag::TextFormat,
            "br" => Tag::Break,
            _ => Tag::Unknown(name),
        };
        if matches!(tag, Tag::Unknown(_)) {
            self.issue(start, self.position, IssueKind::UnknownTag);
        }
        if tag == Tag::Paragraph
            && self
                .stack
                .iter()
                .any(|&i| self.out.elements[i].tag == Tag::Paragraph)
        {
            self.issue(start, self.position, IssueKind::NestedParagraph);
        }
        let mut element = Element {
            tag,
            parent: self.stack.last().copied(),
            opening: self.source(start, self.position),
            closing: None,
            self_closing,
            attributes,
            character: CharacterDeclarations::default(),
            paragraph: ParagraphDeclarations::default(),
        };
        declarations(&mut element, &mut self.out.issues);
        let index = self.out.elements.len();
        let is_break = element.tag == Tag::Break;
        self.out.elements.push(element);
        self.token(
            start,
            self.position,
            if is_break {
                TokenKind::Break { element: index }
            } else {
                TokenKind::Open { element: index }
            },
        )?;
        if !self_closing && !is_break {
            if self.stack.len() == MAX_DEPTH {
                return Err(error(start, ErrorKind::Limit("HTML nesting")));
            }
            self.stack.push(index);
        }
        Ok(())
    }
}

fn tag_name(tag: &Tag) -> &str {
    match tag {
        Tag::Paragraph => "p",
        Tag::Font => "font",
        Tag::Bold => "b",
        Tag::Italic => "i",
        Tag::Underline => "u",
        Tag::TextFormat => "textformat",
        Tag::Break => "br",
        Tag::Unknown(name) => name,
    }
}
fn entity_end(source: &str, start: usize) -> usize {
    for (offset, ch) in source[start + 1..].char_indices().take(32) {
        if ch == ';' {
            return start + 1 + offset + 1;
        }
        if !(ch.is_ascii_alphanumeric() || ch == '#') {
            return start + 1;
        }
    }
    start + 1
}
fn decode_entity(value: &str) -> Option<char> {
    let inner = value.strip_prefix('&')?.strip_suffix(';')?;
    match inner {
        "lt" => Some('<'),
        "gt" => Some('>'),
        "amp" => Some('&'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => {
            let number = inner.strip_prefix('#')?;
            let code = if let Some(hex) = number
                .strip_prefix('x')
                .or_else(|| number.strip_prefix('X'))
            {
                if hex.is_empty() || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return None;
                }
                u32::from_str_radix(hex, 16).ok()?
            } else {
                if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
                    return None;
                }
                number.parse().ok()?
            };
            char::from_u32(code).filter(|&ch| ch != '\0')
        }
    }
}
fn decode_attribute(source: &str) -> Option<String> {
    let mut output = String::new();
    let mut pos = 0;
    while pos < source.len() {
        let ch = source[pos..].chars().next()?;
        if ch == '&' {
            let end = entity_end(source, pos);
            output.push(decode_entity(&source[pos..end])?);
            pos = end;
        } else {
            output.push(ch);
            pos += ch.len_utf8();
        }
    }
    Some(output)
}
fn decimal(source: &str) -> Option<Decimal> {
    let (negative, value) = if let Some(s) = source.strip_prefix('-') {
        (true, s)
    } else {
        (false, source.strip_prefix('+').unwrap_or(source))
    };
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if whole.is_empty()
        || whole.len() > 9
        || fraction.len() > 6
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let mut coefficient = whole
        .parse::<i64>()
        .ok()?
        .checked_mul(10_i64.pow(fraction.len() as u32))?;
    if !fraction.is_empty() {
        coefficient = coefficient.checked_add(fraction.parse::<i64>().ok()?)?;
    }
    if negative {
        coefficient = -coefficient;
    }
    Some(Decimal {
        coefficient,
        decimal_places: fraction.len() as u8,
    })
}
fn declaration<T>(value: T, attribute: &Attribute, evidence: Evidence) -> Declaration<T> {
    Declaration {
        value,
        source: attribute.value_source.clone(),
        evidence,
    }
}
fn declarations(element: &mut Element, issues: &mut Vec<Issue>) {
    let style = Declaration {
        value: true,
        source: element.opening.clone(),
        evidence: Evidence::Documented,
    };
    match element.tag {
        Tag::Bold => element.character.bold = Some(style),
        Tag::Italic => element.character.italic = Some(style),
        Tag::Underline => element.character.underline = Some(style),
        _ => {}
    }
    for attr in &element.attributes {
        let Some(value) = attr.decoded_value.as_deref() else {
            continue;
        };
        let mut recognized = true;
        let mut valid = true;
        match (&element.tag, attr.name.as_str()) {
            (Tag::Paragraph, "align") => {
                let align = match value.to_ascii_lowercase().as_str() {
                    "left" => Some(Alignment::Left),
                    "right" => Some(Alignment::Right),
                    "center" => Some(Alignment::Center),
                    "justify" => Some(Alignment::Justify),
                    _ => None,
                };
                valid = align.is_some();
                element.paragraph.alignment =
                    align.map(|v| declaration(v, attr, Evidence::Documented));
            }
            (Tag::Font, "face") => {
                element.character.face =
                    Some(declaration(value.to_owned(), attr, Evidence::Documented))
            }
            (Tag::Font, "size") => {
                let number = decimal(value);
                valid = number.is_some();
                element.character.size = number.map(|n| {
                    declaration(
                        if value.starts_with(['+', '-']) {
                            FontSize::Relative(n)
                        } else {
                            FontSize::AbsolutePixels(n)
                        },
                        attr,
                        Evidence::Documented,
                    )
                });
            }
            (Tag::Font, "color") => {
                let rgb = value
                    .strip_prefix('#')
                    .filter(|v| v.len() == 6 && v.bytes().all(|b| b.is_ascii_hexdigit()))
                    .and_then(|v| u32::from_str_radix(v, 16).ok())
                    .map(|v| [(v >> 16) as u8, (v >> 8) as u8, v as u8]);
                valid = rgb.is_some();
                element.character.color = rgb.map(|v| declaration(v, attr, Evidence::Documented));
            }
            (Tag::Font, "letterspacing") => {
                let number = decimal(value);
                valid = number.is_some();
                element.character.letter_spacing =
                    number.map(|n| declaration(n, attr, Evidence::ObservedExtension));
                issues.push(Issue {
                    source: attr.name_source.clone(),
                    kind: IssueKind::ObservedFontExtension,
                });
            }
            (Tag::Font, "kerning") => {
                let flag = match value {
                    "0" => Some(false),
                    "1" => Some(true),
                    _ => None,
                };
                valid = flag.is_some();
                element.character.kerning =
                    flag.map(|v| declaration(v, attr, Evidence::ObservedExtension));
                issues.push(Issue {
                    source: attr.name_source.clone(),
                    kind: IssueKind::ObservedFontExtension,
                });
            }
            (
                Tag::TextFormat,
                "blockindent" | "indent" | "leading" | "leftmargin" | "rightmargin",
            ) => {
                let number = decimal(value);
                valid = number.is_some();
                let parsed = number.map(|v| declaration(v, attr, Evidence::Documented));
                match attr.name.as_str() {
                    "blockindent" => element.paragraph.block_indent = parsed,
                    "indent" => element.paragraph.indent = parsed,
                    "leading" => element.paragraph.leading = parsed,
                    "leftmargin" => element.paragraph.left_margin = parsed,
                    _ => element.paragraph.right_margin = parsed,
                }
            }
            (Tag::TextFormat, "tabstops") => {
                let values = if value.is_empty() {
                    Some(Vec::new())
                } else {
                    value
                        .split(',')
                        .map(|s| {
                            let s = s.trim();
                            if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
                                None
                            } else {
                                s.parse::<u32>().ok()
                            }
                        })
                        .collect::<Option<Vec<_>>>()
                };
                valid = values.as_ref().is_some_and(|v| v.len() <= 4096);
                if valid {
                    element.paragraph.tab_stops =
                        values.map(|v| declaration(v, attr, Evidence::Documented));
                }
            }
            _ => recognized = false,
        }
        if !recognized || !valid {
            issues.push(Issue {
                source: attr.name_source.clone(),
                kind: if recognized {
                    IssueKind::InvalidAttributeValue
                } else {
                    IssueKind::UnknownAttribute
                },
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::Rect;

    fn text(doc: &HtmlText<'_>) -> String {
        doc.atoms
            .iter()
            .filter_map(|a| match a.value {
                AtomValue::Scalar(c) => Some(c),
                AtomValue::UnresolvedEntity => None,
            })
            .collect()
    }
    fn coverage(doc: &HtmlText<'_>) {
        let mut end = 0;
        for token in &doc.tokens {
            assert_eq!(token.source.bytes.start, end);
            end = token.source.bytes.end;
            assert!(doc.source.get(token.source.bytes.clone()).is_some());
        }
        assert_eq!(end, doc.source.len());
    }
    fn field(value: &str) -> EditText {
        EditText {
            id: 7,
            bounds: Rect {
                x_min: 0,
                x_max: 1000,
                y_min: 0,
                y_max: 500,
            },
            flags: EditFlags { bits: 0x8002 },
            font_id: Some(19),
            font_class: None,
            font_class_span: None,
            font_height: Some(280),
            color: Some([255, 255, 255, 127]),
            max_length: None,
            layout: Some(EditLayout {
                alignment: 2,
                left_margin: 0,
                right_margin: 0,
                indent: 0,
                leading: -40,
            }),
            variable_name: String::new(),
            variable_name_span: Span { offset: 99, len: 1 },
            initial_text: Some(value.into()),
            initial_text_span: Some(Span {
                offset: 100,
                len: value.len() + 1,
            }),
            body: Span {
                offset: 50,
                len: value.len() + 51,
            },
        }
    }

    #[test]
    fn utf8_utf16_entities_and_supplementary_characters_keep_exact_origins() {
        let doc = HtmlText::parse("é&#x1F600;∑&lt;&gt;&amp;&quot;&apos;&#8226;").unwrap();
        coverage(&doc);
        assert_eq!(text(&doc), "é😀∑<>&\"'•");
        assert_eq!(doc.atoms[0].source.bytes, 0..2);
        assert_eq!(doc.atoms[0].source.utf16, 0..1);
        assert_eq!(doc.atoms[1].source.bytes, 2..11);
        assert_eq!(doc.atoms[1].source.utf16, 1..10);
        assert_eq!(doc.atoms[1].decoded_utf16, Some(1..3));
        assert_eq!(doc.atoms[2].decoded_utf16, Some(3..4));
        assert!(doc.issues.is_empty());
    }

    #[test]
    fn paragraphs_breaks_and_literal_whitespace_are_structural() {
        let doc = HtmlText::parse("<p align='center'> A </p><p>B<br/>C\r\n\t</p>").unwrap();
        coverage(&doc);
        assert_eq!(text(&doc), " A BC\r\n\t");
        assert_eq!(
            doc.elements
                .iter()
                .filter(|e| e.tag == Tag::Paragraph)
                .count(),
            2
        );
        assert_eq!(
            doc.tokens
                .iter()
                .filter(|t| matches!(t.kind, TokenKind::Break { .. }))
                .count(),
            1
        );
        assert_eq!(doc.runs[0].paragraph_element, Some(0));
        assert_eq!(doc.runs[1].paragraph_element, Some(1));
        assert_eq!(
            doc.elements[0].paragraph.alignment.as_ref().unwrap().value,
            Alignment::Center
        );
        assert!(
            doc.unresolved_behavior
                .contains(&UnresolvedBehavior::ParagraphAndBreakToTextMapping)
        );
    }

    #[test]
    fn nested_scopes_restore_original_declared_fonts_without_substitution() {
        let doc = HtmlText::parse(
            "<FONT FACE='Exact&amp;Font'><b>A<font face='Other'>B</font>C</b></FONT>",
        )
        .unwrap();
        coverage(&doc);
        assert_eq!(
            doc.elements[0].character.face.as_ref().unwrap().value,
            "Exact&Font"
        );
        let faces = |run| {
            doc.scopes(run)
                .unwrap()
                .filter_map(|e| e.character.face.as_ref().map(|d| d.value.as_str()))
                .collect::<Vec<_>>()
        };
        assert_eq!(faces(0), ["Exact&Font"]);
        assert_eq!(faces(1), ["Other", "Exact&Font"]);
        assert_eq!(faces(2), ["Exact&Font"]);
        assert_eq!(doc.elements[1].character.bold.as_ref().unwrap().value, true);
        assert!(doc.field_defaults.is_none());
    }

    #[test]
    fn saved_defaults_and_rgb_do_not_discard_original_alpha_or_units() {
        let field = field(
            "<p><font face='$Exact' size='14' color='#ffffff' letterSpacing='0.000000' kerning='1'>•</font></p>",
        );
        let doc = HtmlText::parse_initial(&field).unwrap();
        coverage(&doc);
        let defaults = doc.field_defaults.as_ref().unwrap();
        assert_eq!(defaults.font_id, Some(19));
        assert_eq!(defaults.color_rgba, Some([255, 255, 255, 127]));
        assert_eq!(defaults.layout_twips.unwrap().leading, -40);
        assert_eq!(
            doc.elements[1].character.color.as_ref().unwrap().value,
            [255, 255, 255]
        );
        assert_eq!(
            doc.elements[1].character.kerning.as_ref().unwrap().evidence,
            Evidence::ObservedExtension
        );
        assert_eq!(
            doc.issues
                .iter()
                .filter(|i| i.kind == IssueKind::ObservedFontExtension)
                .count(),
            2
        );
        let atom = &doc.atoms[0];
        assert_eq!(
            atom.source.movie_bytes.unwrap().offset,
            100 + atom.source.bytes.start
        );
        assert_eq!(atom.source.movie_bytes.unwrap().len, 3);
    }

    #[test]
    fn decimal_and_relative_declarations_are_preserved_without_rounding() {
        let doc=HtmlText::parse("<textformat leftmargin='2.25' indent='-4' leading='+3' tabstops='10, 20,30'><font size='+2.5'><i><u>X</u></i></font></textformat>").unwrap();
        assert_eq!(
            doc.elements[0]
                .paragraph
                .left_margin
                .as_ref()
                .unwrap()
                .value,
            Decimal {
                coefficient: 225,
                decimal_places: 2
            }
        );
        assert_eq!(
            doc.elements[0]
                .paragraph
                .indent
                .as_ref()
                .unwrap()
                .value
                .coefficient,
            -4
        );
        assert_eq!(
            doc.elements[0].paragraph.tab_stops.as_ref().unwrap().value,
            [10, 20, 30]
        );
        assert_eq!(
            doc.elements[1].character.size.as_ref().unwrap().value,
            FontSize::Relative(Decimal {
                coefficient: 25,
                decimal_places: 1
            })
        );
        assert!(doc.elements[2].character.italic.as_ref().unwrap().value);
        assert!(doc.elements[3].character.underline.as_ref().unwrap().value);
    }

    #[test]
    fn unknown_markup_attributes_and_entities_remain_explicit() {
        let doc=HtmlText::parse("<!--opaque--><unknown custom='x'><font color='red' face='A&nbsp;B'>X</font>&nbsp;Y&#xD800;</unknown>").unwrap();
        coverage(&doc);
        for kind in [
            IssueKind::OpaqueMarkup,
            IssueKind::UnknownTag,
            IssueKind::UnknownAttribute,
            IssueKind::InvalidAttributeValue,
            IssueKind::UnresolvedEntity,
        ] {
            assert!(doc.issues.iter().any(|i| i.kind == kind));
        }
        assert!(doc.elements[1].character.face.is_none());
        assert!(doc.elements[1].character.color.is_none());
        assert!(
            doc.atoms
                .iter()
                .any(|a| a.value == AtomValue::UnresolvedEntity)
        );
        assert_eq!(doc.atoms.last().unwrap().decoded_utf16, None);
    }

    #[test]
    fn malformed_syntax_fails_without_implicit_html_repairs() {
        for source in [
            "<p>",
            "</p>",
            "<p><b>x</p></b>",
            "<font face=A>x</font>",
            "<font face='A' FACE='B'>x</font>",
            "<font face='A'color='#ffffff'>x</font>",
            "<font face='A>",
            "<!--unfinished",
        ] {
            assert!(
                matches!(
                    HtmlText::parse(source).unwrap_err().kind,
                    ErrorKind::Syntax(_)
                ),
                "{source}"
            );
        }
    }

    #[test]
    fn limits_and_invalid_saved_spans_are_checked() {
        assert!(matches!(
            HtmlText::parse(&"x".repeat(MAX_BYTES + 1))
                .unwrap_err()
                .kind,
            ErrorKind::Limit(_)
        ));
        let source = format!(
            "{}x{}",
            "<b>".repeat(MAX_DEPTH + 1),
            "</b>".repeat(MAX_DEPTH + 1)
        );
        assert!(matches!(
            HtmlText::parse(&source).unwrap_err().kind,
            ErrorKind::Limit(_)
        ));
        let mut field = field("x");
        field.initial_text_span.as_mut().unwrap().offset = usize::MAX;
        assert!(HtmlText::parse_initial(&field).is_err());
        field.initial_text_span = None;
        assert!(HtmlText::parse_initial(&field).is_err());
        field.flags.bits = 0;
        assert!(HtmlText::parse_initial(&field).is_err());
    }
}
