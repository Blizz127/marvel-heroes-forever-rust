//! Native selected-fill matrix arithmetic, separate from generated geometry.
//!
//! The caller supplies the actual current fill matrix, optional grid record,
//! selected-fill bounds and returned image matrix. This does not bind a provider,
//! load an image, infer UVs from saved dimensions or enable a renderer path.
//! Evidence and operation order: `docs/SCALE9_FILL_MATRIX.md`.

use super::native_scale9::{
    self, Matrix2x4, RectF32, SourceInputs, build_finite_ieee_nearest, three_point_affine,
};

/// Explicit native source-record roles for the paint-context constructor.
/// The third constructor matrix is identity on this route; it is not the
/// geometry context's outer matrix. No fields have inferred defaults.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridRecord {
    pub grid: RectF32,
    pub source_bounds: RectF32,
    pub grid_matrix: Matrix2x4,
    pub shape_to_grid: Matrix2x4,
    /// Bounds accumulated from matching provider paths, not DefineShape bounds.
    /// None means a known empty selection. Equality takes the native identity
    /// branch without constructing a grid context or transforming points.
    pub selected_fill_bounds: Option<RectF32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GridBranch {
    /// The caller has established the absence of the context's grid record.
    Absent,
    Present(GridRecord),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImageBranch {
    /// The selected fill's native image pointer is known null.
    Absent,
    /// Exact matrix returned by the selected image's virtual +0xb0.
    /// Construction, dimensions and subsequent resource changes are external.
    ReturnedMatrix(Matrix2x4),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparedFill {
    /// None denotes the absent-grid branch; Some(identity) can instead denote
    /// present-grid empty/degenerate bounds. Their later operations differ.
    pub bounds_transform: Option<Matrix2x4>,
    pub inverse_bounds_transform: Option<Matrix2x4>,
    pub before_image: Matrix2x4,
    pub matrix: Matrix2x4,
}

/// Arithmetic from native c3121 onward. Any preceding morph interpolation is
/// already represented in `current_fill`. This function is pure and neither
/// selects that earlier route nor recovers its inputs.
///
/// Positive selected bounds yield one affine from transformed TL/TR/BR corners.
/// It is not a per-region inverse UV mapping. Present-grid degenerate/empty
/// selection still performs inverse and inline composition with identity.
/// The image matrix is then left-composed using the separate native helper.
/// All operations use the existing finite, normal-sensitive binary32 profile;
/// exceptional/subnormal-sensitive math rejects without a generic fallback.
pub fn prepare_finite_ieee_nearest(
    current_fill: Matrix2x4,
    grid: GridBranch,
    image: ImageBranch,
) -> native_scale9::Result<PreparedFill> {
    let (bounds_transform, inverse_bounds_transform, before_image) = match grid {
        GridBranch::Absent => (None, None, current_fill),
        GridBranch::Present(record) => {
            let transform = selected_bounds_transform(record)?;
            let inverse = transform.inverse_into(Matrix2x4::IDENTITY)?;
            let composed = current_fill.compose_fill_inverse(inverse)?;
            (Some(transform), Some(inverse), composed)
        }
    };
    let matrix = match image {
        ImageBranch::Absent => before_image,
        ImageBranch::ReturnedMatrix(image) => before_image.left_compose(image)?,
    };
    Ok(PreparedFill {
        bounds_transform,
        inverse_bounds_transform,
        before_image,
        matrix,
    })
}

fn selected_bounds_transform(record: GridRecord) -> native_scale9::Result<Matrix2x4> {
    let Some(bounds) = record.selected_fill_bounds else {
        return Ok(Matrix2x4::IDENTITY);
    };
    let [x0, y0, x1, y1] = bounds.coordinates();
    if x0 == x1 || y0 == y1 {
        return Ok(Matrix2x4::IDENTITY);
    }
    let context = build_finite_ieee_nearest(SourceInputs {
        grid: record.grid,
        source_bounds: record.source_bounds,
        grid_matrix: record.grid_matrix,
        shape_to_grid: record.shape_to_grid,
        outer_matrix: Matrix2x4::IDENTITY,
    })?;
    let source = [[x0, y0], [x1, y0], [x1, y1]];
    let destination = [
        context.transform_point(source[0])?.position,
        context.transform_point(source[1])?.position,
        context.transform_point(source[2])?.position,
    ];
    three_point_affine(source, destination)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits(matrix: Matrix2x4) -> [[u32; 4]; 2] {
        matrix.rows().map(|row| row.map(f32::to_bits))
    }
    fn record(bounds: Option<RectF32>) -> GridRecord {
        GridRecord {
            grid: RectF32::from_coordinates([2.0, 2.0, 8.0, 8.0]).unwrap(),
            source_bounds: RectF32::from_coordinates([0.0, 0.0, 10.0, 10.0]).unwrap(),
            grid_matrix: Matrix2x4::IDENTITY,
            shape_to_grid: Matrix2x4::IDENTITY,
            selected_fill_bounds: bounds,
        }
    }

    #[test]
    fn absent_branches_preserve_all_input_bits() {
        let fill = Matrix2x4::from_rows([[-0.0, 2.0, -3.0, 4.0], [5.0, -0.0, 7.0, -8.0]]).unwrap();
        let prepared =
            prepare_finite_ieee_nearest(fill, GridBranch::Absent, ImageBranch::Absent).unwrap();
        assert_eq!(bits(prepared.matrix), bits(fill));
        assert_eq!(prepared.bounds_transform, None);
        assert_eq!(prepared.inverse_bounds_transform, None);
    }

    #[test]
    fn empty_or_degenerate_bounds_skip_invalid_grid_but_still_compose() {
        let fill = Matrix2x4::from_rows([[2.0, 3.0, -7.0, 5.0], [11.0, 13.0, 17.0, 19.0]]).unwrap();
        for bounds in [
            None,
            Some(RectF32::from_coordinates([4.0, 2.0, 4.0, 8.0]).unwrap()),
        ] {
            let mut grid = record(bounds);
            grid.grid = RectF32::from_coordinates([0.0, 0.0, 0.0, 0.0]).unwrap();
            let result =
                prepare_finite_ieee_nearest(fill, GridBranch::Present(grid), ImageBranch::Absent)
                    .unwrap();
            assert_eq!(result.bounds_transform, Some(Matrix2x4::IDENTITY));
            assert_eq!(result.inverse_bounds_transform, Some(Matrix2x4::IDENTITY));
            assert_eq!(
                result.matrix.rows(),
                [[2.0, 3.0, 0.0, 5.0], [11.0, 13.0, 0.0, 19.0]]
            );
        }
    }

    #[test]
    fn image_matrix_is_left_composed_after_the_grid_stage() {
        let fill = Matrix2x4::from_rows([[2.0, 0.0, 0.0, 3.0], [0.0, 4.0, 0.0, 5.0]]).unwrap();
        let image = Matrix2x4::from_rows([[7.0, 0.0, 0.0, 11.0], [0.0, 13.0, 0.0, 17.0]]).unwrap();
        let result = prepare_finite_ieee_nearest(
            fill,
            GridBranch::Absent,
            ImageBranch::ReturnedMatrix(image),
        )
        .unwrap();
        assert_eq!(
            result.matrix.rows(),
            [[14.0, 0.0, 0.0, 32.0], [0.0, 52.0, 0.0, 82.0]]
        );
        assert_eq!(bits(result.before_image), bits(fill));
    }

    #[test]
    fn positive_bounds_expose_invalid_grid_instead_of_using_identity() {
        let mut grid = record(Some(
            RectF32::from_coordinates([0.0, 0.0, 10.0, 10.0]).unwrap(),
        ));
        grid.grid = RectF32::from_coordinates([0.0, 0.0, 0.0, 0.0]).unwrap();
        assert!(matches!(
            prepare_finite_ieee_nearest(
                Matrix2x4::IDENTITY,
                GridBranch::Present(grid),
                ImageBranch::Absent
            ),
            Err(native_scale9::MathError::NonPositiveGrid)
        ));
    }

    #[test]
    fn inline_composition_preserves_negative_zero_without_extra_addition() {
        let fill =
            Matrix2x4::from_rows([[-0.0, -0.0, 3.0, -0.0], [-0.0, -0.0, 4.0, -0.0]]).unwrap();
        let result = fill.compose_fill_inverse(Matrix2x4::IDENTITY).unwrap();
        assert_eq!(bits(result), [[0x80000000, 0x80000000, 0, 0x80000000]; 2]);
        assert_ne!(
            bits(result),
            bits(Matrix2x4::IDENTITY.left_compose(fill).unwrap())
        );
    }
}
