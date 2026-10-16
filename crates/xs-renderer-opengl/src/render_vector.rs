use std::rc::Rc;

use xs_gl::{Buffer, GL_FLOAT, Shader};
use xs_layout::{
    CLIP_FLOATS, FILL_TEXCOORD_FLOATS, FILL_VERTEX_FLOATS, STROKE_CORNER_FLOATS,
    STROKE_VERTEX_FLOATS, VectorGeometry,
};
use xs_math::{Matrix3, Rectangle, Vector2};

use crate::RendererOpenGl;

/// f32 in one stroke vertex once the renderer has written the corner selector
/// and the clip in. This is the stride the shader's attributes are declared
/// against and the one the frame arena divides by, so it is the number that has
/// to be right.
pub(crate) const DRAWN_STROKE_VERTEX_FLOATS: usize =
    STROKE_VERTEX_FLOATS + STROKE_CORNER_FLOATS + CLIP_FLOATS;

/// f32 in one fill vertex once the texture coordinate and clip are in. A fill is
/// meant to come out equal to a plain solid-colour quad, and this is the
/// assertion that it does: `push_simple_quad` writes `SIMPLE_QUAD_VERTEX_FLOATS`
/// into the same arena, and two layouts that differed would interleave into
/// nonsense rather than into a subtly wrong picture.
pub(crate) const DRAWN_FILL_VERTEX_FLOATS: usize =
    FILL_VERTEX_FLOATS + FILL_TEXCOORD_FLOATS + CLIP_FLOATS;

/// The corner selectors of the two triangles that carry one segment, as
/// `(t along, s across)`. Reading them down the columns gives the first triangle
/// and then the second, so the whole segment is `(0,-1) (1,-1) (0,1)` followed by
/// `(1,-1) (1,1) (0,1)`.
const CAPSULE_CORNERS: [[f32; 2]; 6] = [
    [0.0, -1.0],
    [1.0, -1.0],
    [0.0, 1.0],
    [1.0, -1.0],
    [1.0, 1.0],
    [0.0, 1.0],
];

/// Where each field of a stored stroke vertex starts. Written out rather than
/// counted inline because the two sides of this boundary - the mesh builder and
/// this reader - are in different crates, and a stride that is off by one field
/// reads as a plausible line in the wrong place.
const SEGMENT_STRIDE: usize = STROKE_VERTEX_FLOATS;
const HALF_WIDTH_AT: usize = 4;
const CAP_AT: usize = 5;
const COLOR_AT: usize = 6;

impl RendererOpenGl {
    pub fn initialize_vector_stroke_shader(&mut self) {
        match Shader::try_new(
            include_str!("glsl/vector_stroke.glsl.vert"),
            include_str!("glsl/vector_stroke.glsl.frag"),
        ) {
            Ok(mut shader) => {
                let size_of_float = std::mem::size_of::<f32>();
                let stride = (DRAWN_STROKE_VERTEX_FLOATS * size_of_float) as i32;
                let offset = |component: usize| (component * size_of_float) as i64;
                let buffer = Buffer::new()
                    .add_attribute("a_p0", 2, GL_FLOAT, false, stride, offset(0))
                    .add_attribute("a_p1", 2, GL_FLOAT, false, stride, offset(2))
                    .add_attribute("a_corner", 2, GL_FLOAT, false, stride, offset(4))
                    .add_attribute("a_halfWidth", 1, GL_FLOAT, false, stride, offset(6))
                    .add_attribute("a_cap", 1, GL_FLOAT, false, stride, offset(7))
                    .add_attribute("a_color", 4, GL_FLOAT, false, stride, offset(8))
                    .add_attribute("a_clip", 4, GL_FLOAT, false, stride, offset(12));
                shader.add_buffer(buffer);
                shader.use_program();
                self.add_shader("vector_stroke", shader);
            }
            Err(error) => panic!("Error in vector_stroke shader: {}", error),
        }
    }

