//! End to end parses of documents shaped like the ones this crate exists for:
//! an icon set, a sprite sheet that uses `defs` and `use`, and a chart with a
//! view box and stroke styling.

use xs_layout::Color;
use xs_layout::STROKE_VERTEX_FLOATS;
use xs_math::Vector2;
use xs_svg::{SvgError, parse, parse_vector};
use xs_vector::Segment;

/// A 24 unit icon in the shape every icon set is exported in: a `viewBox`, a
/// `fill="none"` root so the strokes show, and one path.
const ICON: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"
     stroke="currentColor" stroke-width="2" stroke-linecap="round">
  <title>A check</title>
  <path d="M20 6 L9 17 L4 12"/>
</svg>"##;

#[test]
fn an_icon_comes_out_as_its_one_stroked_path() {
    let image = parse(ICON).expect("the icon parses");
    let expected = 1;
    assert_eq!(image.shapes.len(), expected);

    let shape = &image.shapes[0];
    let expected = None;
    assert_eq!(shape.fill, expected, "the root says fill: none");

    let stroke = shape.stroke.as_ref().expect("it strokes");
    assert_eq!(stroke.width, 2.0);
    assert_eq!(
        stroke.color,
        Color::BLACK,
        "currentColor with no color of its own is black"
    );
}

#[test]
fn an_icon_takes_its_size_from_its_view_box() {
    let image = parse(ICON).expect("the icon parses");
    let expected = Vector2::new(24.0, 24.0);
    assert_eq!(image.size(), Some(expected));
    assert_eq!(image.view_box.unwrap().width, 24.0);
}

#[test]
fn an_icon_reaches_a_vector_ready_to_draw() {
    let vector = parse_vector(ICON).expect("the icon parses");
    assert_eq!(vector.shapes.len(), 1);
    assert!(vector.view_box.is_some());
    assert_eq!(vector.style.width, Some(24.0));

    // The geometry is untouched: the view box is the node's to apply.
    let expected = Some(&Segment::MoveTo(Vector2::new(20.0, 6.0)));
    assert_eq!(vector.shapes[0].path.segments.first(), expected);
}

#[test]
fn the_view_box_is_what_scales_the_icon_at_draw_time() {
    let vector = parse_vector(ICON).expect("the icon parses");
    // 96px is four times the 24 unit view box, so the geometry the node uploads
    // is in its own pixels rather than in the document's units.
    let geometry = vector.geometry(Vector2::new(96.0, 96.0));
    assert!(
        geometry.stroke_vertices.len() >= STROKE_VERTEX_FLOATS,
        "there is a stroke to draw"
    );
    assert!(geometry.bounds.width > 24.0, "it was scaled up by four");
}

#[test]
fn a_sprite_sheet_draws_each_use_where_it_points() {
    let sprite = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 16">
  <defs>
    <rect id="tile" width="10" height="10"/>
    <g id="pair"><rect width="4" height="4"/><rect x="6" width="4" height="4"/></g>
  </defs>
  <use href="#tile" x="2" y="4" fill="red"/>
  <use href="#pair" x="20" fill="blue"/>
</svg>"##;

    let image = parse(sprite).expect("the sprite parses");
    let expected = 3;
    assert_eq!(
        image.shapes.len(),
        expected,
        "one from the use, two from the pair"
    );

    let first = &image.shapes[0];
    assert_eq!(
        first.path.segments.first(),
        Some(&Segment::MoveTo(Vector2::new(2.0, 4.0)))
    );
    assert_eq!(first.fill.as_ref().expect("it fills").color, Color::RED);

    let last = &image.shapes[2];
    assert_eq!(
        last.path.segments.first(),
        Some(&Segment::MoveTo(Vector2::new(26.0, 0.0))),
        "the pair's own second rect, moved by the use"
    );
    assert_eq!(last.fill.as_ref().expect("it fills").color, Color::BLUE);
}

