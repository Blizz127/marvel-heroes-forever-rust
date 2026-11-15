//! Bounded positive-axis nine-slice geometry. Scene/grid precedence and native
//! bitmap/gradient fill policy are deliberately chosen by the caller.

use crate::gfx::{Matrix, Rect, tessellate::Position};
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity {
    pub scene_node: usize,
    pub grid_movie: usize,
    pub grid_tag: usize,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min: Position,
    pub max: Position,
}
impl Bounds {
    fn valid(self) -> bool {
        finite(self.min) && finite(self.max) && self.min.x < self.max.x && self.min.y < self.max.y
    }
    fn contains(self, p: Position) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_input_triangles: usize,
    pub max_output_triangles: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_input_triangles: 250_000,
            max_output_triangles: 1_000_000,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct WarpedMesh {
    pub identity: Identity,
    /// Grid placement's parent-coordinate twips, including its translation.
    pub positions: Vec<Position>,
    /// Barycentrically interpolated original input source coordinates. This is
    /// appropriate for sliced standalone bitmaps, not an automatic shape-fill policy.
    pub source_positions: Vec<Position>,
    pub indices: Vec<u32>,
    pub input_triangles: usize,
}
#[derive(Clone, Copy, Debug)]
struct Axis {
    min: f64,
    max: f64,
    low: f64,
    high: f64,
    scale: f64,
    translation: f64,
}
impl Axis {
    fn new(
        min: f64,
        max: f64,
        low: f64,
        high: f64,
        scale: f64,
        translation: f64,
    ) -> io::Result<Self> {
        if low < min || high > max || low >= high {
            return Err(unsupported("grid outside source bounds or empty center"));
        }
        let target = (max - min) * scale;
        let borders = (low - min) + (max - high);
        if target <= borders {
            return Err(unsupported(
                "target is no larger than fixed borders; native undersize policy unresolved",
            ));
        }
        Ok(Self {
            min,
            max,
            low,
            high,
            scale,
            translation,
        })
    }
    fn map(self, value: f64) -> f64 {
        if self.scale == 1.0 {
            return value + self.translation;
        }
        let result = if value <= self.low {
            self.min * self.scale + (value - self.min)
        } else if value >= self.high {
            self.max * self.scale + (value - self.max)
        } else {
            let low = self.min * self.scale + (self.low - self.min);
            let high = self.max * self.scale - (self.max - self.high);
            low + (value - self.low) * (high - low) / (self.high - self.low)
        };
        result + self.translation
    }
    fn cuts(self) -> Vec<(f64, f64)> {
        if self.scale == 1.0 {
            return vec![(self.min, self.max)];
        }
        [
            (self.min, self.low),
            (self.low, self.high),
            (self.high, self.max),
        ]
        .into_iter()
        .filter(|(a, b)| a < b)
        .collect()
    }
}
#[derive(Clone, Copy, Debug)]
pub struct GridWarp {
    pub identity: Identity,
    pub source_bounds: Bounds,
    pub splitter: Rect,
    pub local_transform: Matrix,
    x: Axis,
    y: Axis,
}
impl GridWarp {
    /// Caller must select exactly one unambiguous grid, establish source bounds,
    /// and exclude nested-grid semantics. This does not resolve scene ownership.
    pub fn new(
        identity: Identity,
        source_bounds: Bounds,
        splitter: Rect,
        local_transform: Matrix,
    ) -> io::Result<Self> {
        if !source_bounds.valid() {
            return Err(invalid("invalid source bounds"));
        }
        if local_transform.rotate_skew_0 != 0 || local_transform.rotate_skew_1 != 0 {
            return Err(unsupported("rotated or skewed grid placement"));
        }
        if local_transform.scale_x <= 0 || local_transform.scale_y <= 0 {
            return Err(unsupported("nonpositive grid scale"));
        }
        if i64::from(splitter.x_max) - i64::from(splitter.x_min) < 1
            || i64::from(splitter.y_max) - i64::from(splitter.y_min) < 1
        {
            return Err(unsupported(
                "disabled grid: center is smaller than one twip",
            ));
        }
        let x = Axis::new(
            source_bounds.min.x,
            source_bounds.max.x,
            f64::from(splitter.x_min),
            f64::from(splitter.x_max),
            f64::from(local_transform.scale_x) / 65536.0,
            f64::from(local_transform.translate_x),
        )?;
        let y = Axis::new(
            source_bounds.min.y,
            source_bounds.max.y,
            f64::from(splitter.y_min),
            f64::from(splitter.y_max),
            f64::from(local_transform.scale_y) / 65536.0,
            f64::from(local_transform.translate_y),
        )?;
        Ok(Self {
            identity,
            source_bounds,
            splitter,
            local_transform,
            x,
            y,
        })
    }
    pub fn needs_deformation(self) -> bool {
        self.x.scale != 1.0 || self.y.scale != 1.0
    }
    pub fn map(self, p: Position) -> io::Result<Position> {
        if !finite(p) || !self.source_bounds.contains(p) {
            return Err(invalid("point outside declared grid source bounds"));
        }
        let mapped = Position {
            x: self.x.map(p.x),
            y: self.y.map(p.y),
        };
        if !finite(mapped) {
            return Err(invalid("non-finite warped point"));
        }
        Ok(mapped)
    }
    pub fn destination_bounds(self) -> Bounds {
        Bounds {
            min: Position {
                x: self.x.map(self.x.min),
                y: self.y.map(self.y.min),
            },
            max: Position {
                x: self.x.map(self.x.max),
                y: self.y.map(self.y.max),
            },
        }
    }
    /// Geometry enters in the grid's local coordinates. Source values may be in
    /// the descendant shape's local coordinates; they are interpolated separately.
    pub fn triangles(
        self,
        positions: &[Position],
        source_positions: &[Position],
        indices: &[u32],
        limits: Limits,
    ) -> io::Result<WarpedMesh> {
        if positions.len() != source_positions.len() || indices.len() % 3 != 0 {
            return Err(invalid("misaligned scale9 vertex/index arrays"));
        }
        let count = indices.len() / 3;
        if count > limits.max_input_triangles {
            return Err(invalid("scale9 input triangle budget"));
        }
        // Unused vertices are neither interpreted nor copied. Each referenced
        // vertex is checked before clipping, keeping work proportional to input.
        let mut output = WarpedMesh {
            identity: self.identity,
            positions: Vec::new(),
            source_positions: Vec::new(),
            indices: Vec::new(),
            input_triangles: count,
        };
        for triangle in indices.chunks_exact(3) {
            let mut input = Vec::with_capacity(3);
            for &index in triangle {
                let index = index as usize;
                let p = *positions
                    .get(index)
                    .ok_or_else(|| invalid("scale9 vertex index outside input"))?;
                let source = *source_positions
                    .get(index)
                    .ok_or_else(|| invalid("scale9 source index outside input"))?;
                if !finite(p) || !finite(source) || !self.source_bounds.contains(p) {
                    return Err(invalid("non-finite vertex or geometry outside grid bounds"));
                }
                input.push(Vertex {
                    position: p,
                    source,
                });
            }
            if !self.needs_deformation() {
                emit(self, &input, &mut output, limits)?;
                continue;
            }
            for (left, right) in self.x.cuts() {
                for (top, bottom) in self.y.cuts() {
                    let mut polygon = input.clone();
                    polygon = clip(&polygon, 0, left, true)?;
                    polygon = clip(&polygon, 0, right, false)?;
                    polygon = clip(&polygon, 1, top, true)?;
                    polygon = clip(&polygon, 1, bottom, false)?;
                    emit(self, &polygon, &mut output, limits)?;
                }
            }
        }
        Ok(output)
    }
}
#[derive(Clone, Copy)]
struct Vertex {
    position: Position,
    source: Position,
}
fn coordinate(p: Position, axis: usize) -> f64 {
    if axis == 0 { p.x } else { p.y }
}
fn lerp(a: Position, b: Position, t: f64) -> Position {
    Position {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}
fn clip(input: &[Vertex], axis: usize, boundary: f64, greater: bool) -> io::Result<Vec<Vertex>> {
    let Some(mut previous) = input.last().copied() else {
        return Ok(Vec::new());
    };
    let inside = |p: Position| {
        if greater {
            coordinate(p, axis) >= boundary
        } else {
            coordinate(p, axis) <= boundary
        }
    };
    let mut output = Vec::new();
    for &current in input {
        let before = inside(previous.position);
        let after = inside(current.position);
        if before != after {
            let a = coordinate(previous.position, axis);
            let b = coordinate(current.position, axis);
            let t = (boundary - a) / (b - a);
            if !t.is_finite() || !(0.0..=1.0).contains(&t) {
                return Err(invalid("invalid scale9 clipping intersection"));
            }
            let mut position = lerp(previous.position, current.position, t);
            if axis == 0 {
                position.x = boundary;
            } else {
                position.y = boundary;
            }
            output.push(Vertex {
                position,
                source: lerp(previous.source, current.source, t),
            });
        }
        if after {
            output.push(current);
        }
        previous = current;
    }
    Ok(output)
}
fn emit(
    warp: GridWarp,
    polygon: &[Vertex],
    output: &mut WarpedMesh,
    limits: Limits,
) -> io::Result<()> {
    for i in 1..polygon.len().saturating_sub(1) {
        let vertices = [polygon[0], polygon[i], polygon[i + 1]];
        let points = [
            warp.map(vertices[0].position)?,
            warp.map(vertices[1].position)?,
            warp.map(vertices[2].position)?,
        ];
        let area = (points[1].x - points[0].x) * (points[2].y - points[0].y)
            - (points[1].y - points[0].y) * (points[2].x - points[0].x);
        if !area.is_finite() {
            return Err(invalid("non-finite warped triangle"));
        }
        if area == 0.0 {
            continue;
        }
        if output.indices.len() / 3 >= limits.max_output_triangles {
            return Err(invalid("scale9 output triangle budget"));
        }
        let index = u32::try_from(output.positions.len())
            .map_err(|_| invalid("scale9 vertex index overflow"))?;
        index
            .checked_add(2)
            .ok_or_else(|| invalid("scale9 vertex index overflow"))?;
        output.positions.extend(points);
        output.source_positions.extend(vertices.map(|v| v.source));
        output.indices.extend([index, index + 1, index + 2]);
    }
    Ok(())
}
/// Explicit mathematical policy: linearly map destination bounding-box positions
/// back into original shape bounds. Autodesk describes average bounding-box
/// scaling for shape fills but does not specify this exact formula. Callers must
/// label using this helper for native paint semantics as INFERRED.
pub fn bounds_linear_sources(
    source: Bounds,
    destination: Bounds,
    positions: &[Position],
) -> io::Result<Vec<Position>> {
    if !source.valid() || !destination.valid() {
        return Err(invalid("invalid average fill bounds"));
    }
    let mut result = Vec::with_capacity(positions.len());
    for &p in positions {
        if !finite(p) {
            return Err(invalid("non-finite average fill position"));
        }
        let x = source.min.x
            + (p.x - destination.min.x) * (source.max.x - source.min.x)
                / (destination.max.x - destination.min.x);
        let y = source.min.y
            + (p.y - destination.min.y) * (source.max.y - source.min.y)
                / (destination.max.y - destination.min.y);
        let value = Position { x, y };
        if !finite(value) {
            return Err(invalid("non-finite average fill coordinate"));
        }
        result.push(value);
    }
    Ok(result)
}
fn finite(p: Position) -> bool {
    p.x.is_finite() && p.y.is_finite()
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn unsupported(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn bounds() -> Bounds {
        Bounds {
            min: Position { x: 0., y: 0. },
            max: Position { x: 100., y: 60. },
        }
    }
    fn identity() -> Identity {
        Identity {
            scene_node: 3,
            grid_movie: 7,
            grid_tag: 11,
        }
    }
    fn grid() -> Rect {
        Rect {
            x_min: 10,
            x_max: 80,
            y_min: 12,
            y_max: 40,
        }
    }
    fn warp() -> GridWarp {
        GridWarp::new(
            identity(),
            bounds(),
            grid(),
            Matrix {
                scale_x: 131072,
                scale_y: 196608,
                translate_x: -5,
                translate_y: 7,
                ..Matrix::default()
            },
        )
        .unwrap()
    }
    fn rectangle() -> Vec<Position> {
        vec![
            Position { x: 0., y: 0. },
            Position { x: 100., y: 0. },
            Position { x: 100., y: 60. },
            Position { x: 0., y: 60. },
        ]
    }
    fn area(mesh: &WarpedMesh) -> f64 {
        mesh.indices
            .chunks_exact(3)
            .map(|i| {
                let a = mesh.positions[i[0] as usize];
                let b = mesh.positions[i[1] as usize];
                let c = mesh.positions[i[2] as usize];
                ((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)) * 0.5
            })
            .sum()
    }
    #[test]
    fn fixed_borders_origin_and_translation_are_preserved() {
        let w = warp();
        assert_eq!(
            w.map(Position { x: 10., y: 12. }).unwrap(),
            Position { x: 5., y: 19. }
        );
        assert_eq!(
            w.map(Position { x: 80., y: 40. }).unwrap(),
            Position { x: 175., y: 167. }
        );
        assert_eq!(
            w.destination_bounds(),
            Bounds {
                min: Position { x: -5., y: 7. },
                max: Position { x: 195., y: 187. }
            }
        );
        assert!(w.needs_deformation());
        let negative = GridWarp::new(
            identity(),
            Bounds {
                min: Position { x: -50., y: -30. },
                max: Position { x: 50., y: 30. },
            },
            Rect {
                x_min: -40,
                x_max: 30,
                y_min: -18,
                y_max: 10,
            },
            Matrix {
                scale_x: 131072,
                scale_y: 131072,
                ..Matrix::default()
            },
        )
        .unwrap();
        assert_eq!(
            negative.destination_bounds().min,
            Position { x: -100., y: -60. }
        );
        assert_eq!(
            negative.map(Position { x: -40., y: -18. }).unwrap(),
            Position { x: -90., y: -48. }
        );
    }
    #[test]
    fn triangles_split_at_grid_lines_with_independent_source_coordinates() {
        let p = rectangle();
        let source: Vec<_> = p
            .iter()
            .map(|p| Position {
                x: p.x * 2. + 30.,
                y: p.y * 3. - 20.,
            })
            .collect();
        let m = warp()
            .triangles(&p, &source, &[0, 1, 2, 0, 2, 3], Limits::default())
            .unwrap();
        assert_eq!(m.identity, identity());
        assert!(m.indices.len() > 6);
        assert!((area(&m) - 36000.).abs() < 1e-8);
        assert_eq!(m.positions.len(), m.source_positions.len());
        for (&dest, &source) in m.positions.iter().zip(&m.source_positions) {
            let original = Position {
                x: (source.x - 30.) / 2.,
                y: (source.y + 20.) / 3.,
            };
            let expected = warp().map(original).unwrap();
            assert!((dest.x - expected.x).abs() < 1e-9 && (dest.y - expected.y).abs() < 1e-9);
        }
        for tri in m.indices.chunks_exact(3) {
            let p: Vec<_> = tri.iter().map(|&i| m.positions[i as usize]).collect();
            for cut in [5., 175.] {
                assert!(!p.iter().any(|p| p.x < cut - 1e-9) || !p.iter().any(|p| p.x > cut + 1e-9));
            }
            for cut in [19., 167.] {
                assert!(!p.iter().any(|p| p.y < cut - 1e-9) || !p.iter().any(|p| p.y > cut + 1e-9));
            }
        }
    }
    #[test]
    fn unit_grid_is_plain_translation_and_average_policy_is_distinct() {
        let w = GridWarp::new(
            identity(),
            bounds(),
            grid(),
            Matrix {
                translate_x: 9,
                translate_y: -2,
                ..Matrix::default()
            },
        )
        .unwrap();
        assert!(!w.needs_deformation());
        let p = rectangle();
        let m = w
            .triangles(&p, &p, &[0, 1, 2, 0, 2, 3], Limits::default())
            .unwrap();
        assert_eq!(m.indices.len(), 6);
        assert_eq!(area(&m), 6000.);
        let w = warp();
        let p = [Position { x: 5., y: 19. }];
        let average = bounds_linear_sources(bounds(), w.destination_bounds(), &p).unwrap();
        assert_eq!(average[0], Position { x: 5., y: 4. });
        assert_ne!(average[0], Position { x: 10., y: 12. });
    }
    #[test]
    fn unresolved_transforms_grids_and_undersize_fail_explicitly() {
        for matrix in [
            Matrix {
                rotate_skew_0: 1,
                ..Matrix::default()
            },
            Matrix {
                scale_x: -65536,
                ..Matrix::default()
            },
            Matrix {
                scale_y: 0,
                ..Matrix::default()
            },
            Matrix {
                scale_x: 100,
                ..Matrix::default()
            },
        ] {
            assert_eq!(
                GridWarp::new(identity(), bounds(), grid(), matrix)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::Unsupported
            );
        }
        for splitter in [
            Rect {
                x_min: 0,
                x_max: 0,
                y_min: 1,
                y_max: 2,
            },
            Rect {
                x_min: -1,
                x_max: 10,
                y_min: 1,
                y_max: 2,
            },
        ] {
            assert!(GridWarp::new(identity(), bounds(), splitter, Matrix::default()).is_err());
        }
        assert!(warp().map(Position { x: f64::NAN, y: 0. }).is_err());
        assert!(warp().map(Position { x: 101., y: 0. }).is_err());
    }
    #[test]
    fn malformed_arrays_and_triangle_budgets_do_not_return_partial_meshes() {
        let p = rectangle();
        let w = warp();
        for indices in [&[0, 1][..], &[0, 1, 99][..]] {
            assert!(w.triangles(&p, &p, indices, Limits::default()).is_err());
        }
        assert!(
            w.triangles(&p, &p[..3], &[0, 1, 2], Limits::default())
                .is_err()
        );
        assert!(
            w.triangles(
                &p,
                &p,
                &[0, 1, 2, 0, 2, 3],
                Limits {
                    max_input_triangles: 1,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        assert!(
            w.triangles(
                &p,
                &p,
                &[0, 1, 2, 0, 2, 3],
                Limits {
                    max_output_triangles: 1,
                    ..Limits::default()
                }
            )
            .is_err()
        );
        assert!(
            bounds_linear_sources(
                Bounds {
                    min: Position { x: 0., y: 0. },
                    max: Position { x: 0., y: 1. }
                },
                bounds(),
                &p
            )
            .is_err()
        );
    }
}
