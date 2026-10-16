//! Vector shapes, and what the GPU is doing with them.
//!
//! The panels go in rough order of what each one is for:
//!
//! 1. **Widths.** The same circle stroked at four widths. All four send the same
//!    twelve floats per segment, because the pen is a number in the vertex and the
//!    ribbon around the line is built on the GPU - a stroke costs the same at any
//!    thickness. The edges come out clean because the antialiasing is a distance
//!    field rather than smoothed geometry, so a hairline is as good as a brush.
//! 2. **Caps and joins.** Round joins are what you get by *not* implementing joins:
//!    each segment is a capsule, and two capsules meeting at a point cover exactly
//!    the disc that rounds that corner off.
//! 3. **Dashes.** The only part of a stroke decided on the CPU, because where a gap
//!    starts is a question about arc length along the whole contour rather than
//!    about any one pixel.
//! 4. **Holes.** A donut is two contours, not a boolean operation: the inner one is
//!    wound the other way and the nonzero fill rule does the rest.
//! 5. **A path from SVG data** - the same `Path` the helpers build.
//! 6. **Transforms**, and the flattening tolerance underneath all of it.

use xs::{AlignItems, Application, Context, Div, Easing, Path, Render, Shape, Vector2, prelude::*};

/// How tall a row of cards is: the card's own padding, a line of text, the gap and
/// the vector.
///
/// Stated rather than left to the layout because a row with no height of its own
/// fills the cross axis, and four full-height rows push the last one off the screen.
const ROW: f32 = 176.;

struct VectorDemo;

impl Render for VectorDemo {
    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        div()
            .testid("root")
            .width(ctx.app_context.screen.width as _)
            .column()
            .gap(16.)
            .padding(24.)
            .background_color(Color::rgba(0.08, 0.09, 0.11, 1.0))
            .font_size(26)
            .child(
                div()
                    .row()
                    .gap(16.)
                    .height(ROW)
                    .child(panel("widths", widths()))
                    .child(panel("caps", caps())),
            )
            .child(
                div()
                    .row()
                    .gap(16.)
                    .height(ROW)
                    .child(panel("joins", joins()))
                    .child(dashes()),
            )
            .child(
                div()
                    .row()
                    .gap(16.)
                    .height(ROW)
                    .child(panel("holes", holes()))
                    .child(from_svg()),
            )
            .child(div().row().gap(16.).child(rotating()))
    }
}

/// A titled card around one vector.
fn panel(title: &str, shapes: Vec<Shape>) -> Div {
    div()
        .testid("panel")
        .column()
        .gap(8.)
        .align_items(AlignItems::Center)
        .padding(12.)
        .width(CARD)
        .height(ROW)
        .border_radius_all(8.)
        .background_color(Color::rgba(0.14, 0.15, 0.18, 1.0))
        .color(Color::WHITE)
        .child(text(title))
        .child(vector().shapes(shapes).width(200.).height(120.))
}

/// How wide a card is: the vector plus its padding either side.
const CARD: f32 = 224.;

/// Four pens on four circles. The message is that they cost the same.
fn widths() -> Vec<Shape> {
    vec![
        shape::circle(36.0, 60.0, 10.0).stroke(Color::WHITE, 1.0),
        shape::circle(70.0, 60.0, 20.0).stroke(Color::WHITE, 2.0),
        shape::circle(112.0, 60.0, 28.0).stroke(Color::WHITE, 6.0),
        shape::circle(162.0, 60.0, 34.0).stroke(Color::WHITE, 14.0),
    ]
}

/// The three cap kinds, which are the only difference between them.
fn caps() -> Vec<Shape> {
    [xs::LineCap::Butt, xs::LineCap::Round, xs::LineCap::Square]
        .into_iter()
        .enumerate()
        .map(|(index, cap)| {
            let y = 30.0 + index as f32 * 30.0;
            shape::line(20.0, y, 180.0, y)
                .stroke(Color::WHITE, 14.0)
                .cap(cap)
        })
        .collect()
}

