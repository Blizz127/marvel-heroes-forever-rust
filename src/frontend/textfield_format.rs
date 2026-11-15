//! Conditional fresh HTML format derivation. No font discovery or native state
//! observation. See docs/TEXTFIELD_FORMAT_MAPPING.md for the exact route.

use super::{
    runtime_text::FormatBinding,
    scene_content::{
        MAX_RUNTIME_PROVENANCE_BYTES, RuntimeFont, RuntimeTextEffects, RuntimeTextStyle,
    },
    textfield_state::{
        FORCE_REBUILD, HTML_ENABLED, HTML_PARSE_MODE, MAX_TEXT_BYTES, Profile, SidePath, Snapshot,
    },
};
use crate::gfx::{
    CharacterKind, Rect,
    glyphs::FontIdentity,
    scene::DefinitionRef,
    text::{EditFlags, EditLayout, EditText},
};
use std::{fmt, num::NonZeroU64};

/// Caller assertions, never conclusions from saved flags or format handle IDs.
/// No Default: a new or changed runtime association needs explicit admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConditionalProfile {
    pub side_paths: Profile,
    /// The optional DocView+0x30 content-provider route must be absent.
    pub content_provider: SidePath,
    pub definition_is_selected_source: bool,
    pub fresh_html_replacement_completed: bool,
    pub no_later_format_mutation: bool,
    pub successful_ordinary_allocation: bool,
}

