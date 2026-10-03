//! Glyph outlines as quadratic curves, and flattening them for a rasterizer.

/// A point in font units or pixels, y up.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ORIGIN: Point = Point { x: 0.0, y: 0.0 };

    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn translate(self, dx: f32, dy: f32) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }

    #[must_use]
    pub fn lerp(self, other: Point, t: f32) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }
}

impl std::ops::Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Sub for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }
}

/// One piece of a contour: a straight run, or a quadratic Bézier. The start
/// point is the end of the previous segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Segment {
    Line(Point),
    /// Control point, then end point.
    Quad(Point, Point),
}

/// A closed run of segments.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contour {
    pub start: Point,
    pub segments: Vec<Segment>,
}

impl Contour {
    /// The points the contour visits, for bounds and tests.
    pub fn points(&self) -> impl Iterator<Item = Point> + '_ {
        let mut current = self.start;
        let mut segments = self.segments.iter().peekable();
        std::iter::from_fn(move || {
            if let Some(segment) = segments.next() {
                current = match segment {
                    Segment::Line(to) | Segment::Quad(_, to) => *to,
                };
                return Some(current);
            }
            // Close the loop back onto the start.
            if current == self.start {
                None
            } else {
                current = self.start;
                Some(current)
            }
        })
    }
}

/// A glyph's outline: one or more closed contours, in font units.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outline {
    pub contours: Vec<Contour>,
}

impl Outline {
    /// Whether there is nothing to draw.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.contours
            .iter()
            .all(|contour| contour.segments.is_empty())
    }

    /// The tight bounds, or `None` for an empty outline.
    #[must_use]
    pub fn bounds(&self) -> Option<(Point, Point)> {
        let mut min = Point::new(f32::INFINITY, f32::INFINITY);
        let mut max = Point::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut any = false;
        for contour in &self.contours {
            for point in contour.points() {
                any = true;
                min.x = min.x.min(point.x);
                min.y = min.y.min(point.y);
                max.x = max.x.max(point.x);
                max.y = max.y.max(point.y);
            }
        }
        any.then_some((min, max))
    }

    /// The contours as polylines, subdividing curves until they are within
    /// `tolerance` of the true curve.
    ///
    /// Quadratics are flattened recursively: a curve is split in half until each
    /// half's control polygon deviates less than `tolerance`, which is the
    /// standard bound for a quadratic and keeps the flatness uniform as a glyph
    /// is scaled up.
    ///
    /// # Panics
    ///
    /// Panics on a non-finite `tolerance`, which would recurse until the stack
    /// ran out.
    #[must_use]
    pub fn flatten(&self, tolerance: f32) -> Vec<Vec<Point>> {
        self.contours
            .iter()
            .map(|contour| {
                let mut polyline = Vec::new();
                let mut current = contour.start;
                polyline.push(current);
                for segment in &contour.segments {
                    match segment {
                        Segment::Line(to) => {
                            polyline.push(*to);
                            current = *to;
                        }
                        Segment::Quad(control, to) => {
                            flatten_quadratic(current, *control, *to, tolerance, &mut polyline);
                            current = *to;
                        }
                    }
                }
                if polyline.len() > 1 && polyline[0] == *polyline.last().unwrap() {
                    polyline.pop();
                }
                polyline
            })
            .collect()
    }

    /// Moves the outline by `(dx, dy)`, used to place a composite's component.
    pub fn translate(&mut self, dx: f32, dy: f32) {
        for contour in &mut self.contours {
            contour.start = contour.start.translate(dx, dy);
            for segment in &mut contour.segments {
                *segment = match *segment {
                    Segment::Line(to) => Segment::Line(to.translate(dx, dy)),
                    Segment::Quad(control, to) => {
                        Segment::Quad(control.translate(dx, dy), to.translate(dx, dy))
                    }
                };
            }
        }
    }
}

