//! Recovered primary tessellator input staging and triangle-index transfer.
//!
//! This is not a tessellator. Source points, generated vertices and generated
//! triangle connectivity are separate domains. See `docs/SCALE9_GEOMETRY.md`.
//! Callers supply an already selected native point-transform context and native
//! provider style IDs/resource-presence facts; no scene or texture lookup occurs.

use super::native_scale9::{MathError, NativeScale9Context, Point};
use std::fmt;

/// Engineering bounds, not recovered limits on native source arrays.
pub const MAX_SOURCE_POINTS: usize = 65_535;
pub const MAX_SOURCE_PATHS: usize = 32_767;
pub const MAX_STYLE_ID: u32 = 65_535;
/// The recovered output caller submits at most 256 triangles per transfer.
pub const MAX_TRIANGLE_BATCH: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryError {
    Math(MathError),
    PointLimit,
    PathLimit,
    StyleLimit,
    TriangleBatchLimit,
    TriangleRange,
    VertexIndex,
    AllocationFailed,
}
impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Math(error) => error.fmt(f),
            Self::PointLimit => f.write_str("primary source point limit exceeded"),
            Self::PathLimit => f.write_str("primary source path limit exceeded"),
            Self::StyleLimit => f.write_str("primary source style limit exceeded"),
            Self::TriangleBatchLimit => f.write_str("primary triangle batch exceeds 256"),
            Self::TriangleRange => f.write_str("primary triangle range is invalid"),
            Self::VertexIndex => f.write_str("primary triangle refers to an absent output vertex"),
            Self::AllocationFailed => f.write_str("cannot reserve primary geometry storage"),
        }
    }
}
impl std::error::Error for GeometryError {}
impl From<MathError> for GeometryError {
    fn from(value: MathError) -> Self {
        Self::Math(value)
    }
}
pub type Result<T> = std::result::Result<T, GeometryError>;

/// Native provider ID, not a Rust style-bank index. Zero means no fill.
/// `resource_non_null` is the observed provider fill-record pointer test;
/// it is not inferred from a saved bitmap tag or a loaded texture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StyleSide {
    pub id: u32,
    pub resource_non_null: bool,
}

