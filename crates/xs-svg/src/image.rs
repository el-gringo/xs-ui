//! The document walk: XML in, a list of shapes out.
//!
//! The walker keeps two things as it goes: the paint and the matrix a child
//! inherits. Everything an SVG expresses by nesting - groups, transforms, paint
//! inheritance - falls out of those two, which is why there is no second pass
//! and no tree of style objects.

use std::collections::HashMap;

use xs_layout::Color;
use xs_math::{Matrix3, Rectangle, Vector2};
use xs_vector::{Fill, Shape, Vector};
use xs_xml::{Document, Element};

use crate::error::{SvgError, SvgResult};
use crate::geometry::{is_drawable, is_shape, local_name, path_of, transform as transform_path};
use crate::gradient::Gradients;
use crate::paint::{Declarations, Paint};
use crate::transform;
use crate::values::{length, numbers};

/// How many `<use>` elements deep the walker follows.
///
/// `<use href="#a"/>` where `a` contains `<use href="#root"/>` is a cycle, and
/// following it is a stack overflow rather than a hang. Eight is far beyond any
/// real icon set.
const MAX_USE_DEPTH: usize = 8;

/// A parsed SVG document.
///
/// The shapes are in the document's own user units and the `view_box` is left
/// unapplied, which is what makes one file drawable at any size: give the
/// `view_box` to a [`Vector`] and it scales and centres the shapes into whatever
/// box layout gives the node. Applying it here instead would bake one size into
/// the geometry.
///
/// # Group opacity is approximated
///
/// `opacity` on a group means the group is composited offscreen and the result
/// drawn, so two translucent children *overlap* and the overlap is darker. There
/// is no offscreen target here, so the opacity is folded into each child's colour
/// instead: the group fades, and its children do not accumulate. Every shape is
/// drawn separately over the frame instead. For an icon - the case this crate
/// exists for - the two agree, because icons rarely overlap.
///
/// # Not read
///
/// Gradients, clip paths, masks, patterns, filters, markers, `switch`'s
/// conditionals, CSS in a `<style>` block, and text. Each is skipped rather than
/// reported: a real file is full of features that have nothing to do with what
/// ends up on screen, and an icon that renders without its gradient is better
/// than an error. See the crate documentation for the whole list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SvgImage {
    /// The document's own `width`, resolved against the view box if it is a
    /// percentage.
    pub width: Option<f32>,
    /// The document's own `height`.
    pub height: Option<f32>,
    /// The `viewBox`, which is also what sizes a document that has no
    /// `width`/`height`.
    pub view_box: Option<Rectangle>,
    /// Every drawable shape, in document order, in the document's own units.
    pub shapes: Vec<Shape>,
}

impl SvgImage {
    /// The size the document asks to be drawn at, if it says.
    ///
    /// A document with a view box and no `width` is that box's size, which is
    /// the rule that lets an icon with only a `viewBox` size itself.
    #[must_use]
    pub fn size(&self) -> Option<Vector2> {
        if let (Some(width), Some(height)) = (self.width, self.height) {
            return Some(Vector2::new(width, height));
        }
        // A document with a view box and one side missing is that box's side,
        // which is what lets an icon with only a `viewBox` size itself.
        let view_box = self.view_box?;
        Some(Vector2::new(
            self.width.unwrap_or(view_box.width),
            self.height.unwrap_or(view_box.height),
        ))
    }

    /// Whether the document drew anything at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    /// Paints every shape that has paint in one colour.
    ///
    /// For the document whose paint this renderer cannot reproduce: every path in
    /// a wordmark may say `fill: url(#gradient)`, a gradient is a quad shader
    /// feature, and what comes back is the initial black. Retinting keeps the
    /// geometry and drops the colour that was never going to work.
    pub fn tint(&mut self, colour: Color) {
        for shape in &mut self.shapes {
            if shape.fill.is_some() {
                shape.fill = Some(Fill::new(colour));
            }
            if let Some(stroke) = &mut shape.stroke {
                stroke.color = colour;
            }
        }
    }

