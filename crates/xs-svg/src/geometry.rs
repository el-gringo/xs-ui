//! The shape elements, as paths.
//!
//! Every element here is a `<Path>` in the element's own user units, which is
//! what `xs-vector` draws. The transforms are *not* applied: a group with a
//! `transform` keeps its children in their own coordinates and the walker
//! composes one matrix instead, so a scaled group's stroke width can see the
//! scale.

use xs_vector::{Path, Segment};
use xs_xml::Element;

use crate::error::SvgResult;
use crate::values::{length, numbers};

/// The base a percentage is measured against.
///
/// Percentages in a shape's geometry are rare enough that measuring them against
/// the view box's width is a reasonable reading of a value that has no better
/// one available here; what it means for a shape inside a scaled group is a
/// question this crate does not try to answer.
const PERCENT_BASE: f32 = 1.0;

/// The element names this module turns into geometry, which is how the walker
/// knows to hand it an element.
pub const SHAPE_ELEMENTS: [&str; 7] = [
    "path", "rect", "circle", "ellipse", "line", "polyline", "polygon",
];

/// Whether an element is one of the shapes, ignoring any namespace prefix.
#[must_use]
pub fn is_shape(name: &str) -> bool {
    SHAPE_ELEMENTS.contains(&local_name(name))
}

/// The element name without its namespace prefix.
///
/// An icon set in the wild writes `svg:rect` and `<svg:svg>` as happily as the
/// unprefixed names, and the prefix never changes what is drawn.
#[must_use]
pub fn local_name(name: &str) -> &str {
    match name.split_once(':') {
        Some((_, local)) => local,
        None => name,
    }
}

/// The path of a shape element, if it is one.
///
/// # Errors
///
/// [`SvgError::BadValue`](crate::SvgError::BadValue) for a geometry attribute that does
/// not read, and [`SvgError::Path`](crate::SvgError::Path) for bad path data.
pub fn path_of(element: &Element) -> SvgResult<Option<Path>> {
    let path = match local_name(element.name()) {
        "path" => path_data(element)?,
        "rect" => rect(element)?,
        "circle" => circle(element)?,
        "ellipse" => ellipse(element)?,
        "line" => line(element)?,
        "polyline" => points(element, false)?,
        "polygon" => points(element, true)?,
        _ => return Ok(None),
    };
    Ok(Some(path))
}

/// The `d` attribute, which is path data verbatim.
fn path_data(element: &Element) -> SvgResult<Path> {
    // An element with no `d` draws nothing and is not an error: `<path/>` is
    // what a template leaves behind before it is filled in.
    Ok(match element.attribute("d") {
        Some(data) => Path::parse(data)?,
        None => Path::new(),
    })
}

/// `<rect>`, with the corner radii SVG defines: either one rounds both corners
/// and the other follows it, and neither may exceed half its own side.
fn rect(element: &Element) -> SvgResult<Path> {
    let x = length_of(element, "x")?;
    let y = length_of(element, "y")?;
    let width = length_of(element, "width")?;
    let height = length_of(element, "height")?;

    let mut rx = length_of(element, "rx")?;
    let mut ry = length_of(element, "ry")?;
    match (rx, ry) {
        (r, 0.0) if r > 0.0 => ry = r,
        (0.0, r) if r > 0.0 => rx = r,
        _ => {}
    }
    rx = rx.min(width / 2.0).max(0.0);
    ry = ry.min(height / 2.0).max(0.0);

    Ok(Path::build(|path| {
        if rx > 0.0 || ry > 0.0 {
            // Each corner is a quarter turn of an arc, which is the case where
            // an arc's endpoint form has to be right: a sweep the other way draws
            // a bow outside the rectangle.
            path.move_to(x + rx, y)
                .line_to(x + width - rx, y)
                .arc_to(rx, ry, 0.0, false, true, x + width, y + ry)
                .line_to(x + width, y + height - ry)
                .arc_to(rx, ry, 0.0, false, true, x + width - rx, y + height)
                .line_to(x + rx, y + height)
                .arc_to(rx, ry, 0.0, false, true, x, y + height - ry)
                .line_to(x, y + ry)
                .arc_to(rx, ry, 0.0, false, true, x + rx, y);
        } else {
            path.move_to(x, y)
                .line_to(x + width, y)
                .line_to(x + width, y + height)
                .line_to(x, y + height);
        }
        path.close();
    }))
}

