//! A document, parsed once and then used as geometry on every frame.

use std::sync::OnceLock;

use xs_layout::Rectangle;
use xs_vector::Vector;

use crate::image::SvgImage;

/// An SVG document, parsed the first time it is asked for and not again.
///
/// Parsing is not something to do in a frame callback - the tiger here is 76 KB
/// of path data, and the framework rebuilds the element tree before every frame -
/// so this holds the result behind a [`OnceLock`] and the frames after the first
/// get a clone of geometry, which is the deep copy the framework makes of every
/// node anyway.
///
/// It is a `static` in the app rather than a local, because that is the only
/// thing whose address survives between frames:
///
/// ```
/// use xs_svg::Svg;
///
/// static ICON: Svg = Svg::new(
///     r#"<svg viewBox="0 0 10 10"><rect width="4" height="4"/></svg>"#,
/// );
///
/// // Every frame, from here on.
/// let vector = ICON.vector(64., 64.);
/// assert_eq!(vector.shapes.len(), 1);
/// ```
///
/// The node type cannot be what is cached, incidentally: [`Vector`] holds a
/// `Box<dyn Animate>` and so is not `Sync`, while [`SvgImage`] is plain geometry.
/// That is also why this lives in the parser rather than in `xs-vector` -
/// depending on the other way round would be a cycle.
pub struct Svg {
    source: &'static str,
    image: OnceLock<SvgImage>,
}

impl Svg {
    /// The document to parse. Nothing happens until it is asked for.
    #[must_use]
    pub const fn new(source: &'static str) -> Self {
        Self {
            source,
            image: OnceLock::new(),
        }
    }

    /// The parsed document.
    ///
    /// # Panics
    ///
    /// If the document does not parse. It is embedded in the binary and read once,
    /// so a malformed one is a bug in the app rather than bad input at runtime,
    /// and this fails loudly for the same reason [`Vector::path`] does: an icon
    /// that silently draws nothing is much harder to find than a stopped app on
    /// the first frame.
    #[must_use]
    pub fn image(&'static self) -> &'static SvgImage {
        self.image.get_or_init(|| match crate::parse(self.source) {
            Ok(image) => image,
            Err(error) => panic!("{} did not parse: {error}", self.name()),
        })
    }

    /// The document's own view box, or `None` when it has none.
    #[must_use]
    pub fn view_box(&'static self) -> Option<Rectangle> {
        self.image().view_box
    }

    /// How much wider the document is than it is tall, which is what sizes a card
    /// around it.
    ///
    /// From the document rather than a number in the app, so a different file
    /// needs no edit to stay the right shape.
    #[must_use]
    pub fn aspect(&'static self) -> f32 {
        self.view_box()
            .map_or(1.0, |view_box| view_box.width / view_box.height)
    }

    /// The document as a [`Vector`], sized to a box.
    ///
    /// The size given wins: a document states its own in millimetres or user
    /// units, which is not what the layout asked for.
    #[must_use]
    pub fn vector(&'static self, width: f32, height: f32) -> Vector {
        self.vector_with(width, height, |_| {})
    }

    /// [`Svg::vector`], with `adjust` given the parsed shapes first.
    ///
    /// For the document whose paint this renderer cannot reproduce - a gradient is
    /// a quad shader feature - where the shapes are wanted in another colour.
    #[must_use]
    pub fn vector_with(
        &'static self,
        width: f32,
        height: f32,
        adjust: impl FnOnce(&mut SvgImage),
    ) -> Vector {
        let mut image = self.image().clone();
        adjust(&mut image);
        image.apply_to(Vector::new()).width(width).height(height)
    }

    /// The file name, for the message in [`Svg::image`]: the tail of the path it
    /// came from.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.source
            .rsplit_once('/')
            .map_or(self.source, |(_, name)| name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ICON: &str =
        r##"<svg viewBox="0 0 20 10"><rect width="4" height="4" fill="#ff0000"/></svg>"##;
    static ICON_SVG: Svg = Svg::new(ICON);

    #[test]
    fn the_document_is_parsed_once_and_kept() {
        let first = ICON_SVG.image();
        let second = ICON_SVG.image();
        assert!(std::ptr::eq(first, second), "the second call re-parsed");
    }

    #[test]
    fn the_view_box_gives_the_aspect() {
        let expected = 2.0;
        assert_eq!(ICON_SVG.aspect(), expected);
        assert_eq!(ICON_SVG.view_box().map(|v| v.width), Some(20.0));
    }

    #[test]
    fn a_vector_comes_out_at_the_size_asked_for() {
        let vector = ICON_SVG.vector(64., 32.);
        assert_eq!(vector.shapes.len(), 1);
        assert_eq!(vector.style.width, Some(64.0));
        assert_eq!(vector.style.height, Some(32.0));
        assert_eq!(vector.view_box.map(|v| v.width), Some(20.0));
    }

    #[test]
    fn a_document_can_be_retinted_on_the_way_out() {
        let vector = ICON_SVG.vector_with(64., 32., |image| image.tint(xs_layout::Color::WHITE));
        let fill = vector.shapes[0].fill.as_ref().expect("it fills");
        assert_eq!(fill.color, xs_layout::Color::WHITE);
    }

    #[test]
    fn a_document_without_a_view_box_is_aspect_one() {
        static PLAIN: Svg = Svg::new("<svg><rect width=\"2\" height=\"2\"/></svg>");
        let expected = 1.0;
        assert_eq!(PLAIN.aspect(), expected);
    }

    #[test]
    fn the_name_is_the_tail_of_the_path() {
        static NAMED: Svg = Svg::new("../../ressources/images/xs-ui.svg");
        let expected = "xs-ui.svg";
        assert_eq!(NAMED.name(), expected);
        assert_eq!(Svg::new("plain.svg").name(), "plain.svg");
    }
}
