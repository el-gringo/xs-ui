use crate::RendererOpenGl;
use xs_core::{AppContext, Color};
use xs_gl::{
    Buffer, GL_FLOAT, GL_TEXTURE_2D, GL_TEXTURE0, GL_TRIANGLES, GL_UNSIGNED_SHORT, Shader,
    glActiveTexture, glBindTexture, glDrawElements,
};
use xs_layout::{Image, Style};
use xs_math::Rectangle;

#[rustfmt::skip]
const QUAD_VERTICES: &[f32] = &[
    0.0, 0.0,
    1.0, 0.0,
    1.0, 1.0,
    0.0, 1.0,
];

impl RendererOpenGl {
    pub fn initialize_quad_shader(&mut self) {
        match Shader::try_new(
            include_str!("glsl/quad.glsl.vert"),
            include_str!("glsl/quad.glsl.frag"),
        ) {
            Ok(mut shader) => {
                let size_of_float = std::mem::size_of::<f32>();
                let stride = 2 * size_of_float as i32;
                let buffer = Buffer::new()
                    .add_attribute("a_position", 2, GL_FLOAT, false, stride, 0)
                    .add_attribute("a_textureCoords", 2, GL_FLOAT, false, stride, 0);
                buffer.push_data(QUAD_VERTICES);
                shader.add_buffer(buffer);
                shader.use_program();
                self.add_shader("quad", shader)
            }
            Err(error) => panic!("Error in quad shader: {}", error),
        }
    }

    pub fn render_quad(
        &mut self,
        ctx: &AppContext,
        quad: &Rectangle,
        style: &Style,
        image: &Option<Image>,
    ) {
        let texture_place_holder = self.texture_place_holder;
        let shader = self.get_shader("quad");
        shader.use_program();
        let borders = [
            style.border.top,
            style.border.right,
            style.border.bottom,
            style.border.left,
        ];
        let border_color = borders
            .iter()
            .find(|b| b.is_some())
            .map_or(Color::TRANSPARENT, |b| {
                b.map_or(Color::TRANSPARENT, |b| b.color)
            });
        let border_width = borders.map(|b| b.map_or(0.0, |b| b.width));
        let texture = image
            .clone()
            .map_or(texture_place_holder, |img| img.texture);
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, texture);
        shader.uniform_2f(
            "u_resolution",
            ctx.screen.width as _,
            ctx.screen.height as _,
        );
        shader.uniform_4fv("u_box", *quad);
        shader.uniform_4fv("u_color", style.background_color);
        shader.uniform_1f("u_colorAlpha", 0.5);
        shader.uniform_4fv("u_shadow", style.box_shadow);
        shader.uniform_4fv("u_shadowColor", style.box_shadow.color);
        shader.uniform_4fv("u_borderWidth", border_width);
        shader.uniform_4fv("u_borderRadius", style.border_radius);
        shader.uniform_4fv("u_borderColor", border_color);
        shader.uniform_1i("u_texture", 0);
        shader.bind_buffers();
        glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_SHORT, 0);
    }
}
