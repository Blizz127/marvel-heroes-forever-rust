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
    TooManyPrimaryVertices,
    AllocationFailed,
    IntegerConversionOutOfRange,
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
            Self::TooManyPrimaryVertices => {
                f.write_str("primary output vertex admission limit exceeded")
            }
            Self::AllocationFailed => f.write_str("cannot reserve primary output vertices"),
            Self::IntegerConversionOutOfRange => {
                f.write_str("primary coordinate is outside finite signed32 conversion range")
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

    /// Inline fill path 0x1400c31b2: `self * inverse`, with that caller's
    /// scalar operation order. Unlike the general SIMD composition, linear
    /// coefficients receive no additional +0 operation. Unused lanes are
    /// written directly to positive zero without reading their inputs.
    pub(super) fn compose_fill_inverse(self, inverse: Self) -> Result<Self> {
        let [f, g] = self.rows;
        let [i, j] = inverse.rows;
        let rows = [
            [
                add(mul(f[0], i[0])?, mul(j[0], f[1])?)?,
                add(mul(j[1], f[1])?, mul(i[1], f[0])?)?,
                0.0,
                add(add(mul(j[3], f[1])?, mul(i[3], f[0])?)?, f[3])?,
            ],
            [
                add(mul(g[0], i[0])?, mul(j[0], g[1])?)?,
                add(mul(j[1], g[1])?, mul(i[1], g[0])?)?,
                0.0,
                add(add(mul(j[3], g[1])?, mul(i[3], g[0])?)?, g[3])?,
            ],
        ];
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

/// Conservative Rust admission cap, not a universal native tessellator limit.
pub const MAX_PRIMARY_OUTPUT_VERTICES: usize = 65_535;

/// Arithmetic result for explicitly supplied primary-tessellator vertices.
/// No native export selection, output receiver or final pixel space is inferred.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimaryNormalizedVertices {
    /// Input order retained. These are in normalization space when applied,
    /// otherwise in the space produced by inverse_outer.
    pub positions: Vec<Point>,
    /// Native seeded extrema AFTER inverse_outer and BEFORE normalization, in
    /// xmin/ymin/xmax/ymax order. Empty input retains the reversed sentinels.
    pub pre_normalization_extrema: [f32; 4],
    /// Applied first to every supplied point, with recovered +0x38 operation order.
    pub inverse_outer: Matrix2x4,
    /// Maps the post-inverse-outer points to normalization space when applied.
    /// Identity when either final extent fails the strict-positive guard.
    pub forward_normalization: Matrix2x4,
    /// Native inverse-helper result handed to the unbound output receiver.
    /// It does not include outer_matrix or establish final reconstruction.
    pub inverse_normalization: Matrix2x4,
    pub normalization_applied: bool,
    /// Whether the native +0x40 invalid-extrema scan branch was taken.
    pub extrema_rescanned: bool,
}

fn extrema_seed() -> [f32; 4] {
    let high = f32::from_bits(0x7149_f2ca);
    let low = f32::from_bits(0xf149_f2ca);
    [high, high, low, low]
}

fn include_point(extrema: &mut [f32; 4], p: Point) {
    // Strict comparisons retain the first encountered signed zero. Sentinels
    // are the native finite constants, not infinities or the first vertex.
    if extrema[0] > p[0] {
        extrema[0] = p[0];
    }
    if extrema[1] > p[1] {
        extrema[1] = p[1];
    }
    if p[0] > extrema[2] {
        extrema[2] = p[0];
    }
    if p[1] > extrema[3] {
        extrema[3] = p[1];
    }
}

fn primary_inverse_outer_point(matrix: Matrix2x4, p: Point) -> Result<Point> {
    let [x, y] = p;
    let [r0, r1] = matrix.rows;
    Ok([
        add(add(mul(x, r0[0])?, mul(y, r0[1])?)?, r0[3])?,
        add(add(mul(y, r1[1])?, mul(x, r1[0])?)?, r1[3])?,
    ])
}

fn primary_normalization_point(matrix: Matrix2x4, p: Point) -> Result<Point> {
    let [x, y] = p;
    let [r0, r1] = matrix.rows;
    Ok([
        add(add(mul(x, r0[0])?, mul(y, r0[1])?)?, r0[3])?,
        add(add(mul(x, r1[0])?, mul(y, r1[1])?)?, r1[3])?,
    ])
}

/// Replays inverse-outer/+0x38 followed by primary-tessellator +0x40 arithmetic.
/// `all_vertices` must represent ALL positions whose native extrema are wanted;
/// selecting an export subset before this call would change normalization.
///
/// Inputs and intermediates use the existing finite IEEE nearest-rounding
/// profile. Empty input or a nonpositive final seeded axis span retains identity
/// normalization and does not rewrite the post-inverse-outer points. This does
/// not replay the enclosing output routine's mesh/count guards or selection.
/// Caller input is never mutated, including on allocation/arithmetic failure.
pub fn normalize_primary_vertices_finite_ieee_nearest(
    all_vertices: &[Point],
    outer_matrix: Matrix2x4,
) -> Result<PrimaryNormalizedVertices> {
    if all_vertices.len() > MAX_PRIMARY_OUTPUT_VERTICES {
        return Err(MathError::TooManyPrimaryVertices);
    }
    for &point in all_vertices {
        check_point(point)?;
    }
    let inverse_outer = outer_matrix.inverse_into(Matrix2x4::IDENTITY)?;
    let mut positions = Vec::new();
    positions
        .try_reserve_exact(all_vertices.len())
        .map_err(|_| MathError::AllocationFailed)?;
    let mut extrema = extrema_seed();
    for &point in all_vertices {
        let transformed = primary_inverse_outer_point(inverse_outer, point)?;
        include_point(&mut extrema, transformed);
        positions.push(transformed);
    }
    let extrema_rescanned = extrema[0] >= extrema[2] || extrema[1] >= extrema[3];
    if extrema_rescanned {
        extrema = extrema_seed();
        for &point in &positions {
            include_point(&mut extrema, point);
        }
    }
    let normalization_applied = extrema[2] > extrema[0] && extrema[3] > extrema[1];
    let mut forward_normalization = Matrix2x4::IDENTITY;
    let mut inverse_normalization = Matrix2x4::IDENTITY;
    if normalization_applied {
        let [xmin, ymin, xmax, ymax] = extrema;
        let low = f32::from_bits(0xc6ff_f800);
        let high = f32::from_bits(0x46ff_f800);
        forward_normalization = three_point_affine(
            [[xmin, ymin], [xmax, ymin], [xmax, ymax]],
            [[low, low], [high, low], [high, high]],
        )?;
        for point in &mut positions {
            *point = primary_normalization_point(forward_normalization, *point)?;
        }
        // The native destination still contains the forward matrix; its unused
        // lanes therefore survive the nonzero-determinant copy-inverse branch.
        inverse_normalization = forward_normalization.inverse()?;
    }
    Ok(PrimaryNormalizedVertices {
        positions,
        pre_normalization_extrema: extrema,
        inverse_outer,
        forward_normalization,
        inverse_normalization,
        normalization_applied,
        extrema_rescanned,
    })
}

/// Packs one explicitly selected coordinate to its native raw stored word.
/// Standard host f32 floor is used under the finite nearest-rounding profile;
/// loaded MSVCR120 floorf/MXCSR equivalence is unverified. No signed16 clamp or
/// receiver signedness is inferred. Out-of-signed32 CVTT behavior is rejected.
pub fn pack_primary_coordinate_finite_ieee_nearest(value: f32) -> Result<u16> {
    checked(value, "primary packing input")?;
    let adjusted = if value < 0.0 {
        sub(value, 0.5)?
    } else {
        add(value, 0.5)?
    };
    let floored = checked(adjusted.floor(), "floor")?;
    // i32::MAX as f32 rounds UP to 2^31, so a binary32 inclusive-max check
    // would wrongly admit an overflowing CVTT. Check the exact wider value.
    let wide = f64::from(floored);
    if wide < -2_147_483_648.0 || wide >= 2_147_483_648.0 {
        return Err(MathError::IntegerConversionOutOfRange);
    }
    // This cast cannot saturate: the explicit check admits only finite signed32
    // integers. The following low-word conversion deliberately wraps as native.
    let integer = floored as i32;
    Ok((integer as u32 & 0xffff) as u16)
}

/// Packs an explicitly selected normalized (or degenerate unnormalized) point.
/// Selection itself and the other bytes of the native vertex record are outside
/// this arithmetic primitive.
pub fn pack_primary_point_finite_ieee_nearest(point: Point) -> Result<[u16; 2]> {
    Ok([
        pack_primary_coordinate_finite_ieee_nearest(point[0])?,
        pack_primary_coordinate_finite_ieee_nearest(point[1])?,
    ])
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
    fn primary_normalization_uses_post_inverse_outer_extrema() {
        let outer = matrix([[2.0, 0.0, 7.0, 10.0], [0.0, 4.0, 9.0, -8.0]]);
        let input = [[10.0, -8.0], [26.0, -8.0], [10.0, 24.0], [26.0, 24.0]];
        let result = normalize_primary_vertices_finite_ieee_nearest(&input, outer).unwrap();
        assert_eq!(result.pre_normalization_extrema, [0.0, 0.0, 8.0, 8.0]);
        assert!(result.normalization_applied);
        assert!(!result.extrema_rescanned);
        assert_eq!(
            result.forward_normalization.rows(),
            [[8191.0, 0.0, 0.0, -32764.0], [0.0, 8191.0, 0.0, -32764.0]]
        );
        assert_eq!(result.inverse_outer.rows()[0][2].to_bits(), 0);
        assert_eq!(result.inverse_outer.rows()[1][2].to_bits(), 0);
        assert_eq!(
            result.positions,
            [
                [-32764.0, -32764.0],
                [32764.0, -32764.0],
                [-32764.0, 32764.0],
                [32764.0, 32764.0]
            ]
        );
        assert_eq!(
            pack_primary_point_finite_ieee_nearest(result.positions[0]).unwrap(),
            [0x8003, 0x8003]
        );
        assert_eq!(input[0], [10.0, -8.0]);
    }

    #[test]
    fn primary_degenerate_and_empty_sequences_skip_only_normalization() {
        let outer = matrix([[2.0, 0.0, 0.0, 10.0], [0.0, 4.0, 0.0, -8.0]]);
        for (input, expected, bounds) in [
            (
                [[12.0, 0.0], [12.0, 16.0]],
                [[1.0, 2.0], [1.0, 6.0]],
                [1.0, 2.0, 1.0, 6.0],
            ),
            (
                [[12.0, 0.0], [20.0, 0.0]],
                [[1.0, 2.0], [5.0, 2.0]],
                [1.0, 2.0, 5.0, 2.0],
            ),
        ] {
            let result = normalize_primary_vertices_finite_ieee_nearest(&input, outer).unwrap();
            assert_eq!(result.positions, expected);
            assert_eq!(result.pre_normalization_extrema, bounds);
            assert!(!result.normalization_applied);
            assert!(result.extrema_rescanned);
            assert_eq!(result.forward_normalization, Matrix2x4::IDENTITY);
            assert_eq!(result.inverse_normalization, Matrix2x4::IDENTITY);
        }
        let empty =
            normalize_primary_vertices_finite_ieee_nearest(&[], Matrix2x4::IDENTITY).unwrap();
        assert!(empty.positions.is_empty());
        assert!(!empty.normalization_applied);
        assert!(empty.extrema_rescanned);
        assert_eq!(
            empty.pre_normalization_extrema.map(f32::to_bits),
            [0x7149_f2ca, 0x7149_f2ca, 0xf149_f2ca, 0xf149_f2ca]
        );
    }

    #[test]
    fn primary_packing_preserves_negative_floor_and_low_word() {
        for (input, expected) in [
            (-0.0, 0),
            (0.0, 0),
            (-1.0, 0xfffe),
            (1.0, 1),
            (-32764.0, 0x8003),
            (32764.0, 0x7ffc),
            (-32768.0, 0x7fff),
            (65536.0, 0),
            (f32::from_bits(0xbf00_0001), 0xffff),
            (f32::from_bits(0xbf00_0002), 0xfffe),
        ] {
            assert_eq!(
                pack_primary_coordinate_finite_ieee_nearest(input).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn primary_output_rejects_unsupported_conversion_and_resource_profiles() {
        assert_eq!(
            pack_primary_coordinate_finite_ieee_nearest(f32::from_bits(0x4eff_ffff)).unwrap(),
            0xff80
        );
        assert_eq!(
            pack_primary_coordinate_finite_ieee_nearest(-2_147_483_648.0).unwrap(),
            0
        );
        for value in [2_147_483_648.0, f32::from_bits(0xcf00_0001)] {
            assert_eq!(
                pack_primary_coordinate_finite_ieee_nearest(value),
                Err(MathError::IntegerConversionOutOfRange)
            );
        }
        assert!(matches!(
            pack_primary_coordinate_finite_ieee_nearest(f32::INFINITY),
            Err(MathError::NonFinite(_))
        ));
        assert!(matches!(
            pack_primary_coordinate_finite_ieee_nearest(f32::from_bits(1)),
            Err(MathError::SubnormalSensitive(_))
        ));
        let over_budget = vec![[0.0, 0.0]; MAX_PRIMARY_OUTPUT_VERTICES + 1];
        assert_eq!(
            normalize_primary_vertices_finite_ieee_nearest(&over_budget, Matrix2x4::IDENTITY),
            Err(MathError::TooManyPrimaryVertices)
        );
        let original = [[1.0, 2.0], [f32::MAX, 3.0]];
        assert!(
            normalize_primary_vertices_finite_ieee_nearest(
                &original,
                matrix([[0.5, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0]])
            )
            .is_err()
        );
        assert_eq!(original[0], [1.0, 2.0]);
        assert_eq!(original[1][0], f32::MAX);
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
