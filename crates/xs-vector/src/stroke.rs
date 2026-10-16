//! Contours to stroke capsules.
//!
//! The whole stroke is one idea: **a segment is a capsule, and the stroke is the
//! union of its segments' capsules.** Everything else follows from that, and all
//! of it lands on the GPU rather than here.
//!
//! - **Antialiasing** is a distance to the segment in the fragment shader, not a
//!   smoothed outline. Nothing is expanded on the CPU; a stroke of any width costs
//!   two endpoints per segment.
//! - **Round joins are free.** Two capsules meeting at a point cover exactly the
//!   disc of radius half-width that rounds that corner off, so nothing between the
//!   two segments has to know the other exists. Miter and bevel joins would each
//!   need geometry of their own, which is why there is no knob for them.
//! - **Round caps are free** for the same reason - the clamped projection turns
//!   each end into a half-disc. Butt and square are the two that need asking for,
//!   and both are one branch in the fragment shader.
//!
//! What is left on the CPU is the part that genuinely cannot move: dashes, because
//! deciding where a gap starts is a question about arc length along the whole
//! contour, not about any one pixel.

use std::fmt;

use xs_layout::Color;
use xs_math::Vector2;

use crate::flatten::{Contour, MIN_SEGMENT_LENGTH};

/// How a stroke ends.
///
/// Round and butt are what the distance field gives for free; square is the butt
/// rectangle grown by half the pen's width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineCap {
    #[default]
    Round,
    Butt,
    Square,
}

impl LineCap {
    /// The code the fragment shader switches on.
    ///
    /// These are the three numbers the GLSL branches on, kept in one place
    /// because a mismatch between the two sides is a shape that is subtly wrong
    /// rather than an error: a cap that is one code out renders as a cap of a
    /// different kind and nothing complains.
    #[must_use]
    pub fn code(self) -> f32 {
        match self {
            Self::Round => 0.0,
            Self::Butt => 1.0,
            Self::Square => 2.0,
        }
    }
}

/// How a stroke is drawn along a contour.
#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    pub color: Color,
    /// The pen's full width, not its radius. The half width is what the shader
    /// wants and is derived here so no caller has to remember which one it is.
    pub width: f32,
    pub cap: LineCap,
    /// Alternating on/off lengths, starting with an on run. An empty pattern is a
    /// solid line, which is the common case and costs a branch per contour rather
    /// than a second code path.
    pub dash: Vec<f32>,
}

impl Stroke {
    #[must_use]
    pub fn new(color: Color, width: f32) -> Self {
        Self {
            color,
            width,
            cap: LineCap::default(),
            dash: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_cap(mut self, cap: LineCap) -> Self {
        self.cap = cap;
        self
    }

    /// Sets the dash pattern.
    ///
    /// Negative and zero lengths are dropped, and an odd-length pattern is
    /// doubled - both are things SVG defines rather than things this crate gets to
    /// choose, and doing it here means the walker below does not have to think
    /// about either.
    #[must_use]
    pub fn with_dash(mut self, pattern: impl Into<Vec<f32>>) -> Self {
        self.dash = normalise_dash(pattern);
        self
    }

    /// Sets the dash pattern in place, keeping the rest of the stroke.
    pub fn set_dash(&mut self, pattern: impl Into<Vec<f32>>) {
        self.dash = normalise_dash(pattern);
    }

    #[must_use]
    pub fn half_width(&self) -> f32 {
        (self.width * 0.5).max(0.0)
    }

    /// Whether this stroke has anything to draw.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.width > 0.0 && self.color.a > 0.0
    }

    fn is_dashed(&self) -> bool {
        !self.dash.is_empty()
    }
}

/// Puts a dash pattern into the form the walker expects.
///
/// Two things SVG defines rather than anything this crate gets to choose: a
/// non-positive length is no length, and a pattern with an odd number of entries
/// repeats to become even. Doing it here means the walker and every builder that
/// sets a pattern cannot disagree about what one means.
fn normalise_dash(pattern: impl Into<Vec<f32>>) -> Vec<f32> {
    let mut dash: Vec<f32> = pattern.into().into_iter().filter(|l| *l > 0.0).collect();
    if dash.len() % 2 == 1 {
        let doubled = dash.clone();
        dash.extend(doubled);
    }
    dash
}

/// The dash walker's position along a contour.
///
/// The whole point of a type here is that the pattern's current entry, how much of
/// it is left, and whether it paints cannot drift apart: the offset is in the
/// same units as the pattern, and a dash boundary that lands mid-segment has to
/// carry the remainder into the next one. Handing the walker a pattern and an
/// offset separately and getting the relationship wrong does not fail, it offsets
/// every dash - which is why this is not three plain fields at the call site.
#[derive(Clone, Copy, Debug)]
struct DashCursor {
    /// Which entry of the pattern is being consumed.
    index: usize,
    /// How much of that entry is left, in pixels.
    remaining: f32,
    /// Whether the current entry paints.
    painting: bool,
}

impl DashCursor {
    /// Starts at `offset` pixels into the pattern.
    fn new(pattern: &[f32], offset: f32) -> Self {
        let mut cursor = Self {
            index: 0,
            remaining: pattern.first().copied().unwrap_or(0.0),
            painting: true,
        };
        cursor.skip(pattern, offset);
        cursor
    }

