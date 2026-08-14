use xs_core::{Color, TextVertices};
use xs_gl::{Buffer, GL_FLOAT, Shader};
use xs_math::Rectangle;

use crate::RendererOpenGl;

/// Number of f32 components in one interleaved text vertex.
pub(crate) const TEXT_VERTEX_FLOATS: usize = 12;

impl RendererOpenGl {
    pub fn initialize_text_shader(&mut self) {
        match Shader::try_new(
            include_str!("glsl/text_alpha.glsl.vert"),
            include_str!("glsl/text_alpha.glsl.frag"),
        ) {
            Ok(mut shader) => {
                let size_of_float = std::mem::size_of::<f32>();
                let stride = (TEXT_VERTEX_FLOATS * size_of_float) as i32;
                let offset = |component: usize| (component * size_of_float) as i64;
                let buffer = Buffer::new()
                    .add_attribute("a_position", 2, GL_FLOAT, false, stride, offset(0))
                    .add_attribute(
                        "a_textureCoords",
                        2,
                        GL_FLOAT,
                        false,
                        stride,
                        offset(2),
                    )
                    .add_attribute("a_color", 4, GL_FLOAT, false, stride, offset(4))
                    .add_attribute("a_clip", 4, GL_FLOAT, false, stride, offset(8));
                shader.add_buffer(buffer);
                shader.use_program();
                self.add_shader("text_alpha", shader);
            }
            Err(error) => panic!("ERROR: {error}"),
        };
    }

    /// Appends the glyph vertices of one text node to the batch. Positions are
    /// translated into screen space on the CPU and the color is baked per
    /// vertex so all text nodes can be drawn with a single draw call.
    pub fn push_text(
        &self,
        quad: &Rectangle,
        color: &Color,
        text_vertices: &TextVertices,
        clip: Option<Rectangle>,
        out: &mut Vec<f32>,
    ) {
        let num_vertices = text_vertices.vertices.len() / 4;
        if num_vertices == 0 {
            return;
        }

        let default_no_clip = Rectangle::new(0.0, 0.0, f32::INFINITY, f32::INFINITY);
        let clip = clip.as_ref().unwrap_or(&default_no_clip);

        let base = out.len();
        out.resize(base + num_vertices * TEXT_VERTEX_FLOATS, 0.0);

        let color4: [f32; 4] = (*color).into();
        for i in 0..num_vertices {
            let src = i * 4;
            let dst = base + i * TEXT_VERTEX_FLOATS;
            out[dst] = text_vertices.vertices[src] + quad.x;
            out[dst + 1] = text_vertices.vertices[src + 1] + quad.y;
            out[dst + 2] = text_vertices.vertices[src + 2];
            out[dst + 3] = text_vertices.vertices[src + 3];
            out[dst + 4..dst + 8].copy_from_slice(&color4);
            out[dst + 8] = clip.x;
            out[dst + 9] = clip.y;
            out[dst + 10] = clip.width;
            out[dst + 11] = clip.height;
        }
    }
}
