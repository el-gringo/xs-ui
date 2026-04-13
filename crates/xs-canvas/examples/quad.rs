use std::collections::HashMap;

use xs_canvas::Canvas;
use xs_core::{Color, EventLoop, Rectangle};
use xs_gl::{
    GL_BLEND, GL_COLOR_BUFFER_BIT, GL_DEPTH_TEST, GL_ELEMENT_ARRAY_BUFFER, GL_FLOAT, GL_ONE,
    GL_ONE_MINUS_SRC_ALPHA, GL_SRC_ALPHA, GL_STATIC_DRAW, GL_TEXTURE_2D, GL_TEXTURE0, GL_TRIANGLES,
    GL_UNSIGNED_SHORT, ShaderGlSl, glActiveTexture, glBindBuffer, glBindTexture, glBlendFunc,
    glBufferData, glClear, glClearColor, glDisable, glDrawArrays, glDrawElements, glEnable,
    glGenBuffers, glViewport, load_texture,
};

const APP_WIDTH: i32 = 1440;
const APP_HEIGHT: i32 = 810;

#[derive(Default)]
struct QuadCanvas {
    shaders: HashMap<&'static str, ShaderGlSl>,
    tex_place_holder: u32,
}

impl QuadCanvas {
    pub fn new() -> Self {
        Default::default()
    }
}

impl EventLoop for QuadCanvas {
    fn initialize(&mut self, _ctx: &mut xs_core::AppContext) {
        let index_buffer = glGenBuffers(1)[0];
        let indices: &[u16] = &[0, 1, 2, 3, 4, 5];
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, index_buffer);
        glBufferData(GL_ELEMENT_ARRAY_BUFFER, indices, GL_STATIC_DRAW);

        self.tex_place_holder = load_texture(&[0, 0, 0, 0], -1, 1, 1, false);
        eprintln!("self.tex_place_holder: {}", self.tex_place_holder);

        let size_of_float = size_of::<f32>() as i32;
        let stride = 11 * size_of::<f32>() as i32;

        match ShaderGlSl::try_new(
            include_str!("glsl/quad.glsl.vert"),
            include_str!("glsl/quad.glsl.frag"),
            &[
                "u_resolution",
                "u_pixelRatio",
                "u_alpha",
                "u_dimensions",
                "u_texture",
                "u_radius",
                "u_borderWidth",
                "u_shadow",
                "u_shadowColor",
                "u_borderColor",
                "u_color",
            ],
            &[&[
                ("a_position", 2, GL_FLOAT, false, stride, 0),
                (
                    "a_textureCoords",
                    2,
                    GL_FLOAT,
                    false,
                    stride,
                    (2 * size_of_float) as _,
                ),
                (
                    "a_color",
                    4,
                    GL_FLOAT,
                    false,
                    stride,
                    (4 * size_of_float) as _,
                ),
                (
                    "a_textureIndex",
                    1,
                    GL_FLOAT,
                    false,
                    stride,
                    (8 * size_of_float) as _,
                ),
                (
                    "a_nodeCoords",
                    2,
                    GL_FLOAT,
                    false,
                    stride,
                    (9 * size_of_float) as _,
                ),
            ]],
        ) {
            Ok(shader) => {
                self.shaders.insert("quad", shader);
            }
            Err(error) => panic!("ERROR: {error}"),
        };
    }

    fn frame(&mut self, _ctx: &mut xs_core::AppContext) {
        glViewport(0, 0, APP_WIDTH as _, APP_HEIGHT as _);
        glClear(GL_COLOR_BUFFER_BIT);
        glClearColor(0.5, 0.5, 0.5, 1.0);

        glDisable(GL_DEPTH_TEST);
        glEnable(GL_BLEND);
        glBlendFunc(GL_ONE, GL_ONE_MINUS_SRC_ALPHA);

        let background_color = Color::rgba(1., 0., 0., 1.);
        let border_color = Color::rgba(0., 1., 0., 1.);
        let rect = Rectangle::new(100., 100., 200., 100.);
        let min_x = rect.x;
        let min_y = rect.y;
        let max_x = rect.x + rect.width;
        let max_y = rect.y + rect.height;

        let shader = self.shaders.get("quad").unwrap();
        shader.use_program();
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, self.tex_place_holder);
        shader.uniform_2f("u_resolution", APP_WIDTH as _, APP_HEIGHT as _);
        shader.uniform_1f("u_pixelRatio", 1.);
        shader.uniform_1f("u_alpha", 1.);
        shader.uniform_2f("u_dimensions", 300., 100.);
        shader.uniform_1i("u_texture", 0);
        shader.uniform_4f("u_radius", 0., 0., 0., 0.);
        shader.uniform_4f("u_borderWidth", 4., 4., 4., 4.);
        shader.uniform_4fv("u_borderColor", border_color);
        shader.uniform_4f("u_shadow", 0., 0., 0., 0.);
        shader.uniform_4f("u_shadowColor", 0., 0., 0., 1.);
        //shader.uniform_4fv("u_color", background_color);

        let texture_index = 0.;
        let Color { r, g, b, a } = background_color;
        #[rustfmt::skip]
        let data: &[f32] = &[
            min_x, min_y,
            0., 0.,
            r, g, b, a,
            texture_index,
            0., 0.,

            max_x, min_y,
            1., 0.,
            r, g, b, a,
            texture_index,
            1., 0.,

            min_x, max_y,
            0., 1.,
            r, g, b, a,
            texture_index,
            0., 1.,

            max_x, min_y,
            1., 0.,
            r, g, b, a,
            texture_index,
            1., 0.,

            max_x, max_y,
            1., 1.,
            r, g, b, a,
            texture_index,
            1., 1.,

            min_x, max_y,
            0., 1.,
            r, g, b, a,
            texture_index,
            0., 1.,
        ];
        shader.attribute_buffer_data("a_position", data);
        glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_SHORT, 0);
    }
}

pub fn main() {
    let app = QuadCanvas::new();
    Canvas::new().run(app);
}
