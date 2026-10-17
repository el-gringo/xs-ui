//! Paint: what a shape is drawn with, and where that comes from.
//!
//! An SVG attribute is a paint *declaration*, not a paint: `stroke="red"` on a
//! group says what its children inherit unless they say otherwise. So reading an
//! element gives a [`Declarations`] holding only what that element wrote, and
//! [`Declarations::resolve`] merges it over the [`Paint`] its parent resolved to.
//! Only `opacity` is exempt, and it is marked below.

use xs_layout::Color;
use xs_vector::{Fill, LineCap, LinearGradient, Shape, Stroke};
use xs_xml::Element;

use crate::color;
use crate::error::SvgResult;
use crate::gradient::Gradients;
use crate::values::{length, number, numbers};
use xs_vector::Path;

/// The paint a shape is actually drawn with.
///
/// Every value here is resolved: an element that says nothing has its parent's
/// paint, and an element whose parent said nothing has [`Paint::default`],
/// which is SVG's own set of initial values. That is why a document with no
/// paint attributes at all is filled black and stroked nothing - the defaults a
/// browser starts from, rather than ones invented to suit a renderer.
#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    /// The fill colour, or `None` for `fill="none"`.
    pub fill: Option<Color>,
    /// The gradient the fill runs along, when it is a `url(#id)` that resolved.
    pub fill_gradient: Option<LinearGradient>,
    /// What multiplies the fill's alpha.
    pub fill_opacity: f32,
    /// The stroke colour, or `None` for `stroke="none"`.
    pub stroke: Option<Color>,
    /// What multiplies the stroke's alpha.
    pub stroke_opacity: f32,
    /// The pen's full width, not its radius.
    pub stroke_width: f32,
    /// How a stroke ends.
    pub stroke_linecap: LineCap,
    /// Alternating on/off lengths, or `None` for a solid line.
    pub stroke_dasharray: Option<Vec<f32>>,
    /// What `currentColor` resolved to on this element or an ancestor.
    pub current_color: Option<Color>,
    /// The element's own alpha.
    ///
    /// **Not inherited**, which is why [`Declarations::resolve`] leaves it at
    /// the inherited value and the walker multiplies ancestors' opacities into
    /// its own factor instead. Folding opacity into each shape's colour is an
    /// approximation of group opacity, and it differs from the real thing where
    /// a translucent group's children overlap - see [`crate::SvgImage`].
    pub opacity: f32,
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            fill: Some(Color::BLACK),
            fill_gradient: None,
            fill_opacity: 1.0,
            stroke: None,
            stroke_opacity: 1.0,
            stroke_width: 1.0,
            stroke_linecap: LineCap::Round,
            stroke_dasharray: None,
            current_color: None,
            opacity: 1.0,
        }
    }
}

impl Paint {
    /// The shape this paints, at a pen width for a group `scale` sized.
    ///
    /// `scale` is there because an element's transform has already been applied
    /// to its geometry by the time it gets here, so a group scaled by two has to
    /// hand its pen a width of two to come out the same thickness on screen. A
    /// dash pattern is scaled with it for the same reason.
    ///
    /// `alpha` is every `opacity` above this element, product included.
    #[must_use]
    pub fn shape(&self, path: Path, scale: f32, alpha: f32) -> Shape {
        let mut shape = Shape::new(path);

        if let Some(color) = self.fill {
            let faded = fade(color, self.fill_opacity * alpha);
            shape.fill = Some(match &self.fill_gradient {
                // The gradient's own colours carry their stop opacities, and the
                // fill opacity above still multiplies into both of them - which is
                // what `fill-opacity` means over a gradient.
                Some(gradient) => Fill {
                    color: faded,
                    gradient: Some(LinearGradient::new(
                        fade(gradient.start, self.fill_opacity * alpha),
                        fade(gradient.end, self.fill_opacity * alpha),
                        gradient.angle,
                    )),
                },
                None => Fill::new(faded),
            });
        }
        if let Some(color) = self.stroke {
            let mut stroke = Stroke::new(
                fade(color, self.stroke_opacity * alpha),
                self.stroke_width * scale,
            );
            stroke.cap = self.stroke_linecap;
            if let Some(dash) = &self.stroke_dasharray {
                stroke.set_dash(dash.iter().map(|length| length * scale).collect::<Vec<_>>());
            }
            shape.stroke = Some(stroke);
        }
        shape
    }
}

