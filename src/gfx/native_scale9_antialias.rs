//! Conditional native mode-1 antialias geometry for one rectangular contour.
//!
//! Explicit native-role options and the selected branch limits are recorded in
//! docs/SCALE9_RECTANGLE_AA.md. This module does not choose runtime settings,
//! bind a provider, normalize packed positions or enable a renderer.

use super::native_scale9::Point;
use super::native_scale9_geometry::PrimarySource;
use super::native_scale9_tessellation::{
    GeneratedVertex, RectangleProfile, RectangleTrace, TessellationError, tessellate_rectangle,
};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectangleAaProfile {
    /// Actual primary T+0x1c option; must preserve the two source Y rows.
    pub relative_tolerance: f32,
    /// Actual positive width at T+0x3ac, after the native caller's selection.
    pub half_width: f32,
    /// Actual separate T+0x3b4 option used by the corner determinant test.
    /// This descriptive name is not a recovered original SDK member name.
    pub corner_tolerance: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AntialiasError {
    Source(TessellationError),
    UnsupportedProfile,
    NonFinite,
    SubnormalSensitive,
    UnsupportedCornerBranch,
    TriangleCorrectionRequired,
}
impl fmt::Display for AntialiasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => error.fmt(f),
            Self::UnsupportedProfile => f.write_str(
                "AA rectangle requires explicit positive width and nonnegative corner tolerance",
            ),
            Self::NonFinite => f.write_str("nonfinite native AA rectangle arithmetic"),
            Self::SubnormalSensitive => {
                f.write_str("subnormal-sensitive native AA rectangle arithmetic")
            }
            Self::UnsupportedCornerBranch => {
                f.write_str("rectangle corner requires an unsupported fallback, clamp or split")
            }
            Self::TriangleCorrectionRequired => {
                f.write_str("rectangle requires native triangle correction")
            }
        }
    }
}
impl std::error::Error for AntialiasError {}
impl From<TessellationError> for AntialiasError {
    fn from(error: TessellationError) -> Self {
        Self::Source(error)
    }
}
pub type Result<T> = std::result::Result<T, AntialiasError>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectangleAaMesh {
    /// Native generated-index domain: four unused source records, then eight
    /// alternating interior/exterior records. Unused records keep mesh/ordinal
    /// sentinels; they are not silently removed or renumbered.
    pub vertices: [GeneratedVertex; 12],
    pub triangles: [[u32; 3]; 10],
    /// Descriptor0 is empty; descriptor1 owns eight vertices.
    pub descriptors: [[u32; 7]; 2],
    pub source_trace: RectangleTrace,
}

fn checked(value: f32) -> Result<f32> {
    if !value.is_finite() {
        Err(AntialiasError::NonFinite)
    } else if value.is_subnormal() {
        Err(AntialiasError::SubnormalSensitive)
    } else {
        Ok(value)
    }
}
fn operation(exact: f64, rounded: f32) -> Result<f32> {
    if exact != 0.0 && exact.abs() < f64::from(f32::MIN_POSITIVE) {
        return Err(AntialiasError::SubnormalSensitive);
    }
    checked(rounded)
}
fn add(a: f32, b: f32) -> Result<f32> {
    operation(f64::from(a) + f64::from(b), a + b)
}
fn sub(a: f32, b: f32) -> Result<f32> {
    operation(f64::from(a) - f64::from(b), a - b)
}
fn mul(a: f32, b: f32) -> Result<f32> {
    operation(f64::from(a) * f64::from(b), a * b)
}
fn div(a: f32, b: f32) -> Result<f32> {
    operation(f64::from(a) / f64::from(b), a / b)
}
fn sqrt(value: f32) -> Result<f32> {
    checked(value.sqrt())
}

// 10e4c0 computes each outgoing incidence key before sorting its two records.
// All four rectangle corners use distinct axis keys, so the style tie branch
// is excluded. Preserve the squared-length arithmetic and its numeric limits.
fn angle_key(from: Point, to: Point) -> Result<f32> {
    let dx = sub(to[0], from[0])?;
    let dy = sub(to[1], from[1])?;
    let dx_squared = mul(dx, dx)?;
    let dy_squared = mul(dy, dy)?;
    let denominator = mul(add(dy_squared, dx_squared)?, 2.0)?;
    if denominator == 0.0 {
        return Err(AntialiasError::UnsupportedCornerBranch);
    }
    let mut key = div(dx_squared, denominator)?;
    if dx < 0.0 {
        key = -key;
    }
    if dy > 0.0 {
        key = sub(1.0, key)?;
    }
    sub(key, 0.5)
}