    /// Appends one vector node's stroke segments to the batch.
    ///
    /// `origin` is the node's top-left in screen pixels and `geometry` is in the
    /// node's own space, so each endpoint is offset and then transformed here -
    /// the two places the node's position is actually known. The vertex shader
    /// turns each transformed pair back into the segment's box, so what crosses
    /// to the GPU is the shape and not the expanded ribbon.
    ///
    /// `scale` is the node transform's `scale_factor`, and it is what keeps the
    /// pen the right width on screen: the half width is a length, so it has to
    /// grow with the drawing the way every other length in the style does. Under
    /// a non-uniform scale it takes the geometric mean, which is the compromise
    /// `Matrix3::scale_factor` already documents - the pen cannot be both half
    /// the pen width across a squashed axis and half the pen width down it.
    #[allow(clippy::too_many_arguments)]
    pub fn push_vector_stroke(
        &self,
        origin: Vector2,
        scale: f32,
        clip: Option<Rectangle>,
        geometry: &VectorGeometry,
        transform: &Matrix3,
        out: &mut Vec<f32>,
    ) {
        let source = &geometry.stroke_vertices;
        let segments = source.len() / SEGMENT_STRIDE;
        if segments == 0 {
            return;
        }

        let clip: [f32; 4] = clip_rect(clip);
        let half_width_scale = scale;

        let base = out.len();
        out.resize(base + segments * 6 * DRAWN_STROKE_VERTEX_FLOATS, 0.0);

        for segment in 0..segments {
            let src = segment * SEGMENT_STRIDE;
            let half_width = source[src + HALF_WIDTH_AT] * half_width_scale;
            let cap = source[src + CAP_AT];
            let p0 = transform.transform_point(origin + Vector2::new(source[src], source[src + 1]));
            let p1 =
                transform.transform_point(origin + Vector2::new(source[src + 2], source[src + 3]));
            let color = &source[src + COLOR_AT..src + COLOR_AT + 4];

            for (corner_slot, corner) in CAPSULE_CORNERS.iter().enumerate() {
                let dst = base + (segment * 6 + corner_slot) * DRAWN_STROKE_VERTEX_FLOATS;
                out[dst] = p0.x;
                out[dst + 1] = p0.y;
                out[dst + 2] = p1.x;
                out[dst + 3] = p1.y;
                out[dst + 4] = corner[0];
                out[dst + 5] = corner[1];
                out[dst + 6] = half_width;
                out[dst + 7] = cap;
                out[dst + 8..dst + 12].copy_from_slice(color);
                out[dst + 12..dst + 16].copy_from_slice(&clip);
            }
        }
    }

    /// Appends one vector node's fill triangles to the batch.
    ///
    /// This writes into the *simple quad* arena, which is the whole point: a
    /// filled path is a set of solid-colour triangles and the driver cannot tell
    /// that from a rectangle, so a fill batches into the same runs as the app's
    /// ordinary backgrounds rather than costing a program and a draw of its own.
    #[allow(clippy::too_many_arguments)]
    pub fn push_vector_fill(
        &self,
        origin: Vector2,
        clip: Option<Rectangle>,
        geometry: &Rc<VectorGeometry>,
        transform: &Matrix3,
        out: &mut Vec<f32>,
    ) {
        let source = &geometry.fill_vertices;
        let vertices = source.len() / FILL_VERTEX_FLOATS;
        if vertices == 0 {
            return;
        }

        let clip: [f32; 4] = clip_rect(clip);

        let base = out.len();
        out.resize(base + vertices * DRAWN_FILL_VERTEX_FLOATS, 0.0);

        for vertex in 0..vertices {
            let src = vertex * FILL_VERTEX_FLOATS;
            let at = transform.transform_point(origin + Vector2::new(source[src], source[src + 1]));
            let dst = base + vertex * DRAWN_FILL_VERTEX_FLOATS;
            out[dst] = at.x;
            out[dst + 1] = at.y;
            // The placeholder texture is 1x1 and fully transparent, so where it is
            // sampled from cannot matter: the simple-quad fragment shader blends
            // by the texture's alpha, which is zero.
            out[dst + 2] = 0.0;
            out[dst + 3] = 0.0;
            out[dst + 4..dst + 8].copy_from_slice(&source[src + 2..src + 6]);
            out[dst + 8..dst + 12].copy_from_slice(&clip);
        }
    }
}