#[test]
fn a_use_paints_the_thing_it_points_at() {
    let sprite = r##"<svg viewBox="0 0 10 10">
  <defs><rect id="plain" width="4" height="4"/></defs>
  <use href="#plain" fill="orange"/>
</svg>"##;

    let image = parse(sprite).expect("the sprite parses");
    // The keyword's own 8 bit value: 0xFFA500 is not `Color::ORANGE`'s 0.63.
    let expected = Color::rgba(1.0, f32::from(0xA5_u8) / 255.0, 0.0, 1.0);
    assert_eq!(
        image.shapes[0].fill.as_ref().expect("it fills").color,
        expected
    );
}

#[test]
fn a_reference_with_its_own_paint_keeps_it() {
    // The `use` copies the reference into itself, so the reference's own
    // presentation attribute is a declaration and the `use` only supplies what
    // the reference leaves unsaid. A browser draws this green too.
    let sprite = r##"<svg viewBox="0 0 10 10">
  <defs><rect id="painted" width="4" height="4" fill="green"/></defs>
  <use href="#painted" fill="orange"/>
</svg>"##;

    let image = parse(sprite).expect("the sprite parses");
    let green = Color::rgba(0.0, f32::from(128_u8) / 255.0, 0.0, 1.0);
    assert_eq!(
        image.shapes[0].fill.as_ref().expect("it fills").color,
        green
    );
}

#[test]
fn a_chart_reads_its_strokes_and_its_view_box() {
    let chart = r##"<svg viewBox="0 0 200 100" width="400" height="200">
  <g stroke="hsl(210, 80%, 45%)" stroke-width="3" fill="none"
     stroke-linecap="round">
    <path d="M10 90 L50 40 L90 60 L130 10 L190 30"/>
    <path d="M10 95 L190 95" stroke-dasharray="4 4"/>
  </g>
</svg>"##;

    let image = parse(chart).expect("the chart parses");
    let expected = 2;
    assert_eq!(image.shapes.len(), expected);
    assert_eq!(image.size(), Some(Vector2::new(400.0, 200.0)));

    // Every path in the group is stroked the same colour and width.
    for shape in &image.shapes {
        let stroke = shape.stroke.as_ref().expect("it strokes");
        assert_eq!(stroke.width, 3.0);
        assert_eq!(shape.fill, None);
    }

    let dashed = image.shapes[1].stroke.as_ref().expect("it strokes");
    assert_eq!(dashed.dash, vec![4.0, 4.0]);
    let solid = image.shapes[0].stroke.as_ref().expect("it strokes");
    assert!(solid.dash.is_empty());
}

#[test]
fn a_nested_transform_reaches_the_geometry() {
    let nested = r##"<svg viewBox="0 0 100 100">
  <g transform="translate(10 10)">
    <g transform="scale(2)">
      <rect width="10" height="10" stroke="black" stroke-width="1"/>
    </g>
  </g>
</svg>"##;

    let image = parse(nested).expect("the document parses");
    let shape = &image.shapes[0];
    assert_eq!(
        shape.path.segments.first(),
        Some(&Segment::MoveTo(Vector2::new(10.0, 10.0)))
    );
    let far_corner = shape.path.segments[2];
    assert_eq!(far_corner, Segment::LineTo(Vector2::new(30.0, 30.0)));

    // The pen scales with the geometry it belongs to, or the stroke comes out
    // half as thick as the file asked for.
    let stroke = shape.stroke.as_ref().expect("it strokes");
    assert_eq!(stroke.width, 2.0);
}

#[test]
fn a_shape_sprite_of_primitives_needs_no_path_data() {
    let shapes = r##"<svg viewBox="0 0 100 40">
  <rect x="0" y="0" width="20" height="20" rx="4"/>
  <circle cx="30" cy="10" r="10"/>
  <ellipse cx="50" cy="10" rx="10" ry="6"/>
  <line x1="60" y1="4" x2="60" y2="16"/>
  <polyline points="70,16 75,4 80,16"/>
  <polygon points="90,4 98,16 82,16"/>
</svg>"##;

    let image = parse(shapes).expect("the document parses");
    let expected = 6;
    assert_eq!(image.shapes.len(), expected);
    for (index, shape) in image.shapes.iter().enumerate() {
        assert!(
            shape.path.segments.iter().any(Segment::is_drawing),
            "shape {index} has no geometry"
        );
    }
}