/// The allocator may discard the definition's embedded-font wrapper. These
/// alternatives are supplied runtime facts, not detected from saved FontID.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontBindingState {
    Unknown,
    RetainedEmbeddedBinding,
    SelectedAfterWrapperNormalization,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FontSelection {
    /// Source local reference and the destination the existing renderer checks.
    pub requested: RuntimeFont,
    /// Independently supplied final font identity, not inferred from the local ID.
    pub selected: FontIdentity,
    pub final_native_selection_asserted: bool,
    pub binding_state: FontBindingState,
    /// Effective native format style. This helper cannot synthesize styles.
    pub bold: bool,
    pub italic: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectiveLayout {
    /// Explicit wire-shaped layout inputs, not a cast of native target/view bits.
    pub flags: EditFlags,
    /// The current renderer retains the saved bounds: changed bounds reject here.
    pub bounds: Rect,
}

#[derive(Clone, Copy, Debug)]
pub struct FreshHtmlInput<'a> {
    pub definition: DefinitionRef,
    pub domain: NonZeroU64,
    pub snapshot: &'a Snapshot,
    pub conditional: ConditionalProfile,
    pub font: FontSelection,
    pub layout: EffectiveLayout,
    pub provenance: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    UnsupportedCondition(&'static str),
    SidePath {
        feature: &'static str,
        state: SidePath,
    },
    Definition(&'static str),
    Document(&'static str),
    Font(&'static str),
    Layout(&'static str),
    Quantization {
        field: &'static str,
        source_twips: i32,
        native_twips: i32,
    },
    Limit(&'static str),
    Provenance,
    Allocation,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "conditional text format: {self:?}")
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

/// Derive only the recovered scalar/effect inputs and bind them to the supplied
/// document run pair. Does not mutate state, intern formats, choose a font,
/// install a binding, execute an assignment, or validate a live native object.
///
/// DefinitionRef/parsed-field identity and caller assertions are checked
/// structurally. Scene/VM/domain lifetime and actual native selection remain the
/// caller's responsibility. The downstream scene-content stage still verifies
/// the declared font route, decoded font, glyph availability and layout support.
pub fn derive_fresh_html(field: &EditText, input: FreshHtmlInput<'_>) -> Result<FormatBinding> {
    let p = input.conditional;
    for (ok, condition) in [
        (p.definition_is_selected_source, "selected definition"),
        (
            p.fresh_html_replacement_completed,
            "fresh HTML replacement completed",
        ),
        (p.no_later_format_mutation, "no later format mutation"),
        (
            p.successful_ordinary_allocation,
            "successful ordinary allocation",
        ),
    ] {
        if !ok {
            return Err(Error::UnsupportedCondition(condition));
        }
    }
    for (feature, state) in [
        ("watches", p.side_paths.watches),
        ("translation service", p.side_paths.translation_service),
        ("stylesheet", p.side_paths.stylesheet),
        ("editor", p.side_paths.editor),
        ("variable binding", p.side_paths.variable_binding),
        ("image callbacks", p.side_paths.image_callbacks),
        ("content provider", p.content_provider),
    ] {
        if state != SidePath::Absent {
            return Err(Error::SidePath { feature, state });
        }
    }
    if input.definition.kind != CharacterKind::Text
        || input.definition.character_id != field.id
        || field.id == 0
        || input.definition.bounds != Some(field.bounds)
        || field.body.len == 0
        || field.body.offset.checked_add(field.body.len).is_none()
    {
        return Err(Error::Definition("parsed field and declared source differ"));
    }
    if field.bounds.x_min >= field.bounds.x_max || field.bounds.y_min >= field.bounds.y_max {
        return Err(Error::Definition("nonpositive saved bounds"));
    }
    // This increment excludes source HTML/font-class/device paths and requires
    // all three authored inputs. No missing values select reset defaults.
    let source_flags = field.flags;
    if !source_flags.has_font() || !source_flags.has_text_color() || !source_flags.has_layout() {
        return Err(Error::Definition(
            "explicit font, color and layout required",
        ));
    }
    if source_flags.html()
        || source_flags.has_font_class()
        || field.font_class.is_some()
        || field.font_class_span.is_some()
        || !source_flags.use_outlines()
        || source_flags.was_static()
        || !field.variable_name.is_empty()
    {
        return Err(Error::Definition("unsupported authored format path"));
    }
    let font_id = field.font_id.ok_or(Error::Definition("missing FontID"))?;
    let height_twips = field.font_height.ok_or(Error::Definition("missing size"))?;
    let color_rgba = field.color.ok_or(Error::Definition("missing color"))?;
    let saved_layout = field.layout.ok_or(Error::Definition("missing layout"))?;

    let snapshot = input.snapshot;
    if snapshot.raw_target_flags & (HTML_ENABLED | HTML_PARSE_MODE)
        != HTML_ENABLED | HTML_PARSE_MODE
    {
        return Err(Error::Document("HTML enable and parse mode required"));
    }
    if snapshot.raw_target_flags & ((1 << 4) | FORCE_REBUILD) != 0
        || snapshot.raw_view_flags_149 & (1 << 2) != 0
    {
        return Err(Error::Document(
            "unsupported parser flags or pending forced rebuild",
        ));
    }
    // The recovered DocView wordWrap getter reads this exact bit. An explicit
    // wire-shaped "no wrap" input cannot override contradictory native state.
    if snapshot.raw_view_flags_149 & (1 << 3) != 0 {
        return Err(Error::Document(
            "native word wrapping is outside this profile",
        ));
    }
    let text = snapshot.document.text.as_bytes();
    if text.len() > MAX_TEXT_BYTES || snapshot.last_input.len() > MAX_TEXT_BYTES {
        return Err(Error::Limit("text bytes"));
    }
    if text.is_empty()
        || text != snapshot.last_input.as_bytes()
        || text
            .iter()
            .any(|b| !(0x20..=0x7e).contains(b) || matches!(b, b'<' | b'>' | b'&'))
    {
        return Err(Error::Document(
            "nonempty unchanged printable ASCII without markup required",
        ));
    }
    let profile = snapshot
        .document
        .run_format
        .ok_or(Error::Document("missing run format"))?;
    if profile != snapshot.definition_html.run_format {
        return Err(Error::Document(
            "run is not the supplied definition HTML profile",
        ));
    }
    // Current defaults can be a different canonical pair. They never replace
    // the actual run pair in this mapping.

    let font = input.font;
    if !font.final_native_selection_asserted || font.binding_state == FontBindingState::Unknown {
        return Err(Error::Font(
            "final native selection and wrapper state must be supplied",
        ));
    }
    if font.requested.movie_id != input.definition.movie_id
        || font.requested.character_id != font_id
        || font_id == 0
        || font.selected.character_id == 0
        || font.requested.expected_resolved != font.selected
    {
        return Err(Error::Font(
            "declared source route or final identity differs",
        ));
    }
    if font.bold || font.italic {
        return Err(Error::Font(
            "effective bold or italic is outside this renderer profile",
        ));
    }
    if input.layout.bounds != field.bounds {
        return Err(Error::Layout("changed bounds have no renderer override"));
    }
    let flags = input.layout.flags;
    if !flags.has_font() || !flags.has_text_color() || !flags.has_layout() {
        return Err(Error::Layout(
            "explicit effective font, color and layout flags required",
        ));
    }
    if flags.has_font_class()
        || !flags.use_outlines()
        || flags.auto_size()
        || flags.password()
        || flags.word_wrap()
        || flags.was_static()
    {
        return Err(Error::Layout("unsupported effective layout flags"));
    }
    let layout_twips = paragraph(saved_layout)?;
    let prefix = "Conditional native fresh HTML scalar mapping; font/flags/bounds supplied: ";
    let provenance_len = prefix
        .len()
        .checked_add(input.provenance.len())
        .ok_or(Error::Limit("provenance bytes"))?;
    if provenance_len > MAX_RUNTIME_PROVENANCE_BYTES {
        return Err(Error::Limit("provenance bytes"));
    }
    if input.provenance.trim().is_empty() || input.provenance.contains('\0') {
        return Err(Error::Provenance);
    }
    // All unsupported-path checks precede construction/allocation of a binding.
    let mut provenance = String::new();
    provenance
        .try_reserve_exact(provenance_len)
        .map_err(|_| Error::Allocation)?;
    provenance.push_str(prefix);
    provenance.push_str(input.provenance);
    Ok(FormatBinding {
        domain: input.domain,
        profile,
        style: RuntimeTextStyle {
            flags,
            font: font.requested,
            height_twips,
            color_rgba,
            layout_twips,
            effects: RuntimeTextEffects {
                kerning: Some(false),
                letter_spacing_twips: Some(0),
                underline: Some(false),
                block_indent_twips: Some(0),
            },
        },
        provenance,
    })
}

fn paragraph(saved: EditLayout) -> Result<EditLayout> {
    if saved.alignment > 2 {
        return Err(Error::Layout("justify or unknown alignment"));
    }
    // Native reader sign-extends the source indent word; EditLayout stores its
    // wire u16. The current renderer cannot express the negative native case.
    let indent = i32::from(saved.indent as i16);
    if indent < 0 {
        return Err(Error::Layout("negative native indent is not representable"));
    }
    Ok(EditLayout {
        alignment: saved.alignment,
        left_margin: exact_twips(i32::from(saved.left_margin), "left margin")? as u16,
        right_margin: exact_twips(i32::from(saved.right_margin), "right margin")? as u16,
        indent: exact_twips(indent, "indent")? as u16,
        leading: exact_twips(i32::from(saved.leading), "leading")? as i16,
    })
}

fn exact_twips(source_twips: i32, field: &'static str) -> Result<i32> {
    // All callers supply a bounded u16 or i16, so f32 conversion is exact and
    // the product/truncated result lies within the native word's range. Rust's
    // out-of-range float-cast policy is never reached for these inputs.
    let pixels = ((source_twips as f32) * f32::from_bits(0x3d4c_cccd)).trunc() as i32;
    let native_twips = pixels * 20;
    if native_twips != source_twips {
        return Err(Error::Quantization {
            field,
            source_twips,
            native_twips,
        });
    }
    Ok(native_twips)
}

#[cfg(test)]
mod tests {
    use super::super::textfield_state::{
        DefinitionHtmlProfile, Document, FormatProfile, Invalidation, ParagraphFormat, TextFormat,
    };
    use super::*;
    use crate::gfx::Span;

    fn nz(n: u64) -> NonZeroU64 {
        NonZeroU64::new(n).unwrap()
    }
    fn format(n: u64) -> FormatProfile {
        FormatProfile {
            text: TextFormat(nz(n)),
            paragraph: ParagraphFormat(nz(n + 1)),
        }
    }
    fn field() -> EditText {
        EditText {
            id: 7,
            bounds: Rect {
                x_min: -40,
                x_max: 1960,
                y_min: -40,
                y_max: 560,
            },
            flags: EditFlags { bits: 0xad31 },
            font_id: Some(9),
            font_class: None,
            font_class_span: None,
            font_height: Some(320),
            color: Some([12, 34, 56, 255]),
            max_length: None,
            layout: Some(EditLayout {
                alignment: 2,
                left_margin: 0,
                right_margin: 0,
                indent: 0,
                leading: 40,
            }),
            variable_name: String::new(),
            variable_name_span: Span { offset: 50, len: 1 },
            initial_text: Some("placeholder".into()),
            initial_text_span: Some(Span {
                offset: 51,
                len: 12,
            }),
            body: Span {
                offset: 24,
                len: 39,
            },
        }
    }
    fn snapshot() -> Snapshot {
        Snapshot {
            raw_target_flags: HTML_ENABLED | HTML_PARSE_MODE,
            raw_view_flags_149: 0,
            last_input: "TEST LABEL".into(),
            document: Document {
                text: "TEST LABEL".into(),
                run_format: Some(format(10)),
            },
            current_defaults: format(20),
            definition_html: DefinitionHtmlProfile {
                run_format: format(10),
                defaults_after_nonempty: format(20),
            },
            pending_invalidation: Invalidation {
                view_mask: 0x106,
                render_dirty: true,
            },
        }
    }
    fn input<'a>(field: &EditText, snapshot: &'a Snapshot) -> FreshHtmlInput<'a> {
        FreshHtmlInput {
            definition: DefinitionRef {
                movie_id: 3,
                character_id: field.id,
                tag_index: 5,
                kind: CharacterKind::Text,
                bounds: Some(field.bounds),
            },
            domain: nz(100),
            snapshot,
            conditional: ConditionalProfile {
                side_paths: Profile {
                    watches: SidePath::Absent,
                    translation_service: SidePath::Absent,
                    stylesheet: SidePath::Absent,
                    editor: SidePath::Absent,
                    variable_binding: SidePath::Absent,
                    image_callbacks: SidePath::Absent,
                },
                content_provider: SidePath::Absent,
                definition_is_selected_source: true,
                fresh_html_replacement_completed: true,
                no_later_format_mutation: true,
                successful_ordinary_allocation: true,
            },
            font: FontSelection {
                requested: RuntimeFont {
                    movie_id: 3,
                    character_id: 9,
                    expected_resolved: FontIdentity {
                        movie_id: 4,
                        character_id: 2,
                    },
                },
                selected: FontIdentity {
                    movie_id: 4,
                    character_id: 2,
                },
                final_native_selection_asserted: true,
                binding_state: FontBindingState::RetainedEmbeddedBinding,
                bold: false,
                italic: false,
            },
            layout: EffectiveLayout {
                flags: field.flags,
                bounds: field.bounds,
            },
            provenance: "controlled test profile",
        }
    }

    #[test]
    fn derives_run_scalars_without_replacing_defaults_or_supplied_flags() {
        let f = field();
        let s = snapshot();
        let before = (f.clone(), s.clone());
        let b = derive_fresh_html(&f, input(&f, &s)).unwrap();
        assert_eq!(b.profile, format(10));
        assert_ne!(b.profile, s.current_defaults);
        assert_eq!(b.style.height_twips, 320);
        assert_eq!(b.style.color_rgba, [12, 34, 56, 255]);
        assert_eq!(b.style.layout_twips, f.layout.unwrap());
        assert_eq!(
            b.style.effects,
            RuntimeTextEffects {
                kerning: Some(false),
                underline: Some(false),
                letter_spacing_twips: Some(0),
                block_indent_twips: Some(0)
            }
        );
        assert!(!b.style.flags.html()); // Native HTML mode is not cast to wire flags.
        let mut explicit = input(&f, &s);
        explicit.layout.flags.bits |= 2;
        assert!(derive_fresh_html(&f, explicit).unwrap().style.flags.html());
        assert_eq!((f, s), before);
    }

    #[test]
    fn every_runtime_condition_and_side_path_is_explicit() {
        let f = field();
        let s = snapshot();
        for i in 0..4 {
            let mut r = input(&f, &s);
            match i {
                0 => r.conditional.definition_is_selected_source = false,
                1 => r.conditional.fresh_html_replacement_completed = false,
                2 => r.conditional.no_later_format_mutation = false,
                _ => r.conditional.successful_ordinary_allocation = false,
            }
            assert!(matches!(
                derive_fresh_html(&f, r),
                Err(Error::UnsupportedCondition(_))
            ));
        }
        for state in [SidePath::Unknown, SidePath::Present] {
            for i in 0..7 {
                let mut r = input(&f, &s);
                match i {
                    0 => r.conditional.side_paths.watches = state,
                    1 => r.conditional.side_paths.translation_service = state,
                    2 => r.conditional.side_paths.stylesheet = state,
                    3 => r.conditional.side_paths.editor = state,
                    4 => r.conditional.side_paths.variable_binding = state,
                    5 => r.conditional.side_paths.image_callbacks = state,
                    _ => r.conditional.content_provider = state,
                }
                assert!(
                    matches!(derive_fresh_html(&f, r), Err(Error::SidePath { state: got, .. }) if got == state)
                );
            }
        }
    }

    #[test]
    fn rejects_stale_empty_plain_or_unproved_documents() {
        let f = field();
        for bad in ["", "a<b", "a>b", "a&b", "a\nb", "a\0b", "é"] {
            let mut s = snapshot();
            s.document.text = bad.into();
            s.last_input = bad.into();
            assert!(matches!(
                derive_fresh_html(&f, input(&f, &s)),
                Err(Error::Document(_))
            ));
        }
        for i in 0..9 {
            let mut s = snapshot();
            match i {
                0 => s.document.text.push('!'),
                1 => s.document.run_format = None,
                2 => s.document.run_format = Some(format(30)),
                3 => s.raw_target_flags &= !HTML_ENABLED,
                4 => s.raw_target_flags &= !HTML_PARSE_MODE,
                5 => s.raw_target_flags |= FORCE_REBUILD,
                6 => s.raw_target_flags |= 1 << 4,
                7 => s.raw_view_flags_149 |= 1 << 2,
                _ => s.raw_view_flags_149 |= 1 << 3,
            }
            assert!(matches!(
                derive_fresh_html(&f, input(&f, &s)),
                Err(Error::Document(_))
            ));
        }
    }

    #[test]
    fn font_route_is_not_a_native_selection_witness() {
        let f = field();
        let s = snapshot();
        for i in 0..8 {
            let mut r = input(&f, &s);
            match i {
                0 => r.font.final_native_selection_asserted = false,
                1 => r.font.binding_state = FontBindingState::Unknown,
                2 => r.font.requested.movie_id = 99,
                3 => r.font.requested.character_id = 99,
                4 => r.font.selected.movie_id = 99,
                5 => r.font.selected.character_id = 0,
                6 => r.font.bold = true,
                _ => r.font.italic = true,
            }
            assert!(matches!(derive_fresh_html(&f, r), Err(Error::Font(_))));
        }
        let mut r = input(&f, &s);
        r.font.binding_state = FontBindingState::SelectedAfterWrapperNormalization;
        assert_eq!(
            derive_fresh_html(&f, r).unwrap().style.font,
            r.font.requested
        );
    }

    #[test]
    fn native_quantization_and_signed_indent_are_not_silent_rounding() {
        for (source, expected) in [
            (0, 0),
            (20, 20),
            (40, 40),
            (65520, 65520),
            (-40, -40),
            (-32760, -32760),
        ] {
            assert_eq!(exact_twips(source, "test"), Ok(expected));
        }
        for (source, native) in [
            (19, 0),
            (21, 20),
            (65535, 65520),
            (-19, 0),
            (-21, -20),
            (-32768, -32760),
        ] {
            assert_eq!(
                exact_twips(source, "test"),
                Err(Error::Quantization {
                    field: "test",
                    source_twips: source,
                    native_twips: native
                })
            );
        }
        let mut layout = field().layout.unwrap();
        layout.indent = 65516;
        assert!(matches!(paragraph(layout), Err(Error::Layout(_))));
        layout.indent = 32760;
        layout.leading = -40;
        assert_eq!(paragraph(layout), Ok(layout));
        layout.alignment = 3;
        assert!(matches!(paragraph(layout), Err(Error::Layout(_))));
    }

    #[test]
    fn rejects_definition_mismatch_and_unrenderable_layout() {
        let f = field();
        let s = snapshot();
        for i in 0..3 {
            let mut r = input(&f, &s);
            match i {
                0 => r.definition.character_id += 1,
                1 => r.definition.kind = CharacterKind::Shape,
                _ => r.definition.bounds = None,
            }
            assert!(matches!(
                derive_fresh_html(&f, r),
                Err(Error::Definition(_))
            ));
        }
        for flag in [0x80, 0x40, 0x1000, 0x4000, 0x4] {
            let mut r = input(&f, &s);
            r.layout.flags.bits |= flag;
            assert!(matches!(derive_fresh_html(&f, r), Err(Error::Layout(_))));
        }
        let mut r = input(&f, &s);
        r.layout.bounds.x_max += 20;
        assert!(matches!(derive_fresh_html(&f, r), Err(Error::Layout(_))));
        for i in 0..6 {
            let mut changed = f.clone();
            match i {
                0 => changed.color = None,
                1 => changed.font_height = None,
                2 => changed.layout = None,
                3 => changed.font_id = None,
                4 => changed.flags.bits |= 2,
                _ => changed.variable_name = "bound".into(),
            }
            assert!(matches!(
                derive_fresh_html(&changed, input(&changed, &s)),
                Err(Error::Definition(_))
            ));
        }
    }

    #[test]
    fn rejects_budget_and_provenance_before_producing_a_binding() {
        let f = field();
        let s = snapshot();
        for bad in ["", " \t", "source\0name"] {
            let mut r = input(&f, &s);
            r.provenance = bad;
            assert_eq!(derive_fresh_html(&f, r), Err(Error::Provenance));
        }
        let long = "p".repeat(MAX_RUNTIME_PROVENANCE_BYTES);
        let mut r = input(&f, &s);
        r.provenance = &long;
        assert_eq!(
            derive_fresh_html(&f, r),
            Err(Error::Limit("provenance bytes"))
        );
        let mut s = snapshot();
        s.document.text = "A".repeat(MAX_TEXT_BYTES + 1);
        assert_eq!(
            derive_fresh_html(&f, input(&f, &s)),
            Err(Error::Limit("text bytes"))
        );
    }
}
