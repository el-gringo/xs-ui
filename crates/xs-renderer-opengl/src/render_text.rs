use xs_core::{AppContext, Color, TextVertices};
use xs_gl::{
    Buffer, GL_FLOAT, GL_TEXTURE_2D, GL_TEXTURE0, GL_TRIANGLES, Shader, glActiveTexture,
    glBindTexture, glDrawArrays,
};
use xs_math::Rectangle;

use crate::RendererOpenGl;

impl RendererOpenGl {
    pub fn initialize_text_shader(&mut self) {
        match Shader::try_new(
            include_str!("glsl/text_alpha.glsl.vert"),
            include_str!("glsl/text_alpha.glsl.frag"),
        ) {
            Ok(mut shader) => {
                let size_of_float = std::mem::size_of::<f32>();
                let stride = 4 * size_of_float as i32;
                let buffer = Buffer::new()
                    .add_attribute("a_position", 2, GL_FLOAT, false, stride, 0)
                    .add_attribute(
                        "a_textureCoords",
                        2,
                        GL_FLOAT,
                        false,
                        stride,
                        (2 * size_of_float) as _,
                    );
                shader.add_buffer(buffer);
                shader.use_program();
                self.add_shader("text_alpha", shader);
            }
            Err(error) => panic!("ERROR: {error}"),
        };
    }

    pub fn render_text(
        &mut self,
        ctx: &AppContext,
        quad: &Rectangle,
        color: &Color,
        text_vertices: &TextVertices,
        atlas_texture: u32,
    ) {
        let shader = self.get_shader("text_alpha");
        shader.use_program();
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, atlas_texture);
        shader.uniform_2f(
            "u_resolution",
            ctx.screen.width as _,
            ctx.screen.height as _,
        );
        shader.uniform_1f("u_pixelRatio", 1.0);
        shader.uniform_matrix_3fv(
            "u_transform",
            false,
            &[1., 0., 0., 0., 1., 0., quad.x, quad.y, 1.],
        );
        shader.uniform_1i("u_texture", 0);
        shader.uniform_4fv("u_color", *color);

        let num_vertices = text_vertices.vertices.len() / 4;
        let buffer = shader.get_buffer(0);
        buffer.push_data(&text_vertices.vertices);
        glDrawArrays(GL_TRIANGLES, 0, num_vertices as _);
    }
}
