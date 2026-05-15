use crate::RendererOpenGl;
use xs_core::{AppContext, Color};
use xs_gl::{
    Buffer, GL_FLOAT, GL_TEXTURE_2D, GL_TEXTURE0, GL_TRIANGLES, GL_UNSIGNED_SHORT, Shader,
    glActiveTexture, glBindTexture, glDrawElements,
};
use xs_layout::{Background, QuadProps};
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
        props: &QuadProps,
        clip: Option<Rectangle>,
    ) {
        let texture_place_holder = self.texture_place_holder;
        let shader = self.get_shader("quad");
        shader.use_program();
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
        let texture = props
            .background_image
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
        match props.background {
            Background::Color(c) => {
                shader.uniform_4fv("u_color", c);
                shader.uniform_1f("u_gradientAngle", -1.0);
            }
            Background::Gradient(g) => {
                shader.uniform_4fv("u_color", g.start);
                shader.uniform_4fv("u_gradientStart", g.start);
                shader.uniform_4fv("u_gradientEnd", g.end);
                shader.uniform_1f("u_gradientAngle", g.angle);
            }
        }
        shader.uniform_1f("u_colorAlpha", 1.0);
        shader.uniform_4fv("u_shadow", props.box_shadow);
        shader.uniform_4fv("u_shadowColor", props.box_shadow.color);
        shader.uniform_4fv("u_borderWidth", border_width);
        shader.uniform_4fv("u_borderRadius", props.border_radius);
        shader.uniform_4fv("u_borderColor", border_color);
        shader.uniform_1i("u_texture", 0);

        let default_no_clip = Rectangle::new(0.0, 0.0, f32::INFINITY, f32::INFINITY);
        let clip_rect = clip.as_ref().unwrap_or(&default_no_clip);
        shader.uniform_4fv("u_clip", *clip_rect);

        shader.bind_buffers();
        glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_SHORT, 0);
    }
}