/// A zigzag whose every corner is a round join, for free, plus a closed circle.
fn joins() -> Vec<Shape> {
    let zigzag = Shape::new(Path::build(|p| {
        p.move_to(20.0, 90.0)
            .line_to(55.0, 25.0)
            .line_to(90.0, 90.0)
            .line_to(125.0, 25.0)
            .line_to(160.0, 90.0);
    }))
    .stroke(Color::WHITE, 18.0)
    .cap(xs::LineCap::Butt);

    vec![
        zigzag,
        shape::circle(100.0, 30.0, 20.0).stroke(Color::rgba(0.4, 0.8, 1.0, 1.0), 10.0),
    ]
}

/// Two holes in one shape, which is two contours and nothing else.
fn holes() -> Vec<Shape> {
    vec![
        shape::circle(55.0, 60.0, 38.0)
            .fill(Color::rgba(0.3, 0.6, 1.0, 1.0))
            .with_hole(shape::circle(55.0, 60.0, 20.0).path),
        shape::rect(115.0, 20.0, 65.0, 80.0)
            .fill(Color::rgba(0.9, 0.5, 0.2, 1.0))
            .with_hole(shape::circle(132.0, 44.0, 10.0).path)
            .with_hole(shape::circle(163.0, 76.0, 10.0).path),
    ]
}

/// A dash pattern walking along its own line.
///
/// This is animated rather than drawn, because the offset is the one thing about a
/// stroke that is not already on the GPU.
fn dashes() -> Div {
    div()
        .testid("panel")
        .column()
        .gap(8.)
        .align_items(AlignItems::Center)
        .padding(12.)
        .width(CARD)
        .height(ROW)
        .border_radius_all(8.)
        .background_color(Color::rgba(0.14, 0.15, 0.18, 1.0))
        .color(Color::WHITE)
        .child(text("dashes"))
        .child(
            vector()
                .shape(
                    shape::line(20.0, 60.0, 180.0, 60.0)
                        .stroke(Color::WHITE, 12.0)
                        .cap(xs::LineCap::Butt)
                        .dash([14.0, 10.0]),
                )
                .width(200.)
                .height(120.)
                .with_animation(
                    "dash-walk",
                    Easing::new().linear().repeat(u32::MAX),
                    |this, value| this.dash_offset(value * 48.0),
                ),
        )
}

/// A real icon's path data, authored at 24x24 and drawn at 96.
///
/// That is the view box's whole job: the path is untouched, and the node maps it
/// onto whatever space it is given.
fn from_svg() -> Div {
    div()
        .testid("panel")
        .column()
        .gap(8.)
        .align_items(AlignItems::Center)
        .padding(12.)
        .width(CARD)
        .height(ROW)
        .border_radius_all(8.)
        .background_color(Color::rgba(0.14, 0.15, 0.18, 1.0))
        .color(Color::WHITE)
        .child(text("svg path data"))
        .child(
            vector()
                // The icon's own viewBox, from Lucide - which is the point of
                // having one: the path only reaches 2..22, so fitting its bounds
                // instead would zoom past the 2 unit margin the drawing intends.
                .view_box(Vector2::zero(), 24., 24.)
                .width(96.)
                .height(96.)
                .path(
                    "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 \
                     18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8z",
                    Color::rgba(0.4, 0.85, 0.6, 1.0),
                ),
        )
}

/// A rotating star, to show that a transform carries the geometry - and the pen.
fn rotating() -> Div {
    div()
        .testid("panel")
        .column()
        .gap(8.)
        .align_items(AlignItems::Center)
        .padding(12.)
        .width(CARD)
        .height(ROW)
        .border_radius_all(8.)
        .background_color(Color::rgba(0.14, 0.15, 0.18, 1.0))
        .color(Color::WHITE)
        .child(text("rotated"))
        .child(
            vector()
                .shape(
                    shape::star(6, 100.0, 60.0, 46.0, 20.0)
                        .fill(Color::rgba(0.95, 0.4, 0.55, 1.0))
                        .stroke(Color::WHITE, 4.0),
                )
                .width(200.)
                .height(120.)
                .with_animation(
                    "spin",
                    Easing::new().linear().repeat(u32::MAX),
                    |this, value| this.rotate(value),
                ),
        )
}

fn main() {
    Application::new()
        .id("vector")
        .title("Vector")
        .run(|_ctx| VectorDemo);
}
