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
//! 7. **A whole SVG document** - `ressources/images/tiger.svg`, which is what the
//!    parser is for: somebody else's 305 paths, read into geometry.

use xs::{
    AlignItems, Application, Context, Div, Easing, Gradient, JustifyContent, LineCap,
    NavigationNode, Path, Render, Shape, Spring, State, Svg, Vector, Vector2, prelude::*,
};

/// How tall a row of cards is: the card's own padding, a line of text, the gap and
/// the vector.
///
/// Stated rather than left to the layout because a row with no height of its own
/// fills the cross axis, and three full-height rows push the last one off the screen.

struct VectorDemo;

impl Render for VectorDemo {
    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        let screen = ctx.app_context.screen;
        let shown = ctx.use_state(|_| View::Panels);
        let on_screen = shown.get();
        // The two slides are the navigation. A horizontal node with the panels
        // first and the tiger second is all it takes: Left and Right are the
        // framework's own sibling navigation, so there is no key handling here to
        // get wrong, and a slide that is navigated to takes focus - which is what
        // puts it on screen.
        let _slides = ctx.focus_handle(NavigationNode::horizontal().id("slides"));

        // One spring drives everything that moves, so the two slides and the
        // arrow travel as one thing rather than as three animations kept in step.
        // It is asked for its target rather than retriggered, so pressing a key
        // halfway through a slide retargets from wherever it had got to.
        //
        // Measured by integrating the same physics the crate does - 90% of the
        // travel in 183ms, settled at 917ms, 5.4% past the target. Half the
        // stiffness of the one it replaced, which is most of the difference
        // between a slide and a lurch: the overshoot is what says spring.
        let t = ctx.animate(
            Spring::new().stiffness(160.).damping(16.),
            if on_screen == View::Tiger { 1. } else { 0. },
        );

        // Every child of the root needs a height of its own: one without it fills
        // the space the root offers, and the flex pass then grows the root past the
        // screen to fit both of them. The 24 either side of what is left is the
        // tiger card's own padding, which sits outside its content box.
        let body = screen.height as f32 - PADDING * 2.;
        let tiger_height = body - TITLE - GAP - 24.;
        let width = screen.width as f32 - PADDING * 2.;
        // What the slides get once the arrow has taken its end of the row.
        let content = width - ARROW - GAP;
        // How far a slide travels to leave: a full screen and the gap between.
        let travel = width + GAP;

        // Both slides are in the tree for the whole transition, so the tiger is
        // laid out *below* the panels and brought up into place rather than the
        // two being overlapped: this flex pass places siblings one after another
        // and has no way to overlap them. Stated height and gap keep the column
        // from growing to fit both.
        let slides = div()
            .column()
            .width(content)
            .height(body)
            .gap(0.)
            // Both slides are centred in what is left of the row.
            .align_items(AlignItems::Center)
            .child(Slide {
                view: View::Panels,
                title: "vector primitives",
                content: panels(body - TITLE - GAP),
                offset: Vector2::new(-t * travel, 0.),
                shown: shown.clone(),
            })
            .child(Slide {
                view: View::Tiger,
                title: "tiger.svg - 305 paths, 76 KB",
                content: div()
                    .row()
                    .width(content)
                    .height(body - TITLE - GAP)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    // The tiger is portrait and the screen is not, so it is
                    // centred in what is left rather than stretched across it.
                    .child(tiger(tiger_height * TIGER.aspect(), tiger_height)),
                offset: Vector2::new((1. - t) * travel, -body),
                shown: shown.clone(),
            });

        div()
            .testid("root")
            .width(screen.width as _)
            .column()
            .gap(GAP)
            .padding(PADDING)
            .background_color(Color::rgba(0.08, 0.09, 0.11, 1.0))
            .child(
                div()
                    .row()
                    .gap(GAP)
                    .width(width)
                    .height(body)
                    .align_items(AlignItems::Center)
                    .child(slides)
                    // The arrow crosses the screen as the slides go, from the end
                    // the panels leave free to the end the tiger arrives at.
                    .child(arrow(on_screen == View::Panels).translate_xy(-t * width, 0.)),
            )
    }
}