// Exact selected scalar route through 1003d0/f6060. Ordinary finite sqrtf,
// IEEE binary32 nearest, separate operations and successful valid storage are
// required. A false native return also covers fallbacks; admit only the actual
// ordinary line intersection within its native distance limit.
fn corner(a: Point, b: Point, c: Point, width: f32, tolerance: f32) -> Result<Point> {
    let dy0 = sub(b[1], a[1])?;
    let dy0_squared = mul(dy0, dy0)?;
    let dx0 = sub(b[0], a[0])?;
    let dx0_squared = mul(dx0, dx0)?;
    let length0 = sqrt(add(dy0_squared, dx0_squared)?)?;
    let dx1 = sub(c[0], b[0])?;
    let dy1 = sub(c[1], b[1])?;
    let dx1_squared = mul(dx1, dx1)?;
    let dy1_squared = mul(dy1, dy1)?;
    let length1 = sqrt(add(dy1_squared, dx1_squared)?)?;
    if length0 <= 0.0 || length1 <= 0.0 {
        return Err(AntialiasError::UnsupportedCornerBranch);
    }

    let denominator = mul(mul(length0, length1)?, 2.0)?;
    let qx = sub(c[0], b[0])?;
    let py = sub(b[1], a[1])?;
    let px = sub(b[0], a[0])?;
    let qy = sub(c[1], b[1])?;
    let dot_x = mul(qx, px)?;
    let cross0 = mul(qx, py)?;
    let cross1 = mul(qy, px)?;
    let dot_y = mul(qy, py)?;
    let mut turn = div(add(dot_y, dot_x)?, denominator)?;
    if cross0 > cross1 {
        turn = sub(1.0, turn)?;
    }
    turn = sub(turn, 0.5)?;
    let negative = turn < 0.0;

    let mut n1y = sub(b[0], c[0])?;
    let near = mul(add(length1, length0)?, tolerance)?;
    n1y = mul(n1y, width)?;
    let mut n1x = sub(c[1], b[1])?;
    let mut n0x = sub(b[1], a[1])?;
    let mut n0y = sub(a[0], b[0])?;
    n1y = div(n1y, length1)?;
    n1x = mul(n1x, width)?;
    n0x = mul(n0x, width)?;
    n0y = mul(n0y, width)?;
    n1x = div(n1x, length1)?;
    n0x = div(n0x, length0)?;
    let absolute_turn = turn.abs();
    n0y = div(n0y, length0)?;
    if absolute_turn < 0.125 {
        return Err(AntialiasError::UnsupportedCornerBranch);
    }

    let q1y = add(c[1], n1y)?;
    let q1x = add(c[0], n1x)?;
    let q0x = add(b[0], n1x)?;
    let q0y = add(b[1], n1y)?;
    let p1y = add(b[1], n0y)?;
    let p0y = add(a[1], n0y)?;
    let p0x = add(a[0], n0x)?;
    let d0y = sub(p1y, p0y)?;
    let d1y = sub(q1y, q0y)?;
    let p1x = add(b[0], n0x)?;
    let d0x = sub(p1x, p0x)?;
    let d1x = sub(q1x, q0x)?;
    let determinant = sub(mul(d1y, d0x)?, mul(d1x, d0y)?)?;
    if determinant == 0.0 || near > determinant.abs() {
        return Err(AntialiasError::UnsupportedCornerBranch);
    }
    let numerator0 = mul(sub(p0y, q0y)?, d1x)?;
    let numerator1 = mul(sub(p0x, q0x)?, d1y)?;
    let factor = div(sub(numerator0, numerator1)?, determinant)?;
    let mut y = mul(d0y, factor)?;
    let mut x = mul(d0x, factor)?;
    x = add(x, p0x)?;
    y = add(y, p0y)?;
    let dy = sub(y, b[1])?;
    let dx = sub(x, b[0])?;
    let dy_squared = mul(dy, dy)?;
    let dx_squared = mul(dx, dx)?;
    let distance = sqrt(add(dy_squared, dx_squared)?)?;
    let limit = if negative {
        mul(width, 4.0)?
    } else {
        div(
            if length0 < length1 { length0 } else { length1 },
            absolute_turn,
        )?
    };
    if distance > limit {
        return Err(AntialiasError::UnsupportedCornerBranch);
    }
    Ok([x, y])
}

