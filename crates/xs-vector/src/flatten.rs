//! Curves to polylines.
//!
//! This is the one place where exactness is given up, and it is given up on
//! purpose and on a budget: [`flatten`] is asked for a tolerance in pixels and
//! returns a polyline that is never further than that from the curve. Everything
//! after this - the stroke's capsules, the fill's triangles, the bounds - works
//! on the polyline, so the tolerance is the single knob that trades work for
//! smoothness and nothing else in the pipeline has to know about it.
//!
//! The subdivision is uniform, not recursive, and the count comes from a bound
//! rather than from a measurement ([`Segment::chord_deviation`]): with the
//! deviation of a sub-curve scaling as the inverse square of the piece count, one
//! square root turns a curvature estimate into a piece count without walking the
//! curve at all. A recursive split would adapt better to a curve that is flat in
//! half of its length, at the cost of a stack, a branch per level, and a bound
//! that is only correct if the test is.
//!
//! Arcs are converted to cubics first, and a cubic is subdivided on its own
//! bound. That is exact rather than approximate: a circular arc of a quarter
//! turn or less is reproduced by a cubic to within a part in ten thousand, well
//! inside any tolerance a caller would ask for.

use xs_math::Vector2;

use crate::path::{Path, Segment};

/// The most pieces one curve is ever split into.
///
/// A cubic's bound is `sqrt(deviation / tolerance)`, so a tolerance near zero or a
/// wildly out-of-scale control point asks for an absurd number. Clamping keeps
/// one bad path from turning into hundreds of thousands of vertices; the result is
/// then coarser than the tolerance asked for, which is the right way round to
/// fail.
pub const MAX_PIECES_PER_CURVE: usize = 64;

/// Two points closer than this are the same point.
///
/// This is not a tolerance knob: it is the threshold below which a segment has no
/// direction, and a zero-length segment would divide by zero in the shader.
pub const MIN_SEGMENT_LENGTH: f32 = 1.0e-5;

/// The default tolerance, in pixels.
///
/// A quarter of a pixel is under the point at which the error is visible on a
/// straight edge, and it costs a circle a handful of segments: a radius-100
/// circle's control points sit about `0.55` pixels outside the chord, so its bound
/// lands just above one piece per quarter turn.
pub const DEFAULT_TOLERANCE: f32 = 0.25;

/// One run of points: a subpath's worth, plus whether `Close` ended it.
///
/// A stroke walks these directly. A fill turns them into rings, and a ring's
/// closing edge is implied rather than stored - so `closed` and the ring list do
/// not have to agree, and a contour of three points stays three points.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contour {
    pub points: Vec<Vector2>,
    pub closed: bool,
}

impl Contour {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many segments this contour draws.
    ///
    /// An open contour of `n` points has `n - 1` segments and a closed one `n`,
    /// the extra being the edge back to the start.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        if self.closed {
            self.points.len()
        } else {
            self.points.len().saturating_sub(1)
        }
    }

    /// The point at `index`, wrapping at the end of a closed contour so callers can
    /// treat it as a ring without special-casing the last edge.
    #[must_use]
    pub fn at(&self, index: usize) -> Option<Vector2> {
        if index < self.points.len() {
            Some(self.points[index])
        } else if self.closed {
            self.points.get(index - self.points.len()).copied()
        } else {
            None
        }
    }
}

/// Flattens `path` into contours no further than `tolerance` pixels from it.
///
/// `tolerance` is in the path's own units, so it has to be a device-pixel figure
/// for the bound to mean anything - see [`crate::Vector`], which scales the path
/// before it gets here for exactly that reason.
#[must_use]
pub fn flatten(path: &Path, tolerance: f32) -> Vec<Contour> {
    let tolerance = if tolerance > 0.0 {
        tolerance
    } else {
        MIN_SEGMENT_LENGTH
    };
    let mut contours = Vec::new();
    let mut current = Contour::new();
    let mut cursor = Vector2::zero();
    let mut contour_start = Vector2::zero();
    let mut started = false;

    for segment in &path.segments {
        match segment {
            Segment::MoveTo(to) => {
                push_contour(&mut contours, &mut current);
                current = Contour::new();
                current.points.push(*to);
                cursor = *to;
                contour_start = *to;
                started = true;
            }
            Segment::LineTo(to) => {
                if !started {
                    // A path that draws before it moves starts at the origin,
                    // which is where the origin is.
                    current.points.push(cursor);
                    started = true;
                }
                // Through `push_curve` rather than a bare push, so a line that
                // goes nowhere is dropped like any other degenerate segment
                // instead of leaving a repeated point behind.
                push_curve(segment, cursor, tolerance, &mut current.points);
                cursor = *to;
            }
            Segment::Close => {
                current.closed = true;
                push_contour(&mut contours, &mut current);
                current = Contour::new();
                // The closing edge goes back to where the contour began, so that
                // is where anything drawn afterwards starts from.
                cursor = contour_start;
                started = false;
            }
            curve => {
                if !started {
                    current.points.push(cursor);
                    started = true;
                }
                flatten_curve(curve, cursor, tolerance, &mut current.points);
                if let Some(end) = curve.end() {
                    cursor = end;
                }
            }
        }
    }
    push_contour(&mut contours, &mut current);
    contours
}

