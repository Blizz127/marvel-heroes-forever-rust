//! Conditional replay of the owned client's Scale9 point and matrix arithmetic.
//!
//! All inputs are explicit native-record roles, not inferred scene transforms.
//! The admitted arithmetic profile is finite IEEE binary32, nearest rounding,
//! without subnormal-sensitive operations. Loaded CRT/MXCSR equivalence is
//! unverified. This module defines no tessellation, image binding or UV policy.

use std::fmt;

pub type Point = [f32; 2];
pub type Result<T> = std::result::Result<T, MathError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathError {
    NonFinite(&'static str),
    SubnormalSensitive(&'static str),
    UnorderedRectangle,
    NonPositiveGrid,
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite(at) => write!(f, "nonfinite Scale9 value at {at}"),
            Self::SubnormalSensitive(at) => write!(
                f,
                "unsupported subnormal-sensitive Scale9 operation at {at}"
            ),
            Self::UnorderedRectangle => f.write_str("Scale9 rectangle endpoints are reversed"),
            Self::NonPositiveGrid => {
                f.write_str("Scale9 builder requires positive grid dimensions")
            }
        }
    }
}
impl std::error::Error for MathError {}

fn checked(v: f32, at: &'static str) -> Result<f32> {
    if !v.is_finite() {
        Err(MathError::NonFinite(at))
    } else if v.is_subnormal() {
        Err(MathError::SubnormalSensitive(at))
    } else {
        Ok(v)
    }
}

// f64 is used ONLY to admit an operation before binary32 execution. This catches
// tiny nonzero results even if a surrounding floating environment flushes them
// to zero. It never supplies a result or changes native binary32 ordering.
fn admit_exact_magnitude(v: f64, at: &'static str) -> Result<()> {
    if v != 0.0 && v.abs() < f64::from(f32::MIN_POSITIVE) {
        Err(MathError::SubnormalSensitive(at))
    } else {
        Ok(())
    }
}
fn add(a: f32, b: f32) -> Result<f32> {
    admit_exact_magnitude(f64::from(a) + f64::from(b), "add")?;
    checked(a + b, "add")
}
fn sub(a: f32, b: f32) -> Result<f32> {
    admit_exact_magnitude(f64::from(a) - f64::from(b), "subtract")?;
    checked(a - b, "subtract")
}
fn mul(a: f32, b: f32) -> Result<f32> {
    admit_exact_magnitude(f64::from(a) * f64::from(b), "multiply")?;
    checked(a * b, "multiply")
}
fn div(a: f32, b: f32) -> Result<f32> {
    admit_exact_magnitude(f64::from(a) / f64::from(b), "divide")?;
    checked(a / b, "divide")
}
fn flip_sign(v: f32) -> f32 {
    f32::from_bits(v.to_bits() ^ 0x8000_0000)
}
fn check_point(p: Point) -> Result<Point> {
    checked(p[0], "point x")?;
    checked(p[1], "point y")?;
    Ok(p)
}
fn point_add(a: Point, b: Point) -> Result<Point> {
    Ok([add(a[0], b[0])?, add(a[1], b[1])?])
}
fn point_sub(a: Point, b: Point) -> Result<Point> {
    Ok([sub(a[0], b[0])?, sub(a[1], b[1])?])
}
fn point_mul(p: Point, factor: f32) -> Result<Point> {
    Ok([mul(p[0], factor)?, mul(p[1], factor)?])
}

