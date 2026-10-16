//! The `Vector` primitive.
//!
//! A node that draws one or more [`Shape`]s. It is an ordinary box in the box
//! model - it flows, it takes a margin, `translate`/`scale`/`rotate` carry it -
//! and the only thing that makes it different is that what it paints is a path
//! rather than a rectangle.
//!
//! The geometry is built during **layout**, not before it, and that ordering is
//! the whole design:
//!
//! - The box size is not known until layout runs, and the node's box is its
//!   geometry's bounds unless the app says otherwise. Building before layout
//!   would mean guessing, or building twice.
//! - A node's transform is not known until the frame walks the tree, so the
//!   geometry stays in the element's own pixels and the render driver transforms
//!   it. Same reason `TextVertices` holds glyph quads relative to the run's
//!   origin.
//!
//! So the cost is that the mesh is rebuilt every frame the element is built. That
//! is the right trade: `build_element_tree` deep-copies every element before any
//! of this runs, so flattening a polyline is nowhere near where a frame goes.

use std::any::TypeId;
use std::rc::Rc;

use xs_animation::Animate;
use xs_core::{AnimationState, AppContext, Context, ElementObject, IntoElement, ParentElement};
use xs_layout::{
    BoxConstraints, EdgeInsets, Layout, LayoutBox, LayoutTree, Material, Positioned, Rectangle,
    SizedLayoutBox, Style, StylePosition, VectorGeometry, View,
};
use xs_math::{Matrix3, Vector2};

use crate::mesh::{MeshBuilder, Shape};
use crate::path::Path;

/// A node that draws paths.
///
/// The shapes are drawn in the order given, so a later one lands on top of an
/// earlier one exactly as a later `Div` would.
#[derive(Default, Clone)]
pub struct Vector {
    pub testid: Option<&'static str>,
    pub style: Style,
    pub shapes: Vec<Shape>,
    /// Flattening tolerance, in device pixels. See [`Vector::tolerance`].
    pub tolerance: Option<f32>,
    /// When set, the shapes are scaled to fit this box and centred in it.
    ///
    /// This is SVG's `viewBox` with `preserveAspectRatio` left at its default,
    /// and it is what lets one path be authored once at whatever size is
    /// convenient and then drawn at whatever size the layout gives.
    pub view_box: Option<Rectangle>,
    pub animation: Option<VectorAnimation>,
}

/// An animation over a `Vector`'s own setters, the same shape as `Div`'s.
///
/// The callback gets the node back with a value in `0..=1` and returns it with
/// whatever it wants changed, so an animation composes with the setters rather than
/// replacing them - and so a dash offset or a stroke width can be animated with the
/// same three lines a rotation is.
pub struct VectorAnimation {
    pub id: &'static str,
    pub animate: Box<dyn Animate>,
    pub callback: Rc<dyn Fn(Vector, f32) -> Vector>,
}

impl Clone for VectorAnimation {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            animate: self.animate.clone_box(),
            callback: Rc::clone(&self.callback),
        }
    }
}

impl Vector {
    #[must_use]
    pub fn new() -> Self {
        Default::default()
    }

    /// An id that is only used for debug and tests.
    #[must_use]
    pub fn testid(mut self, testid: &'static str) -> Self {
        self.testid = Some(testid);
        self
    }

    // ---------------------------------------------------------------- shapes

    /// Adds a shape. Several shapes make one node, one box, one material.
    #[must_use]
    pub fn shape(mut self, shape: Shape) -> Self {
        self.shapes.push(shape);
        self
    }

    #[must_use]
    pub fn shapes(mut self, shapes: impl IntoIterator<Item = Shape>) -> Self {
        self.shapes.extend(shapes);
        self
    }

    /// Adds a shape parsed from SVG path data, filled with `color`.
    ///
    /// # Panics
    ///
    /// Panics if `data` is not valid path data. This is the one place in the
    /// primitive layer that can fail, and it fails loudly at build time rather
    /// than drawing nothing - a path that silently disappears is much harder to
    /// find than one that stops the app on the first frame.
    #[must_use]
    pub fn path(self, data: &str, color: xs_layout::Color) -> Self {
        let path = Path::parse(data).unwrap_or_else(|error| {
            panic!("Vector::path: invalid SVG path data: {error}\n  {data}")
        });
        self.shape(Shape::new(path).fill(color))
    }