/// `<circle>`, which is an ellipse whose two radii happen to be equal.
fn circle(element: &Element) -> SvgResult<Path> {
    let cx = length_of(element, "cx")?;
    let cy = length_of(element, "cy")?;
    let r = length_of(element, "r")?;
    Ok(ellipse_path(cx, cy, r, r))
}

/// `<ellipse>`, as two half turns of the same sweep: each arc is half a turn, so
/// two of them in one direction are a whole one.
fn ellipse(element: &Element) -> SvgResult<Path> {
    let cx = length_of(element, "cx")?;
    let cy = length_of(element, "cy")?;
    let rx = length_of(element, "rx")?;
    let ry = length_of(element, "ry")?;
    Ok(ellipse_path(cx, cy, rx, ry))
}

fn ellipse_path(cx: f32, cy: f32, rx: f32, ry: f32) -> Path {
    Path::build(|path| {
        path.move_to(cx + rx, cy)
            .arc_to(rx, ry, 0.0, false, false, cx - rx, cy)
            .arc_to(rx, ry, 0.0, false, false, cx + rx, cy)
            .close();
    })
}

/// `<line>`, which has no interior and so only ever carries a stroke.
fn line(element: &Element) -> SvgResult<Path> {
    let (x1, y1) = (length_of(element, "x1")?, length_of(element, "y1")?);
    let (x2, y2) = (length_of(element, "x2")?, length_of(element, "y2")?);
    Ok(Path::build(|path| {
        path.move_to(x1, y1).line_to(x2, y2);
    }))
}

/// `<polyline>` and `<polygon>`, which differ only in whether the contour closes.
fn points(element: &Element, close: bool) -> SvgResult<Path> {
    let points = match element.attribute("points") {
        Some(points) => numbers(points, "points")?,
        None => return Ok(Path::new()),
    };
    Ok(Path::build(|path| {
        let (pairs, _odd) = points.as_chunks::<2>();
        for (index, pair) in pairs.iter().enumerate() {
            if index == 0 {
                path.move_to(pair[0], pair[1]);
            } else {
                path.line_to(pair[0], pair[1]);
            }
        }
        if close {
            path.close();
        }
    }))
}

/// One length attribute, zero when it is absent - which is what every one of
/// these attributes defaults to.
fn length_of(element: &Element, name: &'static str) -> SvgResult<f32> {
    match element.attribute(name) {
        Some(value) => length(value, PERCENT_BASE, name),
        None => Ok(0.0),
    }
}

/// The same path moved by `matrix`.
///
/// An arc is not an affine invariant of itself - an arc is a circle, and a
/// matrix that turns a circle into an ellipse has no `ArcTo` left to write - so
/// arcs become cubics first, which is exact rather than approximated and is the
/// same conversion the flattener already does.
#[must_use]
pub fn transform(path: &Path, matrix: &xs_math::Matrix3) -> Path {
    if *matrix == xs_math::Matrix3::identity() {
        return path.clone();
    }
    let point = |at| matrix.transform_point(at);
    let mut out = Path::new();
    let mut from = xs_math::Vector2::zero();

    for segment in &path.segments {
        match segment {
            Segment::MoveTo(p) => {
                from = *p;
                let p = point(*p);
                out.move_to(p.x, p.y);
            }
            Segment::LineTo(p) => {
                from = *p;
                let p = point(*p);
                out.line_to(p.x, p.y);
            }
            Segment::QuadraticTo { control, to } => {
                from = *to;
                let control = point(*control);
                let to = point(*to);
                out.quadratic_to(control.x, control.y, to.x, to.y);
            }
            Segment::CubicTo { c1, c2, to } => {
                from = *to;
                let c1 = point(*c1);
                let c2 = point(*c2);
                let to = point(*to);
                out.cubic_to(c1.x, c1.y, c2.x, c2.y, to.x, to.y);
            }
            Segment::ArcTo { .. } => {
                for cubic in xs_vector::arc_to_cubics(from, segment) {
                    transform_one(&mut out, &cubic, matrix);
                }
                from = segment.end().unwrap_or(from);
            }
            Segment::Close => {
                out.close();
            }
        }
    }
    out
}