    /// This image on a [`Vector`]: its shapes, its view box, and its own size if
    /// it declared one.
    #[must_use]
    pub fn apply_to(self, vector: Vector) -> Vector {
        // Asked before the move, because `self` is consumed here.
        let size = self.size();
        let vector = vector.shapes(self.shapes);
        let vector = match self.view_box {
            Some(view_box) => vector.view_box(
                Vector2::new(view_box.x, view_box.y),
                view_box.width,
                view_box.height,
            ),
            None => vector,
        };
        // The declared size goes on as the node's own, because that is what the
        // document asked for - and a caller who disagrees can overwrite it.
        match size {
            Some(size) => vector.width(size.x).height(size.y),
            None => vector,
        }
    }
}

/// Parses an SVG document from text.
///
/// # Errors
///
/// [`SvgError::NotSvg`] when the root is not `<svg>`, and as
/// [`xs_xml::parse`] for anything malformed; a bad attribute is a
/// [`SvgError::BadValue`] naming it.
pub fn parse(text: &str) -> SvgResult<SvgImage> {
    image(&xs_xml::parse(text)?)
}

/// Parses an SVG document from bytes, which have to be UTF-8.
///
/// # Errors
///
/// As [`parse`].
pub fn parse_bytes(bytes: &[u8]) -> SvgResult<SvgImage> {
    image(&xs_xml::parse_bytes(bytes)?)
}

/// Parses an SVG document straight into a [`Vector`].
///
/// # Errors
///
/// As [`parse`].
pub fn parse_vector(text: &str) -> SvgResult<Vector> {
    Ok(parse(text)?.apply_to(Vector::new()))
}

/// The shape list for an already parsed document.
fn image(document: &Document) -> SvgResult<SvgImage> {
    let root = document
        .root()
        .ok_or(SvgError::Xml(xs_xml::XmlError::NoRoot))?;
    if local_name(root.name()) != "svg" {
        return Err(SvgError::NotSvg(root.name().to_owned()));
    }

    let view_box = match root.attribute("viewBox") {
        Some(value) => Some(view_box(value)?),
        None => None,
    };
    // A percentage on the root resolves against the view box, which is the only
    // size there is to resolve it against.
    let base = match view_box {
        Some(view_box) => view_box.width,
        None => 1.0,
    };
    let width = match root.attribute("width") {
        Some(value) => Some(length(value, base, "width")?),
        None => None,
    };
    let height = match root.attribute("height") {
        Some(value) => Some(length(value, base, "height")?),
        None => None,
    };

    let mut walker = Walker {
        ids: HashMap::new(),
        gradients: Gradients::collect(root),
        shapes: Vec::new(),
    };
    walker.collect_ids(root);

    // A root with a `transform` of its own is unusual and legal; the root's
    // paint attributes are read by the walk like anyone else's, so the paint it
    // starts from is SVG's initial values.
    let transform = transform::parse(root.attribute("transform").unwrap_or(""))?;
    let paint = Declarations::of(root)?.resolve(&Paint::default(), &Gradients::default())?;
    walker.children(root, &paint, &transform, paint.opacity, 0)?;

    Ok(SvgImage {
        width,
        height,
        view_box,
        shapes: walker.shapes,
    })
}

/// `viewBox`, which is four numbers and has to be exactly four numbers.
fn view_box(value: &str) -> SvgResult<Rectangle> {
    let numbers = numbers(value, "viewBox")?;
    let [x, y, width, height] = numbers[..] else {
        return Err(SvgError::BadValue {
            attribute: "viewBox",
            value: value.to_owned(),
        });
    };
    Ok(Rectangle::new(x, y, width, height))
}

