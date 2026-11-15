//! Conditional TextField document snapshots for exact scene instances.
//! Format handles have no inferred native-to-renderer conversion here.

use super::{
    scene_content::{MAX_RUNTIME_PROVENANCE_BYTES, RuntimeTextOverride, RuntimeTextStyle},
    textfield_host::TextFieldHost,
    textfield_state::{AttachedIdentity, FormatProfile, Invalidation, Profile, SidePath},
};
use crate::{
    avm1::vm::AttachedTextFieldId,
    gfx::{
        CharacterKind, edit_layout,
        scene::{DefinitionRef, Scene},
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    num::NonZeroU64,
};

const MAX_BINDINGS: usize = 100_000;
const MAX_BATCH_BYTES: usize = 64 * 1024 * 1024;

/// Empty documents have no character run. Neither choice claims recovered
/// native empty-paragraph formatting, and there is deliberately no Default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyDocumentStyle {
    Reject,
    /// Caller explicitly selects current defaults as conditional layout inputs.
    UseSuppliedCurrentDefaults,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldBinding {
    pub field: AttachedTextFieldId,
    pub identity: AttachedIdentity,
    pub expected_definition: DefinitionRef,
    pub empty_style: EmptyDocumentStyle,
}

/// Caller-established mapping of both opaque format families in one domain.
/// Font route, RGBA, spacing and layout are supplied; nothing is inherited from
/// an authored style or guessed from the numeric format handles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatBinding {
    pub domain: NonZeroU64,
    pub profile: FormatProfile,
    pub style: RuntimeTextStyle,
    pub provenance: String,
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_fields: usize,
    pub max_formats: usize,
    pub max_document_bytes: usize,
    /// Ceiling independently applied to supplied registry and emitted strings.
    pub max_provenance_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_fields: 4096,
            max_formats: 4096,
            max_document_bytes: 16 * 1024 * 1024,
            max_provenance_bytes: 16 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectedFormatSource {
    DocumentRun,
    /// Explicit renderer policy; native empty paragraph remains unproved.
    ExplicitEmptyCurrentDefaults,
}

/// Observed host snapshot metadata, aligned with TextBatch::overrides().
/// Pending invalidation is retained, never acknowledged by preparation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldSnapshot {
    pub field: AttachedTextFieldId,
    pub identity: AttachedIdentity,
    pub definition: DefinitionRef,
    pub selected_format: FormatProfile,
    pub format_source: SelectedFormatSource,
    pub current_defaults: FormatProfile,
    pub inactive_side_paths: Profile,
    pub raw_target_flags: u32,
    pub raw_view_flags_149: u8,
    pub pending_invalidation: Invalidation,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub fields: usize,
    pub document_bytes: usize,
    pub provenance_bytes: usize,
    pub empty_documents: usize,
}

