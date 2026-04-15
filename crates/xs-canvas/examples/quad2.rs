mod ppm;

use std::collections::HashMap;
use std::mem::size_of;

use xs_canvas::Canvas;
use xs_core::{Color, EventLoop, Rectangle};
use xs_gl::{
    GL_BLEND, GL_COLOR_BUFFER_BIT, GL_DEPTH_TEST, GL_ELEMENT_ARRAY_BUFFER, GL_FLOAT,
    GL_ONE_MINUS_SRC_ALPHA, GL_SRC_ALPHA, GL_STATIC_DRAW, GL_TEXTURE_2D, GL_TEXTURE0, GL_TRIANGLES,
    GL_UNSIGNED_SHORT, ShaderGlSl, glActiveTexture, glBindBuffer, glBindTexture, glBlendFunc,
    glBufferData, glClear, glClearColor, glDisable, glDrawElements, glEnable, glGenBuffers,
    glViewport, load_texture,
};

const APP_WIDTH: i32 = 1440;
const APP_HEIGHT: i32 = 810;
#[rustfmt::skip]
const QUAD_VERTICES: &[f32] = &[
    0.0, 0.0,
    1.0, 0.0,
    1.0, 1.0,
    0.0, 1.0
];

#[derive(Default)]
struct ShadowCanvas {
    shaders: HashMap<&'static str, ShaderGlSl>,
    index_buffer: u32,
    texture_place_holder: u32,
    texture: u32,
}

impl ShadowCanvas {
    pub fn new() -> Self {
        Default::default()
    }
}

impl EventLoop for ShadowCanvas {
    fn initialize(&mut self, _ctx: &mut xs_core::AppContext) {
        let index_buffer = glGenBuffers(1)[0];
        let indices: &[u16] = &[0, 1, 3, 1, 2, 3];
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, index_buffer);
        glBufferData(GL_ELEMENT_ARRAY_BUFFER, indices, GL_STATIC_DRAW);
        self.index_buffer = index_buffer;

        self.texture_place_holder =
            load_texture(&[0, 0, 0, 0], -1, 1, 1, xs_core::ImageFormat::Rgba);
        let bitmap = ppm::PpmImage::try_from(include_bytes!("opengl.ppm")).unwrap();
        self.texture = load_texture(
            &bitmap.data,
            -1,
            bitmap.width,
            bitmap.height,
            xs_core::ImageFormat::Rgb,
        );

        let size_of_float = size_of::<f32>() as i32;
        let stride = 2 * size_of_float;
        match ShaderGlSl::try_new(
            include_str!("glsl/quad2.glsl.vert"),
            include_str!("glsl/quad2.glsl.frag"),
            &[&[
                ("a_position", 2, GL_FLOAT, false, stride, 0),
                ("a_texCoord", 2, GL_FLOAT, false, stride, 0),
            ]],
        ) {
            Ok(shader) => {
                shader.attribute_buffer_data("a_position", QUAD_VERTICES);
                self.shaders.insert("shadow", shader);
            }
            Err(error) => panic!("ERROR: {error}"),
        }
    }

    fn frame(&mut self, _ctx: &mut xs_core::AppContext) {
        glViewport(0, 0, APP_WIDTH, APP_HEIGHT);
        glClear(GL_COLOR_BUFFER_BIT);
        glClearColor(0.75, 0.75, 0.75, 1.0);

        glDisable(GL_DEPTH_TEST);
        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

        let shader = self.shaders.get_mut("shadow").unwrap();
        shader.use_program();
        let quad = Rectangle::new(100., 200., 300., 100.);
        let background_color = Color::rgba(1.0, 0.0, 0.0, 1.0);
        let shadow_color = Color::rgba(0.0, 0.0, 0.0, 1.0);
        let border_color = Color::rgba(1.0, 1.0, 0.0, 1.0);

        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, self.texture);
        shader.uniform_2f("u_resolution", APP_WIDTH as f32, APP_HEIGHT as f32);
        shader.uniform_4fv("u_box", quad);
        shader.uniform_4fv("u_color", background_color);
        shader.uniform_1f("u_colorAlpha", 0.5);
        shader.uniform_4f("u_shadow", 8.0, 8.0, 5.0, 0.0);
        shader.uniform_4fv("u_shadowColor", shadow_color);
        shader.uniform_4f("u_borderWidth", 4.0, 4.0, 4.0, 8.0);
        shader.uniform_1f("u_borderRadius", 10.0);
        shader.uniform_4fv("u_borderColor", border_color);
        shader.uniform_1i("u_texture", 0);
        glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_SHORT, 0);
    }
}

pub fn main() {
    let app = ShadowCanvas::new();
    Canvas::new().run(app);
}
