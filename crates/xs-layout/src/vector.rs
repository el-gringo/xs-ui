use xs_math::Rectangle;

/// f32 per stroke vertex as [`VectorGeometry`] stores it.
///
/// `a_p0(2) a_p1(2) a_halfWidth(1) a_cap(1) a_color(4)`
///
/// `a_p0` and `a_p1` are the two ends of one segment and `a_halfWidth` is the
/// pen's radius, both in **local** pixels - the element's own untransformed
/// space, before the node's transform is applied. They have to be local, not
/// screen, because a transformed node's matrix is not known until the frame
/// walks the tree, whereas the shape is fixed when the element is built.
///
/// The colour is per vertex rather than per material because one node can carry
/// several shapes that each paint their own way.
///
/// Two fields of the drawn vertex are *not* here, and both because the renderer
/// is the only thing that knows them: the corner selector, which is the same six
/// constants for every segment, and the clip rectangle, which is an axis-aligned
/// box in screen space. `TextVertices` leaves the clip out for the same reason.
pub const STROKE_VERTEX_FLOATS: usize = 10;

/// What the renderer inserts per stroke vertex to turn it into the drawn form:
/// `a_corner(2)`, the `(t along, s across)` selector that says which of the four
/// corners of the segment's box this vertex is.
pub const STROKE_CORNER_FLOATS: usize = 2;

/// f32 per fill vertex as [`VectorGeometry`] stores it.
///
/// `a_position(2) a_color(4)`
///
/// A fill is a plain solid-colour triangle, so its drawn form is byte for byte a
/// `Div` with a background and no border - which is why it rides the
/// `simple_quad` shader and its arena instead of needing a program of its own.
/// The renderer adds the texture coordinate and the clip to reach that form.
pub const FILL_VERTEX_FLOATS: usize = 6;

/// What the renderer appends to a fill vertex to make it a simple-quad vertex:
/// `a_texcoord(2) a_clip(4)`. The texture coordinate is always the origin,
/// because the placeholder texture is 1x1 and fully transparent, so
/// `mix(v_color, texColor, texColor.a)` hands back `v_color`.
pub const FILL_TEXCOORD_FLOATS: usize = 2;

/// What the renderer appends to a stroke vertex: `a_clip(4)`.
pub const CLIP_FLOATS: usize = 4;

/// Everything the renderer needs to draw one vector node, in the form the GPU
/// consumes.
///
/// Rebuilt from the path on every frame the element is built, which is the same
/// bargain the rest of the frame makes: `build_element_tree` deep-copies every
/// element before any of this runs, so flattening a polyline is not where a
/// frame is won or lost. Caching it would also need the box size, which is not
/// known until layout runs.
///
/// `bounds` is the local-space extent of everything drawn - the pen's half width
/// included - so a node's layout box can be sized to contain it and a stroke is
/// not clipped by its own element's bounds.
#[derive(PartialEq, Clone, Debug, Default)]
pub struct VectorGeometry {
    /// Interleaved [`STROKE_VERTEX_FLOATS`]-float vertices.
    pub stroke_vertices: Vec<f32>,
    /// Interleaved [`FILL_VERTEX_FLOATS`]-float vertices.
    pub fill_vertices: Vec<f32>,
    /// Local pixels, relative to the element's own top-left corner.
    pub bounds: Rectangle,
}

impl VectorGeometry {
    /// True when there is nothing to draw, which is worth checking before
    /// touching a frame's arenas at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stroke_vertices.is_empty() && self.fill_vertices.is_empty()
    }

    /// The number of stroke segments queued, for tests and for the debug print.
    #[must_use]
    pub fn stroke_segment_count(&self) -> usize {
        self.stroke_vertices.len() / STROKE_VERTEX_FLOATS
    }

    /// The number of fill triangles queued.
    #[must_use]
    pub fn fill_triangle_count(&self) -> usize {
        self.fill_vertices.len() / (FILL_VERTEX_FLOATS * 3)
    }
}