fn original_triangle_gates(original: [Point; 3], inner: [Point; 3]) -> Result<()> {
    let [a, b, c] = original;
    let [ia, ib, ic] = inner;
    let g0 = sub(
        mul(sub(ia[0], c[0])?, sub(c[1], b[1])?)?,
        mul(sub(ia[1], c[1])?, sub(c[0], b[0])?)?,
    )?;
    let g1 = sub(
        mul(sub(a[1], c[1])?, sub(ib[0], a[0])?)?,
        mul(sub(ib[1], a[1])?, sub(a[0], c[0])?)?,
    )?;
    let g2 = sub(
        mul(sub(ic[0], b[0])?, sub(b[1], a[1])?)?,
        mul(sub(ic[1], b[1])?, sub(b[0], a[0])?)?,
    )?;
    if [g0, g1, g2].into_iter().any(|value| value >= 0.0) {
        return Err(AntialiasError::TriangleCorrectionRequired);
    }
    Ok(())
}

// 1009d0 uses mapped interior coordinates. Strictly negative q skips all its
// repair writes; zero is not admitted as an already resolved triangle.
fn resolved_triangle_gate([a, b, c]: [Point; 3]) -> Result<()> {
    let q = sub(
        mul(sub(c[0], b[0])?, sub(b[1], a[1])?)?,
        mul(sub(b[0], a[0])?, sub(c[1], b[1])?)?,
    )?;
    if q >= 0.0 {
        Err(AntialiasError::TriangleCorrectionRequired)
    } else {
        Ok(())
    }
}