/// The walk itself.
struct Walker {
    /// Every element in the document that has an `id`, cloned.
    ///
    /// Collected up front rather than found during the walk because `<use>` may
    /// point forwards at something defined later, which is how `<defs>` is
    /// written.
    ids: HashMap<String, Element>,
    /// Every gradient in the document, which a `fill="url(#id)"` resolves
    /// against. Collected with the ids, for the same reason: it is nearly always
    /// in `<defs>` and the shape that uses it is nowhere near it.
    gradients: Gradients,
    shapes: Vec<Shape>,
}

impl Walker {
    /// Gathers every `id` in the tree.
    fn collect_ids(&mut self, element: &Element) {
        if let Some(id) = element.attribute("id") {
            self.ids.insert(id.to_owned(), element.clone());
        }
        for child in element.children() {
            if let Some(child) = child.as_element() {
                self.collect_ids(child);
            }
        }
    }

    /// Walks one element's children, which is how a group hands its paint and
    /// its matrix down.
    fn children(
        &mut self,
        element: &Element,
        paint: &Paint,
        transform: &Matrix3,
        alpha: f32,
        use_depth: usize,
    ) -> SvgResult<()> {
        for child in element.children() {
            let Some(child) = child.as_element() else {
                continue;
            };
            self.visit(child, paint, transform, alpha, use_depth)?;
        }
        Ok(())
    }

    /// Walks one element.
    fn visit(
        &mut self,
        element: &Element,
        parent: &Paint,
        parent_transform: &Matrix3,
        parent_alpha: f32,
        use_depth: usize,
    ) -> SvgResult<()> {
        let name = local_name(element.name());
        let declarations = Declarations::of(element)?;
        let paint = declarations.resolve(parent, &self.gradients)?;
        let transform = match element.attribute("transform") {
            Some(value) => parent_transform.then(&transform::parse(value)?),
            None => *parent_transform,
        };
        // SVG does not inherit `opacity`: it composites the group. Folding the
        // running product into each shape's colour is the approximation that
        // `SvgImage` documents.
        let alpha = parent_alpha * paint.opacity;

        if is_shape(name) {
            if let Some(shape) = Self::draw(element, &paint, &transform, alpha)? {
                self.shapes.push(shape);
            }
            return Ok(());
        }

        match name {
            // A group of anything: `g`, a link, or a `switch` whose conditions
            // are all satisfied, which is the only branch of `switch` worth
            // having and the one a browser takes too.
            "g" | "a" | "switch" => self.children(element, &paint, &transform, alpha, use_depth),
            // A nested viewport is placed with x/y and otherwise behaves as a
            // group. Its width/height clip, and nothing here clips.
            "svg" => {
                let x = attribute_length(element, "x")?;
                let y = attribute_length(element, "y")?;
                let placed = transform.then(&Matrix3::translation(Vector2::new(x, y)));
                self.children(element, &paint, &placed, alpha, use_depth)
            }
            "use" => self.use_(element, &paint, &transform, alpha, use_depth),
            _ => Ok(()),
        }
    }

    /// The shape of one shape element, painted and placed.
    fn draw(
        element: &Element,
        paint: &Paint,
        transform: &Matrix3,
        alpha: f32,
    ) -> SvgResult<Option<Shape>> {
        let Some(path) = path_of(element)? else {
            return Ok(None);
        };
        // A move and nothing else is a shape that draws no ink: an empty `d`, a
        // one point polyline, a template that has not been filled in yet.
        if !is_drawable(&path) {
            return Ok(None);
        }
        // The geometry is in the element's own units and the pen is in screen
        // units, so a scaled group scales the pen by the same factor.
        let scale = transform.scale_factor();
        Ok(Some(paint.shape(
            transform_path(&path, transform),
            scale,
            alpha,
        )))
    }

