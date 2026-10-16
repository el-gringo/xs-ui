//! The path: an ordered list of segments, and the arithmetic to evaluate it.
//!
//! A path keeps its curves. Nothing here flattens anything, because the whole
//! point of the pipeline is that the exact geometry survives until the frame
//! that draws it, and by then it is a polyline in local pixels and the GPU has
//! the rest.

use std::fmt;

use xs_math::Vector2;

/// One piece of a [`Path`].
///
/// The set is SVG's, minus nothing that matters for drawing: arcs are here in
/// their endpoint form because they are the only command whose parameters are
/// not control points, and dropping them would mean an ellipse could not be
/// written the way anyone writes one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Segment {
    /// Starts a new contour. The path is broken here, so a stroke does not join
    /// across it and a fill treats it as a separate ring.
    MoveTo(Vector2),
    LineTo(Vector2),
    QuadraticTo {
        control: Vector2,
        to: Vector2,
    },
    CubicTo {
        c1: Vector2,
        c2: Vector2,
        to: Vector2,
    },
    /// An elliptical arc, in SVG's endpoint parameterisation.
    ArcTo {
        radius: Vector2,
        /// The ellipse's own rotation, in radians, clockwise as the y axis points
        /// down.
        rotation: f32,
        large_arc: bool,
        sweep: bool,
        to: Vector2,
    },
    /// Closes the current contour with a straight line back to its start.
    Close,
}

impl Segment {
    /// Where this segment ends. `Close` ends where it started, which the caller
    /// knows and this does not, so it reports `None` rather than guessing.
    #[must_use]
    pub fn end(&self) -> Option<Vector2> {
        match self {
            Self::MoveTo(p)
            | Self::LineTo(p)
            | Self::QuadraticTo { to: p, .. }
            | Self::CubicTo { to: p, .. }
            | Self::ArcTo { to: p, .. } => Some(*p),
            Self::Close => None,
        }
    }

    /// Whether this segment draws anything at all. `MoveTo` only repositions, and
    /// a contour of nothing but `MoveTo`s is not a shape.
    #[must_use]
    pub fn is_drawing(&self) -> bool {
        !matches!(self, Self::MoveTo(_) | Self::Close)
    }

    /// The point at `t` along a line, quadratic or cubic.
    ///
    /// `LineTo` is a degree-one Bézier, which is what makes a flattened contour
    /// and an exact curve the same shape to sample. `MoveTo`, `Close` and
    /// `ArcTo` return `None`: an arc is stored as a segment but is only ever
    /// sampled after `arc_to_cubics` has replaced it, and there is no reason to
    /// carry a second evaluator for it.
    #[must_use]
    pub fn point_at(&self, p0: Vector2, t: f32) -> Option<Vector2> {
        let u = 1.0 - t;
        match self {
            Self::LineTo(p) => Some(Vector2::new(u * p0.x + t * p.x, u * p0.y + t * p.y)),
            Self::QuadraticTo { control, to } => {
                let (a, b, c) = (u * u, 2.0 * u * t, t * t);
                Some(Vector2::new(
                    a * p0.x + b * control.x + c * to.x,
                    a * p0.y + b * control.y + c * to.y,
                ))
            }
            Self::CubicTo { c1, c2, to } => {
                let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                Some(Vector2::new(
                    a * p0.x + b * c1.x + c * c2.x + d * to.x,
                    a * p0.y + b * c1.y + c * c2.y + d * to.y,
                ))
            }
            _ => None,
        }
    }