/// What one element wrote about its paint.
///
/// A property that is absent is `None` and the parent's value stands. The two
/// colour fields are `Option<Option<..>>` because a paint has three states -
/// unsaid, `none`, and a colour - and two nested options is the cheapest way to
/// write three.
#[allow(clippy::option_option)]
pub(crate) struct Declarations<'a> {
    /// Raw, because `currentColor` in a fill cannot be resolved until this
    /// element's own `color` and its ancestors' have been.
    color: Option<&'a str>,
    fill: Option<Option<&'a str>>,
    stroke: Option<Option<&'a str>>,
    fill_opacity: Option<f32>,
    stroke_opacity: Option<f32>,
    stroke_width: Option<f32>,
    stroke_linecap: Option<LineCap>,
    stroke_dasharray: Option<Vec<f32>>,
    opacity: Option<f32>,
}

impl<'a> Declarations<'a> {
    pub(crate) fn of(element: &'a Element) -> SvgResult<Self> {
        let style = Style::of(element);
        Ok(Self {
            color: style.get("color"),
            fill: paint_of(&style, "fill"),
            stroke: paint_of(&style, "stroke"),
            fill_opacity: fraction_of(&style, "fill-opacity")?,
            stroke_opacity: fraction_of(&style, "stroke-opacity")?,
            stroke_width: match style.get("stroke-width") {
                Some(value) => Some(length(value, 1.0, "stroke-width")?.max(0.0)),
                None => None,
            },
            // A cap that is not one of the three names is treated as the default
            // rather than as an error: a cap is not worth failing a document over.
            stroke_linecap: style.get("stroke-linecap").map(|value| match value.trim() {
                "butt" => LineCap::Butt,
                "square" => LineCap::Square,
                _ => LineCap::Round,
            }),
            stroke_dasharray: match style.get("stroke-dasharray") {
                Some("none") | None => None,
                Some(value) => Some(numbers(value, "stroke-dasharray")?),
            },
            opacity: fraction_of(&style, "opacity")?,
        })
    }

    /// The paint an element with these declarations paints with, given its
    /// parent's.
    #[allow(clippy::option_option)]
    pub(crate) fn resolve(&self, parent: &Paint, gradients: &Gradients) -> SvgResult<Paint> {
        // `color` is inherited, and it has to be settled before any paint that
        // says `currentColor` can be.
        let current_color = match self.color {
            Some(value) => color::parse(value, parent.current_color)?,
            None => parent.current_color,
        };
        let resolve_paint =
            |declared: &Option<Option<&'a str>>, inherited: Option<Color>| match declared {
                Some(Some(value)) => color::parse(value, current_color),
                // `none` is a value, not an absence: it clears what it inherited.
                Some(None) => Ok(None),
                None => Ok(inherited),
            };
        // A `url(#id)` is a gradient if the document has one by that name, and
        // otherwise nothing this parser can draw - which is the same answer as a
        // paint it could not read, and leaves the shape with the fill it
        // inherits.
        let resolve_fill = |declared: &Option<Option<&'a str>>,
                            parent: Option<Color>,
                            parent_gradient: Option<LinearGradient>|
         -> SvgResult<(Option<Color>, Option<LinearGradient>)> {
            match declared {
                Some(Some(value)) => match gradients.get(value) {
                    Some(gradient) => {
                        // The start colour is also the flat fallback, so a shape
                        // that is later flattened or stroked reads as the top of
                        // the ramp rather than as black.
                        Ok((Some(gradient.start), Some(gradient)))
                    }
                    None if value.trim().starts_with("url(") => Ok((parent, parent_gradient)),
                    None => color::parse(value, current_color).map(|c| (c, None)),
                },
                Some(None) => Ok((None, None)),
                None => Ok((parent, parent_gradient)),
            }
        };
        let (fill, fill_gradient) = resolve_fill(&self.fill, parent.fill, parent.fill_gradient)?;