/// One already-cubic segment, moved by `matrix`.
fn transform_one(out: &mut Path, segment: &Segment, matrix: &xs_math::Matrix3) {
    match segment {
        Segment::MoveTo(p) => {
            let p = matrix.transform_point(*p);
            out.move_to(p.x, p.y);
        }
        Segment::LineTo(p) => {
            let p = matrix.transform_point(*p);
            out.line_to(p.x, p.y);
        }
        Segment::QuadraticTo { control, to } => {
            let control = matrix.transform_point(*control);
            let to = matrix.transform_point(*to);
            out.quadratic_to(control.x, control.y, to.x, to.y);
        }
        Segment::CubicTo { c1, c2, to } => {
            let c1 = matrix.transform_point(*c1);
            let c2 = matrix.transform_point(*c2);
            let to = matrix.transform_point(*to);
            out.cubic_to(c1.x, c1.y, c2.x, c2.y, to.x, to.y);
        }
        Segment::ArcTo { to, .. } => {
            let to = matrix.transform_point(*to);
            out.line_to(to.x, to.y);
        }
        Segment::Close => {
            out.close();
        }
    }
}

/// Whether a path has anything in it, which `<path/>` and an empty `points` do
/// not.
#[must_use]
pub fn is_drawable(path: &Path) -> bool {
    !path.is_empty() && path.segments.iter().any(Segment::is_drawing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xs_xml::parse;

    fn path_of_source(source: &str) -> Path {
        let document = parse(source).expect("parses");
        let element = document.root().expect("has a root");
        path_of(element).expect("reads").expect("is a shape")
    }

    #[test]
    fn a_rect_is_four_lines() {
        let path = path_of_source("<rect x=\"10\" y=\"20\" width=\"30\" height=\"40\"/>");
        let expected = 5;
        assert_eq!(path.segments.len(), expected);
        assert_eq!(path.segments.last(), Some(&Segment::Close));
    }

    #[test]
    fn a_rect_defaults_to_the_origin() {
        let path = path_of_source("<rect width=\"30\" height=\"40\"/>");
        let first = path.segments.first();
        assert_eq!(
            first,
            Some(&Segment::MoveTo(xs_math::Vector2::new(0.0, 0.0)))
        );
    }

    #[test]
    fn one_corner_radius_rounds_both() {
        let path = path_of_source("<rect width=\"30\" height=\"40\" rx=\"5\"/>");
        // Four arcs and four straight sides, where the square version is five
        // segments of which one is the close.
        let arcs = path
            .segments
            .iter()
            .filter(|segment| matches!(segment, Segment::ArcTo { .. }))
            .count();
        assert_eq!(arcs, 4);
    }

    #[test]
    fn a_radius_beyond_half_the_side_is_clamped() {
        let path = path_of_source("<rect width=\"30\" height=\"40\" rx=\"100\"/>");
        let radius = path
            .segments
            .iter()
            .find_map(|segment| match segment {
                Segment::ArcTo { radius, .. } => Some(radius.x),
                _ => None,
            })
            .expect("there is an arc");
        assert_eq!(radius, 15.0);
    }

    #[test]
    fn a_circle_is_two_arcs() {
        let path = path_of_source("<circle cx=\"5\" cy=\"5\" r=\"10\"/>");
        let arcs = path
            .segments
            .iter()
            .filter(|segment| matches!(segment, Segment::ArcTo { .. }))
            .count();
        let expected = 2;
        assert_eq!(arcs, expected);
    }

    #[test]
    fn an_ellipse_keeps_both_radii() {
        let path = path_of_source("<ellipse cx=\"0\" cy=\"0\" rx=\"10\" ry=\"4\"/>");
        let radius = path
            .segments
            .iter()
            .find_map(|segment| match segment {
                Segment::ArcTo { radius, .. } => Some(*radius),
                _ => None,
            })
            .expect("there is an arc");
        let expected = xs_math::Vector2::new(10.0, 4.0);
        assert_eq!(radius, expected);
    }

    #[test]
    fn a_line_is_a_move_and_a_line_to() {
        let path = path_of_source("<line x1=\"1\" y1=\"2\" x2=\"3\" y2=\"4\"/>");
        let expected = vec![
            Segment::MoveTo(xs_math::Vector2::new(1.0, 2.0)),
            Segment::LineTo(xs_math::Vector2::new(3.0, 4.0)),
        ];
        assert_eq!(path.segments, expected);
    }

    #[test]
    fn a_polyline_does_not_close_and_a_polygon_does() {
        let polyline = path_of_source("<polyline points=\"0,0 1,1 2,0\"/>");
        let polygon = path_of_source("<polygon points=\"0,0 1,1 2,0\"/>");
        assert!(!polyline.segments.contains(&Segment::Close));
        assert!(polygon.segments.contains(&Segment::Close));
    }

    #[test]
    fn points_may_be_separated_by_anything() {
        let commas = path_of_source("<polyline points=\"0,0 1,1\"/>");
        let spaces = path_of_source("<polyline points=\"0 0 1 1\"/>");
        assert_eq!(commas, spaces);
    }

    #[test]
    fn a_trailing_odd_point_is_dropped() {
        // An odd count is malformed, and dropping the half point is kinder than
        // refusing the whole file over it.
        let path = path_of_source("<polyline points=\"0,0 1,1 2\"/>");
        let expected = 2;
        assert_eq!(path.segments.len(), expected);
    }

    #[test]
    fn a_path_with_no_d_is_empty_rather_than_an_error() {
        let path = path_of_source("<path/>");
        let expected = false;
        assert_eq!(is_drawable(&path), expected);
    }

    #[test]
    fn path_data_is_read_verbatim() {
        let path = path_of_source("<path d=\"M0 0 L10 10 Z\"/>");
        let expected = 3;
        assert_eq!(path.segments.len(), expected);
        assert!(is_drawable(&path));
    }

    #[test]
    fn bad_path_data_is_an_error() {
        let document = parse("<path d=\"M0 0 Q\"/>").expect("parses");
        let error = path_of(document.root().expect("has a root")).unwrap_err();
        assert!(error.to_string().starts_with("invalid path data"));
    }

    #[test]
    fn a_geometry_value_that_is_not_a_length_is_an_error() {
        let document = parse("<rect width=\"wide\"/>").expect("parses");
        let error = path_of(document.root().expect("has a root")).unwrap_err();
        assert_eq!(error.to_string(), "'wide' is not a valid width");
    }

    #[test]
    fn an_element_that_is_not_a_shape_has_no_path() {
        let document = parse("<g/>").expect("parses");
        let expected = None;
        assert_eq!(
            path_of(document.root().expect("has a root")).unwrap(),
            expected
        );
    }

    #[test]
    fn a_namespace_prefix_does_not_change_the_element() {
        assert!(is_shape("svg:rect"));
        assert!(is_shape("rect"));
        assert!(!is_shape("svg:linearGradient"));
        assert_eq!(local_name("svg:rect"), "rect");
    }

    #[test]
    fn an_identity_transform_leaves_the_arcs_alone() {
        // The arcs of an untransformed circle stay arcs rather than becoming
        // eight cubics, which is both smaller and exact.
        let path = ellipse_path(0.0, 0.0, 10.0, 10.0);
        let same = transform(&path, &xs_math::Matrix3::identity());
        assert_eq!(same, path);
        assert!(
            same.segments
                .iter()
                .any(|segment| matches!(segment, Segment::ArcTo { .. }))
        );
    }

    #[test]
    fn transforming_moves_every_control_point() {
        let path = Path::build(|path| {
            path.move_to(1.0, 1.0)
                .cubic_to(2.0, 2.0, 3.0, 3.0, 4.0, 4.0);
        });
        let moved = transform(
            &path,
            &xs_math::Matrix3::translation(xs_math::Vector2::new(10.0, 20.0)),
        );
        let expected = Segment::CubicTo {
            c1: xs_math::Vector2::new(12.0, 22.0),
            c2: xs_math::Vector2::new(13.0, 23.0),
            to: xs_math::Vector2::new(14.0, 24.0),
        };
        assert_eq!(moved.segments.last(), Some(&expected));
    }

    #[test]
    fn transforming_leaves_no_arcs_behind() {
        // A rotation turns the circle an arc describes into an ellipse, which
        // has to leave as cubics for the rotation to mean anything.
        let path = ellipse_path(0.0, 0.0, 10.0, 10.0);
        let rotated = transform(&path, &xs_math::Matrix3::rotation(0.5));
        assert!(
            !rotated
                .segments
                .iter()
                .any(|segment| matches!(segment, Segment::ArcTo { .. }))
        );
        assert!(is_drawable(&rotated));
    }
}