    /// The largest distance the curve can bow away from its chord, which is the
    /// number a subdivision count is derived from.
    ///
    /// Both cases come from the same bound: a curve stays within `max|B''| / 8`
    /// of its chord, because the chord at parameter `t` is where the curve would
    /// be with no second derivative and the difference integrates up from
    /// `t(1-t) <= 1/4`.
    ///
    /// - A quadratic's second derivative is the constant `2 * (p0 - 2p1 + p2)`,
    ///   so its deviation is exactly `d / 4`. That is not a bound but the real
    ///   peak: it is reached at `t = 0.5`, where the curve sits a quarter of the
    ///   way from the chord to the control point.
    /// - A cubic's second derivative is linear, peaking at `6 * max(d1, d2)`, so
    ///   its deviation is at most `3/4 * max(d1, d2)`.
    ///
    /// This is what makes the flattening count computable without walking the
    /// curve - see [`crate::flatten`].
    #[must_use]
    pub fn chord_deviation(&self, p0: Vector2) -> Option<f32> {
        match self {
            Self::QuadraticTo { control, to } => Some(second_difference(p0, *control, *to) / 4.0),
            Self::CubicTo { c1, c2, to } => {
                let d = second_difference(p0, *c1, *c2).max(second_difference(*c1, *c2, *to));
                Some(d * 3.0 / 4.0)
            }
            Self::LineTo(_) => Some(0.0),
            _ => None,
        }
    }
}

fn second_difference(a: Vector2, b: Vector2, c: Vector2) -> f32 {
    Vector2::new(a.x - 2.0 * b.x + c.x, a.y - 2.0 * b.y + c.y).magnitude()
}

/// A sequence of [`Segment`]s.
///
/// Every mutating method returns `&mut Self` so a path reads as one expression;
/// [`Path::build`] takes a closure for the same reason, which is how a shape is
/// usually written.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path {
    pub segments: Vec<Segment>,
}

impl Path {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `path(|p| { p.move_to(..).line_to(..) })`
    #[must_use]
    pub fn build(f: impl FnOnce(&mut Path)) -> Self {
        let mut path = Path::new();
        f(&mut path);
        path
    }

    /// Parses SVG path data - `M0,0 L10,10 C...` - see [`crate::parser`].
    pub fn parse(data: &str) -> Result<Self, PathError> {
        crate::parser::parse(data)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.segments.iter().any(Segment::is_drawing)
    }

    /// The number of points the drawing segments name, `MoveTo`s included. A
    /// `Close` adds no new point.
    #[must_use]
    pub fn point_count(&self) -> usize {
        self.segments
            .iter()
            .filter(|s| !matches!(s, Segment::Close))
            .count()
    }