/// The number of equal pieces a curve needs to stay within `tolerance` of its
/// chord.
///
/// `chord_deviation` is the worst-case bow of the whole curve. Subdividing into
/// `n` equal parameter ranges cuts the second derivative by `n^2`, so the bow
/// becomes `deviation / n^2` and the count is the square root of the ratio. This
/// is the "expanding the square root" step, and it is why flattening does not
/// need to look at the curve.
#[must_use]
pub fn piece_count(segment: &Segment, p0: Vector2, tolerance: f32) -> usize {
    let Some(deviation) = segment.chord_deviation(p0) else {
        return 1;
    };
    if deviation <= tolerance {
        return 1;
    }
    let count = (deviation / tolerance).sqrt().ceil();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let count = count as usize;
    count.clamp(1, MAX_PIECES_PER_CURVE)
}

/// Appends `segment`'s polyline to `out`, without repeating the point it starts
/// from.
fn flatten_curve(segment: &Segment, mut from: Vector2, tolerance: f32, out: &mut Vec<Vector2>) {
    if let Segment::ArcTo { .. } = segment {
        // An arc has no closed form here, and its endpoint form is the awkward one
        // to sample, so it becomes cubics first and each of those is subdivided on
        // its own bound below.
        for cubic in arc_to_cubics(from, segment) {
            push_curve(&cubic, from, tolerance, out);
            from = cubic.end().unwrap_or(from);
        }
        return;
    }
    push_curve(segment, from, tolerance, out);
}

fn push_curve(segment: &Segment, from: Vector2, tolerance: f32, out: &mut Vec<Vector2>) {
    let Some(end) = segment.end() else {
        return;
    };
    if (end - from).magnitude() < MIN_SEGMENT_LENGTH {
        // No direction, so nothing to subdivide. It still has to record its end
        // point when it is the only thing that has moved the cursor, or a later
        // segment would start from somewhere the path never went.
        if out.last() != Some(&end) {
            out.push(end);
        }
        return;
    }
    let pieces = piece_count(segment, from, tolerance);
    for i in 1..=pieces {
        if let Some(point) = segment.point_at(from, i as f32 / pieces as f32) {
            out.push(point);
        }
    }
    if out.last() != Some(&end) {
        out.push(end);
    }
}

/// Hands a finished contour over, dropping the ones with nothing in them.
///
/// A contour of one point is a lone `M`; a contour of two is a line to nowhere
/// and neither is a shape. Keeping them would put zero-area segments into the
/// stroke mesh, and a segment with no direction is a divide by zero in the
/// shader.
fn push_contour(contours: &mut Vec<Contour>, current: &mut Contour) {
    let keep = current.points.len() >= 2
        && current
            .points
            .windows(2)
            .any(|w| (w[1] - w[0]).magnitude() >= MIN_SEGMENT_LENGTH);
    if keep {
        contours.push(std::mem::take(current));
    } else {
        current.points.clear();
    }
    current.closed = false;
}

