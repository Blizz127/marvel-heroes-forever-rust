//! Conditional store stage for an explicitly selected native primary vertex chunk.
//!
//! Inputs already passed export selection and normalization. Colors are native
//! words after callback/cache resolution, not synthesized bitmap colors. This
//! module neither resolves formats/resources nor enables a rendering path.

use super::native_scale9::{MathError, pack_primary_point_finite_ieee_nearest};
use super::native_scale9_tessellation::GeneratedVertex;
use std::fmt;

/// Recovered per-transfer vertex request in bff70, not a total mesh limit.
pub const MAX_PACKED_VERTEX_CHUNK: usize = 256;
pub const PRIMARY_RECORD_STRIDE: usize = 12;

/// Actual selected format fields, supplied by the caller. Pointer compatibility
/// from the native cache alone does not establish these values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedFormatProfile {
    pub stride: u32,
    /// (byte offset, full element code), including the terminating pair.
    pub elements: [(u32, u32); 5],
}

pub const EXACT_PRIMARY_FORMAT: PackedFormatProfile = PackedFormatProfile {
    stride: 12,
    elements: [(0, 0x132), (4, 0x251), (8, 0x1211), (9, 0x2211), (0, 0)],
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PackingProfile {
    pub format: PackedFormatProfile,
    /// bff70 argument 6; only its zero/default branch is admitted.
    pub forced_style: u32,
    /// Whether the full provider's +0x60 morph owner is nonnull.
    pub morph_owner_present: bool,
    /// Native factor argument. Finite values only; a present owner requires zero.
    pub morph_factor: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedColorWord {
    pub style_id: u16,
    /// Exact native dword after single-style callback/cache resolution. No
    /// channel ordering, white default, interpolation or premultiplication.
    pub word: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackingError {
    UnsupportedFormat,
    UnsupportedProfile,
    TooManyVertices,
    ColorCountMismatch,
    DestinationTooShort,
    UnsupportedFlags { vertex: usize },
    ZeroSelectedStyle { vertex: usize },
    ColorStyleMismatch { vertex: usize },
    InconsistentCachedColor { vertex: usize },
    Arithmetic { vertex: usize, error: MathError },
}

impl fmt::Display for PackingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormat => {
                f.write_str("selected format is not exact primary stride-12 layout")
            }
            Self::UnsupportedProfile => f.write_str("unsupported forced-style or morph profile"),
            Self::TooManyVertices => f.write_str("primary vertex chunk exceeds 256 records"),
            Self::ColorCountMismatch => {
                f.write_str("resolved color count differs from vertex count")
            }
            Self::DestinationTooShort => f.write_str("primary record destination is too short"),
            Self::UnsupportedFlags { vertex } => write!(
                f,
                "vertex {vertex} requires skipped or dual-style color handling"
            ),
            Self::ZeroSelectedStyle { vertex } => {
                write!(f, "vertex {vertex} selects unsupported style zero")
            }
            Self::ColorStyleMismatch { vertex } => {
                write!(f, "vertex {vertex} resolved color has a different style")
            }
            Self::InconsistentCachedColor { vertex } => write!(
                f,
                "vertex {vertex} contradicts consecutive native style-cache reuse"
            ),
            Self::Arithmetic { vertex, error } => write!(f, "vertex {vertex}: {error}"),
        }
    }
}
impl std::error::Error for PackingError {}

