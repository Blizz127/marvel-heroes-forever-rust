//! Conditional native primary tessellation for one admitted rectangular contour.
//!
//! This expands the recovered zero-tolerance, non-AA scan/monotone branches. It
//! is not general polygon tessellation or runtime Scale9 admission. All source
//! points have already passed through the caller-selected point transform.

use super::native_scale9::Point;
use super::native_scale9_geometry::PrimarySource;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectangleProfile {
    /// Actual primary option copied to T+0x1c. Only numerical zero is supported.
    pub relative_tolerance: f32,
    /// Actual half-width passed to fed60. Only numerical zero is supported.
    pub antialias_half_width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TessellationError {
    UnsupportedProfile,
    UnsupportedSource,
    UnsupportedStyles,
    NonFinite,
    SubnormalSensitive,
    SentinelBoundary,
    DegenerateArithmetic,
    UnsupportedBranch,
}
impl fmt::Display for TessellationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnsupportedProfile => "rectangle replay requires zero tolerance and non-AA width",
            Self::UnsupportedSource => {
                "rectangle replay requires one exact five-point rectangular path"
            }
            Self::UnsupportedStyles => {
                "rectangle replay requires native styles [0,1] and resource style 1"
            }
            Self::NonFinite => "nonfinite rectangle arithmetic",
            Self::SubnormalSensitive => "subnormal-sensitive rectangle arithmetic",
            Self::SentinelBoundary => "rectangle crosses native initial-coordinate sentinel",
            Self::DegenerateArithmetic => {
                "rectangle orientation is degenerate after native arithmetic"
            }
            Self::UnsupportedBranch => "rectangle left the recovered scan branch",
        })
    }
}
impl std::error::Error for TessellationError {}
pub type Result<T> = std::result::Result<T, TessellationError>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScanChain {
    pub source_vertex: u32,
    pub direction: i16,
    pub slope: f32,
    /// Inclusive interval in the native edge array, in construction order.
    pub edge_range: [u32; 2],
    pub start: Point,
    pub styles: [u16; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeneratedVertex {
    pub position: Point,
    pub local_ordinal: u32,
    pub styles: [u16; 2],
    pub flags: u16,
    pub mesh: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectangleTrace {
    pub sorted_source_indices: [u32; 5],
    pub retained_y_indices: [u32; 2],
    /// Creation order, before the separate native Y/X/slope chain ordering.
    pub chains: [ScanChain; 2],
    pub ordered_chain_indices: [u32; 2],
    pub scan_bands: [[f32; 2]; 2],
    pub linked_vertex_tags: [u32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectangleMesh {
    pub vertices: [GeneratedVertex; 4],
    /// Global generated-vertex indices, ready for the separate index exporter.
    pub triangles: [[u32; 3]; 2],
    /// Exact seven-u32 native descriptor words, including the empty descriptor0.
    pub descriptors: [[u32; 7]; 2],
    pub trace: RectangleTrace,
}

const INITIAL_COORDINATE: f32 = f32::from_bits(0xf149_f2ca);
const TAG_INDEX_MASK: u32 = 0x0fff_ffff;

fn checked(value: f32) -> Result<f32> {
    if !value.is_finite() {
        Err(TessellationError::NonFinite)
    } else if value.is_subnormal() {
        Err(TessellationError::SubnormalSensitive)
    } else {
        Ok(value)
    }
}
fn operation(exact: f64, rounded: f32) -> Result<f32> {
    if exact != 0.0 && exact.abs() < f64::from(f32::MIN_POSITIVE) {
        return Err(TessellationError::SubnormalSensitive);
    }
    checked(rounded)
}
fn sub(a: f32, b: f32) -> Result<f32> {
    operation(f64::from(a) - f64::from(b), a - b)
}
fn mul(a: f32, b: f32) -> Result<f32> {
    operation(f64::from(a) * f64::from(b), a * b)
}
fn add(a: f32, b: f32) -> Result<f32> {
    operation(f64::from(a) + f64::from(b), a + b)
}
fn div(a: f32, b: f32) -> Result<f32> {
    operation(f64::from(a) / f64::from(b), a / b)
}
fn same(a: f32, b: f32) -> bool {
    a.to_bits() == b.to_bits()
}

// The recovered <=9-element f6e20 branch swaps adjacent items only on strict Y >.
fn source_order(points: &[Point; 5]) -> [u32; 5] {
    let mut order = [0, 1, 2, 3, 4];
    for i in 1..order.len() {
        let mut j = i;
        while j != 0 && points[order[j - 1] as usize][1] > points[order[j] as usize][1] {
            order.swap(j - 1, j);
            j -= 1;
        }
    }
    order
}

fn make_chain(points: &[Point; 5], start: usize, direction: i16, edge: u32) -> Result<ScanChain> {
    let next = (start as isize + direction as isize) as usize;
    let a = points[start];
    let b = points[next];
    Ok(ScanChain {
        source_vertex: start as u32,
        direction,
        slope: div(sub(b[0], a[0])?, sub(b[1], a[1])?)?,
        edge_range: [edge, edge],
        start: a,
        styles: [0, 1],
    })
}

// 103bb0's two directional passes. The admission below guarantees every
// increasing run has one edge; a longer run is rejected, not approximated.
fn construct_chains(points: &[Point; 5]) -> Result<[ScanChain; 2]> {
    let mut found = [None, None];
    let mut count = 0;
    for direction in [1_i16, -1] {
        let mut i = if direction == 1 { 0_isize } else { 4_isize };
        while (direction == 1 && i < 4) || (direction == -1 && i > 0) {
            let j = (i + direction as isize) as usize;
            let at_boundary = i == if direction == 1 { 0 } else { 4 };
            let begins_run = points[j][1] > points[i as usize][1]
                && (at_boundary
                    || points[(i - direction as isize) as usize][1] >= points[i as usize][1]);
            if begins_run {
                let beyond = j as isize + direction as isize;
                if (0..5).contains(&beyond) && points[beyond as usize][1] > points[j][1] {
                    return Err(TessellationError::UnsupportedBranch);
                }
                if count == 2 {
                    return Err(TessellationError::UnsupportedBranch);
                }
                found[count] = Some(make_chain(points, i as usize, direction, count as u32)?);
                count += 1;
            }
            i += direction as isize;
        }
    }
    match found {
        [Some(a), Some(b)] => Ok([a, b]),
        _ => Err(TessellationError::UnsupportedBranch),
    }
}

fn chain_less(a: ScanChain, b: ScanChain) -> bool {
    if a.start[1] != b.start[1] {
        a.start[1] < b.start[1]
    } else if a.start[0] != b.start[0] {
        a.start[0] < b.start[0]
    } else {
        a.slope < b.slope
    }
}

// Exact recovered three-node branch of10cbf0: accumulate cross products about
// last point, pick forward order iff the f32 sum is strictly positive.
fn triangle3(tags: [u32; 3], positions: &[Point; 4]) -> Result<[u32; 3]> {
    let indices = tags.map(|tag| tag & TAG_INDEX_MASK);
    let [a, b, c] = indices.map(|i| positions[i as usize]);
    let dy = sub(c[1], a[1])?;
    let dx = sub(c[0], a[0])?;
    let cross = sub(mul(sub(b[0], c[0])?, dy)?, mul(sub(b[1], c[1])?, dx)?)?;
    let area = add(0.0, cross)?;
    if area == 0.0 {
        return Err(TessellationError::DegenerateArithmetic);
    }
    Ok(if area > 0.0 {
        indices
    } else {
        [indices[2], indices[1], indices[0]]
    })
}

/// Expand the proved fresh/reset mode-1 rectangle route. Requires one path in
/// the exact low-left→low-right→high-right→high-left→low-left order, [0,1]
/// native styles and resource-presence bit1. Coordinates must agree bitwise
/// along the rectangle, remain above native sentinels, and pass finite IEEE
/// binary32/nearest/no-subnormal-sensitive intermediate checks.
///
/// The zero options are explicit supplied native-role inputs, not inferred from
/// a scene or default renderer configuration. No native allocation failure,
/// general contour, AA, UV, normalization or GPU format behavior is emulated.
/// Source state is borrowed and never changed. Output storage is fixed-size.
pub fn tessellate_rectangle(
    source: &PrimarySource,
    profile: RectangleProfile,
) -> Result<RectangleMesh> {
    if profile.relative_tolerance != 0.0 || profile.antialias_half_width != 0.0 {
        return Err(TessellationError::UnsupportedProfile);
    }
    let points: &[Point; 5] = source
        .points()
        .try_into()
        .map_err(|_| TessellationError::UnsupportedSource)?;
    let [path] = source.paths() else {
        return Err(TessellationError::UnsupportedSource);
    };
    if path.first != 0 || path.last != 4 {
        return Err(TessellationError::UnsupportedSource);
    }
    if path.styles != [0, 1] || source.max_style_id() != 1 || !source.style_has_resource(1) {
        return Err(TessellationError::UnsupportedStyles);
    }
    for point in points {
        checked(point[0])?;
        checked(point[1])?;
    }
    let [xl, yl] = points[0];
    let [xr, yh] = points[2];
    if !(xl < xr && yl < yh)
        || !same(points[1][0], xr)
        || !same(points[1][1], yl)
        || !same(points[3][0], xl)
        || !same(points[3][1], yh)
        || !same(points[4][0], xl)
        || !same(points[4][1], yl)
    {
        return Err(TessellationError::UnsupportedSource);
    }
    if xl <= INITIAL_COORDINATE || yl <= INITIAL_COORDINATE {
        return Err(TessellationError::SentinelBoundary);
    }

    let sorted = source_order(points);
    let mut previous_y = INITIAL_COORDINATE;
    let mut retained = [0_u32; 2];
    let mut retained_count = 0;
    for index in sorted {
        let y = points[index as usize][1];
        if sub(y, previous_y)? > mul(y.abs(), profile.relative_tolerance)? {
            if retained_count == 2 {
                return Err(TessellationError::UnsupportedBranch);
            }
            retained[retained_count] = index;
            retained_count += 1;
            previous_y = y;
        } else if !same(y, previous_y) {
            return Err(TessellationError::UnsupportedBranch);
        }
    }
    if retained_count != 2 {
        return Err(TessellationError::UnsupportedBranch);
    }
    let chains = construct_chains(points)?;
    let ordered = if chain_less(chains[1], chains[0]) {
        [1, 0]
    } else {
        [0, 1]
    };
    let left = chains[ordered[0] as usize];
    let right = chains[ordered[1] as usize];
    if left.direction != -1 || right.direction != 1 || left.slope != 0.0 || right.slope != 0.0 {
        return Err(TessellationError::UnsupportedBranch);
    }
    // 103f60 advances these vertical chains through [yl,yh] then[yh,yh].
    // Materialization uses exact edge endpoints on both passes; no interpolation.
    let positions = [
        left.start,
        right.start,
        points[(left.source_vertex as i32 + i32::from(left.direction)) as usize],
        points[(right.source_vertex as i32 + i32::from(right.direction)) as usize],
    ];
    let tags = [0x8000_0000, 1, 0x8000_0002, 3];
    let mut triangles = [[0; 3]; 2];
    let mut emitted = 0;
    let mut stack = [tags[0], tags[1], 0];
    for tag in tags.into_iter().skip(2) {
        let last = stack[1];
        stack[2] = tag;
        if ((last ^ tag) >> 31) == 0 {
            return Err(TessellationError::UnsupportedBranch);
        }
        triangles[emitted] = triangle3(stack, &positions)?;
        emitted += 1;
        stack = [last, tag, 0];
    }
    // Fresh resource style1 creates descriptor1 after the empty descriptor0.
    // All six corner visits reuse the same four vertices for mesh1/style1.
    let vertices = std::array::from_fn(|i| GeneratedVertex {
        position: positions[i],
        local_ordinal: i as u32,
        styles: [1, 1],
        flags: 0x0a,
        mesh: 1,
    });
    Ok(RectangleMesh {
        vertices,
        triangles,
        descriptors: [[0; 7], [1, 1, 1, 0x8008, 0x8000, 0, 4]],
        trace: RectangleTrace {
            sorted_source_indices: sorted,
            retained_y_indices: retained,
            chains,
            ordered_chain_indices: ordered,
            scan_bands: [[yl, yh], [yh, yh]],
            linked_vertex_tags: tags,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::native_scale9::{Matrix2x4, NativeScale9Context, RectF32};
    use crate::gfx::native_scale9_geometry::{export_triangle_indices, StyleSide};

    const PROFILE: RectangleProfile = RectangleProfile {
        relative_tolerance: 0.0,
        antialias_half_width: 0.0,
    };
    fn context() -> NativeScale9Context {
        NativeScale9Context::from_parts(
            Matrix2x4::IDENTITY,
            RectF32::from_coordinates([1.0, 1.0, 2.0, 2.0]).unwrap(),
            [Matrix2x4::IDENTITY; 9],
            Matrix2x4::IDENTITY,
        )
    }
    fn points(bounds: [f32; 4]) -> [Point; 5] {
        let [xl, yl, xr, yh] = bounds;
        [[xl, yl], [xr, yl], [xr, yh], [xl, yh], [xl, yl]]
    }
    fn source(input: &[Point], style: u32, resource: bool) -> PrimarySource {
        let mut result = PrimarySource::new();
        result
            .submit_straight_path(
                &context(),
                input,
                [
                    StyleSide {
                        id: 0,
                        resource_non_null: false,
                    },
                    StyleSide {
                        id: style,
                        resource_non_null: resource,
                    },
                ],
            )
            .unwrap();
        result
    }

    #[test]
    fn recovered_scan_monotone_and_triangle_hand_trace() {
        let input = source(&points([0.0, 0.0, 6.0, 4.0]), 1, true);
        let mesh = tessellate_rectangle(&input, PROFILE).unwrap();
        assert_eq!(mesh.trace.sorted_source_indices, [0, 1, 4, 2, 3]);
        assert_eq!(mesh.trace.retained_y_indices, [0, 2]);
        assert_eq!(
            mesh.trace
                .chains
                .map(|c| (c.source_vertex, c.direction, c.edge_range)),
            [(1, 1, [0, 0]), (4, -1, [1, 1])]
        );
        assert!(mesh
            .trace
            .chains
            .iter()
            .all(|c| c.slope.to_bits() == 0 && c.styles == [0, 1]));
        assert_eq!(mesh.trace.ordered_chain_indices, [1, 0]);
        assert_eq!(mesh.trace.scan_bands, [[0.0, 4.0], [4.0, 4.0]]);
        assert_eq!(
            mesh.trace.linked_vertex_tags,
            [0x80000000, 1, 0x80000002, 3]
        );
        assert_eq!(
            mesh.vertices.map(|v| v.position),
            [[0.0, 0.0], [6.0, 0.0], [0.0, 4.0], [6.0, 4.0]]
        );
        assert_eq!(mesh.triangles, [[0, 1, 2], [3, 2, 1]]);
        assert_eq!(mesh.descriptors, [[0; 7], [1, 1, 1, 0x8008, 0x8000, 0, 4]]);
        for (i, v) in mesh.vertices.iter().enumerate() {
            assert_eq!(
                (v.local_ordinal, v.styles, v.flags, v.mesh),
                (i as u32, [1, 1], 10, 1)
            );
        }
        assert_eq!(
            export_triangle_indices(
                &mesh.vertices.map(|v| v.local_ordinal),
                &mesh.triangles,
                0,
                2
            )
            .unwrap(),
            [0, 1, 2, 3, 2, 1]
        );
    }

    #[test]
    fn translated_positive_and_negative_coordinates_follow_same_native_order() {
        for bounds in [[3.0, 7.0, 11.0, 13.0], [-6.0, -9.0, -2.0, -1.0]] {
            let input = source(&points(bounds), 1, true);
            let mesh = tessellate_rectangle(&input, PROFILE).unwrap();
            let [xl, yl, xr, yh] = bounds;
            assert_eq!(
                mesh.vertices.map(|v| v.position),
                [[xl, yl], [xr, yl], [xl, yh], [xr, yh]]
            );
            assert_eq!(mesh.triangles, [[0, 1, 2], [3, 2, 1]]);
        }
    }

    #[test]
    fn unsupported_options_styles_and_resource_presence_reject() {
        let p = points([0.0, 0.0, 6.0, 4.0]);
        let input = source(&p, 1, true);
        for profile in [
            RectangleProfile {
                relative_tolerance: 1e-6,
                ..PROFILE
            },
            RectangleProfile {
                antialias_half_width: 0.5,
                ..PROFILE
            },
            RectangleProfile {
                relative_tolerance: f32::NAN,
                ..PROFILE
            },
        ] {
            assert_eq!(
                tessellate_rectangle(&input, profile),
                Err(TessellationError::UnsupportedProfile)
            );
        }
        assert_eq!(
            tessellate_rectangle(&source(&p, 7, true), PROFILE),
            Err(TessellationError::UnsupportedStyles)
        );
        assert_eq!(
            tessellate_rectangle(&source(&p, 1, false), PROFILE),
            Err(TessellationError::UnsupportedStyles)
        );
    }

    #[test]
    fn winding_nonrectangular_open_and_multiple_paths_reject() {
        let clockwise = points([0.0, 0.0, 6.0, 4.0]);
        let reversed = [
            clockwise[0],
            clockwise[3],
            clockwise[2],
            clockwise[1],
            clockwise[4],
        ];
        let mut skewed = clockwise;
        skewed[2][0] = 7.0;
        for p in [reversed, skewed] {
            assert_eq!(
                tessellate_rectangle(&source(&p, 1, true), PROFILE),
                Err(TessellationError::UnsupportedSource)
            );
        }
        assert_eq!(
            tessellate_rectangle(&source(&clockwise[..4], 1, true), PROFILE),
            Err(TessellationError::UnsupportedSource)
        );
        let mut multiple = source(&clockwise, 1, true);
        multiple
            .submit_straight_path(
                &context(),
                &clockwise,
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
        assert_eq!(
            tessellate_rectangle(&multiple, PROFILE),
            Err(TessellationError::UnsupportedSource)
        );
    }

    #[test]
    fn native_sentinel_and_finite_intermediate_limits_are_required() {
        let boundary = source(&points([INITIAL_COORDINATE, 0.0, 0.0, 4.0]), 1, true);
        assert_eq!(
            tessellate_rectangle(&boundary, PROFILE),
            Err(TessellationError::SentinelBoundary)
        );
        let y_boundary = source(&points([0.0, INITIAL_COORDINATE, 6.0, 0.0]), 1, true);
        assert_eq!(
            tessellate_rectangle(&y_boundary, PROFILE),
            Err(TessellationError::SentinelBoundary)
        );
        let overflow = source(&points([0.0, 0.0, 1e30, 1e30]), 1, true);
        assert_eq!(
            tessellate_rectangle(&overflow, PROFILE),
            Err(TessellationError::NonFinite)
        );
        let tiny = source(
            &points([0.0, 0.0, f32::MIN_POSITIVE, f32::MIN_POSITIVE]),
            1,
            true,
        );
        assert_eq!(
            tessellate_rectangle(&tiny, PROFILE),
            Err(TessellationError::SubnormalSensitive)
        );
    }
}