    fn advance(&mut self, pattern: &[f32]) {
        self.index = (self.index + 1) % pattern.len();
        self.remaining = pattern[self.index];
        self.painting = !self.painting;
    }

    /// Consume `amount` pixels of pattern, reporting how much of each entry they
    /// covered and whether it paints.
    ///
    /// The remainder of a partly-consumed entry is left where it is, which is what
    /// carries a dash boundary across a vertex instead of restarting the pattern
    /// there.
    fn take(&mut self, pattern: &[f32], amount: f32, out: &mut Vec<(bool, f32)>) {
        let mut left = amount;
        while left > MIN_SEGMENT_LENGTH {
            let run = self.remaining.min(left);
            out.push((self.painting, run));
            left -= run;
            self.remaining -= run;
            if self.remaining <= MIN_SEGMENT_LENGTH {
                self.advance(pattern);
            }
            // A pattern of nothing but sub-threshold lengths would otherwise spin
            // here for ever, since `run` would be zero.
            if run <= MIN_SEGMENT_LENGTH {
                break;
            }
        }
    }

    fn skip(&mut self, pattern: &[f32], amount: f32) {
        let mut sink = Vec::new();
        self.take(pattern, amount.max(0.0), &mut sink);
    }
}

/// Appends `contour`'s segments to `out` as stroke endpoints.
///
/// `out` receives, per segment: `p0.x p0.y p1.x p1.y half_width cap r g b a` -
/// ten floats, which is [`xs_layout::STROKE_VERTEX_FLOATS`].
///
/// Nothing here computes a normal, an offset or a corner. Those are the vertex
/// shader's job, and keeping them there is what makes the cost of a stroke
/// independent of its width: a hairline and a 40-pixel brush send the same twelve
/// floats per segment.
pub fn emit_stroke(contour: &Contour, stroke: &Stroke, dash_offset: f32, out: &mut Vec<f32>) {
    // A zero width or a fully transparent colour still emits geometry, and the
    // renderer would dutifully batch and draw it. This is the same reasoning as
    // the renderer's own skip for a transparent `SimpleQuad`: on hardware that is
    // fill bound, covering pixels to arrive at the colour already there is the
    // expensive way to do nothing.
    if !stroke.is_visible() {
        return;
    }
    let half_width = stroke.half_width();
    let cap = stroke.cap.code();
    let color: [f32; 4] = stroke.color.into();

    let mut push = |from: Vector2, to: Vector2, out: &mut Vec<f32>| {
        out.push(from.x);
        out.push(from.y);
        out.push(to.x);
        out.push(to.y);
        out.push(half_width);
        out.push(cap);
        out.extend_from_slice(&color);
    };

    if stroke.is_dashed() {
        emit_dashed(contour, stroke, dash_offset, &mut push, out);
        return;
    }

    let count = contour.segment_count();
    for index in 0..count {
        let (Some(from), Some(to)) = (contour.at(index), contour.at(index + 1)) else {
            continue;
        };
        // A zero-length segment has no direction, and the vertex shader divides by
        // its length to find the normal.
        if (to - from).magnitude() < MIN_SEGMENT_LENGTH {
            continue;
        }
        push(from, to, out);
    }
}

fn emit_dashed(
    contour: &Contour,
    stroke: &Stroke,
    dash_offset: f32,
    push: &mut impl FnMut(Vector2, Vector2, &mut Vec<f32>),
    out: &mut Vec<f32>,
) {
    let pattern = &stroke.dash;
    let mut cursor = DashCursor::new(pattern, dash_offset);
    let mut runs: Vec<(bool, f32)> = Vec::new();

    for index in 0..contour.segment_count() {
        let (Some(from), Some(to)) = (contour.at(index), contour.at(index + 1)) else {
            continue;
        };
        let length = (to - from).magnitude();
        if length < MIN_SEGMENT_LENGTH {
            continue;
        }
        let direction = (to - from).normalized();

        // The pattern is consumed across the whole contour, so where a boundary
        // falls inside a segment is decided here rather than per segment - and a
        // boundary landing exactly on a vertex just splits one dash into two runs.
        runs.clear();
        cursor.take(pattern, length, &mut runs);

        let mut cursor_at = from;
        for (painting, run) in runs.iter().copied() {
            let end = cursor_at + direction * run;
            if painting {
                push(cursor_at, end, out);
            }
            cursor_at = end;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flatten::flatten;
    use crate::path::Path;
    use xs_layout::STROKE_VERTEX_FLOATS;

    fn stroke_vertices(path: &Path, stroke: &Stroke) -> Vec<f32> {
        let contours = flatten(path, 0.25);
        let mut out = Vec::new();
        for contour in &contours {
            emit_stroke(contour, stroke, 0.0, &mut out);
        }
        out
    }

    fn segments(vertices: &[f32]) -> Vec<((f32, f32), (f32, f32))> {
        vertices
            .chunks_exact(STROKE_VERTEX_FLOATS)
            .map(|c| ((c[0], c[1]), (c[2], c[3])))
            .collect()
    }

    #[test]
    fn a_line_becomes_one_segment() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).line_to(10.0, 0.0);
        });
        let vertices = stroke_vertices(&path, &Stroke::new(Color::WHITE, 4.0));
        assert_eq!(segments(&vertices), vec![((0.0, 0.0), (10.0, 0.0))]);
    }

    /// The pen's *radius* is what the shader wants, and a caller asking for a
    /// width of 4 must not end up with a stroke two pixels wide.
    #[test]
    fn the_vertex_carries_the_half_width() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).line_to(10.0, 0.0);
        });
        let vertices = stroke_vertices(&path, &Stroke::new(Color::WHITE, 7.0));
        assert_eq!(vertices[4], 3.5);
    }

    #[test]
    fn the_vertex_carries_the_colour() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).line_to(10.0, 0.0);
        });
        let vertices = stroke_vertices(&path, &Stroke::new(Color::RED, 2.0));
        assert_eq!(&vertices[6..10], &[1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn each_cap_kind_has_its_own_code() {
        assert_eq!(LineCap::Round.code(), 0.0);
        assert_eq!(LineCap::Butt.code(), 1.0);
        assert_eq!(LineCap::Square.code(), 2.0);
    }

    #[test]
    fn the_cap_reaches_the_vertex() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).line_to(10.0, 0.0);
        });
        let vertices = stroke_vertices(
            &path,
            &Stroke::new(Color::WHITE, 2.0).with_cap(LineCap::Butt),
        );
        assert_eq!(vertices[5], LineCap::Butt.code());
    }

    #[test]
    fn a_closed_contour_closes() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0)
                .line_to(10.0, 0.0)
                .line_to(10.0, 10.0)
                .close();
        });
        let vertices = stroke_vertices(&path, &Stroke::new(Color::WHITE, 2.0));
        // Two drawn edges plus the implied closing one.
        assert_eq!(segments(&vertices).len(), 3);
        let last = segments(&vertices)[2];
        assert_eq!(last.1, (0.0, 0.0), "must run back to the start");
    }

    #[test]
    fn a_zero_length_segment_is_not_emitted() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).line_to(10.0, 0.0).line_to(10.0, 0.0);
        });
        let vertices = stroke_vertices(&path, &Stroke::new(Color::WHITE, 2.0));
        assert_eq!(segments(&vertices).len(), 1);
    }

    #[test]
    fn separate_contours_do_not_join() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0)
                .line_to(10.0, 0.0)
                .move_to(50.0, 0.0)
                .line_to(60.0, 0.0);
        });
        let vertices = stroke_vertices(&path, &Stroke::new(Color::WHITE, 2.0));
        assert_eq!(segments(&vertices).len(), 2);
    }

    #[test]
    fn a_stroke_costs_the_same_at_any_width() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).line_to(100.0, 0.0);
        });
        let hairline = stroke_vertices(&path, &Stroke::new(Color::WHITE, 1.0));
        let brush = stroke_vertices(&path, &Stroke::new(Color::WHITE, 40.0));
        // Identical length: the width is a number in the vertex, not geometry.
        assert_eq!(hairline.len(), brush.len());
        assert_eq!(hairline.len(), STROKE_VERTEX_FLOATS);
    }

    // -------------------------------------------------------------- dashes

    #[test]
    fn a_dash_pattern_splits_a_line() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).line_to(30.0, 0.0);
        });
        let stroke = Stroke::new(Color::WHITE, 2.0).with_dash([10.0, 10.0]);
        let painted = segments(&stroke_vertices(&path, &stroke));
        // 30 long, 10 on and 10 off: painted 0-10 and 20-30.
        assert_eq!(painted.len(), 2, "{painted:?}");
        assert_eq!(painted[0], ((0.0, 0.0), (10.0, 0.0)));
        assert_eq!(painted[1], ((20.0, 0.0), (30.0, 0.0)));
    }

    /// The walker tracks arc length along the whole contour, so the pattern has to
    /// land in the same places whatever the path is made of. The total painted
    /// length cannot show this - a pattern that restarted at every vertex would
    /// cover exactly as much - so this checks where the runs begin and how long
    /// each one is.
    ///
    /// A 20-long contour with a vertex at 3 and a 6/2 pattern. Carrying the
    /// pattern gives runs of 3, 3, 6 and 4 - the first dash split by the vertex -
    /// starting at arc 0, 3, 8 and 16. Restarting at the vertex would give 3, 6,
    /// 6 and 1 starting at 0, 3, 11 and 16.
    #[test]
    fn the_pattern_carries_across_a_polyline() {
        let bent = Path::build(|p| {
            p.move_to(0.0, 0.0).line_to(3.0, 0.0).line_to(3.0, 17.0);
        });
        let stroke = Stroke::new(Color::WHITE, 2.0).with_dash([6.0, 2.0]);
        let contour = flatten(&bent, 0.25).remove(0);
        let mut out = Vec::new();
        emit_stroke(&contour, &stroke, 0.0, &mut out);

        let painted = segments(&out);
        let starts: Vec<f32> = painted
            .iter()
            .map(|(from, _)| arc_length_at(&contour, *from))
            .collect();
        let lengths: Vec<f32> = painted
            .iter()
            .map(|(from, to)| Vector2::new(to.0 - from.0, to.1 - from.1).magnitude())
            .collect();
        assert_eq!(starts, vec![0.0, 3.0, 8.0, 16.0], "{starts:?}");
        assert_eq!(lengths, vec![3.0, 3.0, 6.0, 4.0], "{lengths:?}");
    }

    /// How far along the contour a point sits, by projecting it onto whichever
    /// polyline edge it lies on. Matching against the vertex list is not enough: a
    /// dash boundary inside a segment lands on no vertex at all.
    fn arc_length_at(contour: &Contour, point: (f32, f32)) -> f32 {
        let point = Vector2::new(point.0, point.1);
        let mut walked = 0.0;
        for index in 0..contour.segment_count() {
            let (Some(from), Some(to)) = (contour.at(index), contour.at(index + 1)) else {
                continue;
            };
            let axis = to - from;
            let length = axis.magnitude();
            if length > MIN_SEGMENT_LENGTH {
                let t = (Vector2::dot(point - from, axis) / (length * length)).clamp(0.0, 1.0);
                if (point - (from + axis * t)).magnitude() < 1.0e-3 {
                    return walked + length * t;
                }
            }
            walked += length;
        }
        walked
    }

    #[test]
    fn an_empty_pattern_is_a_solid_line() {
        let stroke = Stroke::new(Color::WHITE, 2.0);
        assert!(!stroke.is_dashed());
    }

    /// Total painted length cannot tell a continuing pattern from a restarting
    /// one - both cover the same arc - so this checks where the boundaries *are*.
    ///
    /// The vertex is at 3 along a 20-long contour with a 6/2 pattern. Restarting
    /// at the vertex would move the boundaries to 3, 11 and 16; carrying the
    /// pattern keeps them at 0, 8 and 16.
    #[test]
    fn a_dash_offset_skips_into_the_first_run() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).line_to(30.0, 0.0);
        });
        let stroke = Stroke::new(Color::WHITE, 2.0).with_dash([10.0, 10.0]);
        let mut out = Vec::new();
        let contour = flatten(&path, 0.25).remove(0);
        emit_stroke(&contour, &stroke, 15.0, &mut out);
        let painted = segments(&out);
        // Offset 15 lands in the second half of the first run, i.e. five pixels
        // before the first gap.
        assert_eq!(painted[0].0, (5.0, 0.0), "{painted:?}");
    }

    #[test]
    fn an_invisible_stroke_emits_nothing() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).line_to(10.0, 0.0);
        });
        assert!(stroke_vertices(&path, &Stroke::new(Color::WHITE, 0.0)).is_empty());
        assert!(stroke_vertices(&path, &Stroke::new(Color::TRANSPARENT, 4.0)).is_empty());
    }

    /// A curve's stroke is the union of its chords' capsules, so it is slightly
    /// *inside* the true stroke on the outside of a bow. The error is bounded by
    /// the flattening tolerance, which is what makes the tolerance the knob that
    /// matters.
    #[test]
    fn a_curved_stroke_follows_its_flattening() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).quadratic_to(50.0, 200.0, 100.0, 0.0);
        });
        let stroke = Stroke::new(Color::WHITE, 2.0);
        let coarse = flatten(&path, 10.0).remove(0);
        let fine = flatten(&path, 0.01).remove(0);
        assert!(fine.points.len() > coarse.points.len());

        // One capsule per polyline edge, whatever the tolerance.
        let mut out = Vec::new();
        emit_stroke(&coarse, &stroke, 0.0, &mut out);
        assert_eq!(out.len(), coarse.segment_count() * STROKE_VERTEX_FLOATS);
        let mut out = Vec::new();
        emit_stroke(&fine, &stroke, 0.0, &mut out);
        assert_eq!(out.len(), fine.segment_count() * STROKE_VERTEX_FLOATS);
    }

    #[test]
    fn a_dash_longer_than_the_contour_draws_one_run() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).line_to(5.0, 0.0);
        });
        let stroke = Stroke::new(Color::WHITE, 2.0).with_dash([100.0, 10.0]);
        let painted = segments(&stroke_vertices(&path, &stroke));
        assert_eq!(painted.len(), 1);
        assert_eq!(painted[0], ((0.0, 0.0), (5.0, 0.0)));
    }
}

impl fmt::Display for LineCap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Round => "round",
            Self::Butt => "butt",
            Self::Square => "square",
        };
        f.write_str(name)
    }
}