/// Inclusive endpoints in the submitted point array. The two style positions
/// retain native argument order; this module does not guess provider sidedness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimaryPath {
    pub first: u32,
    pub last: u32,
    pub styles: [u32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Submission {
    SkippedEqualStyles,
    DiscardedTooShort,
    Stored { path_index: usize },
}

/// Fresh/reset primary input state in the selected caller's mode 1.
/// Native 103700/103a70 retain repeated points, geometric closure and full
/// style IDs. They do not split an edge at Scale9 region boundaries.
#[derive(Debug, Default)]
pub struct PrimarySource {
    points: Vec<Point>,
    paths: Vec<PrimaryPath>,
    resource_style_bits: Vec<u32>,
    max_style: u32,
}
impl PrimarySource {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn points(&self) -> &[Point] {
        &self.points
    }
    pub fn paths(&self) -> &[PrimaryPath] {
        &self.paths
    }
    pub fn max_style_id(&self) -> u32 {
        self.max_style
    }
    pub fn has_resource_styles(&self) -> bool {
        self.resource_style_bits.iter().any(|&word| word != 0)
    }
    pub fn style_has_resource(&self, id: u32) -> bool {
        self.resource_style_bits
            .get((id / 32) as usize)
            .is_some_and(|word| word & (1 << (id & 31)) != 0)
    }
    /// Clears only the modeled input state. No native allocator is emulated.
    pub fn reset(&mut self) {
        self.points.clear();
        self.paths.clear();
        self.resource_style_bits.clear();
        self.max_style = 0;
    }

    /// Submit START followed by every straight END, including a repeated closing
    /// endpoint when supplied by the provider. The native path-reader binding is
    /// a caller obligation. Curves, strokes, morphing and alternate modes are
    /// outside this interface.
    ///
    /// Equal styles skip before transforming, as at c3f24. For unequal styles,
    /// all points are transformed before the short-path finalizer test. Errors
    /// leave observable state unchanged: this is bounded Rust failure policy,
    /// not a claim about native allocation/math failures.
    pub fn submit_straight_path(
        &mut self,
        context: &NativeScale9Context,
        source_points: &[Point],
        sides: [StyleSide; 2],
    ) -> Result<Submission> {
        if sides[0].id == sides[1].id {
            return Ok(Submission::SkippedEqualStyles);
        }
        if source_points.len() > MAX_SOURCE_POINTS
            || self.points.len() > MAX_SOURCE_POINTS - source_points.len()
        {
            return Err(GeometryError::PointLimit);
        }
        let mut transformed = Vec::new();
        transformed
            .try_reserve_exact(source_points.len())
            .map_err(|_| GeometryError::AllocationFailed)?;
        for &point in source_points {
            transformed.push(context.transform_point(point)?.position);
        }
        if transformed.len() < 2 {
            return Ok(Submission::DiscardedTooShort);
        }
        if self.paths.len() == MAX_SOURCE_PATHS {
            return Err(GeometryError::PathLimit);
        }
        let greatest = sides[0].id.max(sides[1].id);
        if greatest > MAX_STYLE_ID {
            return Err(GeometryError::StyleLimit);
        }
        let words = (greatest / 32) as usize + 1;
        self.points
            .try_reserve(transformed.len())
            .map_err(|_| GeometryError::AllocationFailed)?;
        self.paths
            .try_reserve(1)
            .map_err(|_| GeometryError::AllocationFailed)?;
        if self.resource_style_bits.len() < words {
            self.resource_style_bits
                .try_reserve(words - self.resource_style_bits.len())
                .map_err(|_| GeometryError::AllocationFailed)?;
        }
        let first = self.points.len() as u32;
        let last = first + transformed.len() as u32 - 1;
        let path_index = self.paths.len();
        self.points.extend(transformed);
        self.paths.push(PrimaryPath {
            first,
            last,
            styles: [sides[0].id, sides[1].id],
        });
        self.resource_style_bits
            .resize(words.max(self.resource_style_bits.len()), 0);
        for side in sides {
            if side.id != 0 && side.resource_non_null {
                self.resource_style_bits[(side.id / 32) as usize] |= 1 << (side.id & 31);
            }
        }
        self.max_style = self.max_style.max(greatest);
        Ok(Submission::Stored { path_index })
    }
}

/// Native 100160 transfer from ALREADY GENERATED triangles and vertex ordinals.
/// Every triangle contains three global generated-vertex indices. Each vertex
/// supplies native +8's u32 mesh-local ordinal. Transfer preserves triangle and
/// corner order and stores the low 16 bits, exactly as the native helper does.
/// This function neither constructs connectivity nor uses PrimarySource indices.
///
/// Bounds/range failures reject atomically. The native helper trusts its caller;
/// these checks and the caller-observed 256-triangle batch cap are Rust admission.
pub fn export_triangle_indices(
    generated_vertex_ordinals: &[u32],
    generated_triangles: &[[u32; 3]],
    first_triangle: usize,
    triangle_count: usize,
) -> Result<Vec<u16>> {
    if triangle_count > MAX_TRIANGLE_BATCH {
        return Err(GeometryError::TriangleBatchLimit);
    }
    let end = first_triangle
        .checked_add(triangle_count)
        .filter(|&end| end <= generated_triangles.len())
        .ok_or(GeometryError::TriangleRange)?;
    let selected = &generated_triangles[first_triangle..end];
    for triangle in selected {
        for &index in triangle {
            if index as usize >= generated_vertex_ordinals.len() {
                return Err(GeometryError::VertexIndex);
            }
        }
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(triangle_count * 3)
        .map_err(|_| GeometryError::AllocationFailed)?;
    for triangle in selected {
        for &index in triangle {
            result.push(generated_vertex_ordinals[index as usize] as u16);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::native_scale9::{Matrix2x4, RectF32};

    fn context() -> NativeScale9Context {
        NativeScale9Context::from_parts(
            Matrix2x4::IDENTITY,
            RectF32::from_coordinates([2.0, 1.0, 4.0, 3.0]).unwrap(),
            [Matrix2x4::IDENTITY; 9],
            Matrix2x4::IDENTITY,
        )
    }
    fn styles() -> [StyleSide; 2] {
        [
            StyleSide {
                id: 0,
                resource_non_null: false,
            },
            StyleSide {
                id: 1,
                resource_non_null: true,
            },
        ]
    }

    #[test]
    fn closure_and_cross_region_edges_are_preserved_without_splitting() {
        let input = [[0.0, 0.0], [6.0, 0.0], [6.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let mut state = PrimarySource::new();
        assert_eq!(
            state
                .submit_straight_path(&context(), &input, styles())
                .unwrap(),
            Submission::Stored { path_index: 0 }
        );
        assert_eq!(state.points(), input);
        assert_eq!(
            state.paths(),
            [PrimaryPath {
                first: 0,
                last: 4,
                styles: [0, 1]
            }]
        );
        assert!(state.style_has_resource(1));
        assert!(!state.style_has_resource(0));
        assert_eq!(state.max_style_id(), 1);
    }

    #[test]
    fn endpoint_regions_transform_before_staging_without_inserting_intersections() {
        let regions = std::array::from_fn(|i| {
            Matrix2x4::from_rows([
                [1.0, 0.0, 0.0, 100.0 * i as f32],
                [0.0, 1.0, 0.0, 10.0 * i as f32],
            ])
            .unwrap()
        });
        let context = NativeScale9Context::from_parts(
            Matrix2x4::IDENTITY,
            RectF32::from_coordinates([2.0, 1.0, 4.0, 3.0]).unwrap(),
            regions,
            Matrix2x4::IDENTITY,
        );
        let source = [
            [-1.0, -1.0],
            [5.0, -1.0],
            [5.0, 4.0],
            [-1.0, 4.0],
            [2.0, 1.0],
        ];
        let mut state = PrimarySource::new();
        state
            .submit_straight_path(&context, &source, styles())
            .unwrap();
        // Strict outcodes 12,9,3,6,0 select native regions 0,2,8,6,4.
        assert_eq!(
            state.points(),
            [
                [-1.0, -1.0],
                [205.0, 19.0],
                [805.0, 84.0],
                [599.0, 64.0],
                [402.0, 41.0]
            ]
        );
        assert_eq!(state.paths()[0].last, 4);
    }

    #[test]
    fn hand_trace_skip_short_repeated_points_style_union_and_reset() {
        let mut state = PrimarySource::new();
        let one = [StyleSide {
            id: 7,
            resource_non_null: true,
        }; 2];
        assert_eq!(
            state
                .submit_straight_path(&context(), &[[f32::NAN, 0.0]], one)
                .unwrap(),
            Submission::SkippedEqualStyles
        );
        assert_eq!(
            state
                .submit_straight_path(&context(), &[[1.0, 2.0]], styles())
                .unwrap(),
            Submission::DiscardedTooShort
        );
        assert!(state.points().is_empty());
        let a = [
            StyleSide {
                id: 32,
                resource_non_null: true,
            },
            StyleSide {
                id: 63,
                resource_non_null: false,
            },
        ];
        state
            .submit_straight_path(&context(), &[[1.0, 2.0]; 2], a)
            .unwrap();
        let b = [
            StyleSide {
                id: 32,
                resource_non_null: false,
            },
            StyleSide {
                id: 1,
                resource_non_null: true,
            },
        ];
        state
            .submit_straight_path(&context(), &[[3.0, 2.0], [1.0, 2.0]], b)
            .unwrap();
        assert_eq!(
            state.paths(),
            [
                PrimaryPath {
                    first: 0,
                    last: 1,
                    styles: [32, 63]
                },
                PrimaryPath {
                    first: 2,
                    last: 3,
                    styles: [32, 1]
                }
            ]
        );
        assert_eq!(state.max_style_id(), 63);
        assert!(state.style_has_resource(32));
        assert!(state.style_has_resource(1));
        assert!(!state.style_has_resource(63));
        state.reset();
        assert!(state.paths().is_empty() && state.points().is_empty());
        assert!(!state.has_resource_styles());
        assert_eq!(state.max_style_id(), 0);
    }

    #[test]
    fn errors_do_not_publish_partial_source_state() {
        let mut state = PrimarySource::new();
        let input = [[1.0, 1.0], [2.0, 2.0]];
        state
            .submit_straight_path(&context(), &input, styles())
            .unwrap();
        assert!(matches!(
            state.submit_straight_path(&context(), &[[0.0, 0.0], [f32::INFINITY, 0.0]], styles()),
            Err(GeometryError::Math(_))
        ));
        let bad_style = [
            styles()[0],
            StyleSide {
                id: MAX_STYLE_ID + 1,
                resource_non_null: true,
            },
        ];
        assert_eq!(
            state.submit_straight_path(&context(), &input, bad_style),
            Err(GeometryError::StyleLimit)
        );
        let excessive = vec![[0.0, 0.0]; MAX_SOURCE_POINTS];
        assert_eq!(
            state.submit_straight_path(&context(), &excessive, styles()),
            Err(GeometryError::PointLimit)
        );
        assert_eq!(state.points(), input);
        assert_eq!(state.paths().len(), 1);
        assert_eq!(state.max_style_id(), 1);
    }

    #[test]
    fn triangle_transfer_uses_generated_vertex_ordinals_and_exact_corner_order() {
        let ordinals = [0x1234_0007, 2, 0xffff_ffff, 9];
        let triangles = [[0, 1, 3], [3, 0, 2], [2, 1, 0]];
        assert_eq!(
            export_triangle_indices(&ordinals, &triangles, 1, 2).unwrap(),
            [9, 7, 65535, 65535, 2, 7]
        );
        assert!(export_triangle_indices(&[], &[], 0, 0).unwrap().is_empty());
        assert_eq!(
            export_triangle_indices(&ordinals, &[[0, 4, 1]], 0, 1),
            Err(GeometryError::VertexIndex)
        );
        assert_eq!(
            export_triangle_indices(&ordinals, &triangles, usize::MAX, 1),
            Err(GeometryError::TriangleRange)
        );
        assert_eq!(
            export_triangle_indices(&ordinals, &triangles, 0, 257),
            Err(GeometryError::TriangleBatchLimit)
        );
    }
}