/// The arrow: which way a key goes, and nothing else.
///
/// Not focusable: the slide titles already say where you are, so the state of the
/// example is on screen twice over without it. The disc is drawn by the vector
/// rather than being the div's background colour, which is what lets the chevron
/// and the circle be one node on one box - no second thing to align, and no way
/// for the two to drift apart.
fn arrow(towards_right: bool) -> Div {
    div()
        .testid("arrow")
        .width(ARROW)
        .height(ARROW)
        .border_radius_all(ARROW / 2.)
        .child(
            vector()
                .shape(
                    shape::circle(DISC_CENTRE, DISC_CENTRE, DISC_RADIUS)
                        .fill(Color::rgba(0.14, 0.15, 0.18, 1.0)),
                )
                .shape(
                    Shape::new(Path::build(|path| {
                        let (first, rest) = CHEVRON.split_first().expect("three points");
                        path.move_to(first.0, first.1);
                        for point in rest {
                            path.line_to(point.0, point.1);
                        }
                    }))
                    .stroke(Color::WHITE, 12.)
                    // A round cap is the clamped projection, and the corner between
                    // two segments is a disc for free: the same two facts the caps
                    // and joins panels are demonstrating, at icon size.
                    .cap(LineCap::Round),
                )
                // The view box frames the disc rather than the chevron, which is
                // why the chevron ends up centred in the card instead of filling it.
                .view_box(Vector2::new(2., 2.), 96., 96.)
                .width(ARROW)
                .height(ARROW)
                // One shape pointing right, mirrored for the other direction: a
                // negative scale about the node's own centre, which is CSS's
                // transform-origin and what `scale_xy` already does.
                .scale_xy(if towards_right { 1. } else { -1. }, 1.),
        )
}

/// One slide: a title across the top and the slide's own content under it.
///
/// It is an element rather than a `Div` because taking focus is what makes it the
/// slide on screen, and focus belongs to the framework's navigation tree rather
/// than to a div. The title travels with its slide, so it arrives with the thing
/// it names instead of staying put while the artwork slides under it.
struct Slide {
    /// Which view this is: what taking focus sets the shared state to.
    view: View,
    title: &'static str,
    /// The slide's own content, already sized by the caller.
    content: Div,
    /// Where it is in the transition.
    offset: Vector2,
    /// The state the two slides share, which is the one on screen.
    shown: State<View>,
}

impl Render for Slide {
    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        let mut handle = ctx.focus_handle(NavigationNode::leaf());
        let shown = self.shown.clone();
        let view = self.view;
        handle.on_focus(move || shown.set(view));

        // The focus indicator: the slide you are on has its title at full
        // strength and the other one does not. Dimming the title rather than
        // outlining the slide, because a title is what the two slides have in
        // common and the ring would be the only thing on screen moving.
        let title_colour = if handle.has_focus() {
            Color::WHITE
        } else {
            Color::rgba(1., 1., 1., 0.45)
        };

        div()
            .column()
            // No width and no height: the column of slides offers exactly the
            // space a slide is allowed to take, and it is the same for both.
            .gap(GAP)
            .align_items(AlignItems::Center)
            .child(
                div()
                    .height(TITLE)
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::Center)
                    .font_size(TITLE_FONT)
                    .color(title_colour)
                    .child(text(self.title)),
            )
            .child(self.content.clone())
            .translate_xy(self.offset.x, self.offset.y)
    }
}
/// The linear gradient, at four angles.
///
/// The one thing the quad shader does that the vector pipeline does not: a
/// gradient puts a start colour, an end colour and an angle in the vertex, and
/// the fragment projects the box's four corners onto that direction and mixes
/// between the two ends along it. Zero radians runs across the box and a quarter
/// turn runs down it, which is why the same four swatches below are the same
/// gradient rotated rather than four different gradients.
fn gradients(height: f32) -> Div {
    let quarter = std::f32::consts::FRAC_PI_2;
    let swatch = |angle: f32| {
        div()
            .width(44.)
            .height(118.)
            .border_radius_all(8.)
            .background(Gradient::angled(
                Color::rgba(0.35, 0.55, 1.0, 1.0),
                Color::rgba(0.95, 0.35, 0.65, 1.0),
                angle,
            ))
    };
    shell(CARD, height).child(text("linear gradient")).child(
        div()
            .row()
            .gap(8.)
            .child(swatch(0.))
            .child(swatch(quarter / 2.))
            .child(swatch(quarter))
            .child(swatch(3. * quarter / 2.)),
    )
}

/// How wide the panels are: three cards and the two gaps between them.
const PANELS: f32 = CARD * 3. + GAP * 2.;

