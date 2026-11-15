//! The evidenced successful native localization result, from explicit references.
//! No prototype inheritance, fallback locale, formatter or VM heap is invented.

use crate::localization::{LocaleCatalog, StringEntry, format_prototype_reference};
use std::{io, ops::Range};

/// Inputs from a caller that has already resolved the UILocalizedInfoPrototype's
/// effective fields. There is deliberately no conversion from UiKeyBinding's
/// directly serialized optional fields: those do not establish native defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct References {
    pub prototype_id: u64,
    pub display_text_id: u64,
    pub tooltip_text_id: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringSource {
    pub id: u64,
    /// Index in the caller's explicit LocaleCatalog partition list.
    pub partition: usize,
    pub entry_offset: usize,
    pub text_bytes: Range<usize>,
}

/// Fields correspond exactly to the native reflected return structure. Strings
/// retain validated UTF-8 and convert losslessly with encode_utf16 at the VM seam.
/// Stored source IDs/spans are metadata, not additional ActionScript properties.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedObjectData {
    pub localized_prototype_id_str: String,
    pub localized_text: String,
    pub localized_tooltip_text: String,
    pub has_tooltip: bool,
    pub references: References,
    pub display_source: StringSource,
    pub tooltip_source: StringSource,
}

/// Select the original default entry and preserve its text for the established
/// no-marker success path. The native callback supplies selector UINT64_MAX,
/// matching the standard loader's base node before testing any alternates.
///
/// A missing ID, '#' formatter marker or missing-translation sentinel returns
/// an explicit error. No fallback, empty value, textual key match or variant is
/// substituted. Tooltip presence depends on its reference, not text emptiness.
pub fn resolve(catalog: &LocaleCatalog, references: References) -> io::Result<LocalizedObjectData> {
    if references.prototype_id == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "successful localized prototype result requires a nonzero prototype reference",
        ));
    }
    let (display, display_source) = present_default(catalog, references.display_text_id)?;
    let (tooltip, tooltip_source) = present_default(catalog, references.tooltip_text_id)?;
    Ok(LocalizedObjectData {
        localized_prototype_id_str: format_prototype_reference(references.prototype_id),
        localized_text: display.default.text.clone(),
        localized_tooltip_text: tooltip.default.text.clone(),
        has_tooltip: references.tooltip_text_id != 0,
        references,
        display_source,
        tooltip_source,
    })
}

fn present_default(catalog: &LocaleCatalog, id: u64) -> io::Result<(&StringEntry, StringSource)> {
    let (partition, entry) = catalog
        .tables()
        .iter()
        .enumerate()
        .find_map(|(index, table)| table.get(id).map(|entry| (index, entry)))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("localized string {id:#x} is absent; native fallback is unresolved"),
            )
        })?;
    if entry.default.text.contains('#') || entry.default.text.contains("!!Missing translation:") {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "localized string {id:#x} requires the native formatter or missing-translation path"
            ),
        ));
    }
    // Parsed StringTable texts are bounded and exclude their NUL terminator.
    // Native conversion is CP_UTF8/MB_ERR_INVALID_CHARS; Rust's validated UTF-8
    // can be encoded directly as UTF-16 without normalization or replacement.
    Ok((
        entry,
        StringSource {
            id,
            partition,
            entry_offset: entry.source_offset,
            text_bytes: entry.default.source_offset..entry.default.source_end,
        },
    ))
}