/// Native rows `[a, c, unused, tx]`, `[b, d, unused, ty]`.
/// Unused lanes are retained because inverse and composition treat them differently.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix2x4 {
    rows: [[f32; 4]; 2],
}
impl Matrix2x4 {
    pub const IDENTITY: Self = Self {
        rows: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0]],
    };

    pub fn from_rows(rows: [[f32; 4]; 2]) -> Result<Self> {
        for row in rows {
            for v in row {
                checked(v, "matrix input")?;
            }
        }
        Ok(Self { rows })
    }
    pub fn rows(self) -> [[f32; 4]; 2] {
        self.rows
    }

    /// Point-helper order: `(y*c + x*a) + tx`, separately rounded at each step.
    pub fn transform_point(self, p: Point) -> Result<Point> {
        let [x, y] = check_point(p)?;
        let [r0, r1] = self.rows;
        Ok([
            add(add(mul(y, r0[1])?, mul(x, r0[0])?)?, r0[3])?,
            add(add(mul(y, r1[1])?, mul(x, r1[0])?)?, r1[3])?,
        ])
    }

    /// Copy-inverse contract. For nonzero determinant, destination's unused
    /// lanes survive. Exact zero uses native identity/negative-translation fallback.
    pub fn inverse_into(self, destination: Self) -> Result<Self> {
        let [s0, s1] = self.rows;
        let det = sub(mul(s1[1], s0[0])?, mul(s0[1], s1[0])?)?;
        if det == 0.0 {
            return Self::from_rows([
                [1.0, 0.0, 0.0, flip_sign(s0[3])],
                [0.0, 1.0, 0.0, flip_sign(s1[3])],
            ]);
        }
        let q = div(1.0, det)?;
        let mut rows = destination.rows;
        rows[0][0] = mul(s1[1], q)?;
        rows[1][1] = mul(q, s0[0])?;
        rows[0][1] = flip_sign(mul(q, s0[1])?);
        rows[1][0] = flip_sign(mul(q, s1[0])?);
        rows[0][3] = flip_sign(add(mul(s0[3], rows[0][0])?, mul(s1[3], rows[0][1])?)?);
        rows[1][3] = flip_sign(add(mul(s1[3], rows[1][1])?, mul(s0[3], rows[1][0])?)?);
        Ok(Self { rows })
    }

    /// Native in-place wrapper first copies all eight source lanes.
    pub fn inverse(self) -> Result<Self> {
        self.inverse_into(self)
    }

    /// Returns `left * self`, retaining native product/add/translation order.
    /// The unused lanes are computed, then masked to positive zero.
    pub fn left_compose(self, left: Self) -> Result<Self> {
        let mut rows = [[0.0; 4]; 2];
        for (row, output) in rows.iter_mut().enumerate() {
            for (lane, value) in output.iter_mut().enumerate() {
                let products = add(
                    mul(left.rows[row][1], self.rows[1][lane])?,
                    mul(left.rows[row][0], self.rows[0][lane])?,
                )?;
                *value = add(products, if lane == 3 { left.rows[row][3] } else { 0.0 })?;
            }
            output[2] = 0.0;
        }
        Ok(Self { rows })
    }
}