/// Owns a point-in-time document projection and borrows its exact scene.
/// Further host changes do not update this batch. Snapshot again for them.
#[derive(Debug)]
pub struct TextBatch<'scene, 'movie> {
    scene: &'scene Scene<'movie>,
    overrides: Vec<RuntimeTextOverride>,
    snapshots: Vec<FieldSnapshot>,
    statistics: Statistics,
}
impl<'scene, 'movie> TextBatch<'scene, 'movie> {
    pub fn scene(&self) -> &'scene Scene<'movie> {
        self.scene
    }
    pub fn overrides(&self) -> &[RuntimeTextOverride] {
        &self.overrides
    }
    pub fn snapshots(&self) -> &[FieldSnapshot] {
        &self.snapshots
    }
    pub fn statistics(&self) -> Statistics {
        self.statistics
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Limit(&'static str),
    Allocation,
    FormatDomain {
        index: usize,
    },
    DuplicateFormat {
        index: usize,
    },
    InvalidProvenance {
        index: usize,
    },
    UnsupportedStyle {
        index: usize,
        reason: &'static str,
    },
    FieldDomain {
        field: AttachedTextFieldId,
    },
    DuplicateField {
        field: AttachedTextFieldId,
    },
    DuplicateNode {
        node: usize,
    },
    MissingField {
        field: AttachedTextFieldId,
    },
    IdentityMismatch {
        field: AttachedTextFieldId,
    },
    SceneMismatch {
        field: AttachedTextFieldId,
    },
    UnsupportedSidePath {
        field: AttachedTextFieldId,
    },
    InvalidDocument {
        field: AttachedTextFieldId,
    },
    EmptyDocument {
        field: AttachedTextFieldId,
    },
    MissingFormat {
        field: AttachedTextFieldId,
        profile: FormatProfile,
    },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "runtime text preparation: {self:?}")
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

/// Snapshot exact admitted document text and format profiles without calling a
/// getter/serializer, modifying the host, executing scripts or preparing glyphs.
/// The caller must keep the host domain bound to this Scene/VM lifetime. Scene
/// identity and format provenance assertions are checked structurally, not
/// established by a native snapshot or a discovery of runtime format mappings.
///
/// Pass batch.scene() and batch.overrides() together to scene_content's existing
/// prepare_with_text_overrides. That stage verifies library/source font routes,
/// bounds, layout support and glyphs; success here does not claim it will pass.
pub fn snapshot_overrides<'scene, 'movie>(
    host: &TextFieldHost,
    scene: &'scene Scene<'movie>,
    fields: &[FieldBinding],
    formats: &[FormatBinding],
    limits: Limits,
) -> Result<TextBatch<'scene, 'movie>> {
    if limits.max_fields > MAX_BINDINGS
        || limits.max_formats > MAX_BINDINGS
        || limits.max_document_bytes > MAX_BATCH_BYTES
        || limits.max_provenance_bytes > MAX_BATCH_BYTES
    {
        return Err(Error::Limit("configured ceiling"));
    }
    if fields.len() > limits.max_fields || formats.len() > limits.max_formats {
        return Err(Error::Limit("binding count"));
    }
    let mut registry = BTreeMap::new();
    let mut registry_bytes = 0;
    for (index, binding) in formats.iter().enumerate() {
        if binding.domain != host.domain() {
            return Err(Error::FormatDomain { index });
        }
        if binding.provenance.trim().is_empty()
            || binding.provenance.len() > MAX_RUNTIME_PROVENANCE_BYTES
        {
            return Err(Error::InvalidProvenance { index });
        }
        add(
            &mut registry_bytes,
            binding.provenance.len(),
            limits.max_provenance_bytes,
            "format provenance bytes",
        )?;
        let style = binding.style;
        if !style.flags.has_font() || !style.flags.has_text_color() || !style.flags.has_layout() {
            return Err(Error::UnsupportedStyle {
                index,
                reason: "explicit font/color/layout flags required",
            });
        }
        if style.effects.kerning != Some(false)
            || style.effects.letter_spacing_twips != Some(0)
            || style.effects.underline != Some(false)
            || style.effects.block_indent_twips != Some(0)
        {
            return Err(Error::UnsupportedStyle {
                index,
                reason: "effects must be explicitly inactive",
            });
        }
        let key = (
            binding.profile.text.0.get(),
            binding.profile.paragraph.0.get(),
        );
        if registry.insert(key, binding).is_some() {
            return Err(Error::DuplicateFormat { index });
        }
    }
    let mut batch = TextBatch {
        scene,
        overrides: Vec::new(),
        snapshots: Vec::new(),
        statistics: Statistics::default(),
    };
    batch
        .overrides
        .try_reserve_exact(fields.len())
        .map_err(|_| Error::Allocation)?;
    batch
        .snapshots
        .try_reserve_exact(fields.len())
        .map_err(|_| Error::Allocation)?;
    let mut seen_fields = BTreeSet::new();
    let mut seen_nodes = BTreeSet::new();
    for binding in fields {
        let field = binding.field;
        if binding.identity.domain != host.domain() {
            return Err(Error::FieldDomain { field });
        }
        if !seen_fields.insert(field.0) {
            return Err(Error::DuplicateField { field });
        }
        if !seen_nodes.insert(binding.identity.scene_node) {
            return Err(Error::DuplicateNode {
                node: binding.identity.scene_node,
            });
        }
        let state = host.state(field).ok_or(Error::MissingField { field })?;
        if state.identity() != binding.identity {
            return Err(Error::IdentityMismatch { field });
        }
        let node = scene
            .nodes
            .get(binding.identity.scene_node)
            .ok_or(Error::SceneMismatch { field })?;
        let definition = binding.expected_definition;
        let source = scene
            .movies
            .get(&definition.movie_id)
            .ok_or(Error::SceneMismatch { field })?;
        let character = source
            .dictionary
            .get(&definition.character_id)
            .ok_or(Error::SceneMismatch { field })?;
        if node.id != binding.identity.scene_node
            || node.kind.definition() != Some(definition)
            || definition.kind != CharacterKind::Text
            || character.tag_index != definition.tag_index
            || character.kind != definition.kind
            || character.bounds != definition.bounds
            || source
                .tags
                .get(definition.tag_index)
                .is_none_or(|tag| tag.code != 37)
        {
            return Err(Error::SceneMismatch { field });
        }
        let profile = state.profile();
        if [
            profile.watches,
            profile.translation_service,
            profile.stylesheet,
            profile.editor,
            profile.variable_binding,
            profile.image_callbacks,
        ]
        .iter()
        .any(|p| *p != SidePath::Absent)
        {
            return Err(Error::UnsupportedSidePath { field });
        }
        let saved = state.snapshot();
        let document = &saved.document;
        if document.text.is_empty() != document.run_format.is_none() {
            return Err(Error::InvalidDocument { field });
        }
        let (selected, format_source) = match document.run_format {
            Some(format) => (format, SelectedFormatSource::DocumentRun),
            None => match binding.empty_style {
                EmptyDocumentStyle::Reject => return Err(Error::EmptyDocument { field }),
                EmptyDocumentStyle::UseSuppliedCurrentDefaults => {
                    batch.statistics.empty_documents += 1;
                    (
                        saved.current_defaults,
                        SelectedFormatSource::ExplicitEmptyCurrentDefaults,
                    )
                }
            },
        };
        let format = registry
            .get(&(selected.text.0.get(), selected.paragraph.0.get()))
            .ok_or(Error::MissingFormat {
                field,
                profile: selected,
            })?;
        if document.text.len() > edit_layout::MAX_TEXT_BYTES
            || document.text.len() > edit_layout::MAX_TEXT_SCALARS
            || document
                .text
                .bytes()
                .any(|b| !(b' '..=b'~').contains(&b) || matches!(b, b'<' | b'&'))
        {
            return Err(Error::InvalidDocument { field });
        }
        add(
            &mut batch.statistics.document_bytes,
            document.text.len(),
            limits.max_document_bytes,
            "document bytes",
        )?;
        add(
            &mut batch.statistics.provenance_bytes,
            format.provenance.len(),
            limits.max_provenance_bytes,
            "output provenance bytes",
        )?;
        batch.overrides.push(RuntimeTextOverride {
            node: node.id,
            expected_definition: definition,
            text: copy_string(&document.text)?,
            style: format.style,
            provenance: copy_string(&format.provenance)?,
        });
        batch.snapshots.push(FieldSnapshot {
            field,
            identity: binding.identity,
            definition,
            selected_format: selected,
            format_source,
            current_defaults: saved.current_defaults,
            inactive_side_paths: profile,
            raw_target_flags: saved.raw_target_flags,
            raw_view_flags_149: saved.raw_view_flags_149,
            pending_invalidation: saved.pending_invalidation,
        });
        batch.statistics.fields += 1;
    }
    Ok(batch)
}

fn add(total: &mut usize, value: usize, limit: usize, label: &'static str) -> Result<()> {
    *total = total
        .checked_add(value)
        .filter(|n| *n <= limit)
        .ok_or(Error::Limit(label))?;
    Ok(())
}
fn copy_string(source: &str) -> Result<String> {
    let mut result = String::new();
    result
        .try_reserve_exact(source.len())
        .map_err(|_| Error::Allocation)?;
    result.push_str(source);
    Ok(result)
}