    /// `<use>`, which is a group containing whatever the reference points at.
    fn use_(
        &mut self,
        element: &Element,
        paint: &Paint,
        parent_transform: &Matrix3,
        parent_alpha: f32,
        use_depth: usize,
    ) -> SvgResult<()> {
        // A reference into another *file* is a fetch rather than a parse, and
        // this crate does no I/O, so that is skipped. An id that is not in this
        // document is an error: a `<use>` that draws nothing is an icon that is
        // mysteriously half there.
        let reference = element
            .attribute("href")
            .or_else(|| element.attribute("xlink:href"));
        let Some(id) = reference.and_then(|reference| reference.strip_prefix('#')) else {
            return Ok(());
        };
        let Some(target) = self.ids.get(id).cloned() else {
            return Err(SvgError::UnknownReference(id.to_owned()));
        };
        if use_depth >= MAX_USE_DEPTH {
            return Err(SvgError::RecursiveUse);
        }

        // `visit` has already folded the `use`'s own transform and opacity in,
        // so this only adds the placement.
        let x = attribute_length(element, "x")?;
        let y = attribute_length(element, "y")?;
        let transform = parent_transform.then(&Matrix3::translation(Vector2::new(x, y)));

        self.visit(&target, paint, &transform, parent_alpha, use_depth + 1)
    }
}

