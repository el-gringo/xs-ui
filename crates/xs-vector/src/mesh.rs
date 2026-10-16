//! Paths to the vertex data the renderer uploads.
//!
//! This is where a [`Shape`] stops being geometry and becomes a
//! [`VectorGeometry`]: flatten, dash, expand into capsules, triangulate the fill,
//! and measure the result. Nothing after this point knows what a path is.
//!
//! Everything here works in the element's own pixels and stays there. The node's
//! transform is applied by the render driver, because a transformed node's matrix
//! is not known until the frame walks the tree - the same reason `TextVertices`
//! holds glyph quads relative to the run's origin rather than on screen.

use xs_layout::{Color, FILL_VERTEX_FLOATS, STROKE_VERTEX_FLOATS, VectorGeometry};
use xs_math::{Rectangle, Vector2};

use crate::fill::triangulate;
use crate::flatten::{Contour, DEFAULT_TOLERANCE, flatten};
use crate::path::Path;
use crate::stroke::{Stroke, emit_stroke};

/// How a shape paints itself.
///
/// A shape with neither paints nothing, which is a legitimate thing to ask for
/// while a stroke animates in from zero width.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fill {
    pub color: Color,
}

impl Fill {
    #[must_use]
    pub fn new(color: Color) -> Self {
        Self { color }
    }

    #[must_use]
    pub fn is_visible(self) -> bool {
        self.color.a > 0.0
    }
}

/// A path and how it paints.
///
/// Several shapes go into one [`Vector::shapes`](crate::Vector::shapes) list and
/// become one node, one layout box and - because they all land in the same frame
/// arena - potentially one draw call. That is why a shape carries no position of
/// its own: everything is in the node's coordinates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shape {
    pub path: Path,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    /// Where the dash pattern starts, in the same units as the pattern's first
    /// entry. Mostly useful with an animation, to walk a dash along a line.
    pub dash_offset: f32,
}

impl Shape {
    /// A bare path with no paint yet.
    ///
    /// The styling methods - `fill`, `stroke`, `dash`, `cap` - live in
    /// [`crate::shape`] and are the only way to paint a shape, so there is one
    /// obvious route from a path to something that draws.
    #[must_use]
    pub fn new(path: Path) -> Self {
        Self {
            path,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn with_dash_offset(mut self, offset: f32) -> Self {
        self.dash_offset = offset;
        self
    }

    /// Whether this shape can produce any geometry at all.
    ///
    /// Worth asking before flattening: a stroke animating its width up from zero is
    /// invisible for the first few frames, and there is no reason to walk a path
    /// that is going to produce nothing.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.path.is_empty()
            || (self.fill.is_some_and(Fill::is_visible)
                || self.stroke.as_ref().is_some_and(Stroke::is_visible))
    }
}

/// Turns shapes into the vertex data a node draws.
///
/// The result is a plain value in the node's own pixels. Nothing here touches GL,
/// which is what lets the whole geometry pipeline be tested on the host.
#[derive(Default)]
pub struct MeshBuilder {
    geometry: VectorGeometry,
    tolerance: f32,
    /// Reused across shapes so a node with many shapes does not allocate a
    /// contour list per shape per frame.
    scratch: Vec<Contour>,
}

impl MeshBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tolerance: DEFAULT_TOLERANCE,
            ..Default::default()
        }
    }

    /// The flattening tolerance, in the node's own pixels.
    ///
    /// This is the one knob that trades work for smoothness across the whole
    /// pipeline: it sets how finely curves become polylines, and therefore how many
    /// capsules a stroke costs and how many points the fill's ear clipper walks.
    #[must_use]
    pub fn tolerance(mut self, tolerance: f32) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Adds one shape, flattened and expanded.
    pub fn shape(&mut self, shape: &Shape) -> &mut Self {
        if !shape.is_visible() {
            return self;
        }
        let stroke = shape.stroke.as_ref().filter(|s| s.is_visible());
        let fill = shape.fill.filter(|f| f.is_visible());
        if stroke.is_none() && fill.is_none() {
            return self;
        }

        self.scratch.clear();
        self.scratch.extend(flatten(&shape.path, self.tolerance));

        if let Some(stroke) = stroke {
            for contour in &self.scratch {
                emit_stroke(
                    contour,
                    stroke,
                    shape.dash_offset,
                    &mut self.geometry.stroke_vertices,
                );
            }
        }
        if let Some(fill) = fill {
            let triangles = triangulate(&self.scratch);
            let color: [f32; 4] = fill.color.into();
            for vertex in triangles {
                self.geometry.fill_vertices.push(vertex.x);
                self.geometry.fill_vertices.push(vertex.y);
                self.geometry.fill_vertices.extend_from_slice(&color);
            }
        }
        self
    }

    pub fn shapes(&mut self, shapes: &[Shape]) -> &mut Self {
        for shape in shapes {
            self.shape(shape);
        }
        self
    }

    /// The bounds of everything added, in local pixels, the stroke's half width
    /// included.
    ///
    /// A stroke's geometry sits *outside* its centreline, so a node sized to the
    /// path's own bounds would have its own outline clipped away by whatever
    /// ancestor clips. Including the half width is what keeps a stroke inside its
    /// layout box.
    pub fn bounds(&self) -> Rectangle {
        let mut bounds = self.geometry.bounds;
        for vertex in self.geometry.fill_vertices.chunks(FILL_VERTEX_FLOATS) {
            let point = Vector2::new(vertex[0], vertex[1]);
            bounds = union(bounds, point);
        }
        for vertex in self.geometry.stroke_vertices.chunks(STROKE_VERTEX_FLOATS) {
            let half_width = vertex[4];
            bounds = union(
                bounds,
                Vector2::new(vertex[0] - half_width, vertex[1] - half_width),
            );
            bounds = union(
                bounds,
                Vector2::new(vertex[2] + half_width, vertex[3] + half_width),
            );
        }
        bounds
    }

    /// The finished geometry, bounds included.
    #[must_use]
    pub fn build(mut self) -> VectorGeometry {
        self.geometry.bounds = self.bounds();
        self.geometry
    }
}