#[test]
fn the_features_this_parser_skips_do_not_stop_a_file() {
    // The kind of file an icon set ships: gradients, a clip path, CSS, metadata
    // and a `symbol`, none of which the parser reads, and all of which have to
    // be survivable.
    let decorated = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <style>.accent { fill: red; }</style>
  <defs>
    <linearGradient id="g"><stop offset="0" stop-color="red"/></linearGradient>
    <clipPath id="c"><rect width="24" height="24"/></clipPath>
    <symbol id="mark"><circle r="4"/></symbol>
  </defs>
  <metadata>generated</metadata>
  <rect class="accent" width="24" height="24" fill="url(#g)" clip-path="url(#c)"/>
</svg>"##;

    let image = parse(decorated).expect("the file still parses");
    let expected = 1;
    assert_eq!(image.shapes.len(), expected);

    // The `url(#g)` resolves, so the shape carries that gradient - and the flat
    // colour is the top of its ramp, which is what a shape that is later stroked
    // or flattened reads as.
    let fill = image.shapes[0].fill.as_ref().expect("it fills");
    assert_eq!(fill.color, Color::RED, "the gradient's first stop");
    assert!(
        fill.gradient.is_some(),
        "and it carries the gradient itself"
    );
}

#[test]
fn a_gradient_reference_to_nothing_falls_back_to_the_inherited_fill() {
    let dangling = r##"<svg viewBox="0 0 10 10">
  <g fill="blue"><rect width="4" height="4" fill="url(#not-here)"/></g>
</svg>"##;

    let image = parse(dangling).expect("it parses");
    let fill = image.shapes[0].fill.as_ref().expect("it fills");
    assert_eq!(fill.color, Color::BLUE, "what the group said");
    assert_eq!(
        fill.gradient, None,
        "and no gradient, because there was none"
    );
}

#[test]
fn a_gradient_runs_from_its_first_stop_to_its_last() {
    let document = r##"<svg viewBox="0 0 10 10">
  <defs>
    <linearGradient id="g" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#ff0000"/>
      <stop offset="100%" stop-color="#0000ff"/>
    </linearGradient>
  </defs>
  <rect width="10" height="10" fill="url(#g)"/>
</svg>"##;

    let image = parse(document).expect("it parses");
    let gradient = image.shapes[0]
        .fill
        .as_ref()
        .expect("it fills")
        .gradient
        .expect("with a gradient");
    assert_eq!(gradient.start, Color::RED);
    assert_eq!(gradient.end, Color::BLUE);
    let expected = std::f32::consts::FRAC_PI_2;
    assert!((gradient.angle - expected).abs() < 1e-6, "down the shape");
}

#[test]
fn a_malformed_value_is_an_error_that_names_the_attribute() {
    let bad = "<svg><rect width=\"24\" fill=\"rebeccapurples\"/></svg>";
    let error = parse(bad).expect_err("the colour is not one");
    let expected = "'rebeccapurples' is not a valid paint";
    assert_eq!(error.to_string(), expected);

    let bad = "<svg><rect width=\"24\" height=\"24\" transform=\"spin(3)\"/></svg>";
    let error = parse(bad).expect_err("there is no spin transform");
    assert_eq!(error.to_string(), "'spin(3)' is not a valid transform");

    let bad = "<svg><path d=\"M0 0 Q\"/></svg>";
    let error = parse(bad).expect_err("the path data stops early");
    assert!(matches!(error, SvgError::Path(_)));
}

#[test]
fn something_that_is_not_xml_at_all_says_so() {
    let error = parse("not xml at all").expect_err("there is no root element");
    assert!(matches!(error, SvgError::Xml(_)));
    assert!(error.to_string().contains("not valid xml"));
}

#[test]
fn an_svg_with_a_default_namespace_prefix_reads_the_same() {
    let plain =
        parse("<svg viewBox=\"0 0 10 10\"><rect width=\"4\" height=\"4\"/></svg>").expect("parses");
    let prefixed = parse(
        "<svg:svg xmlns:svg=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 10 10\">\
         <svg:rect width=\"4\" height=\"4\"/></svg:svg>",
    )
    .expect("parses");
    assert_eq!(plain.shapes, prefixed.shapes);
    assert_eq!(plain.view_box, prefixed.view_box);
}