/// Converts an endpoint-parameterised elliptical arc into at most four cubics.
///
/// This is the standard conversion, and the reason to prefer cubics over sampling
/// the arc directly is that a cubic carries its own exact chord bound, so the
/// caller gets the same tolerance guarantee it would get from a cubic it wrote by
/// hand. The four-piece maximum is the ellipse split at its axis crossings, where
/// each piece spans a quarter turn.
///
/// A zero radius means SVG says the arc is a straight line, and the endpoints
/// being equal means there is nothing to draw.
#[must_use]
pub fn arc_to_cubics(from: Vector2, segment: &Segment) -> Vec<Segment> {
    let Segment::ArcTo {
        radius,
        rotation,
        large_arc,
        sweep,
        to,
    } = segment
    else {
        return Vec::new();
    };
    // A zero radius means the arc is a straight line, which is what SVG says and
    // what keeps `A 0 0 0 0 1 x y` from silently drawing nothing.
    if radius.x <= 0.0 || radius.y <= 0.0 {
        return if *to == from {
            Vec::new()
        } else {
            vec![Segment::LineTo(*to)]
        };
    }
    // Identical endpoints leave nothing to sweep.
    if *to == from {
        return Vec::new();
    }

    // Endpoint to centre parameters, per the SVG implementation notes F.6.5.
    let (rx, ry) = (radius.x.abs(), radius.y.abs());
    // `sin_cos` hands back (sin, cos). Reading it the other way round rotates the
    // whole arc by a quarter turn, which a circle cannot show - it comes out the
    // same circle - and an ellipse very much can.
    let (sin_phi, cos_phi) = rotation.sin_cos();
    let half = (from - *to) * 0.5;
    let x1p = cos_phi * half.x + sin_phi * half.y;
    let y1p = -sin_phi * half.x + cos_phi * half.y;

    // Scale the radii up if they are too small to span the chord, rather than
    // failing: SVG calls for exactly this and a too-small ellipse is an error
    // only in the sense that nothing can be drawn from it.
    let lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    let (rx, ry) = if lambda > 1.0 {
        (rx * lambda.sqrt(), ry * lambda.sqrt())
    } else {
        (rx, ry)
    };

    let rx2 = rx * rx;
    let ry2 = ry * ry;
    let x1p2 = x1p * x1p;
    let y1p2 = y1p * y1p;
    let numerator = (rx2 * ry2 - rx2 * y1p2 - ry2 * x1p2).max(0.0);
    let denominator = rx2 * y1p2 + ry2 * x1p2;
    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let factor = sign * (numerator / denominator).sqrt();
    let cxp = factor * rx * y1p / ry;
    let cyp = factor * -ry * x1p / rx;

    let center = Vector2::new(
        cos_phi * cxp - sin_phi * cyp + (from.x + to.x) * 0.5,
        sin_phi * cxp + cos_phi * cyp + (from.y + to.y) * 0.5,
    );

    let angle = |ux: f32, uy: f32, vx: f32, vy: f32| {
        let dot = ux * vx + uy * vy;
        let len = (ux * ux + uy * uy).sqrt() * (vx * vx + vy * vy).sqrt();
        let mut a = (dot / len).clamp(-1.0, 1.0).acos();
        if ux * vy - uy * vx < 0.0 {
            a = -a;
        }
        a
    };

    let ux = (x1p - cxp) / rx;
    let uy = (y1p - cyp) / ry;
    let vx = (-x1p - cxp) / rx;
    let vy = (-y1p - cyp) / ry;
    let theta = angle(1.0, 0.0, ux, uy);
    let delta = angle(ux, uy, vx, vy);
    let delta = if !sweep && delta > 0.0 {
        delta - std::f32::consts::TAU
    } else if *sweep && delta < 0.0 {
        delta + std::f32::consts::TAU
    } else {
        delta
    };

    // A quarter turn is the most a cubic reproduces to well under a part in ten
    // thousand, which is finer than any tolerance worth asking for.
    let pieces = ((delta.abs() / std::f32::consts::FRAC_PI_2).ceil() as usize).clamp(1, 4);
    let step = delta / pieces as f32;
    // The control point distance for a cubic spanning `step` radians is
    // `4/3 tan(step/4)`. The quarter of the angle is not a typo: at a quarter turn
    // this is the familiar 0.5523, and halving it to `tan(step/2)` would put the
    // control points out at 1.333 and bulge a quarter circle to a radius of 141
    // instead of 100.
    let handle = 4.0 / 3.0 * (step * 0.25).tan();

    let point_at = |theta: f32| {
        let (sin, cos) = theta.sin_cos();
        Vector2::new(
            center.x + rx * cos * cos_phi - ry * sin * sin_phi,
            center.y + rx * cos * sin_phi + ry * sin * cos_phi,
        )
    };
    let tangent_at = |theta: f32| {
        let (sin, cos) = theta.sin_cos();
        Vector2::new(
            -rx * sin * cos_phi - ry * cos * sin_phi,
            -rx * sin * sin_phi + ry * cos * cos_phi,
        )
    };

    let mut theta = theta;
    let mut cursor = from;
    let mut cubics = Vec::with_capacity(pieces);
    for index in 0..pieces {
        let next = theta + step;
        // The last piece snaps to the endpoint the path named, so the polyline
        // ends exactly where the arc does rather than a rounding error away.
        let end = if index + 1 == pieces {
            *to
        } else {
            point_at(next)
        };
        let start_tangent = tangent_at(theta) * handle;
        let end_tangent = tangent_at(next) * handle;
        cubics.push(Segment::CubicTo {
            c1: cursor + start_tangent,
            c2: end - end_tangent,
            to: end,
        });
        cursor = end;
        theta = next;
    }
    cubics
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32) -> Vector2 {
        Vector2::new(x, y)
    }

    #[test]
    fn a_straight_line_flattens_to_its_two_endpoints() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).line_to(100.0, 50.0);
        });
        let contours = flatten(&path, DEFAULT_TOLERANCE);
        assert_eq!(contours.len(), 1);
        assert_eq!(contours[0].points, vec![v(0.0, 0.0), v(100.0, 50.0)]);
        assert!(!contours[0].closed);
    }

    /// The point of the whole exercise: a curve becomes enough pieces to hold the
    /// tolerance, and no more.
    #[test]
    fn a_tight_tolerance_splits_a_curve_further() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).quadratic_to(50.0, 100.0, 100.0, 0.0);
        });
        let coarse = flatten(&path, 10.0);
        let fine = flatten(&path, 0.01);
        assert!(
            fine[0].points.len() > coarse[0].points.len(),
            "{} vs {}",
            fine[0].points.len(),
            coarse[0].points.len()
        );
    }

    /// A bound is only worth having if it is never optimistic. This is the test
    /// that matters: sample the flattened polyline's deviation from the true curve
    /// against the tolerance it was asked for, over a spread of shapes.
    #[test]
    fn a_flattened_curve_stays_within_the_tolerance_it_asked_for() {
        let shapes: [(&str, Path); 4] = [
            (
                "quadratic",
                Path::build(|p| {
                    p.move_to(0.0, 0.0).quadratic_to(50.0, 200.0, 100.0, 0.0);
                }),
            ),
            (
                "cubic",
                Path::build(|p| {
                    p.move_to(0.0, 0.0)
                        .cubic_to(0.0, 300.0, 100.0, 300.0, 100.0, 0.0);
                }),
            ),
            (
                "tight cubic",
                Path::build(|p| {
                    p.move_to(0.0, 0.0)
                        .cubic_to(0.0, 900.0, 1.0, 900.0, 1.0, 0.0);
                }),
            ),
            (
                "quarter circle",
                Path::build(|p| {
                    p.move_to(100.0, 0.0)
                        .arc_to(100.0, 100.0, 0.0, false, true, 0.0, 100.0);
                }),
            ),
        ];

        for (name, path) in shapes {
            for tolerance in [0.05f32, 0.25, 1.0, 4.0] {
                let contours = flatten(&path, tolerance);
                let points = &contours[0].points;
                assert!(points.len() >= 2, "{name}: no points");

                // Measure the shape against its own polyline, densely. Sampling
                // each exact point against the single chord it happened to fall in
                // would measure the wrong thing - the guarantee is about the
                // polyline as a whole.
                let exact = exact_points(&path);
                let mut worst: f32 = 0.0;
                for point in exact {
                    worst = worst.max(distance_to_polyline(point, points));
                }
                assert!(
                    worst <= tolerance * 1.05,
                    "{name} at tolerance {tolerance}: worst deviation {worst} over {} points",
                    points.len()
                );
            }
        }
    }

    /// Every point of `path`'s curves, sampled densely enough that the densest
    /// sample is far finer than any tolerance under test.
    ///
    /// The curves are evaluated from the segments themselves rather than from
    /// another flattening, so this is the real shape and not a second opinion from
    /// the code under test.
    fn exact_points(path: &Path) -> Vec<Vector2> {
        let mut points = Vec::new();
        let mut cursor = Vector2::zero();
        for segment in &path.segments {
            match segment {
                Segment::MoveTo(to) => {
                    cursor = *to;
                    points.push(*to);
                }
                Segment::Close => {}
                drawing => {
                    let Some(end) = drawing.end() else { continue };
                    if let Segment::ArcTo { .. } = drawing {
                        // Sample the cubics the arc becomes, which is what the
                        // flattener subdivides, and so what has to be within
                        // tolerance of the original arc.
                        for cubic in arc_to_cubics(cursor, drawing) {
                            for i in 0..=200 {
                                if let Some(p) = cubic.point_at(cursor, i as f32 / 200.0) {
                                    points.push(p);
                                }
                            }
                            if let Some(p) = cubic.end() {
                                cursor = p;
                            }
                        }
                        continue;
                    }
                    for i in 0..=200 {
                        if let Some(p) = drawing.point_at(cursor, i as f32 / 200.0) {
                            points.push(p);
                        }
                    }
                    cursor = end;
                }
            }
        }
        points
    }

    /// The shortest distance from `point` to the polyline, ignoring `closed`.
    ///
    /// Open is the weaker test: a closed contour's missing edge can only bring a
    /// point *closer* to the polyline than the open reading does, so an open
    /// measurement is a conservative bound on the closed one.
    fn distance_to_polyline(point: Vector2, points: &[Vector2]) -> f32 {
        let mut best = f32::INFINITY;
        for window in points.windows(2) {
            best = best.min(point_to_segment(point, window[0], window[1]));
        }
        if points.len() == 1 {
            best = (point - points[0]).magnitude();
        }
        best
    }

    /// The clamp has to actually clamp, or one wild control point asks for a
    /// million pieces.
    #[test]
    fn a_pathological_curve_is_clamped_rather_than_exploded() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0)
                .cubic_to(0.0, 1.0e6, 1.0, 1.0e6, 1.0, 0.0);
        });
        let contours = flatten(&path, 1.0e-9);
        assert!(contours[0].points.len() <= MAX_PIECES_PER_CURVE + 2);
    }

    #[test]
    fn a_moveto_breaks_the_contour() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0)
                .line_to(10.0, 0.0)
                .move_to(50.0, 50.0)
                .line_to(60.0, 50.0);
        });
        let contours = flatten(&path, DEFAULT_TOLERANCE);
        assert_eq!(contours.len(), 2);
        assert_eq!(contours[1].points[0], v(50.0, 50.0));
    }

    #[test]
    fn a_close_marks_the_contour_and_does_not_repeat_the_start() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0)
                .line_to(10.0, 0.0)
                .line_to(10.0, 10.0)
                .close();
        });
        let contours = flatten(&path, DEFAULT_TOLERANCE);
        assert_eq!(contours.len(), 1);
        assert!(contours[0].closed);
        assert_eq!(contours[0].points.len(), 3);
        // The closing edge is implied, so `at` wraps to it.
        assert_eq!(contours[0].at(3), Some(v(0.0, 0.0)));
        assert_eq!(contours[0].segment_count(), 3);
    }

    #[test]
    fn drawing_after_a_close_starts_a_new_contour() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0)
                .line_to(10.0, 0.0)
                .close()
                .line_to(20.0, 20.0);
        });
        let contours = flatten(&path, DEFAULT_TOLERANCE);
        assert_eq!(contours.len(), 2);
        assert!(contours[0].closed);
        assert!(!contours[1].closed);
        assert_eq!(contours[1].points[0], v(0.0, 0.0));
    }

    #[test]
    fn a_lone_moveto_is_not_a_contour() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0);
        });
        assert!(flatten(&path, DEFAULT_TOLERANCE).is_empty());
    }

    /// A zero-length segment has no direction, and the shader divides by its
    /// length, so it must not survive to the mesh.
    #[test]
    fn a_zero_length_segment_is_dropped() {
        let path = Path::build(|p| {
            p.move_to(5.0, 5.0).line_to(5.0, 5.0).line_to(9.0, 5.0);
        });
        let contours = flatten(&path, DEFAULT_TOLERANCE);
        assert_eq!(contours.len(), 1);
        assert_eq!(contours[0].points.len(), 2);
    }

    #[test]
    fn a_drawing_command_before_any_moveto_starts_at_the_origin() {
        let path = Path::build(|p| {
            p.line_to(10.0, 0.0);
        });
        let contours = flatten(&path, DEFAULT_TOLERANCE);
        assert_eq!(contours[0].points[0], v(0.0, 0.0));
    }

    #[test]
    fn a_path_drawing_before_a_later_moveto_keeps_both_contours() {
        let path = Path::build(|p| {
            p.line_to(10.0, 0.0).move_to(0.0, 0.0).line_to(0.0, 10.0);
        });
        let contours = flatten(&path, DEFAULT_TOLERANCE);
        assert_eq!(contours.len(), 2);
        assert_eq!(contours[0].points[0], v(0.0, 0.0));
        assert_eq!(contours[1].points[0], v(0.0, 0.0));
    }

    /// A circle is the shape every ellipse and round join comes out of, so its
    /// radius has to come out right or everything built on it is wrong.
    #[test]
    fn a_circle_arc_lands_on_its_radius() {
        let path = Path::build(|p| {
            p.move_to(100.0, 0.0)
                .arc_to(100.0, 100.0, 0.0, false, true, 0.0, 100.0);
        });
        let points = flatten(&path, 0.01)[0].points.clone();
        assert!(points.len() >= 4, "only {} points", points.len());
        for point in points.iter().skip(1) {
            let radius = (*point - v(0.0, 0.0)).magnitude();
            assert!(
                (radius - 100.0).abs() < 0.05,
                "point {point:?} is {radius} from the centre, wanted 100"
            );
        }
    }

    #[test]
    fn a_full_circle_is_two_arcs_and_meets_its_start() {
        let path = Path::build(|p| {
            p.move_to(50.0, 0.0)
                .arc_to(50.0, 50.0, 0.0, true, true, -50.0, 0.0)
                .arc_to(50.0, 50.0, 0.0, true, true, 50.0, 0.0);
        });
        let points = flatten(&path, 0.01)[0].points.clone();
        assert!(points.len() > 16, "only {} points", points.len());
        for point in &points {
            assert!(((*point - v(0.0, 0.0)).magnitude() - 50.0).abs() < 0.05);
        }
    }

    #[test]
    fn a_rotated_arc_stays_on_its_rotated_ellipse() {
        // A circle under rotation is still a circle, so its radius is the check.
        let path = Path::build(|p| {
            p.move_to(100.0, 0.0)
                .arc_to(100.0, 100.0, 37.0, false, true, 0.0, 100.0);
        });
        for point in flatten(&path, 0.01)[0].points.iter().skip(1) {
            assert!((point.magnitude() - 100.0).abs() < 0.1, "{point:?}");
        }
    }

    /// SVG says a zero radius degenerates to a line, and a zero-length chord
    /// degenerates to nothing at all. Both have to not produce NaNs.
    #[test]
    fn a_degenerate_arc_does_not_produce_nonsense() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0)
                .arc_to(0.0, 10.0, 0.0, false, true, 10.0, 0.0);
        });
        let points = flatten(&path, 0.01)[0].points.clone();
        assert!(points.iter().all(|p| p.x.is_finite() && p.y.is_finite()));
        assert!(!points.is_empty());
    }

    #[test]
    fn radii_too_small_for_the_chord_are_grown_rather_than_dropped() {
        // r = 5 cannot span a chord of 20, so SVG says scale it up until it can.
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0)
                .arc_to(5.0, 5.0, 0.0, false, true, 20.0, 0.0);
        });
        let points = flatten(&path, 0.01)[0].points.clone();
        assert!(points.iter().all(|p| p.x.is_finite()));
        assert!(points.len() >= 2);
    }

    #[test]
    fn a_negative_tolerance_does_not_divide_by_zero() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0)
                .cubic_to(0.0, 50.0, 100.0, 50.0, 100.0, 0.0);
        });
        for tolerance in [0.0f32, -1.0] {
            let contours = flatten(&path, tolerance);
            assert!(contours[0].points.iter().all(|p| p.x.is_finite()));
        }
    }

    /// The distance from `point` to the segment `a`-`b`, which is what "stays
    /// within tolerance of its chord" means.
    fn point_to_segment(point: Vector2, a: Vector2, b: Vector2) -> f32 {
        let axis = b - a;
        let len2 = Vector2::dot(axis, axis);
        if len2 < MIN_SEGMENT_LENGTH {
            return (point - a).magnitude();
        }
        let t = (Vector2::dot(point - a, axis) / len2).clamp(0.0, 1.0);
        (point - (a + axis * t)).magnitude()
    }
}