        Ok(Paint {
            fill,
            fill_gradient,
            fill_opacity: self.fill_opacity.unwrap_or(parent.fill_opacity),
            stroke: resolve_paint(&self.stroke, parent.stroke)?,
            stroke_opacity: self.stroke_opacity.unwrap_or(parent.stroke_opacity),
            stroke_width: self.stroke_width.unwrap_or(parent.stroke_width),
            stroke_linecap: self.stroke_linecap.unwrap_or(parent.stroke_linecap),
            stroke_dasharray: self
                .stroke_dasharray
                .clone()
                .or_else(|| parent.stroke_dasharray.clone()),
            current_color,
            // Not inherited: an element that writes no opacity has none, and
            // the walker keeps the running product of its ancestors' instead.
            opacity: self.opacity.unwrap_or(1.0),
        })
    }
}

/// One paint property, where `none` is a value and absence is not.
///
/// `url(#gradient)` is treated as absence rather than as a colour it cannot
/// read: the shape falls back to whatever it inherits, which at the root of a
/// document is black. That is the difference between an icon whose gradient did
/// not make it and one that is refused outright for using a feature the file
/// also does not depend on.
#[allow(clippy::option_option)]
fn paint_of<'a>(style: &Style<'a>, name: &str) -> Option<Option<&'a str>> {
    style.get(name).map(|value| match value.trim() {
        "none" => None,
        _ => Some(value),
    })
}

/// One opacity property, clamped into a fraction.
fn fraction_of(style: &Style<'_>, name: &'static str) -> SvgResult<Option<f32>> {
    match style.get(name) {
        Some(value) => Ok(Some(number(value, name)?.clamp(0.0, 1.0))),
        None => Ok(None),
    }
}

/// Multiplies a colour's alpha, which is where `fill-opacity`, `stroke-opacity`
/// and any group opacity above the shape end up.
fn fade(color: Color, alpha: f32) -> Color {
    Color::rgba(color.r, color.g, color.b, color.a * alpha.clamp(0.0, 1.0))
}

/// A `style="a:b;c:d"` attribute, read as declarations.
struct Style<'a> {
    element: &'a Element,
    declarations: Vec<(&'a str, &'a str)>,
}