/// One length attribute, zero when it is absent.
fn attribute_length(element: &Element, name: &'static str) -> SvgResult<f32> {
    match element.attribute(name) {
        Some(value) => length(value, 1.0, name),
        None => Ok(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xs_layout::Color;
    use xs_vector::Segment;

    #[test]
    fn reads_a_single_rect() {
        let image =
            parse("<svg><rect width=\"10\" height=\"10\" fill=\"red\"/></svg>").expect("parses");
        let expected = 1;
        assert_eq!(image.shapes.len(), expected);
        let fill = image.shapes[0].fill.as_ref().expect("it fills");
        assert_eq!(fill.color, Color::RED);
    }

    #[test]
    fn a_root_that_is_not_svg_is_an_error() {
        let error = parse("<html/>").unwrap_err();
        assert_eq!(error.to_string(), "the root element is <html>, not <svg>");
    }

    #[test]
    fn the_size_comes_from_the_documents_own_attributes() {
        let image = parse("<svg width=\"32\" height=\"16\"/>").expect("parses");
        let expected = Some(Vector2::new(32.0, 16.0));
        assert_eq!(image.size(), expected);
    }

    #[test]
    fn a_view_box_sizes_a_document_that_has_no_width() {
        let image = parse("<svg viewBox=\"0 0 24 24\"/>").expect("parses");
        let expected = Some(Vector2::new(24.0, 24.0));
        assert_eq!(image.size(), expected);
        assert_eq!(image.view_box, Some(Rectangle::new(0.0, 0.0, 24.0, 24.0)));
    }

    #[test]
    fn a_view_box_is_four_numbers() {
        let error = parse("<svg viewBox=\"0 0 24\"/>").unwrap_err();
        assert_eq!(error.to_string(), "'0 0 24' is not a valid viewBox");
    }

    #[test]
    fn the_view_box_is_left_for_the_node_to_apply() {
        let image = parse("<svg viewBox=\"0 0 24 24\"><rect width=\"24\" height=\"24\"/></svg>")
            .expect("parses");
        // Untouched: the geometry is in the document's own units.
        let first = image.shapes[0].path.segments.first();
        assert_eq!(first, Some(&Segment::MoveTo(Vector2::new(0.0, 0.0))));
        let last = image.shapes[0].path.segments.last();
        assert_eq!(last, Some(&Segment::Close));
    }

    #[test]
    fn a_group_transforms_its_children() {
        let image = parse(
            "<svg><g transform=\"translate(10,20)\"><rect width=\"1\" height=\"1\"/></g></svg>",
        )
        .expect("parses");
        let expected = Some(&Segment::MoveTo(Vector2::new(10.0, 20.0)));
        assert_eq!(image.shapes[0].path.segments.first(), expected);
    }

    #[test]
    fn transforms_nest_the_way_svg_composes_them() {
        let image = parse(
            "<svg><g transform=\"translate(10,0)\"><g transform=\"scale(2)\">\
             <rect width=\"1\" height=\"1\"/></g></g></svg>",
        )
        .expect("parses");
        // The inner scale happens first, so the rectangle is 2 wide and then
        // moved 10 along.
        let expected = Some(&Segment::MoveTo(Vector2::new(10.0, 0.0)));
        assert_eq!(image.shapes[0].path.segments.first(), expected);
        let far_corner = image.shapes[0].path.segments[2];
        assert_eq!(far_corner, Segment::LineTo(Vector2::new(12.0, 2.0)));
    }

    #[test]
    fn a_transformed_group_scales_its_own_stroke_width() {
        let image = parse(
            "<svg><g transform=\"scale(3)\"><rect width=\"1\" height=\"1\" \
             stroke=\"black\" stroke-width=\"2\"/></g></svg>",
        )
        .expect("parses");
        let stroke = image.shapes[0].stroke.as_ref().expect("it strokes");
        assert_eq!(stroke.width, 6.0);
    }

    #[test]
    fn paint_is_inherited_by_children() {
        let image = parse("<svg><g fill=\"red\"><rect width=\"1\" height=\"1\"/></g></svg>")
            .expect("parses");
        let fill = image.shapes[0].fill.as_ref().expect("it fills");
        assert_eq!(fill.color, Color::RED);
    }

    #[test]
    fn a_group_opacity_fades_its_children() {
        let image = parse(
            "<svg><g opacity=\"0.5\"><g opacity=\"0.5\"><rect width=\"1\" height=\"1\"/></g></g></svg>",
        )
        .expect("parses");
        let fill = image.shapes[0].fill.as_ref().expect("it fills");
        let expected = 0.25;
        assert!((fill.color.a - expected).abs() < 1e-6);
    }

    #[test]
    fn a_use_draws_what_it_points_at() {
        let image = parse(
            "<svg><defs><rect id=\"box\" width=\"4\" height=\"4\"/></defs>\
             <use href=\"#box\" x=\"10\" fill=\"blue\"/></svg>",
        )
        .expect("parses");
        let expected = 1;
        assert_eq!(image.shapes.len(), expected);
        assert_eq!(
            image.shapes[0].path.segments.first(),
            Some(&Segment::MoveTo(Vector2::new(10.0, 0.0)))
        );
        let fill = image.shapes[0].fill.as_ref().expect("it fills");
        assert_eq!(fill.color, Color::BLUE);
    }

    #[test]
    fn a_use_applies_its_own_opacity_once() {
        let image = parse(
            "<svg><defs><rect id=\"box\" width=\"4\" height=\"4\"/></defs>\
             <use href=\"#box\" opacity=\"0.5\"/></svg>",
        )
        .expect("parses");
        let fill = image.shapes[0].fill.as_ref().expect("it fills");
        let expected = 0.5;
        assert!((fill.color.a - expected).abs() < 1e-6);
    }

    #[test]
    fn a_use_transform_is_applied_alongside_its_x_and_y() {
        let image = parse(
            "<svg><defs><rect id=\"box\" width=\"4\" height=\"4\"/></defs>\
             <use href=\"#box\" x=\"10\" transform=\"scale(2)\"/></svg>",
        )
        .expect("parses");
        // SVG defines `x` as a translate *appended* to the element's own
        // transform list, and a list composes right to left: so the x moves the
        // content to 10 and then the use's own scale doubles that to 20.
        assert_eq!(
            image.shapes[0].path.segments.first(),
            Some(&Segment::MoveTo(Vector2::new(20.0, 0.0)))
        );
        let far_corner = image.shapes[0].path.segments[2];
        assert_eq!(far_corner, Segment::LineTo(Vector2::new(28.0, 8.0)));
    }

    #[test]
    fn a_use_can_point_at_a_group() {
        let image = parse(
            "<svg><defs><g id=\"pair\"><rect width=\"1\" height=\"1\"/><rect x=\"2\" \
             width=\"1\" height=\"1\"/></g></defs><use href=\"#pair\"/></svg>",
        )
        .expect("parses");
        let expected = 2;
        assert_eq!(image.shapes.len(), expected);
    }

    #[test]
    fn a_use_into_another_file_is_skipped() {
        let image = parse("<svg><use href=\"other.svg#box\"/></svg>").expect("parses");
        assert!(image.is_empty());
    }

    #[test]
    fn a_use_pointing_at_nothing_is_an_error() {
        let error = parse("<svg><use href=\"#missing\"/></svg>").unwrap_err();
        assert_eq!(error.to_string(), "no element with id 'missing'");
    }

    #[test]
    fn a_use_that_points_at_itself_is_an_error_rather_than_a_hang() {
        let source = "<svg><use id=\"loop\" href=\"#loop\"/></svg>";
        let error = parse(source).unwrap_err();
        assert_eq!(error.to_string(), "<use> elements are nested too deeply");
    }

    #[test]
    fn the_elements_this_parser_does_not_draw_are_skipped() {
        let image = parse(
            "<svg><title>t</title><desc>d</desc><metadata/><style>rect{}</style>\
             <defs/><linearGradient id=\"g\"/><clipPath id=\"c\"/><mask id=\"m\"/>\
             <filter id=\"f\"/><pattern id=\"p\"/><marker id=\"k\"/><symbol id=\"s\"/>\
             <text>hello</text><rect width=\"1\" height=\"1\"/></svg>",
        )
        .expect("parses");
        let expected = 1;
        assert_eq!(image.shapes.len(), expected);
    }

    #[test]
    fn a_nested_svg_is_placed_by_its_x_and_y() {
        let image = parse("<svg><svg x=\"5\" y=\"5\"><rect width=\"1\" height=\"1\"/></svg></svg>")
            .expect("parses");
        assert_eq!(
            image.shapes[0].path.segments.first(),
            Some(&Segment::MoveTo(Vector2::new(5.0, 5.0)))
        );
    }

    #[test]
    fn shapes_come_out_in_document_order() {
        let image = parse(
            "<svg><rect width=\"1\" height=\"1\"/><circle r=\"1\"/>\
             <path d=\"M0 0 L1 1\"/></svg>",
        )
        .expect("parses");
        let expected = 3;
        assert_eq!(image.shapes.len(), expected);
        // Four sides, then a whole circle - a move and the two arcs of a
        // half turn each - then one line: the shapes are in document order, so
        // index 1 is the circle.
        let expected = 4;
        assert_eq!(image.shapes[1].path.segments.len(), expected);
        assert_eq!(image.shapes[2].path.segments.len(), 2);
    }

    #[test]
    fn an_empty_document_is_an_image_with_no_shapes() {
        let image = parse("<svg/>").expect("parses");
        assert!(image.is_empty());
        assert_eq!(image.size(), None);
    }

    #[test]
    fn a_document_reaches_a_vector_with_its_view_box() {
        let image = parse("<svg viewBox=\"0 0 24 24\"><rect width=\"24\" height=\"24\"/></svg>")
            .expect("parses");
        let vector = image.apply_to(Vector::new());
        assert_eq!(vector.shapes.len(), 1);
        let expected = Some(Rectangle::new(0.0, 0.0, 24.0, 24.0));
        assert_eq!(vector.view_box, expected);
        assert_eq!(vector.style.width, Some(24.0));
        assert_eq!(vector.style.height, Some(24.0));
    }

    #[test]
    fn a_declared_size_goes_on_as_the_nodes_own() {
        let vector = parse_vector("<svg width=\"32\" height=\"16\"/>").expect("parses");
        assert_eq!(vector.style.width, Some(32.0));
        assert_eq!(vector.style.height, Some(16.0));
    }

    #[test]
    fn text_and_comments_are_not_shapes() {
        let image = parse("<svg>text<rect width=\"1\" height=\"1\"/>more</svg>").expect("parses");
        assert_eq!(image.shapes.len(), 1);
    }
}
