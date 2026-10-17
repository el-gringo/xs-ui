//! Building a node's shapes into a mesh, and the key that mesh is cached under.
//!
//! The build itself is not much code, but it is the only place in the crate that
//! knows *everything* that decides what a mesh looks like, and the key has to
//! agree with it exactly. If the key forgets one of them - the box size, the view
//! box, the tolerance - then a node that changed would be handed the mesh from
//! before it changed, and the only symptom would be artwork that is subtly wrong.

use std::rc::Rc;

use xs_layout::{Rectangle, VectorGeometry};
use xs_math::Vector2;

use crate::flatten::DEFAULT_TOLERANCE;
use crate::mesh::{MeshBuilder, Shape};

/// Everything that changes a mesh, and nothing that does not.
///
/// Compared by the cache rather than hashed, so a hit is exact: see
/// [`xs_layout::MeshCache`] for why a fingerprint would not do.
///
/// The fields that are *absent* are the interesting half. A node's transform is
/// not here, because a rotation or a scale is applied to the vertices by the
/// renderer rather than baked into them - so a spinning star reuses one mesh
/// rather than rebuilding it every frame. The clip rectangle is not here either,
/// because it is applied per quad at draw time and says nothing about the
/// geometry. The device pixel ratio is not here, but it is not forgotten: it has
/// already been folded into the shapes by `post_render`, so it moves the key by
/// moving what is compared.
#[derive(PartialEq, Debug)]
pub(crate) struct MeshKey {
    pub(crate) shapes: Vec<Shape>,
    pub(crate) view_box: Option<Rectangle>,
    /// `None` is a real value and not "no tolerance": it means the default, and
    /// the default is a number like any other.
    pub(crate) tolerance: Option<f32>,
    pub(crate) size: Vector2,
}

/// Builds the mesh for `shapes` in a box of `size`.
///
/// The entry point for both the cached path in layout and [`Vector::mesh`], which
/// is the same call without the cache.
pub(crate) fn build_mesh(
    shapes: &[Shape],
    view_box: Option<Rectangle>,
    tolerance: Option<f32>,
    size: Vector2,
) -> Rc<VectorGeometry> {
    let mut builder = MeshBuilder::new().tolerance(tolerance.unwrap_or(DEFAULT_TOLERANCE));

    // Mapped out of the view box before the builder sees them, rather than after
    // it has flattened them: the flattening tolerance is in the node's own pixels,
    // so a curve has to be subdivided at the size it will be drawn at, not at the
    // size it was authored at.
    let mut mapped: Vec<Shape> = Vec::with_capacity(shapes.len());
    for shape in shapes {
        mapped.push(match view_box {
            Some(view) => {
                // A view box with no extent has no scale, and a shape with no
                // placement has nothing to draw - so it is dropped rather than
                // drawn somewhere arbitrary.
                let Some(fit) = view_box_fit(&view, size) else {
                    continue;
                };
                Shape {
                    path: shape.path.scaled(fit.scale).translated(fit.offset),
                    ..shape.clone()
                }
            }
            None => shape.clone(),
        });
    }
    builder.shapes(&mapped);
    Rc::new(builder.build())
}

/// How a view box of authored coordinates covers a node box of local pixels.
struct ViewBoxFit {
    /// What one view box unit is worth in local pixels.
    scale: f32,
    /// Applied after the scale, so that `scale * view.min + offset` is the top
    /// left of the node box.
    offset: Vector2,
}

/// Maps a `viewBox` region onto `size`: SVG's `viewBox` with the default
/// `preserveAspectRatio="xMidYMid meet"`.
///
/// The scale comes from the *view box*, not from the shape, which is the whole
/// difference between a view box and scaling artwork to fit. An icon authored on a
/// 24 unit grid keeps the margin baked into those coordinates instead of being
/// scaled until its ink touches the edge. Nothing is distorted, because the
/// smaller ratio wins, and the scaled view box is centred, which leaves bars on
/// the other axis as SVG does.
///
/// A view box with no extent has no meaningful scale and yields nothing. Shapes
/// *outside* the view box are still drawn, where SVG would clip them to the
/// viewport: this framework has no viewport clip of its own, and inventing one
/// here would hide artwork rather than misplace it.
fn view_box_fit(view: &Rectangle, size: Vector2) -> Option<ViewBoxFit> {
    if view.width <= 0.0 || view.height <= 0.0 || size.x <= 0.0 || size.y <= 0.0 {
        return None;
    }
    let scale = (size.x / view.width).min(size.y / view.height);
    // Centre the scaled view box, then take the view box's own origin back out of
    // the translation, so that `p * scale + offset` maps its min corner rather
    // than (0, 0).
    let centred = Vector2::new(
        (size.x - view.width * scale) * 0.5,
        (size.y - view.height * scale) * 0.5,
    );
    Some(ViewBoxFit {
        scale,
        offset: centred - Vector2::new(view.x, view.y) * scale,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape;
    use xs_layout::Color;

    fn square() -> Vec<Shape> {
        vec![shape::rect(0., 0., 10., 10.).fill(Color::RED)]
    }

    fn key(shapes: &[Shape], size: Vector2) -> MeshKey {
        MeshKey { shapes: shapes.to_vec(), view_box: None, tolerance: Some(1.), size }
    }

    /// The key has to move for every input the build reads. A key that missed one
    /// would hand back a mesh built from a different shape, which no test of the
    /// mesh itself would ever catch.
    #[test]
    fn the_key_moves_for_everything_the_build_reads() {
        let red = square();
        let blue = vec![shape::rect(0., 0., 10., 10.).fill(Color::BLUE)];
        let base = key(&red, Vector2::new(50., 50.));

        assert_ne!(base, key(&red, Vector2::new(51., 50.)), "size is not in it");
        assert_ne!(base, key(&blue, Vector2::new(50., 50.)), "shapes are not in it");

        let mut coarser = base.clone_key();
        coarser.tolerance = Some(4.);
        assert_ne!(base, coarser, "tolerance is not in it");

        let mut boxed = base.clone_key();
        boxed.view_box = Some(Rectangle::new(0., 0., 10., 10.));
        assert_ne!(base, boxed, "the view box is not in it");
    }

    impl MeshKey {
        /// The key's own type is private to this crate, so a test cannot spell out
        /// a literal without naming every field twice; this is the cheap way.
        fn clone_key(&self) -> Self {
            MeshKey {
                shapes: self.shapes.clone(),
                view_box: self.view_box,
                tolerance: self.tolerance,
                size: self.size,
            }
        }
    }

    #[test]
    fn the_build_is_the_same_as_it_was_without_a_cache() {
        let mesh = build_mesh(&square(), None, Some(1.), Vector2::new(50., 50.));
        // Two triangles for a square, at the square's own bounds.
        assert_eq!(mesh.fill_triangle_count(), 2);
        assert_eq!(mesh.bounds, Rectangle::new(0., 0., 10., 10.));
    }

    #[test]
    fn a_degenerate_view_box_produces_nothing_rather_than_nonsense() {
        let mesh = build_mesh(
            &square(),
            Some(Rectangle::new(0., 0., 0., 0.)),
            Some(1.),
            Vector2::new(50., 50.),
        );
        assert!(mesh.is_empty());
    }
}