    pub fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.segments.push(Segment::MoveTo(Vector2::new(x, y)));
        self
    }

    pub fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.segments.push(Segment::LineTo(Vector2::new(x, y)));
        self
    }

    pub fn quadratic_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) -> &mut Self {
        self.segments.push(Segment::QuadraticTo {
            control: Vector2::new(cx, cy),
            to: Vector2::new(x, y),
        });
        self
    }

    /// The smooth continuation of a previous cubic: its first control point is
    /// the mirror of the previous one's second.
    pub fn smooth_cubic_to(&mut self, c2x: f32, c2y: f32, x: f32, y: f32) -> &mut Self {
        let reflected = self.reflected_control();
        self.segments.push(Segment::CubicTo {
            c1: reflected,
            c2: Vector2::new(c2x, c2y),
            to: Vector2::new(x, y),
        });
        self
    }

    pub fn cubic_to(
        &mut self,
        c1x: f32,
        c1y: f32,
        c2x: f32,
        c2y: f32,
        x: f32,
        y: f32,
    ) -> &mut Self {
        self.segments.push(Segment::CubicTo {
            c1: Vector2::new(c1x, c1y),
            c2: Vector2::new(c2x, c2y),
            to: Vector2::new(x, y),
        });
        self
    }

    /// The smooth continuation of a previous quadratic: its control point is the
    /// mirror of the previous one's.
    pub fn smooth_quadratic_to(&mut self, x: f32, y: f32) -> &mut Self {
        let control = self.reflected_control();
        self.segments.push(Segment::QuadraticTo {
            control,
            to: Vector2::new(x, y),
        });
        self
    }

    /// An elliptical arc, in SVG's endpoint form: a radius, the ellipse's own
    /// rotation in degrees, the large-arc and sweep flags, and the endpoint.
    ///
    /// Nine arguments is a lot, and it is still the fewest that describe an arc.
    #[allow(clippy::too_many_arguments)]
    pub fn arc_to(
        &mut self,
        rx: f32,
        ry: f32,
        rotation_degrees: f32,
        large_arc: bool,
        sweep: bool,
        x: f32,
        y: f32,
    ) -> &mut Self {
        self.segments.push(Segment::ArcTo {
            radius: Vector2::new(rx, ry),
            rotation: rotation_degrees.to_radians(),
            large_arc,
            sweep,
            to: Vector2::new(x, y),
        });
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.segments.push(Segment::Close);
        self
    }

    pub fn push(&mut self, segment: Segment) -> &mut Self {
        self.segments.push(segment);
        self
    }

    /// The point `S` reflects: twice the last point minus the outgoing control
    /// point of the curve before it, which is where it inherits from.
    ///
    /// With no curve before it there is nothing to mirror and this is the
    /// current point, which is what SVG says a smooth command does at the start
    /// of a subpath.
    fn reflected_control(&self) -> Vector2 {
        let Some(current) = self.current_point() else {
            return Vector2::zero();
        };
        let previous = self.segments.iter().rev().find(|s| s.is_drawing());
        match previous {
            Some(Segment::CubicTo { c2, to, .. }) => {
                Vector2::new(2.0 * to.x - c2.x, 2.0 * to.y - c2.y)
            }
            Some(Segment::QuadraticTo { control, to }) => {
                Vector2::new(2.0 * to.x - control.x, 2.0 * to.y - control.y)
            }
            // A line and an arc have no outgoing control point to mirror, so the
            // reflection is the current point - which is what SVG says a smooth
            // command does in that case.
            Some(Segment::LineTo(_)) | Some(Segment::ArcTo { .. }) | None => current,
            // `find` only ever yields a drawing segment, and those are the three
            // above.
            Some(_) => current,
        }
    }

    /// Where a segment appended now would start.
    #[must_use]
    pub fn current_point(&self) -> Option<Vector2> {
        for segment in self.segments.iter().rev() {
            match segment {
                Segment::MoveTo(p) | Segment::LineTo(p) => return Some(*p),
                Segment::QuadraticTo { to, .. }
                | Segment::CubicTo { to, .. }
                | Segment::ArcTo { to, .. } => return Some(*to),
                // A `Close` returns to the start of its contour, which the
                // reverse walk cannot see from here; treating the path as
                // continuing from the last named point is what a builder needs
                // and what every caller actually means by asking.
                Segment::Close => continue,
            }
        }
        None
    }

    /// Every point the path names, `Close` excluded. The bounding box of these
    /// plus the curve bulges is the shape's bounds - see [`crate::mesh`], which
    /// is where that is put together properly.
    #[must_use]
    pub fn points(&self) -> Vec<Vector2> {
        self.segments.iter().filter_map(Segment::end).collect()
    }

    /// The bounds of the path's control points, inflated by nothing.
    ///
    /// Always contains the shape, and usually is the shape: for a path whose
    /// curves do not bulge outside their control hull - which is every path made
    /// of lines and of curves that stay inside their own controls - it is exact.
    /// The mesh builder measures the flattened result instead, so this is for
    /// callers that want a cheap conservative answer.
    #[must_use]
    pub fn control_bounds(&self) -> Option<(Vector2, Vector2)> {
        let mut min = Vector2::new(f32::INFINITY, f32::INFINITY);
        let mut max = Vector2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut any = false;
        let mut keep = |p: Vector2| {
            any = true;
            min = Vector2::new(min.x.min(p.x), min.y.min(p.y));
            max = Vector2::new(max.x.max(p.x), max.y.max(p.y));
        };
        for segment in &self.segments {
            match segment {
                Segment::MoveTo(p) | Segment::LineTo(p) => keep(*p),
                Segment::QuadraticTo { control, to } => {
                    keep(*control);
                    keep(*to);
                }
                Segment::CubicTo { c1, c2, to } => {
                    keep(*c1);
                    keep(*c2);
                    keep(*to);
                }
                Segment::ArcTo { radius, to, .. } => {
                    keep(*to);
                    // The ellipse's own extent, generously: its rotated bounding
                    // half-axis is never longer than rx + ry.
                    keep(Vector2::new(
                        to.x - radius.x - radius.y,
                        to.y - radius.x - radius.y,
                    ));
                    keep(Vector2::new(
                        to.x + radius.x + radius.y,
                        to.y + radius.x + radius.y,
                    ));
                }
                Segment::Close => {}
            }
        }
        any.then_some((min, max))
    }

    /// Every coordinate multiplied by `factor`.
    ///
    /// This is what turns a path an app authored in logical pixels into one in
    /// device pixels, and it has to happen in the same place and at the same
    /// moment as `Style::scaled` - after the element leaves the app's hands and
    /// before anything downstream flattens it - or the flattening tolerance
    /// would be measured in the wrong unit.
    pub fn scale_mut(&mut self, factor: f32) {
        if factor == 1.0 {
            return;
        }
        let scale = |p: &mut Vector2| {
            p.x *= factor;
            p.y *= factor;
        };
        let scale_radius = |r: &mut Vector2| {
            r.x *= factor;
            r.y *= factor;
        };
        for segment in &mut self.segments {
            match segment {
                Segment::MoveTo(p) | Segment::LineTo(p) => scale(p),
                Segment::QuadraticTo { control, to } => {
                    scale(control);
                    scale(to);
                }
                Segment::CubicTo { c1, c2, to } => {
                    scale(c1);
                    scale(c2);
                    scale(to);
                }
                Segment::ArcTo { radius, to, .. } => {
                    // Lengths, like everything else. The rotation is an angle and
                    // stays where it is, exactly as `Style::scaled` leaves a
                    // gradient's angle alone.
                    scale_radius(radius);
                    scale(to);
                }
                Segment::Close => {}
            }
        }
    }

    /// The same path with every coordinate multiplied by `factor`.
    #[must_use]
    pub fn scaled(&self, factor: f32) -> Self {
        let mut path = self.clone();
        path.scale_mut(factor);
        path
    }

    /// The same path moved by `offset`, which is how a shape is placed inside a
    /// node without rebuilding it.
    #[must_use]
    pub fn translated(&self, offset: Vector2) -> Self {
        let mut path = self.clone();
        let move_by = |p: &mut Vector2| {
            p.x += offset.x;
            p.y += offset.y;
        };
        for segment in &mut path.segments {
            match segment {
                Segment::MoveTo(p) | Segment::LineTo(p) => move_by(p),
                Segment::QuadraticTo { control, to } => {
                    move_by(control);
                    move_by(to);
                }
                Segment::CubicTo { c1, c2, to } => {
                    move_by(c1);
                    move_by(c2);
                    move_by(to);
                }
                Segment::ArcTo { to, .. } => move_by(to),
                Segment::Close => {}
            }
        }
        path
    }

    /// The same path scaled about `origin` by `factor`, uniformly.
    ///
    /// Scaling about the origin in place is almost never what is meant - it slides
    /// the path off its own anchor as it grows - so this form takes the point to
    /// hold still. `view_box` does *not* use it: a view box scales about its own
    /// min corner, because that corner is what the coordinates are relative to.
    #[must_use]
    pub fn scaled_about(&self, origin: Vector2, factor: f32) -> Self {
        self.translated(-origin).scaled(factor).translated(origin)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for segment in &self.segments {
            match segment {
                Segment::MoveTo(p) => write!(f, "M{},{}", p.x, p.y)?,
                Segment::LineTo(p) => write!(f, " L{},{}", p.x, p.y)?,
                Segment::QuadraticTo { control, to } => {
                    write!(f, " Q{},{} {},{}", control.x, control.y, to.x, to.y)?;
                }
                Segment::CubicTo { c1, c2, to } => {
                    write!(f, " C{},{} {},{} {},{}", c1.x, c1.y, c2.x, c2.y, to.x, to.y)?
                }
                Segment::ArcTo {
                    radius,
                    rotation,
                    large_arc,
                    sweep,
                    to,
                } => {
                    let flags = |b: bool| if b { '1' } else { '0' };
                    write!(
                        f,
                        " A{},{} {} {} {} {},{}",
                        radius.x,
                        radius.y,
                        rotation.to_degrees(),
                        flags(*large_arc),
                        flags(*sweep),
                        to.x,
                        to.y
                    )?;
                }
                Segment::Close => write!(f, " Z")?,
            }
        }
        Ok(())
    }
}

