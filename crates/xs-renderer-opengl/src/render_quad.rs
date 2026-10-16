use crate::RendererOpenGl;
use xs_core::Color;
use xs_gl::{Buffer, GL_FLOAT, Shader};
use xs_layout::{Background, QuadProps};
use xs_math::{Matrix3, Rectangle, Vector2};

/// Number of f32 components in one interleaved quad vertex.
pub(crate) const QUAD_VERTEX_FLOATS: usize = 46;
/// Number of f32 components for a single corner (position + texture coords).
const CORNER_FLOATS: usize = 4;
const VERTICES_PER_QUAD: usize = 6;

/// Simple quad: position(2) + texcoord(2) + color(4) + clip(4) = 12 floats per
/// vertex.
///
/// The arithmetic in that sum used to come out at 14, and the stride was built
/// from it. Nothing broke - the attributes only ever describe the first 12
/// floats, so the two extra were dead padding the CPU zeroed and the GPU skipped -
/// but it was 16% of the vertex bandwidth of the shader most of the frame goes
/// through, on hardware where that is not free. A vector fill shares this format
/// exactly, which is what `a_fill_vertex_is_a_simple_quad_vertex` pins down.
pub(crate) const SIMPLE_QUAD_VERTEX_FLOATS: usize = 12;

impl RendererOpenGl {
    pub fn initialize_quad_shader(&mut self) {
        match Shader::try_new(
            include_str!("glsl/quad.glsl.vert"),
            include_str!("glsl/quad.glsl.frag"),
        ) {
            Ok(mut shader) => {
                let size_of_float = std::mem::size_of::<f32>();
                let stride = (QUAD_VERTEX_FLOATS * size_of_float) as i32;
                let offset = |component: usize| (component * size_of_float) as i64;
                let buffer = Buffer::new()
                    .add_attribute("a_position", 2, GL_FLOAT, false, stride, offset(0))
                    .add_attribute("a_textureCoords", 2, GL_FLOAT, false, stride, offset(2))
                    .add_attribute("a_box", 4, GL_FLOAT, false, stride, offset(4))
                    .add_attribute("a_color", 4, GL_FLOAT, false, stride, offset(8))
                    .add_attribute("a_gradientStart", 4, GL_FLOAT, false, stride, offset(12))
                    .add_attribute("a_gradientEnd", 4, GL_FLOAT, false, stride, offset(16))
                    .add_attribute("a_gradientAngle", 1, GL_FLOAT, false, stride, offset(20))
                    .add_attribute("a_colorAlpha", 1, GL_FLOAT, false, stride, offset(21))
                    .add_attribute("a_shadow", 4, GL_FLOAT, false, stride, offset(22))
                    .add_attribute("a_shadowColor", 4, GL_FLOAT, false, stride, offset(26))
                    .add_attribute("a_borderColor", 4, GL_FLOAT, false, stride, offset(30))
                    .add_attribute("a_borderWidth", 4, GL_FLOAT, false, stride, offset(34))
                    .add_attribute("a_borderRadius", 4, GL_FLOAT, false, stride, offset(38))
                    .add_attribute("a_clip", 4, GL_FLOAT, false, stride, offset(42));
                shader.add_buffer(buffer);
                shader.use_program();
                self.add_shader("quad", shader);
            }
            Err(error) => panic!("Error in quad shader: {}", error),
        }
    }

