//! Original bounded fill tessellation from decoded SWF directed edges.
//! No transforms, texture sampling, stroke expansion or screen-layout guesses.

use super::Matrix;
use super::shapes::{Fill, FillRule, FillStyle, Geometry, Point, Shape, StyleRef};
use std::collections::BTreeMap;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}
impl From<Point> for Position {
    fn from(p: Point) -> Self {
        Self {
            x: f64::from(p.x),
            y: f64::from(p.y),
        }
    }
}
impl Position {
    fn middle(self, other: Self) -> Self {
        Self {
            x: (self.x + other.x) * 0.5,
            y: (self.y + other.y) * 0.5,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// Maximum quadratic-to-chord displacement, in shape twips (20 per pixel).
    pub curve_tolerance: f64,
    pub max_curve_depth: u8,
    pub max_segments: usize,
    /// Bounds active-interval intersection checks and slab crossing visits.
    pub max_work: usize,
    pub max_triangles: usize,
    /// None uses Shape.fill_rule, whose Shape4 nonzero interpretation is inferred.
    pub fill_rule_override: Option<FillRule>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            curve_tolerance: 0.25,
            max_curve_depth: 24,
            max_segments: 250_000,
            max_work: 8_000_000,
            max_triangles: 500_000,
            fill_rule_override: None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectedEdge {
    pub edge_index: usize,
    pub reversed: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Contour {
    /// Original directed-edge identities; right-fill edges are reversed so every
    /// emitted contour has its assigned fill on the same side.
    pub source_edges: Vec<DirectedEdge>,
    /// Closed, flattened path in shape twips; last point equals the first.
    pub points: Vec<Position>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct FillMesh {
    pub style: StyleRef,
    /// Original fill including its exact matrix, stops and bitmap identifier.
    pub paint: FillStyle,
    pub contours: Vec<Contour>,
    pub positions: Vec<Position>,
    /// Triangles use consistent positive algebraic area in shape coordinates.
    pub indices: Vec<u32>,
    /// Multiple outgoing directed edges at a vertex; all closed walks are kept.
    pub branching_vertices: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub shape_id: u16,
    pub fill_rule: FillRule,
    pub inferred_winding: bool,
    pub fills: Vec<FillMesh>,
    /// Stroke geometry is still available in Shape, but not expanded here.
    pub stroke_edges: usize,
    pub unpainted_edges: usize,
    /// Same selected fill on both sides cancels this internal boundary.
    pub internal_edges: usize,
    pub flattened_segments: usize,
    pub work: usize,
}
struct Budget {
    segments: usize,
    work: usize,
    triangles: usize,
    options: Options,
}
impl Budget {
    fn segment(&mut self) -> io::Result<()> {
        self.segments = self
            .segments
            .checked_add(1)
            .ok_or_else(|| invalid("segment overflow"))?;
        if self.segments > self.options.max_segments {
            return Err(invalid("tessellation segment budget"));
        }
        Ok(())
    }
    fn work(&mut self, count: usize) -> io::Result<()> {
        self.work = self
            .work
            .checked_add(count)
            .ok_or_else(|| invalid("work overflow"))?;
        if self.work > self.options.max_work {
            return Err(invalid("tessellation work budget"));
        }
        Ok(())
    }
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn key(p: Point) -> (i32, i32) {
    (p.x, p.y)
}
fn endpoints(shape: &Shape, edge: DirectedEdge) -> (Point, Point) {
    let e = &shape.edges[edge.edge_index];
    if edge.reversed {
        (e.to, e.from)
    } else {
        (e.from, e.to)
    }
}

pub fn tessellate(shape: &Shape, options: Options) -> io::Result<Mesh> {
    if !options.curve_tolerance.is_finite()
        || options.curve_tolerance <= 0.0
        || options.max_curve_depth > 32
    {
        return Err(invalid("invalid tessellation tolerance or depth"));
    }
    if !shape.unsupported.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "shape has unsupported semantic fields",
        ));
    }
    let fill_rule = options.fill_rule_override.unwrap_or(shape.fill_rule);
    let inferred_winding =
        options.fill_rule_override.is_none() && shape.flags.is_some_and(|f| f.raw & 4 != 0);
    let mut mesh = Mesh {
        shape_id: shape.id,
        fill_rule,
        inferred_winding,
        fills: Vec::new(),
        stroke_edges: 0,
        unpainted_edges: 0,
        internal_edges: 0,
        flattened_segments: 0,
        work: 0,
    };
    let mut groups: BTreeMap<(usize, u16), Vec<DirectedEdge>> = BTreeMap::new();
    let mut budget = Budget {
        segments: 0,
        work: 0,
        triangles: 0,
        options,
    };
    for (edge_index, e) in shape.edges.iter().enumerate() {
        budget.work(1)?;
        if e.line.is_some() {
            mesh.stroke_edges += 1;
        }
        if e.fill_left.is_none() && e.fill_right.is_none() {
            mesh.unpainted_edges += 1;
        }
        if e.fill_left == e.fill_right {
            if e.fill_left.is_some() {
                mesh.internal_edges += 1;
            }
            continue;
        }
        for (style, reversed) in [(e.fill_left, false), (e.fill_right, true)] {
            if let Some(style) = style {
                if shape.fill(style).is_none() {
                    return Err(invalid("unresolved fill reference"));
                }
                groups
                    .entry((style.set, style.index))
                    .or_default()
                    .push(DirectedEdge {
                        edge_index,
                        reversed,
                    });
            }
        }
    }
    for ((set, index), edges) in groups {
        let style = StyleRef { set, index };
        let (contours, branching_vertices) = contours(shape, &edges, &mut budget)?;
        let (positions, indices) = triangulate(&contours, fill_rule, &mut budget)?;
        mesh.fills.push(FillMesh {
            style,
            paint: shape.fill(style).unwrap().clone(),
            contours,
            positions,
            indices,
            branching_vertices,
        });
    }
    mesh.flattened_segments = budget.segments;
    mesh.work = budget.work;
    Ok(mesh)
}

fn contours(
    shape: &Shape,
    edges: &[DirectedEdge],
    budget: &mut Budget,
) -> io::Result<(Vec<Contour>, usize)> {
    let mut outgoing: BTreeMap<(i32, i32), Vec<usize>> = BTreeMap::new();
    let mut balance: BTreeMap<(i32, i32), i64> = BTreeMap::new();
    for (i, &edge) in edges.iter().enumerate() {
        let (from, to) = endpoints(shape, edge);
        outgoing.entry(key(from)).or_default().push(i);
        *balance.entry(key(from)).or_default() += 1;
        *balance.entry(key(to)).or_default() -= 1;
    }
    if let Some((p, _)) = balance.iter().find(|(_, n)| **n != 0) {
        return Err(invalid(&format!(
            "open directed fill contour at ({},{})",
            p.0, p.1
        )));
    }
    let branching = outgoing.values().filter(|v| v.len() > 1).count();
    // Reverse adjacency once, then consume the original source order by pop.
    for list in outgoing.values_mut() {
        list.reverse();
    }
    let mut used = vec![false; edges.len()];
    let mut result = Vec::new();
    for seed in 0..edges.len() {
        if used[seed] {
            continue;
        }
        let (start, _) = endpoints(shape, edges[seed]);
        let mut point = start;
        let mut contour = Contour {
            source_edges: Vec::new(),
            points: vec![start.into()],
        };
        loop {
            let list = outgoing
                .get_mut(&key(point))
                .ok_or_else(|| invalid("missing contour adjacency"))?;
            while list.last().is_some_and(|i| used[*i]) {
                list.pop();
            }
            let next = list.pop().ok_or_else(|| invalid("unclosed fill contour"))?;
            used[next] = true;
            let edge = edges[next];
            let (from, to) = endpoints(shape, edge);
            let geometry = &shape.edges[edge.edge_index].geometry;
            match *geometry {
                Geometry::Line => push_point(&mut contour.points, to.into(), budget)?,
                Geometry::Quadratic { control } => flatten(
                    from.into(),
                    control.into(),
                    to.into(),
                    &mut contour.points,
                    budget,
                )?,
            }
            contour.source_edges.push(edge);
            point = to;
            if point == start {
                break;
            }
        }
        result.push(contour);
    }
    Ok((result, branching))
}
fn push_point(points: &mut Vec<Position>, point: Position, budget: &mut Budget) -> io::Result<()> {
    if points.last() == Some(&point) {
        return Ok(());
    }
    budget.segment()?;
    points.push(point);
    Ok(())
}
fn flatten(
    from: Position,
    control: Position,
    to: Position,
    points: &mut Vec<Position>,
    budget: &mut Budget,
) -> io::Result<()> {
    let mut stack = vec![(from, control, to, 0u8)];
    while let Some((a, b, c, depth)) = stack.pop() {
        budget.work(1)?;
        // B(t)-chord(t) = t(1-t)*(2B-A-C), bounded by |2B-A-C|/4.
        let error = (2.0 * b.x - a.x - c.x).hypot(2.0 * b.y - a.y - c.y) * 0.25;
        if error <= budget.options.curve_tolerance {
            push_point(points, c, budget)?;
            continue;
        }
        if depth >= budget.options.max_curve_depth {
            return Err(invalid("quadratic subdivision depth limit"));
        }
        let ab = a.middle(b);
        let bc = b.middle(c);
        let middle = ab.middle(bc);
        stack.push((middle, bc, c, depth + 1));
        stack.push((a, ab, middle, depth + 1));
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Segment {
    a: Position,
    b: Position,
    winding: i64,
}
impl Segment {
    fn new(a: Position, b: Position) -> Option<Self> {
        if a.y == b.y {
            return None;
        }
        Some(if a.y < b.y {
            Self { a, b, winding: 1 }
        } else {
            Self {
                a: b,
                b: a,
                winding: -1,
            }
        })
    }
    fn x(self, y: f64) -> f64 {
        self.a.x + (self.b.x - self.a.x) * ((y - self.a.y) / (self.b.y - self.a.y))
    }
}
fn cross(a: Position, b: Position) -> f64 {
    a.x * b.y - a.y * b.x
}
fn sub(a: Position, b: Position) -> Position {
    Position {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}
fn intersection_y(a: Segment, b: Segment) -> Option<f64> {
    let r = sub(a.b, a.a);
    let s = sub(b.b, b.a);
    let denominator = cross(r, s);
    if denominator == 0.0 {
        return None;
    }
    let q = sub(b.a, a.a);
    let t = cross(q, s) / denominator;
    let u = cross(q, r) / denominator;
    if t > 0.0 && t < 1.0 && u > 0.0 && u < 1.0 {
        let y = a.a.y + t * r.y;
        if y > a.a.y.max(b.a.y) && y < a.b.y.min(b.b.y) {
            return Some(y);
        }
    }
    None
}
fn triangulate(
    contours: &[Contour],
    rule: FillRule,
    budget: &mut Budget,
) -> io::Result<(Vec<Position>, Vec<u32>)> {
    let mut segments = Vec::new();
    let mut events = Vec::new();
    for contour in contours {
        if contour.points.first() != contour.points.last() {
            return Err(invalid("flattened contour is open"));
        }
        for pair in contour.points.windows(2) {
            if let Some(segment) = Segment::new(pair[0], pair[1]) {
                events.push(segment.a.y);
                events.push(segment.b.y);
                segments.push(segment);
            }
        }
    }
    segments.sort_by(|a, b| a.a.y.total_cmp(&b.a.y).then(a.a.x.total_cmp(&b.a.x)));
    let mut active: Vec<usize> = Vec::new();
    // Only segments with overlapping Y extents can cross; work is explicitly bounded.
    for (i, &segment) in segments.iter().enumerate() {
        active.retain(|&j| segments[j].b.y > segment.a.y);
        budget.work(active.len())?;
        for &j in &active {
            if let Some(y) = intersection_y(segment, segments[j]) {
                events.push(y);
            }
        }
        active.push(i);
    }
    events.sort_by(f64::total_cmp);
    events.dedup();
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    active.clear();
    let mut next = 0;
    for pair in events.windows(2) {
        let (bottom, top) = (pair[0], pair[1]);
        let middle = bottom + (top - bottom) * 0.5;
        if middle == bottom || middle == top {
            continue;
        }
        while next < segments.len() && segments[next].a.y <= bottom {
            active.push(next);
            next += 1;
        }
        active.retain(|&j| segments[j].b.y >= top);
        budget.work(active.len())?;
        let mut crossings: Vec<(f64, usize)> =
            active.iter().map(|&j| (segments[j].x(middle), j)).collect();
        crossings.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut winding = 0i64;
        for pair in crossings.windows(2) {
            winding += segments[pair[0].1].winding;
            let inside = match rule {
                FillRule::EvenOdd => winding % 2 != 0,
                FillRule::NonZero => winding != 0,
            };
            if !inside || pair[0].0 == pair[1].0 {
                continue;
            }
            let left = segments[pair[0].1];
            let right = segments[pair[1].1];
            let p = [
                Position {
                    x: left.x(bottom),
                    y: bottom,
                },
                Position {
                    x: right.x(bottom),
                    y: bottom,
                },
                Position {
                    x: right.x(top),
                    y: top,
                },
                Position {
                    x: left.x(top),
                    y: top,
                },
            ];
            triangle([p[0], p[1], p[2]], &mut positions, &mut indices, budget)?;
            triangle([p[0], p[2], p[3]], &mut positions, &mut indices, budget)?;
        }
        if let Some(last) = crossings.last() {
            winding += segments[last.1].winding;
        }
        if winding != 0 {
            return Err(invalid("unbalanced scanline winding"));
        }
    }
    Ok((positions, indices))
}
fn triangle(
    mut p: [Position; 3],
    positions: &mut Vec<Position>,
    indices: &mut Vec<u32>,
    budget: &mut Budget,
) -> io::Result<()> {
    let area = cross(sub(p[1], p[0]), sub(p[2], p[0]));
    if !area.is_finite() {
        return Err(invalid("non-finite triangle"));
    }
    if area == 0.0 {
        return Ok(());
    }
    if area < 0.0 {
        p.swap(1, 2);
    }
    budget.triangles = budget
        .triangles
        .checked_add(1)
        .ok_or_else(|| invalid("triangle count overflow"))?;
    if budget.triangles > budget.options.max_triangles {
        return Err(invalid("triangle output budget"));
    }
    let index =
        u32::try_from(positions.len()).map_err(|_| invalid("triangle vertex index overflow"))?;
    index
        .checked_add(2)
        .ok_or_else(|| invalid("triangle vertex index overflow"))?;
    positions.extend(p);
    indices.extend([index, index + 1, index + 2]);
    Ok(())
}

/// Invert a preserved SWF fill matrix. The output is bitmap pixel space or the
/// gradient's native coordinate space, not normalized UVs. Solid fills return None.
pub fn fill_coordinates(fill: &Fill, point: Position) -> io::Result<Option<Position>> {
    match fill {
        Fill::Solid(_) => Ok(None),
        Fill::Bitmap { matrix, .. } | Fill::Gradient { matrix, .. } => {
            inverse_matrix(*matrix, point).map(Some)
        }
    }
}
fn inverse_matrix(matrix: Matrix, point: Position) -> io::Result<Position> {
    let a = f64::from(matrix.scale_x) / 65536.0;
    let b = f64::from(matrix.rotate_skew_1) / 65536.0;
    let c = f64::from(matrix.rotate_skew_0) / 65536.0;
    let d = f64::from(matrix.scale_y) / 65536.0;
    let det = a * d - b * c;
    if det == 0.0 || !point.x.is_finite() || !point.y.is_finite() {
        return Err(invalid("singular fill matrix or non-finite position"));
    }
    let x = point.x - f64::from(matrix.translate_x);
    let y = point.y - f64::from(matrix.translate_y);
    let value = Position {
        x: (d * x - b * y) / det,
        y: (a * y - c * x) / det,
    };
    if !value.x.is_finite() || !value.y.is_finite() {
        return Err(invalid("non-finite fill coordinates"));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::super::shapes::{BitSpan, Color, Edge, Shape4Flags, StyleSet};
    use super::super::{Rect, Span};
    use super::*;
    fn shape() -> Shape {
        Shape {
            source: Span { offset: 0, len: 0 },
            tag_code: 32,
            version: 3,
            id: 1,
            bounds: Rect {
                x_min: 0,
                x_max: 100,
                y_min: 0,
                y_max: 100,
            },
            edge_bounds: None,
            fill_rule: FillRule::EvenOdd,
            flags: None,
            styles: vec![StyleSet {
                source: Span { offset: 0, len: 0 },
                fills: vec![FillStyle {
                    source: Span { offset: 0, len: 0 },
                    fill: Fill::Solid(Color {
                        red: 10,
                        green: 20,
                        blue: 30,
                        alpha: 255,
                    }),
                }],
                lines: Vec::new(),
            }],
            records: Vec::new(),
            edges: Vec::new(),
            unsupported: Vec::new(),
        }
    }
    fn polygon(shape: &mut Shape, points: &[(i32, i32)], right: bool) {
        let style = Some(StyleRef { set: 0, index: 1 });
        for i in 0..points.len() {
            let a = points[i];
            let b = points[(i + 1) % points.len()];
            shape.edges.push(Edge {
                source: BitSpan {
                    start: shape.edges.len() * 8,
                    len: 8,
                },
                from: Point { x: a.0, y: a.1 },
                to: Point { x: b.0, y: b.1 },
                geometry: Geometry::Line,
                fill_left: if right { None } else { style },
                fill_right: if right { style } else { None },
                line: None,
            });
        }
    }
    fn area(mesh: &Mesh) -> f64 {
        mesh.fills
            .iter()
            .map(|f| {
                f.indices
                    .chunks_exact(3)
                    .map(|i| {
                        let a = f.positions[i[0] as usize];
                        let b = f.positions[i[1] as usize];
                        let c = f.positions[i[2] as usize];
                        cross(sub(b, a), sub(c, a)) / 2.0
                    })
                    .sum::<f64>()
            })
            .sum()
    }
    fn covers(mesh: &Mesh, p: Position) -> bool {
        mesh.fills.iter().any(|f| {
            f.indices.chunks_exact(3).any(|i| {
                let a = f.positions[i[0] as usize];
                let b = f.positions[i[1] as usize];
                let c = f.positions[i[2] as usize];
                cross(sub(b, a), sub(p, a)) >= 0.0
                    && cross(sub(c, b), sub(p, b)) >= 0.0
                    && cross(sub(a, c), sub(p, c)) >= 0.0
            })
        })
    }
    #[test]
    fn holes_respect_evenodd_and_both_nonzero_orientations() {
        let mut s = shape();
        polygon(&mut s, &[(0, 0), (10, 0), (10, 10), (0, 10)], false);
        polygon(&mut s, &[(3, 3), (7, 3), (7, 7), (3, 7)], false);
        let even = tessellate(&s, Options::default()).unwrap();
        assert!((area(&even) - 84.0).abs() < 1e-10);
        assert!(!covers(&even, Position { x: 5.0, y: 5.0 }));
        assert!(covers(&even, Position { x: 1.0, y: 5.0 }));
        let nonzero = tessellate(
            &s,
            Options {
                fill_rule_override: Some(FillRule::NonZero),
                ..Options::default()
            },
        )
        .unwrap();
        assert!((area(&nonzero) - 100.0).abs() < 1e-10);
        assert!(covers(&nonzero, Position { x: 5.0, y: 5.0 }));
        for edge in &mut s.edges[4..] {
            std::mem::swap(&mut edge.from, &mut edge.to);
        }
        let hole = tessellate(
            &s,
            Options {
                fill_rule_override: Some(FillRule::NonZero),
                ..Options::default()
            },
        )
        .unwrap();
        assert!((area(&hole) - 84.0).abs() < 1e-10);
        assert!(!covers(&hole, Position { x: 5.0, y: 5.0 }));
    }
    #[test]
    fn disjoint_concave_and_crossing_contours_keep_actual_area() {
        let mut s = shape();
        polygon(
            &mut s,
            &[(0, 0), (10, 0), (10, 5), (5, 5), (5, 10), (0, 10)],
            false,
        );
        polygon(&mut s, &[(20, 0), (25, 0), (25, 5), (20, 5)], true);
        let mesh = tessellate(&s, Options::default()).unwrap();
        assert!((area(&mesh) - 100.0).abs() < 1e-10);
        assert_eq!(mesh.fills[0].contours.len(), 2);
        assert!(!covers(&mesh, Position { x: 7.0, y: 7.0 }));
        let mut s = shape();
        polygon(&mut s, &[(0, 0), (10, 10), (0, 10), (10, 0)], false);
        let mesh = tessellate(&s, Options::default()).unwrap();
        assert!((area(&mesh) - 50.0).abs() < 1e-10);
        assert!(
            mesh.fills[0]
                .positions
                .contains(&Position { x: 5.0, y: 5.0 })
        );
    }
    #[test]
    fn coincident_edges_cancel_and_touching_branches_do_not_bridge() {
        let mut s = shape();
        polygon(&mut s, &[(0, 0), (10, 0), (10, 10), (0, 10)], false);
        polygon(&mut s, &[(10, 10), (20, 10), (20, 20), (10, 20)], false);
        let mesh = tessellate(&s, Options::default()).unwrap();
        assert_eq!(mesh.fills[0].branching_vertices, 1);
        assert!((area(&mesh) - 200.0).abs() < 1e-10);
        assert!(!covers(&mesh, Position { x: 5.0, y: 15.0 }));
        let mut s = shape();
        polygon(&mut s, &[(0, 0), (10, 0), (10, 10), (0, 10)], false);
        polygon(&mut s, &[(0, 0), (10, 0), (10, 10), (0, 10)], false);
        assert_eq!(area(&tessellate(&s, Options::default()).unwrap()), 0.0);
        assert_eq!(
            area(
                &tessellate(
                    &s,
                    Options {
                        fill_rule_override: Some(FillRule::NonZero),
                        ..Options::default()
                    }
                )
                .unwrap()
            ),
            100.0
        );
    }
    #[test]
    fn quadratic_flattening_is_bounded_and_preserves_reversal() {
        let mut s = shape();
        polygon(&mut s, &[(0, 0), (20, 0)], false);
        s.edges[0].geometry = Geometry::Quadratic {
            control: Point { x: 10, y: 20 },
        };
        let mesh = tessellate(
            &s,
            Options {
                curve_tolerance: 0.001,
                ..Options::default()
            },
        )
        .unwrap();
        assert!((area(&mesh) - 400.0 / 3.0).abs() < 0.02);
        let contour = &mesh.fills[0].contours[0];
        assert_eq!(contour.points.first(), contour.points.last());
        assert!(contour.points.len() > 8);
        for edge in &mut s.edges {
            edge.fill_right = edge.fill_left;
            edge.fill_left = None;
        }
        let reversed = tessellate(
            &s,
            Options {
                curve_tolerance: 0.001,
                ..Options::default()
            },
        )
        .unwrap();
        assert!((area(&mesh) - area(&reversed)).abs() < 1e-8);
        assert!(
            reversed.fills[0]
                .contours
                .iter()
                .flat_map(|c| &c.source_edges)
                .all(|e| e.reversed)
        );
        assert!(
            tessellate(
                &s,
                Options {
                    max_curve_depth: 0,
                    ..Options::default()
                }
            )
            .is_err()
        );
    }
    #[test]
    fn style_banks_and_paint_transforms_are_retained() {
        let mut s = shape();
        polygon(&mut s, &[(0, 0), (10, 0), (10, 10), (0, 10)], false);
        let matrix = Matrix {
            scale_x: 131072,
            scale_y: 196608,
            rotate_skew_0: -16384,
            rotate_skew_1: 32768,
            translate_x: 10,
            translate_y: -20,
        };
        s.styles[0].fills[0].fill = Fill::Bitmap {
            character_id: 44,
            matrix,
            repeating: false,
            smoothed: true,
        };
        s.styles.push(s.styles[0].clone());
        polygon(&mut s, &[(20, 0), (30, 0), (30, 10), (20, 10)], false);
        for edge in &mut s.edges[4..] {
            edge.fill_left = Some(StyleRef { set: 1, index: 1 });
        }
        let mesh = tessellate(&s, Options::default()).unwrap();
        assert_eq!(mesh.fills.len(), 2);
        assert_eq!(mesh.fills[1].style, StyleRef { set: 1, index: 1 });
        assert_eq!(mesh.fills[0].paint, s.styles[0].fills[0]);
        // Matrix maps native (4,8) to shape (22,3).
        assert_eq!(
            fill_coordinates(&mesh.fills[0].paint.fill, Position { x: 22.0, y: 3.0 }).unwrap(),
            Some(Position { x: 4.0, y: 8.0 })
        );
        assert!(
            inverse_matrix(
                Matrix {
                    scale_x: 0,
                    scale_y: 0,
                    ..Matrix::default()
                },
                Position { x: 0.0, y: 0.0 }
            )
            .is_err()
        );
    }
    #[test]
    fn open_paths_and_budgets_fail_without_fabricated_closure() {
        let mut s = shape();
        polygon(&mut s, &[(0, 0), (10, 0), (10, 10), (0, 10)], false);
        for options in [
            Options {
                max_segments: 1,
                ..Options::default()
            },
            Options {
                max_work: 1,
                ..Options::default()
            },
            Options {
                max_triangles: 1,
                ..Options::default()
            },
            Options {
                curve_tolerance: f64::NAN,
                ..Options::default()
            },
            Options {
                curve_tolerance: 0.0,
                ..Options::default()
            },
        ] {
            assert!(tessellate(&s, options).is_err());
        }
        s.edges.pop();
        assert!(
            tessellate(&s, Options::default())
                .unwrap_err()
                .to_string()
                .contains("open directed")
        );
    }
    #[test]
    fn internal_boundaries_and_unrendered_strokes_are_explicit() {
        let mut s = shape();
        polygon(&mut s, &[(0, 0), (10, 0), (10, 10), (0, 10)], false);
        let mut edge = s.edges[0].clone();
        edge.fill_right = edge.fill_left;
        edge.line = Some(StyleRef { set: 0, index: 1 });
        s.edges.push(edge);
        s.flags = Some(Shape4Flags {
            raw: 4,
            uses_non_scaling_strokes: false,
            uses_scaling_strokes: false,
        });
        s.fill_rule = FillRule::NonZero;
        let mesh = tessellate(&s, Options::default()).unwrap();
        assert_eq!(mesh.internal_edges, 1);
        assert_eq!(mesh.stroke_edges, 1);
        assert_eq!(area(&mesh), 100.0);
        assert!(mesh.inferred_winding);
        assert!(
            !tessellate(
                &s,
                Options {
                    fill_rule_override: Some(FillRule::EvenOdd),
                    ..Options::default()
                }
            )
            .unwrap()
            .inferred_winding
        );
    }
}