/// The root's padding, and the gap between its children.
const PADDING: f32 = 24.;
const GAP: f32 = 16.;

/// The disc, and so the card's height: the card has no padding, because the disc
/// is the card.
const ARROW: f32 = 72.;

/// A slide's title: how much height it takes off the slide, and how big the text
/// in it is. Stated rather than left to the text, because the slide's content has
/// to be told how much room is left over.
const TITLE: f32 = 68.;
const TITLE_FONT: u16 = 48;

/// Which of the two things the example has to show is on screen.
#[derive(Clone, Copy, PartialEq)]
enum View {
    /// The nine cards: what each panel is for.
    Panels,
    /// The whole document.
    Tiger,
}

/// The cards, three by three: what each panel is for.
///
/// What each card draws inside it stays its own size, so a taller card is air
/// above and below the drawing - the shell centres it - rather than a bigger
/// drawing.
///
/// What each card draws inside it stays its own size, so a taller row is air
/// above and below the drawing rather than a bigger drawing.
fn panels(height: f32) -> Div {
    // A third of the slide each, so the grid is the slide's height on any window
    // rather than a fixed size that fits one of them.
    let tall = (height - GAP * 2.) / 3.;
    let row = |cards: [Div; 3]| {
        let mut row = div().row().gap(GAP).height(tall);
        for tall in cards {
            row = row.child(tall);
        }
        row
    };
    div()
        .column()
        .width(PANELS)
        .height(height)
        .gap(GAP)
        .child(row([
            panel("widths", widths(), tall),
            panel("caps", caps(), tall),
            panel("joins", joins(), tall),
        ]))
        .child(row([
            panel("holes", holes(), tall),
            dashes(tall),
            from_svg(tall),
        ]))
        .child(row([
            rotating(tall),
            gradients(tall),
            card("xs-ui.svg", logo_vector(200., 120.), tall),
        ]))
}

/// The box every panel is: dark, rounded, and holding what is inside it in the
/// middle.
///
/// No padding: `align_items` centres across the card and `justify_content` down
/// it, so the slack a padding would have taken is the padding. Stating the
/// padding as well used to put every card's contents 12 pixels left of where the
/// card's own box is, because the two insets are applied in different places.
fn shell(width: f32, height: f32) -> Div {
    div()
        .testid("panel")
        .column()
        .gap(8.)
        .width(width)
        .height(height)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .font_size(CARD_TITLE)
        .color(Color::WHITE)
        .border_radius_all(8.)
        .background_color(Color::rgba(0.14, 0.15, 0.18, 1.0))
}

/// A titled card around one vector.
fn card(title: &str, ink: Vector, height: f32) -> Div {
    shell(CARD, height).child(text(title)).child(ink)
}

/// A titled card around a list of shapes, which is the panel's usual case.
fn panel(title: &str, shapes: Vec<Shape>, height: f32) -> Div {
    card(
        title,
        vector().shapes(shapes).width(200.).height(120.),
        height,
    )
}

/// How wide a card is.
const CARD: f32 = 224.;