    // ------------------------------------------------------------- geometry

    /// How finely curves become polylines, in device pixels.
    ///
    /// This is the one knob that trades work for smoothness across the whole
    /// pipeline: it sets how many capsules a stroke costs and how many points the
    /// fill's ear clipper walks. The default is a quarter of a pixel, which is
    /// finer than the error is visible on a straight edge.
    #[must_use]
    pub fn tolerance(mut self, tolerance: f32) -> Self {
        self.tolerance = Some(tolerance);
        self
    }

    /// Shows the `view_box` region of the shapes' own coordinates in the node,
    /// SVG's `viewBox`.
    ///
    /// The scale is uniform - the smaller of the two ratios wins, so nothing is
    /// distorted - and the scaled view box is centred, which is
    /// `preserveAspectRatio="xMidYMid meet"`. Padding that is baked into the
    /// authored coordinates is part of the artwork, so an icon on a 24 unit grid
    /// keeps its margin instead of being scaled until its ink touches the edge.
    ///
    /// Shapes outside the view box are still drawn; see [`view_box_fit`].
    #[must_use]
    pub fn view_box(mut self, origin: Vector2, width: f32, height: f32) -> Self {
        self.view_box = Some(Rectangle::new(origin.x, origin.y, width, height));
        self
    }

    // ------------------------------------------------------------- placement

    #[must_use]
    pub fn absolute(mut self) -> Self {
        self.style.position = StylePosition::Absolute;
        self
    }

    #[must_use]
    pub fn top(mut self, value: f32) -> Self {
        self.style.top = Some(value);
        self
    }

    #[must_use]
    pub fn right(mut self, value: f32) -> Self {
        self.style.right = Some(value);
        self
    }

    #[must_use]
    pub fn bottom(mut self, value: f32) -> Self {
        self.style.bottom = Some(value);
        self
    }

    #[must_use]
    pub fn left(mut self, value: f32) -> Self {
        self.style.left = Some(value);
        self
    }

    #[must_use]
    pub fn width(mut self, size: f32) -> Self {
        self.style.width = Some(size);
        self
    }

    #[must_use]
    pub fn height(mut self, size: f32) -> Self {
        self.style.height = Some(size);
        self
    }

    #[must_use]
    pub fn margin(mut self, margin: f32) -> Self {
        self.style.margin = EdgeInsets::all(margin);
        self
    }

    #[must_use]
    pub fn margin_top(mut self, size: f32) -> Self {
        self.style.margin.top = size;
        self
    }

    #[must_use]
    pub fn margin_bottom(mut self, size: f32) -> Self {
        self.style.margin.bottom = size;
        self
    }

    #[must_use]
    pub fn margin_left(mut self, size: f32) -> Self {
        self.style.margin.left = size;
        self
    }

    #[must_use]
    pub fn margin_right(mut self, size: f32) -> Self {
        self.style.margin.right = size;
        self
    }

    // -------------------------------------------------------------- transform

    /// Replace this element's whole transform.
    #[must_use]
    pub fn transform(mut self, transform: Matrix3) -> Self {
        self.style.transform = transform;
        self
    }

    /// Move the element and its subtree by `offset` pixels.
    #[must_use]
    pub fn translate(mut self, offset: Vector2) -> Self {
        self.style.transform = self.style.transform * Matrix3::translation(offset);
        self
    }

    #[must_use]
    pub fn translate_xy(self, x: f32, y: f32) -> Self {
        self.translate(Vector2::new(x, y))
    }

    /// Scale by the same factor on both axes.
    ///
    /// A stroke's width follows this, through the transform's `scale_factor` in
    /// the renderer: the pen is a length, so it has to grow with the drawing the
    /// way every other length in the style does.
    #[must_use]
    pub fn scale(mut self, factor: f32) -> Self {
        self.style.transform = self.style.transform * Matrix3::uniform_scale(factor);
        self
    }