/// Subdivides `quadratic` until both halves are flat enough.
fn flatten_quadratic(
    start: Point,
    control: Point,
    end: Point,
    tolerance: f32,
    out: &mut Vec<Point>,
) {
    // The distance from the control point to the chord is the upper bound on the
    // curve's deviation, so it is the thing to test against.
    let deviation = (control.x - start.x.midpoint(end.x)).powi(2)
        + (control.y - start.y.midpoint(end.y)).powi(2);
    let chord_x = end.x - start.x;
    let chord_y = end.y - start.y;
    let chord_sq = chord_x * chord_x + chord_y * chord_y;
    let flat = deviation <= tolerance * tolerance * chord_sq.max(1.0);

    if flat || out.len() > 64 {
        out.push(end);
        return;
    }
    // de Casteljau at t = 0.5.
    let start_control = start.lerp(control, 0.5);
    let control_end = control.lerp(end, 0.5);
    let middle = start_control.lerp(control_end, 0.5);
    flatten_quadratic(start, start_control, middle, tolerance, out);
    flatten_quadratic(middle, control_end, end, tolerance, out);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn polyline(segments: Vec<Segment>, start: Point) -> Vec<Vec<Point>> {
        let outline = Outline {
            contours: vec![Contour { start, segments }],
        };
        outline.flatten(0.05)
    }

    #[test]
    fn a_line_is_not_subdivided() {
        let actual = polyline(vec![Segment::Line(Point::new(10.0, 0.0))], Point::ORIGIN);
        let expected = vec![vec![Point::ORIGIN, Point::new(10.0, 0.0)]];
        assert_eq!(actual, expected);
    }

    #[test]
    fn a_flat_curve_collapses_to_its_end() {
        // Control point on the chord: nothing to subdivide.
        let actual = polyline(
            vec![Segment::Quad(Point::new(5.0, 0.0), Point::new(10.0, 0.0))],
            Point::ORIGIN,
        );
        let expected = vec![vec![Point::ORIGIN, Point::new(10.0, 0.0)]];
        assert_eq!(actual, expected);
    }

    #[test]
    fn a_curved_arc_is_subdivided_and_stays_on_the_curve() {
        let control = Point::new(5.0, 10.0);
        let end = Point::new(10.0, 0.0);
        let points = polyline(vec![Segment::Quad(control, end)], Point::ORIGIN);
        let line = points[0].clone();
        assert!(line.len() > 3, "expected subdivision, got {line:?}");

        // Every interior point must sit on the true curve. For
        // P0=(0,0) P1=(5,10) P2=(10,0) that is x = 10t and y = 20t(1-t).
        for point in line.iter().skip(1).take(line.len().saturating_sub(2)) {
            let t = point.x / 10.0;
            let expected_y = 20.0 * t * (1.0 - t);
            assert!(
                (point.y - expected_y).abs() < 0.1,
                "{point:?} is not on the curve, expected y {expected_y}"
            );
        }
    }

    #[test]
    fn the_flattened_polyline_stays_inside_the_control_polygon() {
        let control = Point::new(5.0, 10.0);
        let points = polyline(
            vec![Segment::Quad(control, Point::new(10.0, 0.0))],
            Point::ORIGIN,
        );
        for point in &points[0] {
            assert!(point.x >= -0.01 && point.x <= 10.01, "x {point:?}");
            assert!(point.y >= -0.01 && point.y <= 10.01, "y {point:?}");
        }
    }

    #[test]
    fn bounds_cover_every_contour() {
        let outline = Outline {
            contours: vec![
                Contour {
                    start: Point::new(-5.0, 0.0),
                    segments: vec![Segment::Line(Point::new(0.0, 7.0))],
                },
                Contour {
                    start: Point::new(1.0, -3.0),
                    segments: vec![Segment::Line(Point::new(9.0, 2.0))],
                },
            ],
        };
        let actual = outline.bounds().unwrap();
        let expected = (Point::new(-5.0, -3.0), Point::new(9.0, 7.0));
        assert_eq!(actual, expected);
    }

    #[test]
    fn an_empty_outline_has_no_bounds() {
        assert_eq!(Outline::default().bounds(), None);
        assert!(Outline::default().is_empty());
    }

    #[test]
    fn translation_moves_every_point() {
        let mut outline = Outline {
            contours: vec![Contour {
                start: Point::new(1.0, 2.0),
                segments: vec![Segment::Quad(Point::new(3.0, 4.0), Point::new(5.0, 6.0))],
            }],
        };
        outline.translate(10.0, 20.0);
        let contour = &outline.contours[0];
        assert_eq!(contour.start, Point::new(11.0, 22.0));
        assert_eq!(
            contour.segments[0],
            Segment::Quad(Point::new(13.0, 24.0), Point::new(15.0, 26.0))
        );
    }

    #[test]
    fn a_contour_closes_back_onto_its_start() {
        let contour = Contour {
            start: Point::new(0.0, 0.0),
            segments: vec![
                Segment::Line(Point::new(10.0, 0.0)),
                Segment::Line(Point::new(10.0, 10.0)),
            ],
        };
        let points: Vec<Point> = contour.points().collect();
        let expected = vec![
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 0.0),
        ];
        assert_eq!(points, expected);
    }
}