/// What went wrong reading SVG path data.
///
/// Carries the byte offset rather than a line and column: path data is normally
/// a single line, often a long one, and the offset is the thing that actually
/// locates the mistake.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at offset {}", self.message, self.offset)
    }
}

impl std::error::Error for PathError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32) -> Vector2 {
        Vector2::new(x, y)
    }

    #[test]
    fn a_line_is_a_degree_one_bezier() {
        let segment = Segment::LineTo(v(10.0, 0.0));
        assert_eq!(segment.point_at(v(0.0, 0.0), 0.0), Some(v(0.0, 0.0)));
        assert_eq!(segment.point_at(v(0.0, 0.0), 0.5), Some(v(5.0, 0.0)));
        assert_eq!(segment.point_at(v(0.0, 0.0), 1.0), Some(v(10.0, 0.0)));
    }

    #[test]
    fn a_straight_cubic_is_exactly_its_line() {
        let segment = Segment::CubicTo {
            c1: v(10.0 / 3.0, 0.0),
            c2: v(20.0 / 3.0, 0.0),
            to: v(10.0, 0.0),
        };
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            let point = segment.point_at(v(0.0, 0.0), t).unwrap();
            assert!(
                (point.x - t * 10.0).abs() < 1.0e-4,
                "at t={t} got {point:?}"
            );
        }
    }

    #[test]
    fn a_quadratic_peaks_half_way_to_its_control_point() {
        // p0 = 0, control = 10, to = 0. At t = 0.5 the curve is a quarter of the
        // way along and half of the way to the control, i.e. 2 * 0.5 * 0.5 * 10.
        let segment = Segment::QuadraticTo {
            control: v(10.0, 0.0),
            to: v(0.0, 0.0),
        };
        let peak = segment.point_at(v(0.0, 0.0), 0.5).unwrap();
        assert!((peak.x - 5.0).abs() < 1.0e-5, "peak was {}", peak.x);
    }

    #[test]
    fn chord_deviation_is_exact_for_a_quadratic() {
        // Control at 10 over a chord from 0 to 0. The curve peaks at 5, and
        // |0 - 2*10 + 0| / 4 is 5 too - this is the peak, not a bound on it.
        let segment = Segment::QuadraticTo {
            control: v(10.0, 0.0),
            to: v(0.0, 0.0),
        };
        let deviation = segment.chord_deviation(v(0.0, 0.0)).unwrap();
        assert!((deviation - 5.0).abs() < 1.0e-5, "was {deviation}");
    }

    #[test]
    fn a_flat_curve_has_no_deviation() {
        let segment = Segment::CubicTo {
            c1: v(10.0, 0.0),
            c2: v(20.0, 0.0),
            to: v(30.0, 0.0),
        };
        let deviation = segment.chord_deviation(v(0.0, 0.0)).unwrap();
        assert!(deviation < 1.0e-5, "was {deviation}");
    }

    /// The bound is only worth having if it is never below the truth. Sampled
    /// densely against the real curve, a quadratic's widest bow has to sit under
    /// what `chord_deviation` claims.
    #[test]
    fn chord_deviation_bounds_the_real_deviation() {
        let segment = Segment::QuadraticTo {
            control: v(30.0, 40.0),
            to: v(10.0, 0.0),
        };
        let p0 = v(0.0, 0.0);
        let claimed = segment.chord_deviation(p0).unwrap();
        let chord_at = |t: f32| Vector2::new(p0.x + t * (10.0 - p0.x), p0.y + t * (0.0 - p0.y));
        let mut worst: f32 = 0.0;
        for i in 0..=2000 {
            let t = i as f32 / 2000.0;
            let on_curve = segment.point_at(p0, t).unwrap();
            let straight = chord_at(t);
            worst = worst.max((on_curve - straight).magnitude());
        }
        assert!(
            worst <= claimed + 1.0e-3,
            "worst {worst} > claimed {claimed}"
        );
        // And it is tight, or the flattening would be doing more work than it has to.
        assert!(
            claimed - worst < 1.0e-2,
            "bound {claimed} is loose, worst {worst}"
        );
    }

    #[test]
    fn current_point_walks_back_to_the_last_named_position() {
        let mut path = Path::new();
        assert_eq!(path.current_point(), None);
        path.move_to(1.0, 2.0).line_to(3.0, 4.0);
        assert_eq!(path.current_point(), Some(v(3.0, 4.0)));
        path.cubic_to(0.0, 0.0, 0.0, 0.0, 9.0, 9.0);
        assert_eq!(path.current_point(), Some(v(9.0, 9.0)));
    }

    #[test]
    fn a_smooth_cubic_mirrors_the_previous_control_point() {
        let mut path = Path::new();
        path.move_to(0.0, 0.0)
            .cubic_to(0.0, 10.0, 10.0, 10.0, 10.0, 0.0);
        path.smooth_cubic_to(20.0, -10.0, 30.0, 0.0);
        let last = path.segments.last().unwrap();
        let Segment::CubicTo { c1, .. } = last else {
            panic!("expected a cubic");
        };
        // Mirror of the previous cubic's c2 = (10, 10) about its end (10, 0).
        assert_eq!(*c1, v(10.0, -10.0));
    }

    #[test]
    fn a_smooth_command_with_nothing_before_it_starts_where_it_is() {
        let mut path = Path::new();
        path.move_to(4.0, 5.0);
        path.smooth_quadratic_to(9.0, 9.0);
        let last = path.segments.last().unwrap();
        let Segment::QuadraticTo { control, .. } = last else {
            panic!("expected a quadratic");
        };
        assert_eq!(*control, v(4.0, 5.0));
    }

    #[test]
    fn scaling_a_path_by_one_changes_nothing() {
        let mut path = Path::build(|p| {
            p.move_to(0.0, 0.0)
                .line_to(10.0, 0.0)
                .quadratic_to(10.0, 10.0, 0.0, 10.0)
                .cubic_to(-5.0, 10.0, -5.0, -10.0, 0.0, 0.0)
                .arc_to(5.0, 5.0, 30.0, true, false, 20.0, 0.0)
                .close();
        });
        let before = path.clone();
        path.scale_mut(1.0);
        assert_eq!(path, before);
    }

    #[test]
    fn scaling_multiplies_lengths_and_leaves_angles_alone() {
        let mut path = Path::build(|p| {
            p.move_to(0.0, 0.0)
                .arc_to(5.0, 5.0, 90.0, true, false, 20.0, 0.0);
        });
        path.scale_mut(2.0);
        let Segment::ArcTo {
            radius, rotation, ..
        } = path.segments[1]
        else {
            panic!("expected an arc");
        };
        assert_eq!(radius, v(10.0, 10.0));
        assert!((rotation.to_degrees() - 90.0).abs() < 1.0e-4);
    }

    #[test]
    fn translating_moves_every_control_point() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).cubic_to(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        });
        let moved = path.translated(v(10.0, 20.0));
        assert_eq!(moved.segments[0], Segment::MoveTo(v(10.0, 20.0)));
        assert_eq!(
            moved.segments[1],
            Segment::CubicTo {
                c1: v(11.0, 22.0),
                c2: v(13.0, 24.0),
                to: v(15.0, 26.0),
            }
        );
    }

    #[test]
    fn scaling_about_a_point_leaves_that_point_put() {
        let path = Path::build(|p| {
            p.move_to(10.0, 10.0).line_to(20.0, 20.0);
        });
        let scaled = path.scaled_about(v(10.0, 10.0), 2.0);
        assert_eq!(scaled.segments[0], Segment::MoveTo(v(10.0, 10.0)));
        assert_eq!(scaled.segments[1], Segment::LineTo(v(30.0, 30.0)));
    }

    #[test]
    fn a_path_of_only_moves_draws_nothing() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0).move_to(5.0, 5.0);
        });
        assert!(path.is_empty());
    }

    #[test]
    fn display_round_trips_through_the_parser() {
        let path = Path::build(|p| {
            p.move_to(0.0, 0.0)
                .line_to(10.0, 0.0)
                .quadratic_to(15.0, 5.0, 10.0, 10.0)
                .cubic_to(5.0, 15.0, 0.0, 15.0, 0.0, 10.0)
                .close();
        });
        let printed = path.to_string();
        assert_eq!(Path::parse(&printed).unwrap(), path);
    }
}