    #[must_use]
    pub fn scale_xy(mut self, x: f32, y: f32) -> Self {
        self.style.transform = self.style.transform * Matrix3::scale(Vector2::new(x, y));
        self
    }

    /// Rotate in radians, about the element's own centre. Positive is clockwise,
    /// because this framework's screen space has y growing downwards.
    #[must_use]
    pub fn rotate(mut self, radians: f32) -> Self {
        self.style.transform = self.style.transform * Matrix3::rotation(radians);
        self
    }

    #[must_use]
    pub fn rotate_degrees(self, degrees: f32) -> Self {
        self.rotate(degrees.to_radians())
    }

    /// Animates this node through `callback`, which receives the node and a value
    /// in `0..=1` and hands back the node to draw.
    ///
    /// `id` names the animation's state, so two animations on one node need
    /// different names and the same animation reused across a list can share one.
    #[must_use]
    pub fn with_animation<A: Animate + 'static>(
        mut self,
        id: &'static str,
        animate: A,
        callback: impl Fn(Vector, f32) -> Vector + 'static,
    ) -> Self {
        self.animation = Some(VectorAnimation {
            id,
            animate: Box::new(animate),
            callback: Rc::new(callback),
        });
        self
    }

    /// Moves every dash pattern along its own path, by `offset` pixels.
    ///
    /// A convenience for the one thing a stroke is usually animated *by*. The
    /// offset is in the same units as the pattern's own entries, so this is an
    /// absolute distance along the contour rather than a fraction of it.
    #[must_use]
    pub fn dash_offset(mut self, offset: f32) -> Self {
        for shape in &mut self.shapes {
            shape.dash_offset = offset;
        }
        self
    }

    #[must_use]
    pub fn when(self, condition: bool, mut callback: impl FnMut(Self) -> Self) -> Self {
        if condition { callback(self) } else { self }
    }

    /// The colour inherited from the parent element, used by shapes that do not
    /// name one of their own.
    #[must_use]
    pub fn color(mut self, color: xs_layout::Color) -> Self {
        self.style.color = color;
        self
    }

    /// Builds the geometry for a box of `size`, in the node's own pixels.
    ///
    /// Public because it is the only piece of this that has real logic, and
    /// because a test that wants to know what a node will emit has no other way to
    /// ask.
    #[must_use]
    pub fn geometry(&self, size: Vector2) -> VectorGeometry {
        let mut builder = MeshBuilder::new();
        if let Some(tolerance) = self.tolerance {
            builder = builder.tolerance(tolerance);
        }
        let mut scaled: Vec<Shape> = Vec::with_capacity(self.shapes.len());
        for shape in &self.shapes {
            scaled.push(match &self.view_box {
                Some(view) => {
                    let Some(fit) = view_box_fit(view, size) else {
                        continue;
                    };
                    Shape {
                        // `p * scale + offset`, in that order. Mapping about the
                        // *view box's* corner rather than the shape's own centre is
                        // the whole point of a view box: the padding baked into
                        // authored coordinates is part of the artwork, so an icon
                        // drawn on a 24 unit grid keeps its 2 unit margin instead of
                        // being scaled up to fill its box edge to edge.
                        path: shape.path.scaled(fit.scale).translated(fit.offset),
                        ..shape.clone()
                    }
                }
                None => shape.clone(),
            });
        }
        builder.shapes(&scaled);
        builder.build()
    }
}

/// Where a shape's centre goes and how much it is scaled by, to fit `box_rect`.
/// How a view box of authored coordinates covers a node box of local pixels.
struct ViewBoxFit {
    /// How much a view box unit is worth in local pixels.
    scale: f32,
    /// Applied after the scale: it lands the view box's own top-left corner, so
    /// that `scale * view.min + offset` is the top-left of the node box.
    offset: Vector2,
}

