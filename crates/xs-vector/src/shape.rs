//! Ready-made shapes.
//!
//! Every one of these is a [`Path`] with a paint style, and every one can be
//! replaced by hand-written path data. They are here because a circle written as
//! two arcs is a line you have to get right every time, and this crate has already
//! got it right.

use xs_layout::Color;
use xs_math::Vector2;

use crate::mesh::Shape;
use crate::path::Path;
use crate::stroke::{LineCap, Stroke};

/// An axis-aligned rectangle, corner at `(x, y)`.
#[must_use]
pub fn rect(x: f32, y: f32, width: f32, height: f32) -> Shape {
    Shape::new(Path::build(|p| {
        p.move_to(x, y)
            .line_to(x + width, y)
            .line_to(x + width, y + height)
            .line_to(x, y + height)
            .close();
    }))
}

/// A rectangle with rounded corners.
///
/// Each corner is a quarter turn of an arc, which is the case where an arc's
/// endpoint parameterisation has to be right: getting the sweep direction wrong
/// draws a bow outside the rectangle instead of inside it.
#[must_use]
pub fn round_rect(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Shape {
    let r = radius.min(width * 0.5).min(height * 0.5).max(0.0);
    Shape::new(Path::build(|p| {
        p.move_to(x + r, y)
            .line_to(x + width - r, y)
            .arc_to(r, r, 0.0, false, true, x + width, y + r)
            .line_to(x + width, y + height - r)
            .arc_to(r, r, 0.0, false, true, x + width - r, y + height)
            .line_to(x + r, y + height)
            .arc_to(r, r, 0.0, false, true, x, y + height - r)
            .line_to(x, y + r)
            .arc_to(r, r, 0.0, false, true, x + r, y)
            .close();
    }))
}

/// A circle, as two half arcs.
///
/// Two rather than four quarters because a full circle cannot be one arc: SVG's
/// endpoint form has no way to say "and back to where you started", so a whole
/// turn is two of them.
#[must_use]
pub fn circle(center_x: f32, center_y: f32, radius: f32) -> Shape {
    ellipse(center_x, center_y, radius, radius)
}

/// An ellipse, as two half arcs.
///
/// The two arcs sweep the **same** way, which reads as a mistake until it is not:
/// each is a half turn, and two half turns in one direction make a whole one. It is
/// giving them *opposite* flags that breaks it - that traces the same half over
/// again and leaves the other half undrawn. And it breaks quietly, because every
/// point of the result is still exactly on the ellipse, so a check on the radii
/// passes and only the filled area comes out halved.
#[must_use]
pub fn ellipse(center_x: f32, center_y: f32, radius_x: f32, radius_y: f32) -> Shape {
    Shape::new(Path::build(|p| {
        p.move_to(center_x + radius_x, center_y)
            .arc_to(
                radius_x,
                radius_y,
                0.0,
                false,
                false,
                center_x - radius_x,
                center_y,
            )
            .arc_to(
                radius_x,
                radius_y,
                0.0,
                false,
                false,
                center_x + radius_x,
                center_y,
            )
            .close();
    }))
}

/// A line from `(x0, y0)` to `(x1, y1)`, which has no interior and so only ever
/// carries a stroke.
#[must_use]
pub fn line(x0: f32, y0: f32, x1: f32, y1: f32) -> Shape {
    Shape::new(Path::build(|p| {
        p.move_to(x0, y0).line_to(x1, y1);
    }))
}

/// A regular polygon, first vertex straight up.
#[must_use]
pub fn polygon(sides: u32, center_x: f32, center_y: f32, radius: f32) -> Shape {
    let points = regular_points(sides, center_x, center_y, radius);
    Shape::new(closed_path(&points)).fill(Color::WHITE)
}

/// A star, with `points` spikes alternating between `outer` and `inner`.
///
/// `inner` is a fraction of `outer` rather than its own radius, because that is the
/// proportion anyone actually has an opinion about.
#[must_use]
pub fn star(points: u32, center_x: f32, center_y: f32, outer: f32, inner: f32) -> Shape {
    let sides = points.max(3) * 2;
    let vertices = regular_points(sides, center_x, center_y, outer);
    let mut alternating = Vec::with_capacity(vertices.len());
    for (index, point) in vertices.into_iter().enumerate() {
        if index % 2 == 0 {
            alternating.push(point);
        } else {
            let from_center = point - Vector2::new(center_x, center_y);
            alternating.push(Vector2::new(center_x, center_y) + from_center * (inner / outer));
        }
    }
    Shape::new(closed_path(&alternating)).fill(Color::WHITE)
}

/// The corners of a regular polygon, first vertex straight up.
///
/// Straight up rather than along an axis because that is how a star is drawn, and
/// having the two agree matters more than either convention being the only one.
fn regular_points(sides: u32, center_x: f32, center_y: f32, radius: f32) -> Vec<Vector2> {
    let sides = sides.max(3);
    let center = Vector2::new(center_x, center_y);
    (0..sides)
        .map(|index| {
            let angle =
                -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * index as f32 / sides as f32;
            center + Vector2::new(angle.cos() * radius, angle.sin() * radius)
        })
        .collect()
}

fn closed_path(points: &[Vector2]) -> Path {
    Path::build(|p| {
        for (index, point) in points.iter().enumerate() {
            if index == 0 {
                p.move_to(point.x, point.y);
            } else {
                p.line_to(point.x, point.y);
            }
        }
        p.close();
    })
}

impl Shape {
    /// Fills with `color`.
    #[must_use]
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(crate::mesh::Fill::new(color));
        self
    }

    /// Strokes with `color` at `width` pixels.
    #[must_use]
    pub fn stroke(mut self, color: Color, width: f32) -> Self {
        self.stroke = Some(Stroke::new(color, width));
        self
    }

    /// Sets the stroke's cap, keeping the rest.
    #[must_use]
    pub fn cap(mut self, cap: LineCap) -> Self {
        if let Some(stroke) = self.stroke.as_mut() {
            stroke.cap = cap;
        }
        self
    }

    /// Sets the dash pattern, keeping the rest.
    ///
    /// The pattern is normalised - non-positive entries dropped, an odd-length one
    /// doubled - so this and `Stroke::with_dash` cannot disagree about what a
    /// pattern means.
    #[must_use]
    pub fn dash(mut self, pattern: impl Into<Vec<f32>>) -> Self {
        if let Some(stroke) = self.stroke.as_mut() {
            stroke.set_dash(pattern);
        }
        self
    }

    /// Subtracts `hole` from this shape's fill.
    ///
    /// Added as a second contour rather than by any kind of boolean operation: a
    /// hole is a contour wound the other way, and that is all the fill rule needs.
    #[must_use]
    pub fn with_hole(mut self, hole: Path) -> Self {
        self.path.segments.extend(hole.segments);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flatten::flatten;
    use crate::mesh::MeshBuilder;

    fn flat_points(shape: &Shape, tolerance: f32) -> Vec<Vector2> {
        flatten(&shape.path, tolerance)
            .into_iter()
            .flat_map(|c| c.points)
            .collect()
    }

    #[test]
    fn a_rectangle_has_four_corners() {
        let points = flat_points(&rect(0.0, 0.0, 10.0, 20.0), 0.25);
        assert_eq!(points.len(), 4);
        assert_eq!(points[0], Vector2::new(0.0, 0.0));
        assert_eq!(points[2], Vector2::new(10.0, 20.0));
    }

    #[test]
    fn a_rectangles_fill_is_two_triangles() {
        let mut builder = MeshBuilder::new();
        builder.shape(&rect(0.0, 0.0, 10.0, 20.0).fill(Color::RED));
        assert_eq!(builder.build().fill_triangle_count(), 2);
    }

    /// The four corner arcs have to bow *inwards*. An arc drawn the other way
    /// leaves the rectangle's own bounds and the corner tips stick out, which is
    /// the classic sign of a wrong sweep flag.
    #[test]
    fn a_round_rect_stays_inside_its_bounds() {
        let shape = round_rect(0.0, 0.0, 40.0, 20.0, 5.0);
        for point in flat_points(&shape, 0.01) {
            assert!(
                point.x >= -1.0e-3
                    && point.x <= 40.0 + 1.0e-3
                    && point.y >= -1.0e-3
                    && point.y <= 20.0 + 1.0e-3,
                "{point:?} is outside the rectangle"
            );
        }
    }

    #[test]
    fn a_round_rects_corners_are_really_round() {
        // A corner arc must reach closer to the corner than the straight cut would.
        let shape = round_rect(0.0, 0.0, 40.0, 40.0, 10.0);
        let points = flat_points(&shape, 0.01);
        let corner_rounded = points.iter().any(|p| {
            (p.x - 5.0).abs() < 3.0 && (p.y - 5.0).abs() < 3.0 && *p != Vector2::new(0.0, 0.0)
        });
        assert!(corner_rounded, "no point inside the corner arc");
    }

    #[test]
    fn a_round_rect_radius_is_clamped_to_the_box() {
        let shape = round_rect(0.0, 0.0, 10.0, 4.0, 100.0);
        for point in flat_points(&shape, 0.01) {
            assert!(
                point.x >= -1.0e-3
                    && point.x <= 10.0 + 1.0e-3
                    && point.y >= -1.0e-3
                    && point.y <= 4.0 + 1.0e-3,
                "{point:?} escaped"
            );
        }
    }

    #[test]
    fn a_circle_is_on_its_radius() {
        for point in flat_points(&circle(50.0, 50.0, 30.0), 0.01) {
            assert!(
                ((point - Vector2::new(50.0, 50.0)).magnitude() - 30.0).abs() < 0.05,
                "{point:?}"
            );
        }
    }

    #[test]
    fn a_circle_starts_and_ends_on_its_centre_line() {
        let points = flat_points(&circle(50.0, 50.0, 30.0), 0.01);
        assert!(
            (points[0].x - 80.0).abs() < 0.1 && (points[0].y - 50.0).abs() < 1.0e-3,
            "started at {:?}",
            points[0]
        );
    }

    #[test]
    fn an_ellipse_is_on_its_radii() {
        for point in flat_points(&ellipse(0.0, 0.0, 40.0, 10.0), 0.01) {
            let normalized = Vector2::new(point.x / 40.0, point.y / 10.0);
            assert!((normalized.magnitude() - 1.0).abs() < 0.01, "{point:?}");
        }
    }

    /// A circle is the one shape where both arcs must sweep the same way, or the
    /// two halves do not meet.
    /// Both halves have to sweep the same way. Opposite flags trace the same half
    /// twice and leave the other half undrawn, and they do it quietly: every point
    /// of the result is still exactly on the circle, so a check on the radii passes
    /// and only the area is wrong - by half.
    #[test]
    fn a_circle_meets_itself() {
        let mut builder = MeshBuilder::new();
        builder.shape(&circle(50.0, 50.0, 30.0).fill(Color::RED));
        let geometry = builder.build();
        assert!(geometry.fill_triangle_count() >= 20);

        // A flattened circle is an inscribed polygon, so its area is necessarily a
        // little *under* the true circle and can never reach it.
        let circle = std::f32::consts::PI * 900.0;
        let area = triangulated_area(&geometry);
        assert!(area < circle, "an inscribed polygon cannot exceed it");
        assert!(
            (area - circle).abs() < circle * 0.01,
            "area {area} against circle {circle}"
        );
    }

    /// The tolerance has to actually converge on the shape. A flattened curve is an
    /// inscribed polygon, so its area approaches the true area from below as the
    /// tolerance falls - and that is a stronger statement than "close enough",
    /// because a shape traced twice would sit still instead of climbing.
    #[test]
    fn a_finer_tolerance_gets_the_fill_closer_to_the_circle() {
        let area_at = |tolerance: f32| {
            let mut builder = MeshBuilder::new().tolerance(tolerance);
            builder.shape(&circle(0.0, 0.0, 30.0).fill(Color::RED));
            triangulated_area(&builder.build())
        };
        let circle = std::f32::consts::PI * 900.0;
        let coarse = area_at(4.0);
        let fine = area_at(0.02);
        assert!(fine > coarse, "{fine} should exceed {coarse}");
        assert!(
            (circle - fine).abs() < circle * 0.001,
            "fine {fine} against circle {circle}"
        );
    }

    #[test]
    fn a_polygon_has_the_right_number_of_sides() {
        assert_eq!(
            flat_points(&polygon(6, 0.0, 0.0, 10.0), 0.25).len(),
            6,
            "a hexagon"
        );
        // Fewer than three is not a polygon, so it is clamped rather than producing
        // a shape that cannot be filled.
        assert_eq!(flat_points(&polygon(2, 0.0, 0.0, 10.0), 0.25).len(), 3);
    }

    #[test]
    fn a_polygon_is_on_its_circumference() {
        for point in flat_points(&polygon(5, 0.0, 0.0, 20.0), 0.25) {
            assert!((point.magnitude() - 20.0).abs() < 1.0e-3, "{point:?}");
        }
    }

    #[test]
    fn a_star_alternates_radii() {
        let points = flat_points(&star(5, 0.0, 0.0, 100.0, 40.0), 0.25);
        assert_eq!(points.len(), 10);
        for (index, point) in points.iter().enumerate() {
            let radius = point.magnitude();
            let expected = if index % 2 == 0 { 100.0 } else { 40.0 };
            assert!((radius - expected).abs() < 1.0e-3, "{point:?}");
        }
    }

    /// The whole point of the star: the alternating radii have to make it concave,
    /// which is the case an ear clipper is easiest to get wrong on.
    #[test]
    fn a_star_fills_less_than_its_circumscribed_circle() {
        let mut builder = MeshBuilder::new();
        builder.shape(&star(5, 0.0, 0.0, 100.0, 40.0).fill(Color::RED));
        let area = triangulated_area(&builder.build());
        let circle = std::f32::consts::PI * 100.0 * 100.0;
        assert!(area < circle, "area {area} vs circle {circle}");
        // Ten wedges of 1/2 * R * r over 36 degrees.
        let expected = 10.0 * 0.5 * 100.0 * 40.0 * (std::f32::consts::TAU / 10.0).sin();
        assert!(
            (area - expected).abs() < 1.0,
            "area {area} wanted {expected}"
        );
    }

    #[test]
    fn a_hole_is_a_second_contour() {
        let shape = circle(50.0, 50.0, 40.0)
            .fill(Color::RED)
            .with_hole(circle(50.0, 50.0, 20.0).path);
        assert_eq!(flatten(&shape.path, 0.25).len(), 2);
    }

    /// The area the fill triangles actually cover.
    fn triangulated_area(geometry: &xs_layout::VectorGeometry) -> f32 {
        let stride = xs_layout::FILL_VERTEX_FLOATS;
        geometry
            .fill_vertices
            .chunks_exact(3 * stride)
            .map(|t| {
                let (a, b, c) = (
                    Vector2::new(t[0], t[1]),
                    Vector2::new(t[stride], t[stride + 1]),
                    Vector2::new(t[2 * stride], t[2 * stride + 1]),
                );
                (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)
            })
            .sum::<f32>()
            .abs()
            / 2.0
    }

    #[test]
    fn a_shape_can_be_styled_without_being_rebuilt() {
        let shape = line(0.0, 0.0, 10.0, 0.0)
            .stroke(Color::WHITE, 4.0)
            .dash([2.0, 2.0]);
        assert_eq!(shape.stroke.as_ref().unwrap().dash, vec![2.0, 2.0]);
        assert_eq!(shape.stroke.as_ref().unwrap().width, 4.0);
    }
}