/// Writes only bytes 0..10 of each selected 12-byte record, retaining padding
/// bytes 10/11 and all trailing destination bytes. Borrowed inputs are unchanged.
/// Any rejected profile, length, style, color or arithmetic leaves the entire
/// destination unchanged; staging is a bounded stack array and cannot allocate.
///
/// `vertices.position` must already be the native normalization result. This
/// function does not select by mesh/ordinal, advance a descriptor cursor, run
/// callbacks, or claim the current scene has the asserted format/profile.
/// Resolved colors include prior native cache behavior. Adjacent equal selected
/// styles must retain their word; initial cache state and cross-call continuity
/// remain caller responsibilities. Style zero is deliberately outside this
/// bounded profile, as are vertex bits 0x8000 and 0x10.
pub fn write_primary_records(
    vertices: &[GeneratedVertex],
    colors: &[ResolvedColorWord],
    profile: PackingProfile,
    destination: &mut [u8],
) -> Result<(), PackingError> {
    if profile.format != EXACT_PRIMARY_FORMAT {
        return Err(PackingError::UnsupportedFormat);
    }
    if profile.forced_style != 0
        || !profile.morph_factor.is_finite()
        || (profile.morph_owner_present && profile.morph_factor != 0.0)
    {
        return Err(PackingError::UnsupportedProfile);
    }
    if vertices.len() > MAX_PACKED_VERTEX_CHUNK {
        return Err(PackingError::TooManyVertices);
    }
    if colors.len() != vertices.len() {
        return Err(PackingError::ColorCountMismatch);
    }
    if destination.len() < vertices.len() * PRIMARY_RECORD_STRIDE {
        return Err(PackingError::DestinationTooShort);
    }

    const FACTOR_BYTES: [u8; 4] = [0, 127, 255, 255];
    let mut staged = [[0u8; 10]; MAX_PACKED_VERTEX_CHUNK];
    let mut last: Option<ResolvedColorWord> = None;
    for (i, (vertex, color)) in vertices.iter().zip(colors).enumerate() {
        if vertex.flags & 0x8010 != 0 {
            return Err(PackingError::UnsupportedFlags { vertex: i });
        }
        let selected_style = vertex.styles[usize::from((vertex.flags >> 5) & 1)];
        if selected_style == 0 {
            return Err(PackingError::ZeroSelectedStyle { vertex: i });
        }
        if color.style_id != selected_style {
            return Err(PackingError::ColorStyleMismatch { vertex: i });
        }
        if let Some(previous) = last {
            if previous.style_id == selected_style && previous.word != color.word {
                return Err(PackingError::InconsistentCachedColor { vertex: i });
            }
        }
        last = Some(*color);
        let [x, y] = pack_primary_point_finite_ieee_nearest(vertex.position)
            .map_err(|error| PackingError::Arithmetic { vertex: i, error })?;
        let record = &mut staged[i];
        record[0..2].copy_from_slice(&x.to_le_bytes());
        record[2..4].copy_from_slice(&y.to_le_bytes());
        record[4..8].copy_from_slice(&color.word.to_le_bytes());
        record[8] = FACTOR_BYTES[usize::from(vertex.flags & 3)];
        record[9] = FACTOR_BYTES[usize::from((vertex.flags >> 2) & 3)];
    }
    for (record, output) in staged[..vertices.len()]
        .iter()
        .zip(destination.chunks_exact_mut(PRIMARY_RECORD_STRIDE))
    {
        output[..10].copy_from_slice(record);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> PackingProfile {
        PackingProfile {
            format: EXACT_PRIMARY_FORMAT,
            forced_style: 0,
            morph_owner_present: false,
            morph_factor: 0.0,
        }
    }
    fn vertex(position: [f32; 2], flags: u16) -> GeneratedVertex {
        GeneratedVertex {
            position,
            local_ordinal: 31,
            styles: [1, 2],
            flags,
            mesh: 7,
        }
    }
    fn color(style_id: u16, word: u32) -> ResolvedColorWord {
        ResolvedColorWord { style_id, word }
    }

    #[test]
    fn exact_bytes_negative_rounding_style_choice_padding_and_tail() {
        let vertices = [vertex([-32764.0, 32764.0], 0x0a), vertex([-0.5, 1.5], 0x21)];
        let saved = vertices;
        let colors = [color(1, 0x12345678), color(2, 0xa1b2c3d4)];
        let mut out = [0x9b; 31];
        write_primary_records(&vertices, &colors, profile(), &mut out).unwrap();
        // Negative values use floor(x-0.5), including an integral negative input.
        assert_eq!(
            &out[..24],
            &[
                0x03, 0x80, 0xfc, 0x7f, 0x78, 0x56, 0x34, 0x12, 0xff, 0xff, 0x9b, 0x9b, 0xff, 0xff,
                0x02, 0x00, 0xd4, 0xc3, 0xb2, 0xa1, 0x7f, 0x00, 0x9b, 0x9b
            ]
        );
        assert_eq!(&out[24..], &[0x9b; 7]);
        assert_eq!(vertices, saved);
    }

    #[test]
    fn all_factor_entries_and_negative_zero_are_explicit() {
        for low in 0..4u16 {
            for high in 0..4u16 {
                let vertices = [vertex([-0.0, 0.0], low | (high << 2))];
                let mut out = [0xa5; 12];
                write_primary_records(&vertices, &[color(1, 0)], profile(), &mut out).unwrap();
                let expected = [0, 127, 255, 255];
                assert_eq!(&out[..8], &[0; 8]);
                assert_eq!(out[8], expected[low as usize]);
                assert_eq!(out[9], expected[high as usize]);
                assert_eq!(&out[10..], &[0xa5; 2]);
            }
        }
    }

    #[test]
    fn late_rejections_are_atomic_and_cache_words_are_checked() {
        let first = vertex([1.0, 2.0], 0x0a);
        for (second, supplied, expected) in [
            (
                vertex([f32::NAN, 0.0], 0x0a),
                color(1, 7),
                PackingError::Arithmetic {
                    vertex: 1,
                    error: MathError::NonFinite("primary packing input"),
                },
            ),
            (
                vertex([0.0, 0.0], 0x10),
                color(1, 7),
                PackingError::UnsupportedFlags { vertex: 1 },
            ),
            (
                vertex([0.0, 0.0], 0x8000),
                color(1, 7),
                PackingError::UnsupportedFlags { vertex: 1 },
            ),
            (
                vertex([0.0, 0.0], 0x20),
                color(1, 7),
                PackingError::ColorStyleMismatch { vertex: 1 },
            ),
            (
                vertex([0.0, 0.0], 0x0a),
                color(1, 8),
                PackingError::InconsistentCachedColor { vertex: 1 },
            ),
        ] {
            let mut out = [0x6e; 29];
            assert_eq!(
                write_primary_records(
                    &[first, second],
                    &[color(1, 7), supplied],
                    profile(),
                    &mut out
                ),
                Err(expected)
            );
            assert_eq!(out, [0x6e; 29]);
        }
        let mut zero_style = first;
        zero_style.styles[0] = 0;
        let mut out = [0x6e; 24];
        assert_eq!(
            write_primary_records(
                &[first, zero_style],
                &[color(1, 7), color(0, 7)],
                profile(),
                &mut out
            ),
            Err(PackingError::ZeroSelectedStyle { vertex: 1 })
        );
        assert_eq!(out, [0x6e; 24]);
    }

    #[test]
    fn exact_layout_and_profile_required_even_without_vertices() {
        let mut p = profile();
        let mut out = [0x41; 12];
        p.format.stride = 24;
        assert_eq!(
            write_primary_records(&[], &[], p, &mut out),
            Err(PackingError::UnsupportedFormat)
        );
        p = profile();
        p.format.elements[1].0 = 5;
        assert_eq!(
            write_primary_records(&[], &[], p, &mut out),
            Err(PackingError::UnsupportedFormat)
        );
        for p in [
            PackingProfile {
                forced_style: 1,
                ..profile()
            },
            PackingProfile {
                morph_owner_present: true,
                morph_factor: 0.5,
                ..profile()
            },
            PackingProfile {
                morph_factor: f32::NAN,
                ..profile()
            },
        ] {
            assert_eq!(
                write_primary_records(&[], &[], p, &mut out),
                Err(PackingError::UnsupportedProfile)
            );
        }
        write_primary_records(
            &[],
            &[],
            PackingProfile {
                morph_owner_present: true,
                morph_factor: -0.0,
                ..profile()
            },
            &mut out,
        )
        .unwrap();
        assert_eq!(out, [0x41; 12]);
    }

    #[test]
    fn chunk_and_length_limits_leave_every_byte_unchanged() {
        let vertices = [vertex([0.0, 0.0], 0x0a); 257];
        let colors = [color(1, 0x10203040); 257];
        let mut out = [0x77; 257 * 12 + 1];
        assert_eq!(
            write_primary_records(&vertices, &colors, profile(), &mut out),
            Err(PackingError::TooManyVertices)
        );
        assert_eq!(
            write_primary_records(&vertices[..2], &colors[..1], profile(), &mut out),
            Err(PackingError::ColorCountMismatch)
        );
        assert_eq!(
            write_primary_records(&vertices[..2], &colors[..2], profile(), &mut out[..23]),
            Err(PackingError::DestinationTooShort)
        );
        assert_eq!(out, [0x77; 257 * 12 + 1]);
        write_primary_records(&vertices[..256], &colors[..256], profile(), &mut out).unwrap();
        assert_eq!(&out[256 * 12..], &[0x77; 13]);
        for chunk in out[..256 * 12].chunks_exact(12) {
            assert_eq!(
                chunk,
                &[0, 0, 0, 0, 0x40, 0x30, 0x20, 0x10, 0xff, 0xff, 0x77, 0x77]
            );
        }
    }
}