impl<'a> Style<'a> {
    fn of(element: &'a Element) -> Self {
        let declarations = element
            .attribute("style")
            .map(|style| {
                style
                    .split(';')
                    .filter_map(|declaration| declaration.split_once(':'))
                    .map(|(name, value)| (name.trim(), value.trim()))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            element,
            declarations,
        }
    }

    /// The value of one property.
    ///
    /// A `style` declaration beats the presentation attribute of the same name,
    /// which is the cascade's own order and the reason an element can carry both
    /// and still mean the declaration.
    fn get(&self, name: &str) -> Option<&'a str> {
        self.declarations
            .iter()
            .rev()
            .find(|(declared, _)| *declared == name)
            .map(|(_, value)| *value)
            .or_else(|| self.element.attribute(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SvgError;
    use xs_xml::parse;

    fn element(source: &str) -> Element {
        parse(source)
            .expect("parses")
            .root()
            .expect("has a root")
            .clone()
    }

    fn paint_of_source(source: &str, parent: &Paint) -> Paint {
        let element = element(source);
        Declarations::of(&element)
            .expect("reads")
            .resolve(parent, &Gradients::default())
            .expect("resolves")
    }

    #[test]
    fn the_defaults_are_svgs_own() {
        let paint = Paint::default();
        assert_eq!(paint.fill, Some(Color::BLACK));
        assert_eq!(paint.stroke, None);
        assert_eq!(paint.stroke_width, 1.0);
        assert_eq!(paint.opacity, 1.0);
    }

    #[test]
    fn an_element_that_says_nothing_inherits_the_defaults() {
        let expected = Paint::default();
        assert_eq!(paint_of_source("<rect/>", &Paint::default()), expected);
    }

    #[test]
    fn reads_a_presentation_attribute() {
        let paint = paint_of_source("<rect fill=\"red\" stroke=\"#00ff00\"/>", &Paint::default());
        assert_eq!(paint.fill, Some(Color::RED));
        assert_eq!(paint.stroke, Some(Color::GREEN));
    }

    #[test]
    fn reads_the_style_attribute() {
        let paint = paint_of_source(
            "<rect style=\"fill: blue; stroke-width: 3\"/>",
            &Paint::default(),
        );
        assert_eq!(paint.fill, Some(Color::BLUE));
        assert_eq!(paint.stroke_width, 3.0);
    }

    #[test]
    fn a_paint_server_reference_falls_back_to_the_inherited_paint() {
        let parent = paint_of_source("<g fill=\"red\"/>", &Paint::default());
        let paint = paint_of_source("<rect fill=\"url(#gradient)\"/>", &parent);
        let expected = Color::RED;
        assert_eq!(paint.fill, Some(expected));
    }

    #[test]
    fn a_style_declaration_wins_over_the_presentation_attribute() {
        let paint = paint_of_source(
            "<rect fill=\"red\" style=\"fill: blue\"/>",
            &Paint::default(),
        );
        let expected = Color::BLUE;
        assert_eq!(paint.fill, Some(expected));
    }

    #[test]
    fn a_style_attribute_without_colons_is_ignored() {
        let paint = paint_of_source("<rect fill=\"red\" style=\"nonsense\"/>", &Paint::default());
        let expected = Color::RED;
        assert_eq!(paint.fill, Some(expected));
    }

    #[test]
    fn none_clears_the_fill() {
        let paint = paint_of_source("<rect fill=\"none\"/>", &Paint::default());
        assert_eq!(paint.fill, None);
    }

    #[test]
    fn a_child_inherits_what_it_does_not_say() {
        let parent = paint_of_source(
            "<g fill=\"red\" stroke=\"blue\" stroke-width=\"2\"/>",
            &Paint::default(),
        );
        let child = paint_of_source("<rect/>", &parent);
        assert_eq!(child.fill, Some(Color::RED));
        assert_eq!(child.stroke, Some(Color::BLUE));
        assert_eq!(child.stroke_width, 2.0);
    }

    #[test]
    fn a_child_overrides_what_it_does_say() {
        let parent = paint_of_source("<g fill=\"red\"/>", &Paint::default());
        let expected = Color::BLUE;
        assert_eq!(
            paint_of_source("<rect fill=\"blue\"/>", &parent).fill,
            Some(expected)
        );
    }

    #[test]
    fn fill_none_in_a_child_beats_an_inherited_fill() {
        let parent = paint_of_source("<g fill=\"red\"/>", &Paint::default());
        assert_eq!(paint_of_source("<rect fill=\"none\"/>", &parent).fill, None);
    }

    #[test]
    fn an_inherited_opacity_is_replaced_not_multiplied() {
        let parent = paint_of_source("<g fill-opacity=\"0.5\"/>", &Paint::default());
        // CSS has this as an override, not a product, and an icon with two
        // nested 50% groups is a lot more common than one that means 25%.
        let child = paint_of_source("<rect fill-opacity=\"0.5\"/>", &parent);
        let expected = 0.5;
        assert!((child.fill_opacity - expected).abs() < 1e-6);

        let grandchild = paint_of_source("<rect/>", &child);
        assert!((grandchild.fill_opacity - expected).abs() < 1e-6);
    }

    #[test]
    fn opacity_is_not_inherited() {
        let parent = paint_of_source(
            "<g opacity=\"0.5\" fill-opacity=\"0.5\"/>",
            &Paint::default(),
        );
        let child = paint_of_source("<rect/>", &parent);
        assert_eq!(child.opacity, 1.0, "the child's own opacity is its own");
        assert_eq!(child.fill_opacity, 0.5, "but this one is inherited");
    }

    #[test]
    fn current_color_follows_the_documents_colour() {
        let parent = paint_of_source("<g color=\"orange\"/>", &Paint::default());
        let child = paint_of_source("<rect fill=\"currentColor\"/>", &parent);
        let orange = Color::rgba(1.0, f32::from(0xA5_u8) / 255.0, 0.0, 1.0);
        assert_eq!(child.fill, Some(orange));
    }

    #[test]
    fn current_color_on_the_shape_itself_works_too() {
        let paint = paint_of_source(
            "<rect color=\"blue\" fill=\"currentColor\"/>",
            &Paint::default(),
        );
        let expected = Color::BLUE;
        assert_eq!(paint.fill, Some(expected));
    }

    #[test]
    fn a_paint_value_that_is_not_a_colour_is_an_error() {
        let element = element("<rect fill=\"mauve\"/>");
        let declarations = Declarations::of(&element).expect("reads");
        let Err(error) = declarations.resolve(&Paint::default(), &Gradients::default()) else {
            panic!("mauve is not a paint");
        };
        assert!(matches!(error, SvgError::BadValue { .. }));
    }

    #[test]
    fn a_stroke_width_with_a_unit_is_read() {
        let paint = paint_of_source("<rect stroke-width=\"2pt\"/>", &Paint::default());
        let expected = 96.0 / 36.0;
        assert!((paint.stroke_width - expected).abs() < 1e-5);
    }

    #[test]
    fn shaping_fades_the_colour() {
        let paint = paint_of_source(
            "<rect fill=\"red\" fill-opacity=\"0.5\"/>",
            &Paint::default(),
        );
        let shape = paint.shape(Path::new(), 1.0, 1.0);
        let fill = shape.fill.expect("it fills");
        assert!((fill.color.a - 0.5).abs() < 1e-6);
    }

    #[test]
    fn an_ancestors_opacity_fades_the_shape_too() {
        let paint = paint_of_source("<rect fill=\"red\"/>", &Paint::default());
        let shape = paint.shape(Path::new(), 1.0, 0.25);
        let fill = shape.fill.expect("it fills");
        assert!((fill.color.a - 0.25).abs() < 1e-6);
    }

    #[test]
    fn shaping_scales_the_pen_with_its_group() {
        let paint = paint_of_source(
            "<rect stroke=\"black\" stroke-width=\"2\"/>",
            &Paint::default(),
        );
        let shape = paint.shape(Path::new(), 3.0, 1.0);
        let stroke = shape.stroke.expect("it strokes");
        assert_eq!(stroke.width, 6.0);
    }

    #[test]
    fn a_dash_pattern_is_scaled_with_its_group() {
        let paint = paint_of_source(
            "<rect stroke=\"black\" stroke-dasharray=\"2 4\"/>",
            &Paint::default(),
        );
        let shape = paint.shape(Path::new(), 2.0, 1.0);
        let stroke = shape.stroke.expect("it strokes");
        assert_eq!(stroke.dash, vec![4.0, 8.0]);
    }

    #[test]
    fn a_stroke_of_nothing_is_not_shaped() {
        let shape = Paint::default().shape(Path::new(), 1.0, 1.0);
        assert_eq!(shape.stroke, None);
        assert!(shape.fill.is_some());
    }
}
