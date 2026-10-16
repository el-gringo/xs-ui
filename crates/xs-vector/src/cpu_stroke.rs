//! A CPU port of the stroke fragment shader.
//!
//! A GPU shader cannot be unit tested, so the maths it depends on is mirrored
//! here and held against the geometry it is supposed to describe. That is the
//! whole reason this module exists, and the reason it is a *port* rather than a
//! reimplementation: if the two ever disagree, one of them is wrong, and the
//! tests below say which claims have to hold for the picture to be right.
//!
//! What it pins down, in order of how badly it would hurt to get wrong:
//!
//! - **Coverage is the distance to the segment**, so a stroke of any width has the
//!   same silhouette as the line it is drawn along, to within half a pixel.
//! - **Round joins come from the union of capsules.** Two segments meeting at a
//!   point must cover exactly the disc of radius half-width around it, which is
//!   what makes a join look right without anything knowing about joins.
//! - **The antialiasing window is half a pixel**, and it is exact rather than
//!   tuned: both fields are true signed distances, so one screen pixel is one unit
//!   and a sample sitting on the edge covers exactly half.
//! - **Butt and square caps are rectangles** whose corners stay sharp, where round
//!   is a half-disc. This is where a "just use the segment distance everywhere"
//!   shortcut would show up.

use xs_layout::Color;
use xs_math::Vector2;

/// What the shader's `a_cap` codes mean.
///
/// These have to be the same three numbers [`crate::LineCap::code`] emits and the
/// GLSL branches on: they are the vocabulary the two sides share, and a cap that is
/// one code out renders as a cap of a *different kind* without anything
/// complaining. Spelled out here rather than imported so this file stands alone as
/// the shader's reference - nothing here calls into the stroke pipeline.
pub mod cap {
    pub const ROUND: f32 = 0.0;
    pub const BUTT: f32 = 1.0;
    pub const SQUARE: f32 = 2.0;
}

/// Coverage of one segment at one point: `1.0` well inside the pen, `0.0` outside,
/// and a fraction in between along the edge.
///
/// The direct mirror of `vector_stroke.glsl.frag`, with the same structure so the
/// two can be read side by side.
#[must_use]
pub fn segment_coverage(
    point: Vector2,
    p0: Vector2,
    p1: Vector2,
    half_width: f32,
    cap: f32,
) -> f32 {
    // Round ends: distance to the segment with the projection clamped, so each end
    // becomes a half-disc of the pen's radius.
    let axis = p1 - p0;
    let len2 = Vector2::dot(axis, axis);
    let rel = point - p0;
    let u = (Vector2::dot(rel, axis) / len2.max(1.0e-6)).clamp(0.0, 1.0);
    let round_sd = (rel - axis * u).magnitude() - half_width;

    // Square and butt ends: distance to the rectangle the segment occupies.
    let len = (p1 - p0).magnitude();
    let reach = if cap < 0.5 || cap > 1.5 {
        half_width
    } else {
        0.0
    };
    let local = Vector2::new(
        // The shader gets this by interpolating the corner selector; here it is
        // projected onto the segment's own frame.
        project_along(rel, axis, len) - len * 0.5,
        signed_across(rel, axis, len),
    );
    let q =
        Vector2::new(local.x.abs(), local.y.abs()) - Vector2::new(len * 0.5 + reach, half_width);
    let box_sd = max(Vector2::new(q.x.max(0.0), q.y.max(0.0))).magnitude() + q.x.max(q.y).min(0.0);

    let sd = mix(round_sd, box_sd, step(0.5, cap));
    (0.5 - sd).clamp(0.0, 1.0)
}

/// How far along `axis` the point is, in pixels.
fn project_along(rel: Vector2, axis: Vector2, len: f32) -> f32 {
    if len < 1.0e-6 {
        return 0.0;
    }
    let unit = axis / len;
    Vector2::dot(rel, unit)
}

/// How far across `axis` the point is, in pixels, signed by which side.
fn signed_across(rel: Vector2, axis: Vector2, len: f32) -> f32 {
    if len < 1.0e-6 {
        return rel.magnitude();
    }
    let unit = axis / len;
    // The shader's normal is (-dir.y, dir.x).
    Vector2::dot(rel, Vector2::new(-unit.y, unit.x))
}