fn union(into: Rectangle, point: Vector2) -> Rectangle {
    if into.width <= 0.0 && into.height <= 0.0 && into.x == 0.0 && into.y == 0.0 {
        // Nothing has been added yet, so this is the first point rather than a
        // degenerate rectangle at the origin.
        return Rectangle::new(point.x, point.y, 0.0, 0.0);
    }
    let max_x = (into.x + into.width).max(point.x);
    let max_y = (into.y + into.height).max(point.y);
    let min_x = into.x.min(point.x);
    let min_y = into.y.min(point.y);
    Rectangle::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape;
    use xs_layout::Color;

    fn bounds_of(geometry: &VectorGeometry) -> Rectangle {
        geometry.bounds
    }

    #[test]
    fn a_filled_square_produces_two_triangles() {
        let mut builder = MeshBuilder::new();
        builder.shape(&shape::rect(0.0, 0.0, 10.0, 10.0).fill(Color::RED));
        let geometry = builder.build();
        assert_eq!(geometry.fill_triangle_count(), 2);
        assert!(geometry.stroke_vertices.is_empty());
    }

    #[test]
    fn a_stroked_line_produces_one_segment() {
        let mut builder = MeshBuilder::new();
        builder.shape(&shape::line(0.0, 0.0, 10.0, 0.0).stroke(Color::WHITE, 2.0));
        let geometry = builder.build();
        assert_eq!(geometry.stroke_segment_count(), 1);
        assert!(geometry.fill_vertices.is_empty());
    }

    /// The node's box has to contain the stroke, not just the centreline, or an
    /// ancestor's `overflow: hidden` eats half the outline.
    #[test]
    fn the_bounds_include_the_pens_half_width() {
        let mut builder = MeshBuilder::new();
        builder.shape(&shape::line(10.0, 10.0, 20.0, 10.0).stroke(Color::WHITE, 6.0));
        let bounds = bounds_of(&builder.build());
        assert_eq!(bounds.x, 7.0, "3px of pen to the left of the start");
        assert_eq!(bounds.width, 16.0, "10px of line plus 3px each end");
        assert_eq!(bounds.y, 7.0);
        assert_eq!(bounds.height, 6.0);
    }

    #[test]
    fn the_bounds_of_a_fill_are_the_path_bounds() {
        let mut builder = MeshBuilder::new();
        builder.shape(&shape::rect(2.0, 4.0, 10.0, 20.0).fill(Color::RED));
        let bounds = bounds_of(&builder.build());
        assert_eq!((bounds.x, bounds.y), (2.0, 4.0));
        assert_eq!((bounds.width, bounds.height), (10.0, 20.0));
    }

    #[test]
    fn an_empty_builder_has_empty_geometry() {
        let geometry = MeshBuilder::new().build();
        assert!(geometry.is_empty());
        assert_eq!(geometry.fill_triangle_count(), 0);
        assert_eq!(geometry.stroke_segment_count(), 0);
    }

    #[test]
    fn an_invisible_shape_costs_nothing() {
        let mut builder = MeshBuilder::new();
        builder.shape(&shape::rect(0.0, 0.0, 10.0, 10.0).fill(Color::TRANSPARENT));
        builder.shape(&shape::rect(0.0, 0.0, 10.0, 10.0).stroke(Color::WHITE, 0.0));
        assert!(builder.build().is_empty());
    }

    #[test]
    fn an_empty_path_costs_nothing() {
        let mut builder = MeshBuilder::new();
        builder.shape(&Shape::new(Path::new()).fill(Color::RED));
        assert!(builder.build().is_empty());
    }

    /// Several shapes into one node is the point of the builder: one node, and the
    /// renderer batches them into the same arena.
    #[test]
    fn several_shapes_land_in_one_geometry() {
        let mut builder = MeshBuilder::new();
        builder.shapes(&[
            shape::rect(0.0, 0.0, 10.0, 10.0).fill(Color::RED),
            shape::rect(20.0, 0.0, 10.0, 10.0).fill(Color::BLUE),
            shape::line(0.0, 20.0, 30.0, 20.0).stroke(Color::WHITE, 2.0),
        ]);
        let geometry = builder.build();
        assert_eq!(geometry.fill_triangle_count(), 4);
        assert_eq!(geometry.stroke_segment_count(), 1);
        // 30 long, plus half the pen at each end.
        assert_eq!(bounds_of(&geometry).width, 32.0);
    }

    /// The tighter the tolerance, the more capsules a curve costs. This is the
    /// knob the docs point at, so it has to actually turn.
    #[test]
    fn the_tolerance_changes_how_much_geometry_a_curve_costs() {
        let path = shape::circle(100.0, 100.0, 100.0);
        let count = |tolerance: f32| {
            let mut builder = MeshBuilder::new().tolerance(tolerance);
            builder.shape(&path.clone().stroke(Color::WHITE, 2.0));
            builder.build().stroke_segment_count()
        };
        assert!(
            count(0.05) > count(4.0),
            "{} vs {}",
            count(0.05),
            count(4.0)
        );
    }

    #[test]
    fn a_shape_can_be_filled_and_stroked_at_once() {
        let mut builder = MeshBuilder::new();
        builder.shape(
            &shape::rect(0.0, 0.0, 10.0, 10.0)
                .fill(Color::RED)
                .stroke(Color::WHITE, 2.0),
        );
        let geometry = builder.build();
        assert_eq!(geometry.fill_triangle_count(), 2);
        assert_eq!(geometry.stroke_segment_count(), 4);
    }

    #[test]
    fn a_circle_has_a_hole_when_it_is_only_stroked() {
        let mut builder = MeshBuilder::new();
        builder.shape(&shape::circle(50.0, 50.0, 40.0).stroke(Color::WHITE, 3.0));
        let geometry = builder.build();
        assert!(geometry.fill_vertices.is_empty());
        assert!(geometry.stroke_segment_count() > 12);
    }

    #[test]
    fn a_donut_subtracts_its_hole() {
        let mut builder = MeshBuilder::new();
        builder.shape(
            &Shape::new(shape::circle(50.0, 50.0, 40.0).path)
                .fill(Color::RED)
                .with_hole(shape::circle(50.0, 50.0, 20.0).path),
        );
        let geometry = builder.build();
        // A ring needs at least the two contours' worth of triangles.
        assert!(geometry.fill_triangle_count() >= 8);
        assert!(!geometry.is_empty());
    }

    #[test]
    fn a_path_is_rebuilt_the_same_way_twice() {
        let build = || {
            let mut builder = MeshBuilder::new();
            builder.shape(&shape::star(5, 50.0, 50.0, 40.0, 16.0).fill(Color::RED));
            builder.build()
        };
        // The builder is rebuilt from the path on every frame, so two builds of the
        // same shape have to be identical or the node's material would differ from
        // one frame to the next for no reason.
        assert_eq!(build(), build());
    }
}