/// Maps the `view_box` region onto `box_size`, SVG's `viewBox` with the default
/// `preserveAspectRatio="xMidYMid meet"`.
///
/// The scale comes from the *view box*, not from the shape: that is the whole
/// difference between a view box and scaling artwork to fit. An icon authored on a
/// 24 unit grid with a 2 unit margin keeps that margin when drawn at 96px, whereas
/// fitting its own bounds would zoom to the ink and eat it. Nothing is distorted,
/// because the smaller ratio wins, and the scaled view box is centred in the node
/// box - which leaves bars on the other axis, as SVG does.
///
/// A view box with no extent, or a node box with no extent, has no meaningful
/// scale and yields nothing. Note that shapes *outside* the view box are still
/// drawn, where SVG would clip them to the viewport: this framework has no
/// viewport clip of its own, and inventing one here would hide artwork rather than
/// misplace it.
fn view_box_fit(view: &Rectangle, box_size: Vector2) -> Option<ViewBoxFit> {
    if view.width <= 0.0 || view.height <= 0.0 || box_size.x <= 0.0 || box_size.y <= 0.0 {
        return None;
    }
    let scale = (box_size.x / view.width).min(box_size.y / view.height);
    // Centre the scaled view box, then back the view box's own origin out of the
    // translation so that `p * scale + offset` maps its min corner, not (0, 0).
    let centred = Vector2::new(
        (box_size.x - view.width * scale) * 0.5,
        (box_size.y - view.height * scale) * 0.5,
    );
    Some(ViewBoxFit {
        scale,
        offset: centred - Vector2::new(view.x, view.y) * scale,
    })
}

impl IntoElement for Vector {
    fn into_element(self) -> xs_core::AnyElement {
        xs_core::AnyElement::new(self)
    }
}

impl ElementObject for Vector {
    fn tag(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn clone_box(&self) -> Box<dyn ElementObject> {
        Box::new(self.clone())
    }

    fn id(&self) -> &'static str {
        self.testid.unwrap_or("vector")
    }

    fn children(&self) -> &[xs_core::AnyElement] {
        &[]
    }

    fn post_render(&mut self, ctx: &mut Context) {
        if let Some(animation) = &self.animation {
            let now = ctx.app_context.now;
            let initial = animation.animate.clone_box();
            let incoming = animation.animate.clone_box();
            let callback = animation.callback.clone();

            let state = ctx.use_keyed_state::<AnimationState>(animation.id, move |_| {
                AnimationState::new_boxed(initial, now)
            });
            state.update(|s| s.retrigger_if_changed(incoming, now));
            let finished = state.read().is_finished();
            let value = if finished {
                state.read().value()
            } else {
                let mut value = 0.0;
                state.update(|s| value = s.tick(now));
                value
            };
            if !finished {
                ctx.app_context.dirty = true;
            }
            let this = (callback)(self.clone(), value);
            self.shapes = this.shapes;
            self.style = this.style;
        }

        // The same conversion `Div` does, and it has to be the same conversion:
        // everything downstream of this point - the constraints, the layout, the
        // flattening tolerance - is in device pixels, and the style is where a
        // logical length becomes one. The shapes are scaled here too, rather than
        // being handed to the mesh builder unscaled, because the flattening
        // tolerance is in pixels and would otherwise be measured in the wrong unit.
        let scale = ctx.app_context.screen.scale;
        self.style = self.style.scaled(scale);
        for shape in &mut self.shapes {
            shape.path.scale_mut(scale);
            if let Some(stroke) = shape.stroke.as_mut() {
                // The pen's width is a length like any other. Doing it here means
                // the renderer does not have to know about it, and a stroke keeps
                // its weight on a 2x display instead of looking thinner than the
                // shapes around it.
                stroke.width *= scale;
            }
        }
        // The view box is expressed in the shapes' own coordinates, which have just
        // been converted to device pixels above. Leaving it behind would mix units
        // in `view_box_fit`: a 24 unit grid under a 2x screen would ask for a scale
        // of 96/24 out of coordinates that are now twice as large as they were, and
        // draw the icon at half the size asked for.
        if let Some(view) = self.view_box.as_mut() {
            view.x *= scale;
            view.y *= scale;
            view.width *= scale;
            view.height *= scale;
        }

        if let Some(text_properties) = self.style.text_properties.clone() {
            ctx.text_properties = text_properties;
        }
        ctx.color = self.style.color;
        if let Some(w) = self.style.width {
            ctx.constraints.max.x = w;
        }
        if let Some(h) = self.style.height {
            ctx.constraints.max.y = h;
        }
    }

