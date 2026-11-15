//! Conditional attached TextField replacement, not attachment, layout, or a VM.
//! Native contracts and explicit boundaries: docs/TEXTFIELD_RUNTIME.md.

use std::{fmt, num::NonZeroU64};

pub const HTML_ENABLED: u32 = 1 << 1;
pub const HTML_PARSE_MODE: u32 = 1 << 12;
pub const TEXT_DIRTY: u32 = 1 << 13;
pub const FORCE_REBUILD: u32 = 1 << 16;
pub const VIEW_INVALIDATION_MASK: u32 = 0x106;
pub const MAX_TEXT_BYTES: usize = 1_048_576;
const TARGET_PARSER_FLAG: u32 = 1 << 4;
const VIEW_PARSER_FLAG: u8 = 1 << 2;

/// Caller-established association. IDs are local handles, not native addresses.
/// A domain must change when its Scene/VM association is replaced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttachedIdentity {
    pub domain: NonZeroU64,
    pub scene_node: usize,
    pub avm_owner: usize,
}

/// Handle to a caller-validated effective text profile in the identity's domain.
/// Non-null profiles are a restriction of this module, not a universal native rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextFormat(pub NonZeroU64);

/// Separate handle type prevents accidentally interchanging format families.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParagraphFormat(pub NonZeroU64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormatProfile {
    pub text: TextFormat,
    pub paragraph: ParagraphFormat,
}

/// Both results must be established by the caller. Native format factories may
/// normalize/intern defaults; equality with the input run profile is not assumed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefinitionHtmlProfile {
    pub run_format: FormatProfile,
    pub defaults_after_nonempty: FormatProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidePath {
    Absent,
    Present,
    Unknown,
}

/// No Default: absence is an explicit caller assertion, never a fresh-state guess.
/// Re-admit a new snapshot if external activity changes any of these conditions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub watches: SidePath,
    pub translation_service: SidePath,
    pub stylesheet: SidePath,
    pub editor: SidePath,
    pub variable_binding: SidePath,
    pub image_callbacks: SidePath,
}

/// Projection of the admitted single, uniform, no-markup text run. Empty text
/// has no character run; this does not describe native empty-paragraph objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub text: String,
    pub run_format: Option<FormatProfile>,
}

/// Pending requests for a future layout/render host, not observed rendered state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invalidation {
    pub view_mask: u32,
    pub render_dirty: bool,
}