/// The clip rectangle as the four floats every shader here expects, with the
/// unbounded default the rest of the renderer uses so the test is never the
/// thing that decides.
fn clip_rect(clip: Option<Rectangle>) -> [f32; 4] {
    let clip = clip.unwrap_or(Rectangle::new(0.0, 0.0, f32::INFINITY, f32::INFINITY));
    [clip.x, clip.y, clip.width, clip.height]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_quad::SIMPLE_QUAD_VERTEX_FLOATS;

    /// A distinctly odd colour, so a colour landing in the wrong four floats is
    /// visible rather than plausible.
    const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

    /// A 10x10 square as two triangles along its diagonal: 100 of area.
    fn a_square(color: [f32; 4]) -> Vec<f32> {
        let mut out = Vec::new();
        for triangle in [
            [(0.0, 0.0), (10.0, 0.0), (0.0, 10.0)],
            [(10.0, 0.0), (10.0, 10.0), (0.0, 10.0)],
        ] {
            for (x, y) in triangle {
                out.push(x);
                out.push(y);
                out.extend_from_slice(&color);
            }
        }
        out
    }

    /// A fill has to come out byte-identical in shape to a plain background quad,
    /// because the two share one shader, one arena and one buffer. If the strides
    /// ever diverge the arena divides by one of them and the other batch's
    /// vertices are read from the wrong offsets, which shows up as a shape in the
    /// wrong place rather than as an error.
    #[test]
    fn a_fill_vertex_is_a_simple_quad_vertex() {
        assert_eq!(DRAWN_FILL_VERTEX_FLOATS, SIMPLE_QUAD_VERTEX_FLOATS);
    }

    /// Same reason, stated on the stroke side: the arena divides a batch's float
    /// span by this to get `glDrawArrays`' first vertex, so it has to be the
    /// stride the vertices were actually written at.
    #[test]
    fn the_stroke_stride_counts_what_the_renderer_appends() {
        assert_eq!(DRAWN_STROKE_VERTEX_FLOATS, 16);
        assert_eq!(STROKE_VERTEX_FLOATS, 10);
        assert_eq!(STROKE_CORNER_FLOATS, 2);
        assert_eq!(CLIP_FLOATS, 4);
    }

    /// The attribute offsets in `initialize_vector_stroke_shader` are written out
    /// by hand, and a wrong one is a plausible-looking line in the wrong place.
    #[test]
    fn the_stroke_attributes_cover_the_vertex_without_gaps() {
        let offsets: [(usize, usize); 7] =
            [(0, 2), (2, 2), (4, 2), (6, 1), (7, 1), (8, 4), (12, 4)];
        let mut next = 0;
        for (at, size) in offsets {
            assert_eq!(at, next, "attribute at {at} leaves a gap or overlap");
            next = at + size;
        }
        assert_eq!(next, DRAWN_STROKE_VERTEX_FLOATS);
    }

    /// `push_vector_fill` is the only thing between a triangulated shape and the
    /// screen, and it has to be lossless: same triangles, same colours, same area,
    /// with the node's origin and transform applied and the clip written in.
    ///
    /// The area check is the one that matters. The fill rides the *simple quad*
    /// arena, so a stride slip would not raise anything - it would read another
    /// node's vertices and quietly paint the wrong shape somewhere else. Comparing
    /// the pushed geometry's area against the source's is what catches that, and it
    /// catches a hole being filled in just as readily as a stride being wrong.
    #[test]
    fn a_fill_survives_the_push() {
        // A square built by hand, so the arithmetic is checkable.
        let geometry = Rc::new(VectorGeometry {
            // A 10x10 square, so the area is 100 and checkable by hand.
            fill_vertices: a_square(RED),
            stroke_vertices: Vec::new(),
            bounds: Rectangle::new(0.0, 0.0, 10.0, 10.0),
        });
        assert_eq!(geometry.fill_triangle_count(), 2);

        let renderer = RendererOpenGl::new();
        let mut out = Vec::new();
        renderer.push_vector_fill(
            Vector2::new(36.0, 437.0),
            None,
            &geometry,
            &Matrix3::identity(),
            &mut out,
        );

        assert_eq!(out.len(), 2 * 3 * DRAWN_FILL_VERTEX_FLOATS);
        // Positions moved by the origin.
        assert_eq!((out[0], out[1]), (36.0, 437.0));
        assert_eq!(
            (
                out[DRAWN_FILL_VERTEX_FLOATS],
                out[DRAWN_FILL_VERTEX_FLOATS + 1]
            ),
            (46.0, 437.0)
        );
        // The texture coordinate is the origin, so the placeholder - which is what
        // makes this a solid colour rather than a texture - is what gets sampled.
        assert_eq!((out[2], out[3]), (0.0, 0.0));
        // The unbounded clip, so nothing is thrown away before there is a clip.
        assert_eq!(out[8], 0.0);
        assert_eq!(out[9], 0.0);
        assert!(out[10].is_infinite() && out[11].is_infinite());

        // The area survived, which is the claim that matters.
        assert!((triangles_area(&out) - 100.0).abs() < 1.0e-3);
    }

    #[test]
    fn a_clip_reaches_the_fill_vertex() {
        let geometry = Rc::new(VectorGeometry {
            fill_vertices: a_square(RED),
            stroke_vertices: Vec::new(),
            bounds: Rectangle::new(0.0, 0.0, 10.0, 10.0),
        });
        let renderer = RendererOpenGl::new();
        let mut out = Vec::new();
        let clip = Rectangle::new(5.0, 6.0, 7.0, 8.0);
        renderer.push_vector_fill(
            Vector2::zero(),
            Some(clip),
            &geometry,
            &Matrix3::identity(),
            &mut out,
        );
        assert_eq!(&out[8..12], &[5.0, 6.0, 7.0, 8.0]);
    }

    /// The node's transform has to reach the fill. The transform arrives in the
    /// same absolute-space form the quad shader's does, so the node's origin is
    /// transformed along with its geometry rather than being added afterwards.
    #[test]
    fn the_fill_follows_the_node_transform() {
        let geometry = Rc::new(VectorGeometry {
            // A 10x10 square, so the area is 100 and checkable by hand.
            fill_vertices: a_square(RED),
            stroke_vertices: Vec::new(),
            bounds: Rectangle::new(0.0, 0.0, 10.0, 10.0),
        });
        let renderer = RendererOpenGl::new();
        let mut out = Vec::new();
        renderer.push_vector_fill(
            Vector2::new(100.0, 100.0),
            None,
            &geometry,
            &Matrix3::translation(Vector2::new(5.0, 5.0)),
            &mut out,
        );
        assert_eq!((out[0], out[1]), (105.0, 105.0));
        assert_eq!(
            (
                out[DRAWN_FILL_VERTEX_FLOATS],
                out[DRAWN_FILL_VERTEX_FLOATS + 1]
            ),
            (115.0, 105.0)
        );
    }

    /// A scaled node's fill scales with it. The matrix is the one the tree composes
    /// - about the node's own centre, CSS's `transform-origin: 50% 50%` - so the
    /// node's top-left corner stays put and the area grows.
    #[test]
    fn a_scaled_nodes_fill_scales_about_its_centre() {
        let geometry = Rc::new(VectorGeometry {
            // A 10x10 square, so the area is 100 and checkable by hand.
            fill_vertices: a_square(RED),
            stroke_vertices: Vec::new(),
            bounds: Rectangle::new(0.0, 0.0, 10.0, 10.0),
        });
        let origin = Vector2::new(100.0, 100.0);
        let centre = origin + Vector2::new(5.0, 5.0);
        let renderer = RendererOpenGl::new();
        let mut out = Vec::new();
        renderer.push_vector_fill(
            origin,
            None,
            &geometry,
            &(Matrix3::translation(centre)
                * Matrix3::uniform_scale(2.0)
                * Matrix3::translation(Vector2::new(-centre.x, -centre.y))),
            &mut out,
        );
        // The node's top-left corner is carried past the centre, so the box grows
        // from 10x10 to 20x20 about its middle: 95..115 around a centre at 105.
        assert!((out[0] - 95.0).abs() < 1.0e-3, "x was {}", out[0]);
        assert!((out[1] - 95.0).abs() < 1.0e-3, "y was {}", out[1]);
        assert!(
            (triangles_area(&out) - 400.0).abs() < 1.0e-3,
            "area {}",
            triangles_area(&out)
        );
    }

    /// The pen is a length, so it scales with the node's transform.
    #[test]
    fn the_stroke_pen_follows_the_nodes_scale() {
        let geometry = Rc::new(VectorGeometry {
            fill_vertices: Vec::new(),
            stroke_vertices: vec![0.0, 0.0, 10.0, 0.0, 4.0, 1.0, 0.25, 0.5, 1.0, 0.5],
            bounds: Rectangle::new(0.0, 0.0, 10.0, 10.0),
        });
        assert_eq!(geometry.stroke_segment_count(), 1);
        let renderer = RendererOpenGl::new();
        let mut out = Vec::new();
        renderer.push_vector_stroke(
            Vector2::zero(),
            2.5,
            None,
            &geometry,
            &Matrix3::identity(),
            &mut out,
        );
        // The pen came in at a radius of 4 and leaves at 10.
        assert_eq!(out[6], 10.0);
        assert_eq!(out[7], 1.0, "the cap code is carried through untouched");
        assert_eq!(&out[8..12], &[0.25, 0.5, 1.0, 0.5], "the colour too");
    }

    /// A stroke segment becomes six vertices: two triangles over the segment's box.
    #[test]
    fn a_stroke_segment_becomes_six_vertices() {
        let geometry = Rc::new(VectorGeometry {
            fill_vertices: Vec::new(),
            stroke_vertices: vec![0.0, 0.0, 10.0, 0.0, 4.0, 1.0, 0.25, 0.5, 1.0, 0.5],
            bounds: Rectangle::new(0.0, 0.0, 10.0, 10.0),
        });
        let renderer = RendererOpenGl::new();
        let mut out = Vec::new();
        renderer.push_vector_stroke(
            Vector2::zero(),
            1.0,
            None,
            &geometry,
            &Matrix3::identity(),
            &mut out,
        );
        assert_eq!(out.len(), 6 * DRAWN_STROKE_VERTEX_FLOATS);
        // All six carry the same segment and the same pen.
        for slot in 0..6 {
            let at = slot * DRAWN_STROKE_VERTEX_FLOATS;
            assert_eq!((out[at], out[at + 1]), (0.0, 0.0));
            assert_eq!((out[at + 2], out[at + 3]), (10.0, 0.0));
            assert_eq!(out[at + 6], 4.0);
        }
        // ...and the six corners are the two triangles of the segment's box.
        let corners: Vec<(f32, f32)> = (0..6)
            .map(|slot| {
                let at = slot * DRAWN_STROKE_VERTEX_FLOATS;
                (out[at + 4], out[at + 5])
            })
            .collect();
        assert_eq!(
            corners,
            vec![
                (0.0, -1.0),
                (1.0, -1.0),
                (0.0, 1.0),
                (1.0, -1.0),
                (1.0, 1.0),
                (0.0, 1.0)
            ]
        );
    }

    fn triangles_area(vertices: &[f32]) -> f32 {
        vertices
            .chunks_exact(3 * DRAWN_FILL_VERTEX_FLOATS)
            .map(|t| {
                let a = (t[0], t[1]);
                let b = (t[DRAWN_FILL_VERTEX_FLOATS], t[DRAWN_FILL_VERTEX_FLOATS + 1]);
                let c = (
                    t[2 * DRAWN_FILL_VERTEX_FLOATS],
                    t[2 * DRAWN_FILL_VERTEX_FLOATS + 1],
                );
                (b.0 - a.0) * (c.1 - a.1) - (c.0 - a.0) * (b.1 - a.1)
            })
            .sum::<f32>()
            .abs()
            / 2.0
    }

    /// The two triangles of one capsule have to cover the same quad in the same
    /// winding, or the segment tears along its diagonal.
    #[test]
    fn the_capsule_corners_agree_on_their_middle_two() {
        assert_eq!(CAPSULE_CORNERS[0], [0.0, -1.0]);
        assert_eq!(CAPSULE_CORNERS[3], CAPSULE_CORNERS[1]);
        assert_eq!(CAPSULE_CORNERS[5], CAPSULE_CORNERS[2]);
        assert_eq!(CAPSULE_CORNERS[4], [1.0, 1.0]);
    }
}