/// Replay only the recovered no-repair AA rectangle branch. The three options
/// are supplied values; no current native renderer state is inferred. Source
/// coordinates remain unchanged on success and error.
pub fn tessellate_rectangle_aa(
    source: &PrimarySource,
    profile: RectangleAaProfile,
) -> Result<RectangleAaMesh> {
    if !profile.half_width.is_finite()
        || profile.half_width <= 0.0
        || !profile.corner_tolerance.is_finite()
        || profile.corner_tolerance < 0.0
    {
        return Err(AntialiasError::UnsupportedProfile);
    }
    checked(profile.half_width)?;
    checked(profile.corner_tolerance)?;
    // Reuse the independently proved common source scan, linked-node order and
    // three-node triangulation. Its non-AA metadata is not reused as AA state.
    let base = tessellate_rectangle(
        source,
        RectangleProfile {
            relative_tolerance: profile.relative_tolerance,
            antialias_half_width: 0.0,
        },
    )?;
    let positions = base.vertices.map(|v| v.position);
    let neighbors = [[1, 2], [3, 0], [0, 3], [2, 1]];
    let mut inside = [[0.0; 2]; 4];
    let mut outside = [[0.0; 2]; 4];
    for (i, [high, low]) in neighbors.into_iter().enumerate() {
        let high_key = angle_key(positions[i], positions[high])?;
        let low_key = angle_key(positions[i], positions[low])?;
        if high_key == low_key {
            return Err(AntialiasError::UnsupportedCornerBranch);
        }
        inside[i] = corner(
            positions[high],
            positions[i],
            positions[low],
            profile.half_width,
            profile.corner_tolerance,
        )?;
        outside[i] = corner(
            positions[low],
            positions[i],
            positions[high],
            profile.half_width,
            profile.corner_tolerance,
        )?;
    }
    for triangle in base.triangles {
        original_triangle_gates(
            triangle.map(|i| positions[i as usize]),
            triangle.map(|i| inside[i as usize]),
        )?;
        resolved_triangle_gate(triangle.map(|i| inside[i as usize]))?;
    }
    let vertices = std::array::from_fn(|index| {
        if index < 4 {
            GeneratedVertex {
                position: positions[index],
                local_ordinal: u32::MAX,
                styles: [u16::MAX; 2],
                flags: 2,
                mesh: u16::MAX,
            }
        } else {
            let ordinal = index - 4;
            let original = ordinal / 2;
            let inner = ordinal % 2 == 0;
            GeneratedVertex {
                position: if inner {
                    inside[original]
                } else {
                    outside[original]
                },
                local_ordinal: ordinal as u32,
                styles: [1, 1],
                flags: if inner { 0x0a } else { 0x08 },
                mesh: 1,
            }
        }
    });
    let mut triangles = [[0; 3]; 10];
    for (index, triangle) in base.triangles.into_iter().enumerate() {
        triangles[index] = triangle.map(|i| 4 + 2 * i);
    }
    // 10bb30's sorted directed edge keys, skipping tagged reverse entries.
    // All four node pointers belong to one fresh linked-node page, so native
    // unsigned pointer order equals this logical source-index order.
    for (edge, [a, b]) in [[0, 1], [1, 3], [2, 0], [3, 2]].into_iter().enumerate() {
        let ia = 4 + 2 * a;
        let ib = 4 + 2 * b;
        let oa = ia + 1;
        let ob = ib + 1;
        triangles[2 + 2 * edge] = [ia, ib, oa];
        triangles[3 + 2 * edge] = [ob, oa, ib];
    }
    Ok(RectangleAaMesh {
        vertices,
        triangles,
        descriptors: [[0; 7], [1, 1, 1, 0x8008, 0x8000, 0, 8]],
        source_trace: base.trace,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::native_scale9::{Matrix2x4, NativeScale9Context, RectF32};
    use crate::gfx::native_scale9_geometry::{StyleSide, export_triangle_indices};

    const PROFILE: RectangleAaProfile = RectangleAaProfile {
        relative_tolerance: f32::from_bits(0x3727_c5ac),
        half_width: f32::from_bits(0x3ef3_3333),
        corner_tolerance: f32::from_bits(0x3a83_126f),
    };
    fn source([xl, yl, xr, yh]: [f32; 4]) -> PrimarySource {
        let context = NativeScale9Context::from_parts(
            Matrix2x4::IDENTITY,
            RectF32::from_coordinates([371.0, 364.0, 892.0, 896.0]).unwrap(),
            [Matrix2x4::IDENTITY; 9],
            Matrix2x4::IDENTITY,
        );
        let mut source = PrimarySource::new();
        source
            .submit_straight_path(
                &context,
                &[[xl, yl], [xr, yl], [xr, yh], [xl, yh], [xl, yl]],
                [
                    StyleSide {
                        id: 0,
                        resource_non_null: false,
                    },
                    StyleSide {
                        id: 1,
                        resource_non_null: true,
                    },
                ],
            )
            .unwrap();
        source
    }

    #[test]
    fn native_square_corner_bits_keep_cancellation_order() {
        let mesh = tessellate_rectangle_aa(&source([0.0, 0.0, 1280.0, 1280.0]), PROFILE).unwrap();
        let expected = [
            [0x3ef33000, 0x3ef33333],
            [0xbef33333, 0xbef33000],
            [0x449ff0cd, 0x3ef33000],
            [0x44a00f33, 0xbef33333],
            [0x3ef33333, 0x449ff0cd],
            [0xbef33000, 0x44a00f33],
            [0x449ff0cd, 0x449ff0cd],
            [0x44a00f33, 0x44a00f33],
        ];
        for (vertex, bits) in mesh.vertices[4..].iter().zip(expected) {
            assert_eq!(vertex.position.map(f32::to_bits), bits);
        }
        assert_ne!(
            mesh.vertices[4].position[0].to_bits(),
            PROFILE.half_width.to_bits()
        );
        assert_eq!(mesh.source_trace.retained_y_indices, [0, 2]);
    }

    #[test]
    fn global_indices_unused_sources_and_fringe_order_match_native_records() {
        let input = source([0.0, 0.0, 1280.0, 1280.0]);
        let before = (input.points().to_vec(), input.paths().to_vec());
        let mesh = tessellate_rectangle_aa(&input, PROFILE).unwrap();
        assert_eq!(
            mesh.triangles,
            [
                [4, 6, 8],
                [10, 8, 6],
                [4, 6, 5],
                [7, 5, 6],
                [6, 10, 7],
                [11, 7, 10],
                [8, 4, 9],
                [5, 9, 4],
                [10, 8, 11],
                [9, 11, 8],
            ]
        );
        assert_eq!(mesh.descriptors, [[0; 7], [1, 1, 1, 0x8008, 0x8000, 0, 8]]);
        for vertex in &mesh.vertices[..4] {
            assert_eq!(
                (
                    vertex.local_ordinal,
                    vertex.styles,
                    vertex.flags,
                    vertex.mesh
                ),
                (u32::MAX, [u16::MAX; 2], 2, u16::MAX)
            );
        }
        for (ordinal, vertex) in mesh.vertices[4..].iter().enumerate() {
            assert_eq!(
                (
                    vertex.local_ordinal,
                    vertex.styles,
                    vertex.flags,
                    vertex.mesh
                ),
                (
                    ordinal as u32,
                    [1, 1],
                    if ordinal % 2 == 0 { 10 } else { 8 },
                    1
                )
            );
        }
        assert_eq!(
            export_triangle_indices(
                &mesh.vertices.map(|v| v.local_ordinal),
                &mesh.triangles,
                0,
                10
            )
            .unwrap(),
            [
                0, 2, 4, 6, 4, 2, 0, 2, 1, 3, 1, 2, 2, 6, 3, 7, 3, 6, 4, 0, 5, 1, 5, 0, 6, 4, 7, 5,
                7, 4
            ]
        );
        assert_eq!(
            (input.points(), input.paths()),
            (before.0.as_slice(), before.1.as_slice())
        );
    }

    #[test]
    fn correction_and_corner_fallback_are_rejected_without_source_changes() {
        let input = source([0.0, 0.0, 0.5, 1.0]);
        let before = input.points().to_vec();
        assert_eq!(
            tessellate_rectangle_aa(&input, PROFILE),
            Err(AntialiasError::TriangleCorrectionRequired)
        );
        assert_eq!(input.points(), before.as_slice());
        let large_tolerance = RectangleAaProfile {
            corner_tolerance: 10000.0,
            ..PROFILE
        };
        assert_eq!(
            tessellate_rectangle_aa(&source([0.0, 0.0, 1280.0, 1280.0]), large_tolerance),
            Err(AntialiasError::UnsupportedCornerBranch)
        );
        assert_eq!(
            resolved_triangle_gate([[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]),
            Err(AntialiasError::TriangleCorrectionRequired)
        );
    }

    #[test]
    fn explicit_options_and_numeric_limits_remain_bounded() {
        let input = source([0.0, 0.0, 1280.0, 1280.0]);
        for half_width in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                tessellate_rectangle_aa(
                    &input,
                    RectangleAaProfile {
                        half_width,
                        ..PROFILE
                    }
                ),
                Err(AntialiasError::UnsupportedProfile)
            );
        }
        assert_eq!(
            tessellate_rectangle_aa(
                &input,
                RectangleAaProfile {
                    half_width: f32::from_bits(1),
                    ..PROFILE
                }
            ),
            Err(AntialiasError::SubnormalSensitive)
        );
        assert_eq!(
            tessellate_rectangle_aa(
                &input,
                RectangleAaProfile {
                    corner_tolerance: -1.0,
                    ..PROFILE
                }
            ),
            Err(AntialiasError::UnsupportedProfile)
        );
        assert!(matches!(
            tessellate_rectangle_aa(
                &source([0.0, 1.0, 6.0, 2.0]),
                RectangleAaProfile {
                    relative_tolerance: 0.5,
                    ..PROFILE
                }
            ),
            Err(AntialiasError::Source(TessellationError::UnsupportedBranch))
        ));
        assert_eq!(
            tessellate_rectangle_aa(&source([0.0, 0.0, 1e20, 1.0]), PROFILE),
            Err(AntialiasError::NonFinite)
        );
    }
}