/// The panels' own title size, which is the one piece of type they share.
const CARD_TITLE: u16 = 22;

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
fn dashes(height: f32) -> Div {
    shell(CARD, height).child(text("dashes")).child(
        vector()
            .shape(
                shape::line(20.0, 60.0, 180.0, 60.0)
                    .stroke(Color::WHITE, 12.0)
                    .cap(LineCap::Butt)
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
fn from_svg(height: f32) -> Div {
    shell(CARD, height).child(text("svg path data")).child(
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
fn rotating(height: f32) -> Div {
    shell(CARD, height).child(text("rotated")).child(
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

/// The chevron's three points, in the 100 unit space the disc is drawn in too.
///
/// A builder rather than path data, since this one is authored here rather than
/// arriving in a file. A chevron and not the tail-and-head arrow it replaced: at
/// 72 pixels that tail is a stub either side of the head, and the whole thing
/// reads as a smudge rather than as a direction.
const CHEVRON: [(f32, f32); 3] = [(36., 30.), (64., 50.), (36., 70.)];

/// Where the disc is in that space, and how big it is.
///
/// The chevron's ink runs 30..70 by 24..76, the stroke width included, so a radius
/// of 44 leaves 24 units of clear space across and 18 down - which is the way round
/// it should be, a chevron being taller than it is wide.
const DISC_CENTRE: f32 = 50.;
const DISC_RADIUS: f32 = 44.;
/// A whole SVG document, rather than path data this file wrote itself.
///
/// `tiger.svg` is the Ghostscript tiger in the form icon sets are drawn in: a
/// `viewBox`, three nested groups whose transforms include a `matrix()` with a
/// negative d, and 305 paths whose paint arrives in `style` attributes rather
/// than presentation ones. The parser reads all of it and hands back geometry -
/// the node below is the same [`Vector`] the panels build by hand, at a size
/// worth looking at.
///
/// No title of its own: the slide above it has one, which is the point of the
/// slide having a title.
fn tiger(width: f32, height: f32) -> Div {
    shell(width, height).child(tiger_vector(width, height))
}

/// The tiger: 305 paths of somebody else's drawing, and the heaviest thing here.
///
/// Compiled in like every other asset here - shaders and fonts are `include_str!`
/// and nothing in this workspace is fetched at runtime. `Svg` parses it the first
/// time something asks for it and never again, which is the whole reason it is
/// there: the mesh is rebuilt every frame and the parse must not be.
///
/// The tolerance is coarser than the default, and worth measuring rather than
/// guessing. Its own strokes come out about a third of a pixel wide at panel size,
/// so subdividing them below a pixel buys nothing visible - and a node rebuilds
/// its mesh every frame, so this is a frame cost and not a one-off. Medians over
/// 30 runs of the whole 305 paths, in a dev build and a 1253x944 box: 24ms at one
/// pixel, 12.5ms at two, 7.3ms at four.
static TIGER: Svg = Svg::new(include_str!("../../../ressources/images/tiger.svg"));

/// This project's own mark: three paths and a gradient.
static LOGO: Svg = Svg::new(include_str!("../../../ressources/images/xs-ui.svg"));

/// The tiger as a [`Vector`], at the tolerance its own strokes need.
fn tiger_vector(width: f32, height: f32) -> Vector {
    TIGER.vector(width, height).tolerance(2.)
}

/// The project's mark, in its own paint.
///
/// Its paint is a linear gradient - every path says `fill: url(#gradient)` - which
/// reaches here as two colours on a direction rather than as one colour, because a
/// fill's vertices are coloured one by one and a ramp between them is what a
/// gradient is.
///
/// No tint here: the logo's orange is the file's own `linearGradient`, and a tint
/// over it would replace both of its stops with the one colour it was given. The
/// card behind it is near-black because that is what the logo is drawn on.
fn logo_vector(width: f32, height: f32) -> Vector {
    LOGO.vector(width, height)
}

fn main() {
    Application::new()
        .id("vector")
        .title("Vector")
        .run(|_ctx| VectorDemo);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The logo is three paths and one gradient, and it is drawn on a near-black
    /// card - so a gradient that stopped resolving is not a slightly wrong
    /// picture, it is three black shapes against black.
    #[test]
    fn the_logo_carries_its_own_gradient() {
        let image = LOGO.image();
        assert_eq!(image.shapes.len(), 3, "three paths in the file");
        for shape in &image.shapes {
            let fill = shape.fill.as_ref().expect("every path fills");
            let gradient = fill
                .gradient
                .as_ref()
                .expect("every path is a url(#gradient)");
            assert_eq!(
                gradient.start,
                Color::rgba(249. / 255., 170. / 255., 101. / 255., 1.)
            );
            assert_eq!(
                gradient.end,
                Color::rgba(254. / 255., 72. / 255., 3. / 255., 1.)
            );
            let expected = std::f32::consts::FRAC_PI_2;
            assert!((gradient.angle - expected).abs() < 1e-6, "y1=0 to y2=1");
        }
    }

    /// `tint` is still there for what it is - the tiger card, whose shapes are one
    /// colour - and it drops the gradient rather than pushing the colour through
    /// both stops. Which is why the logo is not tinted: the tint would be the
    /// whole of what the paint is.
    #[test]
    fn a_tint_replaces_a_gradient_entirely() {
        let mut image = LOGO.image().clone();
        image.tint(Color::WHITE);
        for shape in &image.shapes {
            let fill = shape.fill.as_ref().expect("every path fills");
            assert_eq!(fill.color, Color::WHITE);
            assert_eq!(
                fill.gradient, None,
                "tint is one colour, so it is not a ramp"
            );
        }
    }
}