/// Ordered endpoints, including equality. Builder separately requires a positive
/// grid; explicit point contexts may retain a degenerate ordered splitter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectF32 {
    coordinates: [f32; 4],
}
impl RectF32 {
    pub fn from_coordinates(coordinates: [f32; 4]) -> Result<Self> {
        for v in coordinates {
            checked(v, "rectangle input")?;
        }
        if coordinates[0] > coordinates[2] || coordinates[1] > coordinates[3] {
            return Err(MathError::UnorderedRectangle);
        }
        Ok(Self { coordinates })
    }
    /// `[xmin, ymin, xmax, ymax]`, with no conversion to pixels or twips.
    pub fn coordinates(self) -> [f32; 4] {
        self.coordinates
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformedPoint {
    pub position: Point,
    /// The native return value, not the selected matrix index.
    pub raw_outcode: u8,
    pub region: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeScale9Context {
    pre_transform: Matrix2x4,
    splitter: RectF32,
    regions: [Matrix2x4; 9],
    post_transform: Matrix2x4,
}
impl NativeScale9Context {
    /// Explicit already-initialized working-record parts. No scene admission,
    /// source identity, cache freshness or coordinate convention is inferred.
    pub fn from_parts(
        pre_transform: Matrix2x4,
        splitter: RectF32,
        regions: [Matrix2x4; 9],
        post_transform: Matrix2x4,
    ) -> Self {
        Self {
            pre_transform,
            splitter,
            regions,
            post_transform,
        }
    }
    pub fn pre_transform(&self) -> Matrix2x4 {
        self.pre_transform
    }
    pub fn splitter(&self) -> RectF32 {
        self.splitter
    }
    pub fn regions(&self) -> &[Matrix2x4; 9] {
        &self.regions
    }
    pub fn post_transform(&self) -> Matrix2x4 {
        self.post_transform
    }

    pub fn transform_point(&self, point: Point) -> Result<TransformedPoint> {
        let p = self.pre_transform.transform_point(point)?;
        let [xmin, ymin, xmax, ymax] = self.splitter.coordinates;
        let code = u8::from(p[0] > xmax)
            | (u8::from(p[1] > ymax) << 1)
            | (u8::from(xmin > p[0]) << 2)
            | (u8::from(ymin > p[1]) << 3);
        const REGION: [u8; 16] = [4, 5, 7, 8, 3, 0, 6, 0, 1, 2, 0, 0, 0, 0, 0, 0];
        let region = REGION[usize::from(code)];
        let middle = self.regions[usize::from(region)].transform_point(p)?;
        Ok(TransformedPoint {
            position: self.post_transform.transform_point(middle)?,
            raw_outcode: code,
            region,
        })
    }
}

fn basis(points: [Point; 3]) -> Result<Matrix2x4> {
    let [p0, p1, p2] = points;
    for point in points {
        check_point(point)?;
    }
    Matrix2x4::from_rows([
        [sub(p1[0], p0[0])?, sub(p2[0], p0[0])?, 0.0, p0[0]],
        [sub(p1[1], p0[1])?, sub(p2[1], p0[1])?, 0.0, p0[1]],
    ])
}

/// Recovered three-point helper, including its exact-zero inverse fallback.
/// Builder points are top-left, top-right, bottom-right (not bottom-left).
pub fn three_point_affine(source: [Point; 3], destination: [Point; 3]) -> Result<Matrix2x4> {
    basis(source)?.inverse()?.left_compose(basis(destination)?)
}

/// Explicit constructor roles. These labels do not establish world/local space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceInputs {
    pub grid: RectF32,
    pub source_bounds: RectF32,
    /// Source record +0x50; nine-matrix builder's G.
    pub grid_matrix: Matrix2x4,
    /// Source record +0x30; point pre-transform C.
    pub shape_to_grid: Matrix2x4,
    /// Constructor's third matrix argument M.
    pub outer_matrix: Matrix2x4,
}

fn edge_length(start: Point, end: Point) -> Result<f32> {
    let delta = point_sub(end, start)?;
    let square = add(mul(delta[1], delta[1])?, mul(delta[0], delta[0])?)?;
    // Standard Rust binary32 sqrt under the declared profile. This is not a
    // claim that the unobserved loaded MSVCR120 sqrtf/MXCSR behaves identically.
    let length = checked(square.sqrt(), "sqrt")?;
    Ok(if length == 0.0 {
        f32::from_bits(0x3a83_126f)
    } else {
        length
    })
}

/// Builds the recovered nine affines and `M * inverse(C) * inverse(G)` post
/// transform under finite IEEE binary32 nearest-rounding arithmetic. No host
/// floating-environment state is read or changed. Subnormal-sensitive and
/// nonfinite operations return errors; no generic nine-slice fallback occurs.
pub fn build_finite_ieee_nearest(inputs: SourceInputs) -> Result<NativeScale9Context> {
    let [gx1, gy1, gx2, gy2] = inputs.grid.coordinates;
    if gx1 == gx2 || gy1 == gy2 {
        return Err(MathError::NonPositiveGrid);
    }
    let mut bounds = inputs.source_bounds.coordinates;
    let repair = f32::from_bits(0x3f66_6666);
    if bounds[0] >= gx1 {
        bounds[0] = sub(gx1, repair)?;
    }
    if bounds[1] >= gy1 {
        bounds[1] = sub(gy1, repair)?;
    }
    // The native upper repair deliberately uses the LOWER grid coordinate.
    if gx2 >= bounds[2] {
        bounds[2] = add(gx1, repair)?;
    }
    if gy2 >= bounds[3] {
        bounds[3] = add(gy1, repair)?;
    }
    let [x0, y0, x3, y3] = bounds;
    let [r0, r1] = inputs.grid_matrix.rows;
    let ax0 = mul(r0[0], x0)?;
    let ax3 = mul(r0[0], x3)?;
    let cy0 = mul(r0[1], y0)?;
    let cy3 = mul(r0[1], y3)?;
    let bx0 = mul(r1[0], x0)?;
    let bx3 = mul(r1[0], x3)?;
    let dy0 = mul(r1[1], y0)?;
    let dy3 = mul(r1[1], y3)?;
    let tl = [add(add(ax0, cy0)?, r0[3])?, add(add(bx0, dy0)?, r1[3])?];
    let tr = [add(add(ax3, cy0)?, r0[3])?, add(add(bx3, dy0)?, r1[3])?];
    let br = [add(add(cy3, ax3)?, r0[3])?, add(add(dy3, bx3)?, r1[3])?];
    let bl = [add(add(cy3, ax0)?, r0[3])?, add(add(dy3, bx0)?, r1[3])?];
    let width = edge_length(tl, tr)?;
    let height = edge_length(tr, br)?;
    let mut left = div(sub(gx1, x0)?, width)?;
    let mut right = div(sub(x3, gx2)?, width)?;
    let mut bottom = div(sub(y3, gy2)?, height)?;
    let mut top = div(sub(gy1, y0)?, height)?;
    let sum = add(right, left)?;
    if sum > 1.0 {
        let denominator = add(sum, f32::from_bits(0x3d4c_cccd))?;
        left = div(left, denominator)?;
        right = div(right, denominator)?;
    }
    let sum = add(bottom, top)?;
    if sum > 1.0 {
        let denominator = add(sum, f32::from_bits(0x3d4c_cccd))?;
        bottom = div(bottom, denominator)?;
        top = div(top, denominator)?;
    }

    // Four separately rounded differences are essential even when G is affine.
    let upper = point_sub(tr, tl)?;
    let right_edge = point_sub(br, tr)?;
    let left_edge = point_sub(bl, tl)?;
    let lower = point_sub(br, bl)?;
    let ul = point_mul(upper, left)?;
    let ur = point_mul(upper, right)?;
    let ll = point_mul(lower, left)?;
    let lr = point_mul(lower, right)?;
    let left_top = point_add(point_mul(left_edge, top)?, tl)?;
    let left_bottom = point_sub(bl, point_mul(left_edge, bottom)?)?;
    let right_top = point_add(point_mul(right_edge, top)?, tr)?;
    let right_bottom = point_sub(br, point_mul(right_edge, bottom)?)?;
    let points = [
        [tl, point_add(ul, tl)?, point_sub(tr, ur)?, tr],
        [
            left_top,
            point_add(ul, left_top)?,
            point_sub(right_top, ur)?,
            right_top,
        ],
        [
            left_bottom,
            point_add(ll, left_bottom)?,
            point_sub(right_bottom, lr)?,
            right_bottom,
        ],
        [bl, point_add(ll, bl)?, point_sub(br, lr)?, br],
    ];
    let xs = [x0, gx1, gx2, x3];
    let ys = [y0, gy1, gy2, y3];
    let mut regions = [Matrix2x4::IDENTITY; 9];
    for row in 0..3 {
        for col in 0..3 {
            regions[row * 3 + col] = three_point_affine(
                [
                    [xs[col], ys[row]],
                    [xs[col + 1], ys[row]],
                    [xs[col + 1], ys[row + 1]],
                ],
                [
                    points[row][col],
                    points[row][col + 1],
                    points[row + 1][col + 1],
                ],
            )?;
        }
    }
    let inverse_g = inputs.grid_matrix.inverse_into(Matrix2x4::IDENTITY)?;
    let inverse_c = inputs.shape_to_grid.inverse_into(Matrix2x4::IDENTITY)?;
    let post = inverse_g
        .left_compose(inverse_c)?
        .left_compose(inputs.outer_matrix)?;
    Ok(NativeScale9Context::from_parts(
        inputs.shape_to_grid,
        inputs.grid,
        regions,
        post,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn matrix(rows: [[f32; 4]; 2]) -> Matrix2x4 {
        Matrix2x4::from_rows(rows).unwrap()
    }
    fn rectangle(v: [f32; 4]) -> RectF32 {
        RectF32::from_coordinates(v).unwrap()
    }
    fn bits(p: Point) -> [u32; 2] {
        p.map(f32::to_bits)
    }

    #[test]
    fn regions_and_splitter_equalities_use_raw_outcode() {
        let context = NativeScale9Context::from_parts(
            Matrix2x4::IDENTITY,
            rectangle([2.0, 3.0, 6.0, 7.0]),
            [Matrix2x4::IDENTITY; 9],
            Matrix2x4::IDENTITY,
        );
        let codes = [12, 8, 9, 4, 0, 1, 6, 2, 3];
        for (row, y) in [1.0, 5.0, 9.0].into_iter().enumerate() {
            for (col, x) in [0.0, 4.0, 8.0].into_iter().enumerate() {
                let result = context.transform_point([x, y]).unwrap();
                assert_eq!(result.region, (row * 3 + col) as u8);
                assert_eq!(result.raw_outcode, codes[row * 3 + col]);
                assert_eq!(bits(result.position), bits([x, y]));
            }
        }
        for p in [
            [2.0, 3.0],
            [6.0, 7.0],
            [2.0, 5.0],
            [6.0, 5.0],
            [4.0, 3.0],
            [4.0, 7.0],
        ] {
            let result = context.transform_point(p).unwrap();
            assert_eq!((result.raw_outcode, result.region), (0, 4));
        }
    }

    #[test]
    fn stages_are_not_combined_and_products_are_not_fused() {
        let pre = matrix([[1.0, 2.0, 0.0, 3.0], [0.0, 1.0, 0.0, 4.0]]);
        let middle = matrix([[2.0, 0.0, 0.0, 5.0], [1.0, 3.0, 0.0, 6.0]]);
        let post = matrix([[0.0, 1.0, 0.0, 7.0], [-1.0, 0.0, 0.0, 8.0]]);
        let context = NativeScale9Context::from_parts(
            pre,
            rectangle([-100.0, -100.0, 100.0, 100.0]),
            [middle; 9],
            post,
        );
        assert_eq!(
            context.transform_point([1.0, 2.0]).unwrap().position,
            [39.0, -13.0]
        );
        let a = f32::from_bits(0x3f80_0001);
        let b = f32::from_bits(0x3f7f_fffe);
        let cancellation = matrix([[a, -1.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0]]);
        assert_eq!(
            cancellation.transform_point([b, 1.0]).unwrap()[0].to_bits(),
            0
        );
        assert_ne!(a.mul_add(b, -1.0), 0.0);
    }

    #[test]
    fn inverse_zero_fallback_and_lane_contracts() {
        let singular = matrix([[0.0, 2.0, 7.0, 5.0], [0.0, 4.0, 9.0, -6.0]]);
        assert_eq!(
            singular.inverse().unwrap().rows(),
            [[1.0, 0.0, 0.0, -5.0], [0.0, 1.0, 0.0, 6.0]]
        );
        let source = matrix([[2.0, 0.0, 7.0, 5.0], [0.0, 4.0, 9.0, -6.0]]);
        let destination = matrix([[0.0, 0.0, 17.0, 0.0], [0.0, 0.0, 19.0, 0.0]]);
        let inverse = source.inverse_into(destination).unwrap().rows();
        assert_eq!((inverse[0][2], inverse[1][2]), (17.0, 19.0));
        assert_eq!(inverse[0][1].to_bits(), (-0.0f32).to_bits());
        let translated = matrix([[1.0, 0.0, 7.0, 10.0], [0.0, 1.0, 9.0, 20.0]]);
        let scaled = matrix([[2.0, 0.0, 0.0, 0.0], [0.0, 3.0, 0.0, 0.0]]);
        assert_eq!(
            translated.left_compose(scaled).unwrap().rows(),
            [[2.0, 0.0, 0.0, 20.0], [0.0, 3.0, 0.0, 60.0]]
        );
    }

    #[test]
    fn builder_preserves_fixed_edges_and_expands_center() {
        // Synthetic power-of-two profile has independently hand-checkable
        // exact matrices: two-unit borders survive in a sixteen-unit target.
        let scale = matrix([[2.0, 0.0, 0.0, 0.0], [0.0, 2.0, 0.0, 0.0]]);
        let inputs = SourceInputs {
            grid: rectangle([2.0, 2.0, 6.0, 6.0]),
            source_bounds: rectangle([0.0, 0.0, 8.0, 8.0]),
            grid_matrix: scale,
            shape_to_grid: Matrix2x4::IDENTITY,
            outer_matrix: scale,
        };
        let context = build_finite_ieee_nearest(inputs).unwrap();
        assert_eq!(context.regions()[0], Matrix2x4::IDENTITY);
        assert_eq!(
            context.regions()[4].rows(),
            [[3.0, 0.0, 0.0, -4.0], [0.0, 3.0, 0.0, -4.0]]
        );
        assert_eq!(
            context.regions()[8].rows(),
            [[1.0, 0.0, 0.0, 8.0], [0.0, 1.0, 0.0, 8.0]]
        );
        assert_eq!(context.post_transform(), Matrix2x4::IDENTITY);
        for (source, expected) in [
            (0.0, 0.0),
            (1.0, 1.0),
            (2.0, 2.0),
            (4.0, 8.0),
            (6.0, 14.0),
            (8.0, 16.0),
        ] {
            assert_eq!(
                bits(context.transform_point([source, source]).unwrap().position),
                bits([expected, expected])
            );
        }
        assert_eq!(
            build_finite_ieee_nearest(SourceInputs {
                grid: rectangle([2.0, 2.0, 2.0, 6.0]),
                ..inputs
            }),
            Err(MathError::NonPositiveGrid)
        );
    }

    #[test]
    fn rejects_unsupported_arithmetic_without_fallback() {
        assert_eq!(
            RectF32::from_coordinates([2.0, 0.0, 1.0, 1.0]),
            Err(MathError::UnorderedRectangle)
        );
        assert!(matches!(
            Matrix2x4::from_rows([[f32::NAN; 4]; 2]),
            Err(MathError::NonFinite(_))
        ));
        assert!(matches!(
            Matrix2x4::IDENTITY.transform_point([f32::from_bits(1), 0.0]),
            Err(MathError::SubnormalSensitive(_))
        ));
        assert!(matches!(
            mul(f32::MIN_POSITIVE, 0.5),
            Err(MathError::SubnormalSensitive(_))
        ));
        assert!(matches!(
            mul(f32::MIN_POSITIVE, f32::MIN_POSITIVE),
            Err(MathError::SubnormalSensitive(_))
        ));
        assert!(matches!(
            sub(f32::from_bits(0x0080_0001), f32::MIN_POSITIVE),
            Err(MathError::SubnormalSensitive(_))
        ));
        assert!(matches!(mul(f32::MAX, 2.0), Err(MathError::NonFinite(_))));
    }
}