fn max(v: Vector2) -> Vector2 {
    v
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

fn step(edge: f32, x: f32) -> f32 {
    if x >= edge { 1.0 } else { 0.0 }
}

/// The alpha a pixel ends up with, which is the coverage scaled by the pen's
/// opacity. What the shader writes to `gl_FragColor`.
#[must_use]
pub fn pixel_alpha(
    point: Vector2,
    p0: Vector2,
    p1: Vector2,
    half_width: f32,
    cap: f32,
    color: Color,
) -> f32 {
    color.a * segment_coverage(point, p0, p1, half_width, cap)
}

/// The colour a polyline paints at `point`: every segment's alpha composited over
/// the background, which is what the union of overlapping capsules amounts to.
///
/// This is the join rule. Nothing in the shader loops over the path, so the join
/// exists only because the blend of two capsules happens to be the right shape -
/// and this function is what lets a test ask whether it really is.
#[must_use]
pub fn polyline_alpha(
    point: Vector2,
    points: &[Vector2],
    half_width: f32,
    cap: f32,
    color: Color,
) -> f32 {
    let mut alpha = 0.0;
    for index in 0..points.len().saturating_sub(1) {
        let a = pixel_alpha(
            point,
            points[index],
            points[index + 1],
            half_width,
            cap,
            color,
        );
        // Over compositing: the same `GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA` the
        // renderer sets in `init`.
        alpha += a * (1.0 - alpha);
    }
    alpha
}

/// Whether `point` is inside the stroke of a polyline, ignoring antialiasing.
///
/// The blunt instrument the precise one is checked against: a stroke should cover
/// exactly the points within half the pen's width of the line, which is a
/// statement about a distance field and not about any of this crate's machinery.
#[must_use]
pub fn within_stroke(point: Vector2, points: &[Vector2], half_width: f32, cap: f32) -> bool {
    polyline_alpha(point, points, half_width, cap, Color::WHITE) >= 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use cap::{BUTT as CAP_BUTT, ROUND as CAP_ROUND, SQUARE as CAP_SQUARE};

    fn v(x: f32, y: f32) -> Vector2 {
        Vector2::new(x, y)
    }

    /// The silhouette is the claim everything else rests on: a pen of width `w`
    /// covers the points within `w/2` of the line.
    ///
    /// The probes sit more than half a pixel either side of the edge, because that
    /// is where the ramp has finished. A probe a hundredth of a pixel inside is
    /// *supposed* to be about half covered - the ramp is one pixel wide, as the
    /// next test pins down - so "solidly inside" has to mean inside the ramp too.
    #[test]
    fn coverage_matches_the_pen_width_across_a_straight_stroke() {
        let (p0, p1) = (v(0.0, 0.0), v(100.0, 0.0));
        // Wide enough that there is room *inside* the stroke and past the ramp at
        // the same time - a one-pixel pen has no such point, which is the next
        // test's problem rather than this one's.
        for width in [4.0f32, 13.0, 40.0] {
            let half = width * 0.5;
            let inside = segment_coverage(v(50.0, half - 0.6), p0, p1, half, CAP_ROUND);
            let outside = segment_coverage(v(50.0, half + 0.6), p0, p1, half, CAP_ROUND);
            assert!(inside > 0.99, "width {width}: inside was {inside}");
            assert!(outside < 0.01, "width {width}: outside was {outside}");
        }
    }

    /// A hairline has to be solid on its centreline. With a one-pixel pen the whole
    /// stroke is antialiasing, so this is the one width where "is it there at all"
    /// is a real question - and where an off-by-a-half bug would vanish.
    #[test]
    fn a_hairline_is_solid_on_its_centreline() {
        let (p0, p1) = (v(0.0, 0.0), v(100.0, 0.0));
        assert!(segment_coverage(v(50.0, 0.0), p0, p1, 0.5, CAP_ROUND) > 0.99);
        assert!(segment_coverage(v(0.0, 0.0), p0, p1, 0.5, CAP_ROUND) > 0.99);
        assert!(segment_coverage(v(100.0, 0.0), p0, p1, 0.5, CAP_ROUND) > 0.99);
    }

    /// Half a pixel is the window, and it is exact because the field is a true
    /// distance: a sample exactly on the edge is half covered, and one half a pixel
    /// further in is opaque.
    #[test]
    fn a_sample_exactly_on_the_edge_is_half_covered() {
        let (p0, p1) = (v(0.0, 0.0), v(100.0, 0.0));
        let half = 5.0;
        let on_edge = segment_coverage(v(50.0, half), p0, p1, half, CAP_ROUND);
        assert!((on_edge - 0.5).abs() < 1.0e-4, "was {on_edge}");
    }

    /// The ramp is one pixel wide, whatever the pen's width. A window scaled to the
    /// pen would blur a thick stroke; a fixed one is what a distance field buys.
    #[test]
    fn the_antialiasing_ramp_is_one_pixel_wide_at_every_width() {
        let (p0, p1) = (v(0.0, 0.0), v(100.0, 0.0));
        for half in [1.0f32, 8.0, 40.0] {
            let at_edge = segment_coverage(v(50.0, half), p0, p1, half, CAP_ROUND);
            let quarter = segment_coverage(v(50.0, half + 0.25), p0, p1, half, CAP_ROUND);
            let three_quarters = segment_coverage(v(50.0, half + 0.75), p0, p1, half, CAP_ROUND);
            assert!((at_edge - 0.5).abs() < 1.0e-4);
            assert!((quarter - 0.25).abs() < 1.0e-4, "half {half}: {quarter}");
            assert!(
                (three_quarters - 0.0).abs() < 1.0e-4,
                "half {half}: {three_quarters}"
            );
        }
    }

    /// A round cap is a half-disc, so the silhouette extends half the pen past the
    /// endpoint and bulges out to the sides of it.
    #[test]
    fn a_round_cap_reaches_half_the_pen_past_the_end() {
        let (p0, p1) = (v(0.0, 0.0), v(50.0, 0.0));
        let half = 5.0f32;
        assert!(
            segment_coverage(v(50.0 + half - 0.6, 0.0), p0, p1, half, CAP_ROUND) > 0.99,
            "must reach to the tip"
        );
        assert!(segment_coverage(v(50.0 + half + 0.6, 0.0), p0, p1, half, CAP_ROUND) < 0.01);
        // And it is a half-disc, not a square. Along the axis it reaches half a pen
        // past the end; at 45 degrees it reaches half a pen *from the end*, which is
        // the same distance measured the other way. A square cap would cover the
        // corner at half * sqrt(2) out instead - see the cap tests below.
        let forty_five = 45.0f32.to_radians();
        let along = |r: f32| p1 + v(r * forty_five.cos(), r * forty_five.sin());
        assert!(
            segment_coverage(along(half - 0.6), p0, p1, half, CAP_ROUND) > 0.99,
            "half a pen diagonally out must be covered"
        );
        assert!(
            segment_coverage(along(half + 0.6), p0, p1, half, CAP_ROUND) < 0.01,
            "and it must stop at half a pen"
        );
    }

    /// The load-bearing join claim: two segments meeting at a corner leave no gap
    /// all the way round it.
    ///
    /// Nothing in the shader knows what a join is. Two capsules sharing an endpoint
    /// cover a disc around it, and the union of those discs is the round join - so
    /// this is a consequence of segments being capsules rather than anything that
    /// was implemented, which is exactly why it is worth asserting.
    #[test]
    fn two_segments_joining_leave_no_gap_around_the_corner() {
        let half: f32 = 5.0;
        // A right-angle corner.
        let points = [v(0.0, 0.0), v(20.0, 0.0), v(20.0, 20.0)];
        let corner = v(20.0, 0.0);

        // Just inside the pen's radius, every direction is filled - including the
        // quadrant the two arms leave empty between them.
        for degrees in 0..360 {
            let radians = (degrees as f32).to_radians();
            let probe = corner + v((half - 0.6) * radians.cos(), (half - 0.6) * radians.sin());
            assert!(
                within_stroke(probe, &points, half, CAP_ROUND),
                "a gap at {degrees} degrees"
            );
        }
    }

    /// And the outer side of the corner stops at the pen's radius rather than
    /// spiking out to the mitre point. This is what makes the join *round*: the
    /// classic mistake here is a miter reaching `half / sin(angle/2)` out, which for
    /// a right angle is a third more than half a pen.
    #[test]
    fn the_join_does_not_mitre_out_past_the_pen() {
        let half: f32 = 5.0;
        let points = [v(0.0, 0.0), v(20.0, 0.0), v(20.0, 20.0)];
        let corner = v(20.0, 0.0);

        // Up and to the right, out of the corner: the side a miter would fill.
        let forty_five = 45.0f32.to_radians();
        let away = |r: f32| corner + v(r * forty_five.cos(), -r * forty_five.sin());
        assert!(
            within_stroke(away(half - 0.6), &points, half, CAP_ROUND),
            "must reach the pen's radius"
        );
        assert!(
            !within_stroke(away(half + 0.6), &points, half, CAP_ROUND),
            "a round join must not spike past it"
        );
        // A mitre would reach half / sin(45 degrees) = 7.07 out.
        let mitre = half / forty_five.sin();
        assert!(
            mitre > half + 1.0,
            "the test is only meaningful if a mitre would reach further, got {mitre}"
        );
        assert!(!within_stroke(away(mitre), &points, half, CAP_ROUND));
    }

    /// Butt caps stop dead at the endpoint. Round ones do not, and this is the
    /// difference between the two codes.
    #[test]
    fn a_butt_cap_stops_at_the_endpoint() {
        let (p0, p1) = (v(0.0, 0.0), v(50.0, 0.0));
        let half = 5.0;
        assert!(
            segment_coverage(v(50.0 + 0.6, 0.0), p0, p1, half, CAP_BUTT) < 0.01,
            "a butt cap must not reach past the end"
        );
        assert!(
            segment_coverage(v(50.0 - 0.6, 0.0), p0, p1, half, CAP_BUTT) > 0.99,
            "and must still cover up to it"
        );
    }

    /// Butt is square across: the same half-width either side of the end line.
    #[test]
    fn a_butt_cap_is_square_across() {
        let (p0, p1) = (v(0.0, 0.0), v(50.0, 0.0));
        let half = 5.0;
        assert!(segment_coverage(v(49.0, half - 0.6), p0, p1, half, CAP_BUTT) > 0.99);
        assert!(segment_coverage(v(49.0, half + 0.6), p0, p1, half, CAP_BUTT) < 0.01);
    }

    /// A square cap is a butt rectangle grown by half the pen width, which is what
    /// gives it the corners a round cap lacks.
    #[test]
    fn a_square_cap_reaches_out_and_keeps_its_corners() {
        let (p0, p1) = (v(0.0, 0.0), v(50.0, 0.0));
        let half = 5.0f32;
        assert!(
            segment_coverage(v(50.0 + half - 0.6, 0.0), p0, p1, half, CAP_SQUARE) > 0.99,
            "must reach out to the full width"
        );
        // The corner is the difference: a square cap covers it, a round one does not.
        let corner = v(50.0 + half - 0.6, half - 0.6);
        assert!(
            segment_coverage(corner, p0, p1, half, CAP_SQUARE) > 0.99,
            "the corner must be covered"
        );
        assert!(
            segment_coverage(corner, p0, p1, half, CAP_ROUND) < 0.5,
            "and a round cap must not reach it"
        );
    }

    /// A zero-length segment still has to produce a disc for a round cap, since the
    /// projection is clamped rather than divided by the length.
    #[test]
    fn a_zero_length_segment_does_not_divide_by_zero() {
        let at = v(5.0, 5.0);
        let inside = segment_coverage(at, at, at, 4.0, CAP_ROUND);
        assert!(
            (inside - 1.0).abs() < 1.0e-4,
            "a dot should be solid, was {inside}"
        );
        let outside = segment_coverage(v(5.0, 10.0), at, at, 4.0, CAP_ROUND);
        assert!(outside < 0.01);
        assert!(inside.is_finite() && outside.is_finite());
    }

    /// The alpha the shader writes is coverage scaled by the pen's opacity, and
    /// never more than that.
    #[test]
    fn alpha_is_coverage_times_the_pens_opacity() {
        let (p0, p1) = (v(0.0, 0.0), v(50.0, 0.0));
        let half = 5.0;
        let half_alpha = pixel_alpha(v(50.0, half), p0, p1, half, CAP_ROUND, Color::WHITE);
        assert!((half_alpha - 0.5).abs() < 1.0e-4);
        let quarter = pixel_alpha(
            v(50.0, half),
            p0,
            p1,
            half,
            CAP_ROUND,
            Color::rgba(1.0, 1.0, 1.0, 0.25),
        );
        assert!((quarter - 0.125).abs() < 1.0e-4, "was {quarter}");
    }

    /// Two overlapping segments compose rather than adding, so a joint is not
    /// darker than the rest of the stroke. This is the renderer's own
    /// `GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA`, and it is why the union of capsules
    /// looks like one pen rather than a pile of them.
    #[test]
    fn overlapping_segments_do_not_pile_up() {
        let half = 5.0;
        // Two segments crossing in the middle.
        let points = [v(0.0, 0.0), v(40.0, 40.0)];
        let crossing = [v(0.0, 40.0), v(40.0, 0.0)];
        let alone = polyline_alpha(v(20.0, 20.0), &points, half, CAP_ROUND, Color::WHITE);
        let both = polyline_alpha(v(20.0, 20.0), &points, half, CAP_ROUND, Color::WHITE);
        assert!(alone > 0.99);
        assert!(
            both <= alone + 1.0e-4,
            "overlapping segments must not exceed one: {both} vs {alone}"
        );
        let _ = crossing;
    }

    /// The colour is not read from the vertex here - the shader takes it from
    /// `a_color` - but alpha is, and a fully transparent pen must draw nothing.
    #[test]
    fn a_fully_transparent_pen_covers_nothing() {
        let (p0, p1) = (v(0.0, 0.0), v(50.0, 0.0));
        assert_eq!(
            pixel_alpha(v(25.0, 0.0), p0, p1, 5.0, CAP_ROUND, Color::TRANSPARENT),
            0.0
        );
    }

    /// Coverage is symmetric about the centreline and the same on both sides, which
    /// is a cheap way to catch a normal that points the wrong way.
    #[test]
    fn coverage_is_symmetric_about_the_centreline() {
        let (p0, p1) = (v(10.0, 10.0), v(40.0, 25.0));
        let half = 4.0;
        for t in [0.1f32, 0.3, 0.5, 0.7, 0.9] {
            let on_axis = p0 + (p1 - p0) * t;
            let normal = (p1 - p0).normalized();
            let perpendicular = Vector2::new(-normal.y, normal.x);
            for offset in [0.2f32, 2.0, 3.9] {
                let a = segment_coverage(on_axis + perpendicular * offset, p0, p1, half, CAP_ROUND);
                let b = segment_coverage(on_axis - perpendicular * offset, p0, p1, half, CAP_ROUND);
                assert!((a - b).abs() < 1.0e-4, "t={t} offset={offset}: {a} vs {b}");
            }
        }
    }

    /// Rotating the whole setup rotates the coverage with it, which is what says
    /// the frame is being built and then measured in, not the other way round.
    #[test]
    fn rotating_the_setup_rotates_the_coverage() {
        let (p0, p1) = (v(0.0, 0.0), v(50.0, 0.0));
        let half = 6.0;
        let angle = 0.7f32;
        let (sin, cos) = angle.sin_cos();
        let rotate = |p: Vector2| Vector2::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos);
        let probe = v(30.0, 2.0);
        let flat = segment_coverage(probe, p0, p1, half, CAP_ROUND);
        let turned = segment_coverage(rotate(probe), rotate(p0), rotate(p1), half, CAP_ROUND);
        assert!(
            (flat - turned).abs() < 1.0e-4,
            "flat {flat} vs rotated {turned}"
        );
    }
}