    /// Appends the six triangles of one quad to the batch, expanding the
    /// shadow padding into the geometry so no per-quad uniforms are needed.
    pub fn push_quad(
        &self,
        quad: &Rectangle,
        props: &QuadProps,
        clip: Option<Rectangle>,
        transform: &Matrix3,
        out: &mut Vec<f32>,
    ) {
        let borders = [
            props.border.top,
            props.border.right,
            props.border.bottom,
            props.border.left,
        ];
        let border_color = borders
            .iter()
            .find(|b| b.is_some())
            .map_or(Color::TRANSPARENT, |b| {
                b.map_or(Color::TRANSPARENT, |b| b.color)
            });
        let border_width = borders.map(|b| b.map_or(0.0, |b| b.width));

        let (color, gradient_start, gradient_end, gradient_angle) = match props.background {
            Background::Color(c) => (c, c, c, -1.0),
            Background::Gradient(g) => (g.start, g.start, g.end, g.angle),
        };

        let shadow: [f32; 4] = props.box_shadow.into();
        let shadow_color: [f32; 4] = props.box_shadow.color.into();
        let border_radius: [f32; 4] = props.border_radius.into();
        let color: [f32; 4] = color.into();
        let gradient_start: [f32; 4] = gradient_start.into();
        let gradient_end: [f32; 4] = gradient_end.into();
        let border_color: [f32; 4] = border_color.into();

        let default_no_clip = Rectangle::new(0.0, 0.0, f32::INFINITY, f32::INFINITY);
        let clip = clip.as_ref().unwrap_or(&default_no_clip);
        let clip: [f32; 4] = [clip.x, clip.y, clip.width, clip.height];

        let box_: [f32; 4] = [quad.x, quad.y, quad.width, quad.height];

        // Shadow padding covers the box plus the gaussian falloff (the shader
        // drops to <1% beyond 3 sigma), the shadow offset and the spread.
        let sigma = shadow[2] + 0.25;
        let padding = 4.0 * sigma + shadow[0].abs().max(shadow[1].abs()) + shadow[3].max(0.0);
        let x1 = quad.x - padding;
        let y1 = quad.y - padding;
        let x2 = quad.x + quad.width + padding;
        let y2 = quad.y + quad.height + padding;

        let mut vertex = [0.0f32; QUAD_VERTEX_FLOATS];
        vertex[4..8].copy_from_slice(&box_);
        vertex[8..12].copy_from_slice(&color);
        vertex[12..16].copy_from_slice(&gradient_start);
        vertex[16..20].copy_from_slice(&gradient_end);
        vertex[20] = gradient_angle;
        vertex[21] = 1.0;
        vertex[22..26].copy_from_slice(&shadow);
        vertex[26..30].copy_from_slice(&shadow_color);
        vertex[30..34].copy_from_slice(&border_color);
        vertex[34..38].copy_from_slice(&border_width);
        vertex[38..42].copy_from_slice(&border_radius);
        vertex[42..46].copy_from_slice(&clip);

        // UVs are mapped to the true box (not the padded geometry) so the
        // shadow region samples the box edge texels via CLAMP_TO_EDGE.
        let inv_w = if quad.width > 0.0 {
            1.0 / quad.width
        } else {
            0.0
        };
        let inv_h = if quad.height > 0.0 {
            1.0 / quad.height
        } else {
            0.0
        };
        let corners = [
            (x1, y1, (x1 - quad.x) * inv_w, (y1 - quad.y) * inv_h),
            (x2, y1, (x2 - quad.x) * inv_w, (y1 - quad.y) * inv_h),
            (x2, y2, (x2 - quad.x) * inv_w, (y2 - quad.y) * inv_h),
            (x1, y2, (x1 - quad.x) * inv_w, (y2 - quad.y) * inv_h),
        ];
        let triangle_order = [0usize, 1, 3, 1, 2, 3];

        let base = out.len();
        out.resize(base + VERTICES_PER_QUAD * QUAD_VERTEX_FLOATS, 0.0);
        for (slot, &corner) in triangle_order.iter().enumerate() {
            let (cx, cy, cu, cv) = corners[corner];
            let start = base + slot * QUAD_VERTEX_FLOATS;
            let (dst, head) = out[start..start + QUAD_VERTEX_FLOATS].split_at_mut(CORNER_FLOATS);
            // Only the position is moved. `a_box` and everything derived from it stay
            // in the element's own untransformed pixels, which is the space the
            // fragment shader evaluates its rounded corners, border and shadow in -
            // so a rotated quad still has round corners in its own space, and they
            // rotate with it. The shader recovers this local position from `a_box`
            // and the texture coordinates.
            let moved = transform.transform_point(Vector2::new(cx, cy));
            dst[0] = moved.x;
            dst[1] = moved.y;
            dst[2] = cu;
            dst[3] = cv;
            head.copy_from_slice(&vertex[CORNER_FLOATS..]);
        }
    }

    pub fn initialize_simple_quad_shader(&mut self) {
        match Shader::try_new(
            include_str!("glsl/simple_quad.glsl.vert"),
            include_str!("glsl/simple_quad.glsl.frag"),
        ) {
            Ok(mut shader) => {
                let size_of_float = std::mem::size_of::<f32>();
                let stride = (SIMPLE_QUAD_VERTEX_FLOATS * size_of_float) as i32;
                let offset = |component: usize| (component * size_of_float) as i64;
                let buffer = Buffer::new()
                    .add_attribute("a_position", 2, GL_FLOAT, false, stride, offset(0))
                    .add_attribute("a_texcoord", 2, GL_FLOAT, false, stride, offset(2))
                    .add_attribute("a_color", 4, GL_FLOAT, false, stride, offset(4))
                    .add_attribute("a_clip", 4, GL_FLOAT, false, stride, offset(8));
                shader.add_buffer(buffer);
                shader.use_program();
                self.add_shader("simple_quad", shader);
            }
            Err(error) => panic!("Error in simple_quad shader: {}", error),
        }
    }

    /// Appends the six triangles of a solid-color quad to the batch. This is
    /// the fast path for quads that have no border, shadow, gradient, or
    /// border-radius. Background images are supported via texture sampling.
    pub fn push_simple_quad(
        &self,
        quad: &Rectangle,
        color: Color,
        clip: Option<Rectangle>,
        transform: &Matrix3,
        out: &mut Vec<f32>,
    ) {
        let default_no_clip = Rectangle::new(0.0, 0.0, f32::INFINITY, f32::INFINITY);
        let clip = clip.as_ref().unwrap_or(&default_no_clip);
        let clip: [f32; 4] = [clip.x, clip.y, clip.width, clip.height];
        let color: [f32; 4] = color.into();

        let corners = [
            (quad.x, quad.y, 0.0, 0.0),
            (quad.x + quad.width, quad.y, 1.0, 0.0),
            (quad.x + quad.width, quad.y + quad.height, 1.0, 1.0),
            (quad.x, quad.y + quad.height, 0.0, 1.0),
        ];
        let triangle_order = [0usize, 1, 3, 1, 2, 3];

        let base = out.len();
        out.resize(base + VERTICES_PER_QUAD * SIMPLE_QUAD_VERTEX_FLOATS, 0.0);
        for (slot, &corner) in triangle_order.iter().enumerate() {
            let (cx, cy, u, v) = corners[corner];
            let start = base + slot * SIMPLE_QUAD_VERTEX_FLOATS;
            let moved = transform.transform_point(Vector2::new(cx, cy));
            out[start] = moved.x;
            out[start + 1] = moved.y;
            out[start + 2] = u;
            out[start + 3] = v;
            out[start + 4..start + 8].copy_from_slice(&color);
            out[start + 8..start + 12].copy_from_slice(&clip);
        }
    }
}