    fn paint(&self, _app_context: &mut AppContext, _views: &[View]) -> View {
        let view: View = Rc::new(VectorView {
            style: self.style.clone(),
            vector: self.clone(),
        });
        match self.style.position {
            StylePosition::Absolute => Rc::new(Positioned {
                position: Vector2::new(
                    self.style.left.unwrap_or(0.0),
                    self.style.top.unwrap_or(0.0),
                ),
                child: view,
            }),
            StylePosition::Static => view,
        }
    }
}

impl ParentElement for Vector {
    fn child(self, _child: impl Into<xs_core::AnyElement>) -> Self {
        // A path has no interior to put a child in, and silently dropping it would
        // be worse than saying so.
        panic!("Vector takes no children: a path has no interior to lay them out in")
    }

    fn children(self, _children: impl IntoIterator<Item = impl Into<xs_core::AnyElement>>) -> Self {
        panic!("Vector takes no children: a path has no interior to lay them out in")
    }
}

/// The layout node: one box, one material, sized to the geometry.
///
/// It does **not** go through `Flex`, unlike `Div`. That is deliberate. `Flex`
/// carries its material on both its own box and the box its parent makes for it,
/// so a material reaches the renderer twice - harmless for a translucent
/// background, which is exactly why it has gone unnoticed, and visibly wrong for a
/// stroke, which would be drawn twice over.
struct VectorView {
    style: Style,
    vector: Vector,
}

impl Layout for VectorView {
    fn position(&self) -> StylePosition {
        self.style.position
    }

    fn margin(&self) -> EdgeInsets {
        self.style.margin
    }

    fn padding(&self) -> EdgeInsets {
        self.style.padding
    }

    /// The transform is reported to the *parent*, which puts it on this node's box
    /// in the flex pass. The box inserted below carries the identity, so it is
    /// applied once rather than twice.
    fn transform(&self) -> Matrix3 {
        self.style.transform
    }

    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let geometry = self.vector.geometry(Vector2::new(
            self.style.width.unwrap_or(constraints.max.x),
            self.style.height.unwrap_or(constraints.max.y),
        ));

        let mut width = self.style.width.unwrap_or(constraints.max.x);
        let mut height = self.style.height.unwrap_or(constraints.max.y);
        // Without an explicit size the node is as big as what it draws, rather than
        // filling the space it is offered - which is what makes
        // `vector().shape(circle(...))` sit in a flex row without a `.width()`.
        if self.style.width.is_none() {
            width = geometry.bounds.width;
        }
        if self.style.height.is_none() {
            height = geometry.bounds.height;
        }

        let box_id = tree.insert(LayoutBox {
            bounds: Rectangle::new(0.0, 0.0, width, height),
            margin: self.style.margin,
            children: Vec::new(),
            // `then_some` gives `Some` when its condition holds, and the condition
            // here is that there is something to draw.
            material: (!geometry.is_empty()).then_some(Material::Vector(Rc::new(geometry))),
            // The identity, deliberately: `transform()` above already handed the
            // transform to the parent, which applies it to this very box.
            transform: Matrix3::identity(),
        });

        SizedLayoutBox {
            margin: self.style.margin,
            size: Vector2::new(width, height),
            children: vec![box_id],
            // No material here. The parent's `LayoutBox::from_child` copies this
            // onto its own box, and a material on both would draw twice.
            material: None,
            transform: self.style.transform,
        }
    }
}