/// Every native-derived value is supplied. Last input may differ from document
/// text; construction must not collapse those two stores or synthesize defaults.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub raw_target_flags: u32,
    pub raw_view_flags_149: u8,
    pub last_input: String,
    pub document: Document,
    pub current_defaults: FormatProfile,
    pub definition_html: DefinitionHtmlProfile,
    pub pending_invalidation: Invalidation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub max_text_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_text_bytes: MAX_TEXT_BYTES,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseMode {
    Plain,
    Html,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatSource {
    CurrentDocumentDefaults,
    DefinitionHtml,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Replacement {
    /// Native core returned early. The native property setter still succeeds.
    Unchanged,
    Replaced {
        mode: ParseMode,
        format_source: FormatSource,
        invalidation: Invalidation,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HtmlFlagChange {
    pub before: bool,
    pub after: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputFeature {
    NonAscii,
    ControlCharacter,
    MarkupOrEntity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Limit(&'static str),
    UnsupportedProfile {
        feature: &'static str,
        state: SidePath,
    },
    UnsupportedTargetParserFlag,
    UnsupportedViewParserFlag,
    UnsupportedInput {
        field: &'static str,
        byte_offset: usize,
        feature: InputFeature,
    },
    InvalidDocument(&'static str),
    HtmlSerializationUnproved,
    Allocation,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TextField state: {self:?}")
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct TextFieldState {
    identity: AttachedIdentity,
    profile: Profile,
    snapshot: Snapshot,
    limits: Limits,
}
impl TextFieldState {
    pub fn from_snapshot(
        identity: AttachedIdentity,
        profile: Profile,
        snapshot: Snapshot,
        limits: Limits,
    ) -> Result<Self> {
        if limits.max_text_bytes > MAX_TEXT_BYTES {
            return Err(Error::Limit("configured text ceiling"));
        }
        for (feature, state) in [
            ("watches", profile.watches),
            ("translation service", profile.translation_service),
            ("stylesheet", profile.stylesheet),
            ("editor", profile.editor),
            ("variable binding", profile.variable_binding),
            ("image callbacks", profile.image_callbacks),
        ] {
            if state != SidePath::Absent {
                return Err(Error::UnsupportedProfile { feature, state });
            }
        }
        // These are native target/view bits, never SWF EditFlags. This initial
        // profile requires both parser booleans false rather than guessing them.
        if snapshot.raw_target_flags & TARGET_PARSER_FLAG != 0 {
            return Err(Error::UnsupportedTargetParserFlag);
        }
        if snapshot.raw_view_flags_149 & VIEW_PARSER_FLAG != 0 {
            return Err(Error::UnsupportedViewParserFlag);
        }
        validate_input(&snapshot.last_input, "last input", limits)?;
        validate_input(&snapshot.document.text, "document", limits)?;
        if snapshot.document.text.is_empty() != snapshot.document.run_format.is_none() {
            return Err(Error::InvalidDocument(
                "nonempty text requires one format; empty text has no character run",
            ));
        }
        Ok(Self {
            identity,
            profile,
            snapshot,
            limits,
        })
    }

    pub fn identity(&self) -> AttachedIdentity {
        self.identity
    }
    pub fn profile(&self) -> Profile {
        self.profile
    }
    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }
    pub fn html(&self) -> bool {
        self.snapshot.raw_target_flags & HTML_ENABLED != 0
    }
    pub fn parse_mode(&self) -> ParseMode {
        if self.snapshot.raw_target_flags & HTML_PARSE_MODE == 0 {
            ParseMode::Plain
        } else {
            ParseMode::Html
        }
    }
    pub fn force_rebuild(&self) -> bool {
        self.snapshot.raw_target_flags & FORCE_REBUILD != 0
    }
    pub fn text(&self) -> &str {
        &self.snapshot.document.text
    }
    pub fn html_text(&self) -> Result<&str> {
        if self.html() {
            // No stylesheet is admitted, so the native cached-input exception
            // cannot apply. Even empty/plain-looking HTML needs its serializer.
            Err(Error::HtmlSerializationUnproved)
        } else {
            Ok(self.text())
        }
    }

    /// Actual Boolean input only: no coercion or implicit parse/invalidation.
    pub fn set_html(&mut self, enabled: bool) -> HtmlFlagChange {
        let before = self.html();
        if enabled {
            self.snapshot.raw_target_flags |= HTML_ENABLED;
        } else {
            self.snapshot.raw_target_flags &= !HTML_ENABLED;
        }
        HtmlFlagChange {
            before,
            after: enabled,
        }
    }

    /// Actual string input only. The selection/edit pre-helper is inert under
    /// the admitted absent-editor profile. Public `html` remains unchanged.
    pub fn set_text(&mut self, input: &str) -> Result<Replacement> {
        self.replace(input, ParseMode::Plain)
    }

    pub fn set_html_text(&mut self, input: &str) -> Result<Replacement> {
        let mode = if self.html() {
            ParseMode::Html
        } else {
            ParseMode::Plain
        };
        self.replace(input, mode)
    }

    fn replace(&mut self, input: &str, mode: ParseMode) -> Result<Replacement> {
        validate_input(input, "replacement", self.limits)?;
        if input == self.snapshot.last_input && !self.force_rebuild() {
            // The comparison precedes mode/default changes and invalidation.
            return Ok(Replacement::Unchanged);
        }
        // Reserve both bounded stores before mutation. Rejected/failed requests
        // leave the complete admitted snapshot unchanged.
        let last_input = copy_string(input)?;
        let document_text = copy_string(input)?;
        let (format, format_source) = match mode {
            ParseMode::Plain => (
                self.snapshot.current_defaults,
                FormatSource::CurrentDocumentDefaults,
            ),
            ParseMode::Html => (
                self.snapshot.definition_html.run_format,
                FormatSource::DefinitionHtml,
            ),
        };
        self.snapshot.last_input = last_input;
        self.snapshot.document = Document {
            text: document_text,
            run_format: (!input.is_empty()).then_some(format),
        };
        if mode == ParseMode::Html && !input.is_empty() {
            // 0x140177ed5/ee1 store defaults in the token-3 completion path.
            // Empty HTML returns earlier, before those calls.
            self.snapshot.current_defaults = self.snapshot.definition_html.defaults_after_nonempty;
        }
        self.snapshot.raw_target_flags &= !(FORCE_REBUILD | HTML_PARSE_MODE);
        if mode == ParseMode::Html {
            self.snapshot.raw_target_flags |= HTML_PARSE_MODE;
        }
        self.snapshot.raw_target_flags |= TEXT_DIRTY;
        self.snapshot.pending_invalidation.view_mask |= VIEW_INVALIDATION_MASK;
        self.snapshot.pending_invalidation.render_dirty = true;
        Ok(Replacement::Replaced {
            mode,
            format_source,
            invalidation: Invalidation {
                view_mask: VIEW_INVALIDATION_MASK,
                render_dirty: true,
            },
        })
    }
}

fn validate_input(input: &str, field: &'static str, limits: Limits) -> Result<()> {
    if input.len() > limits.max_text_bytes {
        return Err(Error::Limit(field));
    }
    for (byte_offset, byte) in input.bytes().enumerate() {
        let feature = if !byte.is_ascii() {
            Some(InputFeature::NonAscii)
        } else if !(b' '..=b'~').contains(&byte) {
            Some(InputFeature::ControlCharacter)
        } else if matches!(byte, b'<' | b'&') {
            Some(InputFeature::MarkupOrEntity)
        } else {
            None
        };
        if let Some(feature) = feature {
            return Err(Error::UnsupportedInput {
                field,
                byte_offset,
                feature,
            });
        }
    }
    Ok(())
}

fn copy_string(input: &str) -> Result<String> {
    let mut result = String::new();
    result
        .try_reserve_exact(input.len())
        .map_err(|_| Error::Allocation)?;
    result.push_str(input);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format(id: u64) -> FormatProfile {
        FormatProfile {
            text: TextFormat(NonZeroU64::new(id).unwrap()),
            paragraph: ParagraphFormat(NonZeroU64::new(id + 100).unwrap()),
        }
    }
    fn profile() -> Profile {
        Profile {
            watches: SidePath::Absent,
            translation_service: SidePath::Absent,
            stylesheet: SidePath::Absent,
            editor: SidePath::Absent,
            variable_binding: SidePath::Absent,
            image_callbacks: SidePath::Absent,
        }
    }
    fn snapshot() -> Snapshot {
        Snapshot {
            raw_target_flags: 0x8000_0000,
            raw_view_flags_149: 0x80,
            last_input: "cached input".into(),
            document: Document {
                text: "visible document".into(),
                run_format: Some(format(1)),
            },
            current_defaults: format(2),
            definition_html: DefinitionHtmlProfile {
                run_format: format(3),
                defaults_after_nonempty: format(4),
            },
            pending_invalidation: Invalidation {
                view_mask: 0x80,
                render_dirty: false,
            },
        }
    }
    fn identity() -> AttachedIdentity {
        AttachedIdentity {
            domain: NonZeroU64::new(1).unwrap(),
            scene_node: 0,
            avm_owner: 0,
        }
    }
    fn state(snapshot: Snapshot) -> TextFieldState {
        TextFieldState::from_snapshot(identity(), profile(), snapshot, Limits::default()).unwrap()
    }

    #[test]
    fn getters_and_html_toggle_do_not_echo_or_reparse_cached_input() {
        let mut field = state(snapshot());
        assert_eq!(field.text(), "visible document");
        assert_eq!(field.html_text().unwrap(), "visible document");
        let before = field.snapshot().clone();
        assert_eq!(
            field.set_html(true),
            HtmlFlagChange {
                before: false,
                after: true
            }
        );
        assert_eq!(
            field.set_html_text("cached input").unwrap(),
            Replacement::Unchanged
        );
        assert_eq!(field.parse_mode(), ParseMode::Plain);
        assert_eq!(field.snapshot().document, before.document);
        assert_eq!(field.snapshot().current_defaults, before.current_defaults);
        assert_eq!(
            field.snapshot().pending_invalidation,
            before.pending_invalidation
        );
        assert_eq!(
            field.snapshot().raw_target_flags,
            before.raw_target_flags | HTML_ENABLED
        );
        assert_eq!(field.html_text(), Err(Error::HtmlSerializationUnproved));
        field.set_html(false);
        assert_eq!(field.snapshot(), &before);
    }

    #[test]
    fn force_rebuild_applies_same_input_and_preserves_unrelated_bits() {
        let mut seed = snapshot();
        seed.raw_target_flags |= FORCE_REBUILD | HTML_ENABLED;
        let mut field = state(seed);
        let change = field.set_html_text("cached input").unwrap();
        assert!(matches!(
            change,
            Replacement::Replaced {
                mode: ParseMode::Html,
                ..
            }
        ));
        assert_eq!(field.text(), "cached input");
        assert_eq!(field.snapshot().document.run_format, Some(format(3)));
        assert_eq!(field.snapshot().current_defaults, format(4));
        assert_eq!(field.snapshot().raw_target_flags, 0x8000_3002);
        assert_eq!(field.snapshot().raw_view_flags_149, 0x80);
        assert_eq!(
            field.snapshot().pending_invalidation,
            Invalidation {
                view_mask: 0x186,
                render_dirty: true
            }
        );
        assert!(!field.force_rebuild());
    }

    #[test]
    fn plain_overwrite_retains_html_enable_and_uses_current_defaults() {
        let mut seed = snapshot();
        seed.raw_target_flags |= HTML_ENABLED | HTML_PARSE_MODE;
        let mut field = state(seed);
        assert!(matches!(
            field.set_text("new plain").unwrap(),
            Replacement::Replaced {
                mode: ParseMode::Plain,
                format_source: FormatSource::CurrentDocumentDefaults,
                ..
            }
        ));
        assert!(field.html());
        assert_eq!(field.parse_mode(), ParseMode::Plain);
        assert_eq!(field.snapshot().document.run_format, Some(format(2)));
        assert_eq!(field.snapshot().current_defaults, format(2));
        assert_eq!(field.html_text(), Err(Error::HtmlSerializationUnproved));
        let before = field.snapshot().clone();
        assert_eq!(
            field.set_html_text("new plain").unwrap(),
            Replacement::Unchanged
        );
        assert_eq!(field.snapshot(), &before);
    }

    #[test]
    fn html_nonempty_and_empty_have_different_default_store_paths() {
        let mut field = state(snapshot());
        field.set_html(true);
        field.set_html_text("READY").unwrap();
        assert_eq!(field.snapshot().current_defaults, format(4));
        field.set_text("plain after HTML").unwrap();
        assert_eq!(field.snapshot().document.run_format, Some(format(4)));
        field.set_html_text("").unwrap();
        assert_eq!(field.snapshot().current_defaults, format(4));
        assert_eq!(field.snapshot().document.run_format, None);
        assert_eq!(field.text(), "");
        assert_eq!(field.parse_mode(), ParseMode::Html);
        assert_eq!(field.html_text(), Err(Error::HtmlSerializationUnproved));
        // Empty HTML on a different initial profile must not install format(4).
        let mut other = state(snapshot());
        other.set_html(true);
        other.set_html_text("").unwrap();
        assert_eq!(other.snapshot().current_defaults, format(2));
    }

    #[test]
    fn rejected_inputs_are_transactional_and_limits_are_exact() {
        let mut field = state(snapshot());
        for input in [
            "a\0b", "a\rb", "a\nb", "a\tb", "é", "<p>x", "a&amp;b", "a\u{7f}",
        ] {
            let before = field.snapshot().clone();
            assert!(matches!(
                field.set_text(input),
                Err(Error::UnsupportedInput { .. })
            ));
            assert_eq!(field.snapshot(), &before);
        }
        let mut seed = snapshot();
        seed.last_input.clear();
        seed.document = Document {
            text: String::new(),
            run_format: None,
        };
        let mut field = TextFieldState::from_snapshot(
            identity(),
            profile(),
            seed,
            Limits { max_text_bytes: 5 },
        )
        .unwrap();
        field.set_text("READY").unwrap();
        let before = field.snapshot().clone();
        assert_eq!(field.set_text("READY!"), Err(Error::Limit("replacement")));
        assert_eq!(field.snapshot(), &before);
    }

    #[test]
    fn unknown_and_present_side_paths_and_inconsistent_documents_are_rejected() {
        for status in [SidePath::Present, SidePath::Unknown] {
            for index in 0..6 {
                let mut p = profile();
                *match index {
                    0 => &mut p.watches,
                    1 => &mut p.translation_service,
                    2 => &mut p.stylesheet,
                    3 => &mut p.editor,
                    4 => &mut p.variable_binding,
                    _ => &mut p.image_callbacks,
                } = status;
                assert!(
                    matches!(TextFieldState::from_snapshot(identity(), p, snapshot(), Limits::default()), Err(Error::UnsupportedProfile { state, .. }) if state == status)
                );
            }
        }
        let mut seed = snapshot();
        seed.raw_target_flags |= TARGET_PARSER_FLAG;
        assert!(matches!(
            TextFieldState::from_snapshot(identity(), profile(), seed, Limits::default()),
            Err(Error::UnsupportedTargetParserFlag)
        ));
        let mut seed = snapshot();
        seed.raw_view_flags_149 |= VIEW_PARSER_FLAG;
        assert!(matches!(
            TextFieldState::from_snapshot(identity(), profile(), seed, Limits::default()),
            Err(Error::UnsupportedViewParserFlag)
        ));
        let mut seed = snapshot();
        seed.document.run_format = None;
        assert!(matches!(
            TextFieldState::from_snapshot(identity(), profile(), seed, Limits::default()),
            Err(Error::InvalidDocument(_))
        ));
        assert!(matches!(
            TextFieldState::from_snapshot(
                identity(),
                profile(),
                snapshot(),
                Limits {
                    max_text_bytes: MAX_TEXT_BYTES + 1
                }
            ),
            Err(Error::Limit(_))
        ));
    }
}