/// `vector()`, for symmetry with `div()` and `text()`.
#[must_use]
pub fn vector() -> Vector {
    Vector::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape;
    use xs_core::{Render, TextSystemDummy};
    use xs_layout::Color;

    fn layout_of(view: &VectorView, constraints: BoxConstraints) -> (SizedLayoutBox, LayoutTree) {
        let mut tree = LayoutTree::new();
        let sized = view.layout(&mut tree, &constraints);
        (sized, tree)
    }

    fn view_of(vector: &Vector) -> VectorView {
        VectorView {
            style: vector.style.clone(),
            vector: vector.clone(),
        }
    }

    fn material_of(sized: &SizedLayoutBox, tree: &LayoutTree) -> Option<Rc<VectorGeometry>> {
        let box_id = sized.children.first()?;
        match tree.get(*box_id)?.material.as_ref()? {
            Material::Vector(geometry) => Some(Rc::clone(geometry)),
            other => panic!("expected a vector material, got {other:?}"),
        }
    }

    #[test]
    fn a_circle_without_a_size_is_as_big_as_its_own_geometry() {
        let vector = vector().shape(shape::circle(50.0, 50.0, 40.0).fill(Color::RED));
        let view = view_of(&vector);
        let (sized, tree) = layout_of(&view, BoxConstraints::from_max(Vector2::new(800.0, 600.0)));
        // The geometry's bounds are 10..90, and a circle flattens to *just* inside
        // its radius, so the box is a shade under 80.
        assert!(
            (sized.size.x - 80.0).abs() < 1.0,
            "width was {}",
            sized.size.x
        );
        assert!(
            (sized.size.y - 80.0).abs() < 1.0,
            "height was {}",
            sized.size.y
        );
        assert!(material_of(&sized, &tree).is_some());
    }

    /// An explicit size has to win over the geometry's own, or a shape in a fixed
    /// slot would not fit the slot.
    #[test]
    fn an_explicit_size_wins_over_the_geometrys_bounds() {
        let vector = vector()
            .width(120.0)
            .height(30.0)
            .shape(shape::circle(50.0, 50.0, 40.0).fill(Color::RED));
        let (sized, _) = layout_of(
            &view_of(&vector),
            BoxConstraints::from_max(Vector2::new(800.0, 600.0)),
        );
        assert_eq!(sized.size, Vector2::new(120.0, 30.0));
    }

    /// A stroke's box has to contain the pen, not just the centreline.
    #[test]
    fn a_stroked_shape_is_as_big_as_its_outline() {
        let vector = vector().shape(shape::circle(50.0, 50.0, 40.0).stroke(Color::WHITE, 10.0));
        let (sized, _) = layout_of(
            &view_of(&vector),
            BoxConstraints::from_max(Vector2::new(800.0, 600.0)),
        );
        // 40 either side of the centre, plus 5 of pen each way.
        assert!(
            (sized.size.x - 90.0).abs() < 1.5,
            "width was {}",
            sized.size.x
        );
    }

    /// The material has to reach exactly one box. Two would draw the stroke twice,
    /// which for an antialiased edge is a visibly darker line.
    #[test]
    fn the_material_reaches_exactly_one_box() {
        let vector = vector().shape(shape::circle(50.0, 50.0, 40.0).fill(Color::RED));
        let (sized, tree) = layout_of(
            &view_of(&vector),
            BoxConstraints::from_max(Vector2::new(800.0, 600.0)),
        );
        assert!(
            sized.material.is_none(),
            "the parent must not carry it as well"
        );
        let boxes_with_material = tree
            .boxes
            .iter()
            .filter(|b| matches!(b.material, Some(Material::Vector(_))))
            .count();
        assert_eq!(boxes_with_material, 1, "expected exactly one vector box");
    }

    /// The transform is applied once. Reported to the parent, carried as the
    /// identity on the box itself - if either half were missing, or both applied,
    /// the shape would be rotated twice or not at all.
    #[test]
    fn the_transform_is_applied_exactly_once() {
        let vector = vector()
            .shape(shape::circle(50.0, 50.0, 40.0).fill(Color::RED))
            .rotate(0.5);
        let view = view_of(&vector);
        assert_eq!(view.transform(), Matrix3::rotation(0.5));
        let (_, tree) = layout_of(&view, BoxConstraints::from_max(Vector2::new(800.0, 600.0)));
        let box_id = tree.boxes.len() - 1;
        assert_eq!(
            tree.boxes[box_id].transform,
            Matrix3::identity(),
            "the box must not carry it too"
        );
    }

    #[test]
    fn a_shape_with_nothing_to_draw_has_no_material() {
        let vector = vector().width(50.0).height(50.0);
        let (sized, tree) = layout_of(
            &view_of(&vector),
            BoxConstraints::from_max(Vector2::new(800.0, 600.0)),
        );
        assert!(sized.material.is_none());
        assert!(material_of(&sized, &tree).is_none());
    }

    /// A shape with nothing to draw has no geometry, and a node sized to its
    /// geometry therefore has nothing to size itself to.
    ///
    /// This is the contract, and it is worth being explicit about because it is the
    /// one case where "the box is as big as what it draws" bites: an element whose
    /// fill animates to fully transparent collapses. Giving the node an explicit
    /// size - or a `view_box` - pins the box, which is what an app that animates
    /// opacity wants and what CSS's `width: auto` on an empty box does too.
    #[test]
    fn an_invisible_shape_has_no_geometry_and_so_no_size() {
        let vector = vector().shape(shape::circle(50.0, 50.0, 40.0).fill(Color::TRANSPARENT));
        let (sized, tree) = layout_of(
            &view_of(&vector),
            BoxConstraints::from_max(Vector2::new(800.0, 600.0)),
        );
        assert_eq!(sized.size, Vector2::zero());
        assert!(material_of(&sized, &tree).is_none());
    }

    /// ...and an explicit size is the way out of it, which is why this is a
    /// contract rather than a bug.
    #[test]
    fn an_explicit_size_holds_an_invisible_shape_open() {
        let vector = vector()
            .width(80.0)
            .height(80.0)
            .shape(shape::circle(50.0, 50.0, 40.0).fill(Color::TRANSPARENT));
        let (sized, _) = layout_of(
            &view_of(&vector),
            BoxConstraints::from_max(Vector2::new(800.0, 600.0)),
        );
        assert_eq!(sized.size, Vector2::new(80.0, 80.0));
    }

    /// The view box is SVG's, and the scale has to be the smaller ratio or the
    /// shape is distorted.
    #[test]
    /// The scale comes from the view box, not from the shape: the old
    /// implementation divided the node's size by the shape's own extent and read
    /// nothing at all out of `view_box`'s width and height, so all three of these
    /// were the same call.
    #[test]
    fn a_view_box_sizes_the_artwork_and_not_the_artwork_sizes_the_box() {
        let fits = |view: (f32, f32)| {
            vector()
                .width(200.0)
                .height(200.0)
                .view_box(Vector2::zero(), view.0, view.1)
                // 100 units of ink centred in a 200 unit view box: half of it is
                // margin, and that margin is the thing being tested.
                .shape(shape::rect(50.0, 50.0, 100.0, 100.0).fill(Color::RED))
                .geometry(Vector2::new(200.0, 200.0))
                .bounds
        };
        // A 200 unit view box in a 200px box is 1:1, so the ink lands where it was
        // authored: the 50 unit margin survives.
        assert_eq!(fits((200.0, 200.0)), Rectangle::new(50.0, 50.0, 100.0, 100.0));
        // Half the view box is 2:1.
        assert_eq!(fits((100.0, 100.0)), Rectangle::new(100.0, 100.0, 200.0, 200.0));
        // A wide view box scales by its height - 0.5 - and is centred on the other
        // axis, as SVG does with `xMidYMid meet`.
        assert_eq!(fits((400.0, 200.0)), Rectangle::new(25.0, 75.0, 50.0, 50.0));
    }

    #[test]
    fn a_view_box_maps_its_own_min_corner_and_not_the_origin() {
        // Authored on a grid starting at 20, so the view box's min is not (0, 0).
        let bounds = vector()
            .width(100.0)
            .height(100.0)
            .view_box(Vector2::new(20.0, 30.0), 10.0, 10.0)
            // Exactly fills the view box, so it lands exactly on the node's box.
            .shape(shape::rect(20.0, 30.0, 10.0, 10.0).fill(Color::RED))
            .geometry(Vector2::new(100.0, 100.0))
            .bounds;
        // The min corner lands on the box's min corner and the shape fills it.
        assert_eq!(bounds, Rectangle::new(0.0, 0.0, 100.0, 100.0));
    }

    fn a_degenerate_view_box_produces_nothing_rather_than_nonsense() {
        let vector = vector()
            .width(0.0)
            .height(0.0)
            .view_box(Vector2::zero(), 0.0, 0.0)
            .shape(shape::rect(0.0, 0.0, 10.0, 10.0).fill(Color::RED));
        let geometry = vector.geometry(Vector2::zero());
        assert!(geometry.is_empty() || geometry.bounds.width.is_finite());
    }

    #[test]
    fn an_empty_path_through_the_view_box_is_skipped() {
        let vector = vector()
            .width(100.0)
            .height(100.0)
            .view_box(Vector2::zero(), 0.0, 0.0)
            .shape(Shape::new(Path::new()).fill(Color::RED));
        assert!(vector.geometry(Vector2::new(100.0, 100.0)).is_empty());
    }

    /// Scaling to the display happens once, in `post_render`, alongside the style.
    /// If the pen's width were missed the stroke would look thinner than everything
    /// else at 2x, which is the kind of thing nobody reports as a bug.
    #[test]
    fn post_render_scales_the_style_the_shapes_and_the_pen() {
        use xs_core::{AppContext, TextSystemDummy};
        let mut vector = vector()
            .width(100.0)
            .shape(shape::line(0.0, 0.0, 50.0, 0.0).stroke(Color::WHITE, 4.0));
        let mut app_context = AppContext::new(Box::new(TextSystemDummy::default()));
        app_context.screen.scale = 2.0;
        let mut ctx = Context::new(
            &mut app_context,
            (TypeId::of::<Vector>(), 0, 0),
            0,
            xs_layout::TextProperties::default(),
            Color::WHITE,
            BoxConstraints::from_max(Vector2::new(1920.0, 1080.0)),
        );
        vector.post_render(&mut ctx);
        assert_eq!(vector.style.width, Some(200.0));
        assert_eq!(
            vector.shapes[0].stroke.as_ref().unwrap().width,
            8.0,
            "the pen is a length and has to scale"
        );
        assert_eq!(
            vector.shapes[0].path.segments[1].end().unwrap().x,
            100.0,
            "the path has to be in device pixels too"
        );
    }

    #[test]
    fn post_render_pushes_the_size_into_the_constraints() {
        use xs_core::{AppContext, TextSystemDummy};
        let mut vector = vector().width(64.0).height(32.0);
        let mut app_context = AppContext::new(Box::new(TextSystemDummy::default()));
        let mut ctx = Context::new(
            &mut app_context,
            (TypeId::of::<Vector>(), 0, 0),
            0,
            xs_layout::TextProperties::default(),
            Color::WHITE,
            BoxConstraints::from_max(Vector2::new(1920.0, 1080.0)),
        );
        vector.post_render(&mut ctx);
        assert_eq!(ctx.constraints.max, Vector2::new(64.0, 32.0));
    }

    #[test]
    fn an_absolute_vector_paints_a_positioned_view() {
        let vector = vector()
            .absolute()
            .left(20.0)
            .top(30.0)
            .shape(shape::circle(50.0, 50.0, 40.0).fill(Color::RED));
        let mut app_context = AppContext::new(Box::new(TextSystemDummy::default()));
        let view = vector.paint(&mut app_context, &[]);
        let mut tree = LayoutTree::new();
        let sized = view.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(800.0, 600.0)),
        );
        // The outer box sits at the requested corner; the inner one holds the
        // material at the origin.
        let outer = tree.get(sized.children[0]).unwrap();
        assert_eq!((outer.bounds.x, outer.bounds.y), (20.0, 30.0));
        assert!(outer.material.is_none());
        let inner = tree.get(outer.children[0]).unwrap();
        assert!(matches!(inner.material, Some(Material::Vector(_))));
    }

    /// A path has no interior, so a child could only ever be silently dropped.
    /// Saying so at the call site is worth more than quietly ignoring it.
    #[test]
    #[should_panic(expected = "no children")]
    fn a_vector_refuses_children() {
        let _ = vector().child(Nothing);
    }

    struct Nothing;

    impl Render for Nothing {
        fn render(&self, _ctx: &mut Context) -> impl IntoElement {
            vector()
        }
    }
}
