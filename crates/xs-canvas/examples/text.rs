use std::collections::HashMap;

use xs_canvas::{Canvas, TextSystem};
use xs_core::{AppContext, Color, EventLoop, TextProperties, TextVertices};
use xs_gl::{
    Buffer, GL_BLEND, GL_COLOR_BUFFER_BIT, GL_DEPTH_TEST, GL_FLOAT, GL_ONE_MINUS_SRC_ALPHA,
    GL_SRC_ALPHA, GL_TEXTURE_2D, GL_TEXTURE0, GL_TRIANGLES, Shader, glActiveTexture, glBindTexture,
    glBlendFunc, glClear, glClearColor, glDisable, glDrawArrays, glEnable, glViewport,
    load_texture,
};

#[cfg(not(feature = "text"))]
compile_error! {"This example requires xs-canvas to feature \'text\'"}
#[cfg(feature = "text")]
use xs_text_system::{Atlas, AtlasAlpha, AtlasMsdf, AtlasSdf};

#[derive(Default)]
struct AtlasFormat {
    texture: u32,
    text_vertices: TextVertices,
}

struct TextCanvas {
    text_properties: TextProperties,
    shaders: HashMap<&'static str, Shader>,
    atlas_alpha: AtlasFormat,
    atlas_sdf: AtlasFormat,
    atlas_msdf: AtlasFormat,
}

fn render_text(
    ctx: &AppContext,
    shader: &mut Shader,
    left: f32,
    top: f32,
    atlas: &AtlasFormat,
    color: Color,
) {
    let vertices = &atlas.text_vertices.vertices;
    let font_scale = atlas.text_vertices.font_scale;
    let distance_range = atlas.text_vertices.distance_range;
    shader.use_program();
    glActiveTexture(GL_TEXTURE0);
    glBindTexture(GL_TEXTURE_2D, atlas.texture);
    shader.uniform_2f(
        "u_resolution",
        ctx.screen.width as _,
        ctx.screen.height as _,
    );
    shader.uniform_1f("u_pixelRatio", 1.0);
    shader.uniform_matrix_3fv(
        "u_transform",
        false,
        &[1., 0., 0., 0., 1., 0., left, top, 1.], // Translate to (100, 100)
    );
    shader.uniform_1i("u_texture", 0);
    shader.uniform_4fv("u_color", color);
    shader.uniform_1f("u_size", font_scale);
    shader.uniform_1f("u_distanceRange", distance_range);

    let num_vertices = vertices.len() / 4;
    let buffer = shader.get_buffer(0);
    buffer.push_data(vertices);
    glDrawArrays(GL_TRIANGLES, 0, num_vertices as _);
}

#[allow(clippy::too_many_arguments)]
impl TextCanvas {
    fn create_text<A: Atlas>(
        &mut self,
        font_family: String,
        font_weight: u16,
        font_data: Vec<u8>,
        max_width: f32,
        max_height: f32,
        text: &str,
    ) -> AtlasFormat {
        let mut text_system_alpha = xs_text_system::TextSystem::<A>::new();
        let atlas_alpha = A::generate(&font_data);
        let atlas_alpha_image = &atlas_alpha.info().image;
        let texture = load_texture(
            &atlas_alpha_image.bytes,
            -1,
            atlas_alpha_image.width as _,
            atlas_alpha_image.height as _,
            atlas_alpha_image.format.clone(),
        );
        text_system_alpha.load_font(font_family.clone(), font_weight, font_data.clone());
        let text_vertices =
            text_system_alpha.render_text(text, &self.text_properties, max_width, max_height);
        AtlasFormat {
            texture,
            text_vertices,
        }
    }
}

impl EventLoop for TextCanvas {
    fn initialize(&mut self, _ctx: &mut xs_core::AppContext) {
        let font_family = self.text_properties.font_family.clone();
        let font_weight = self.text_properties.font_weight;
        let font_data =
            include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf").to_vec();

        self.atlas_alpha = self.create_text::<AtlasAlpha>(
            font_family.clone(),
            font_weight,
            font_data.clone(),
            600.,
            200.,
            "Alpha Text",
        );

        self.atlas_sdf = self.create_text::<AtlasSdf>(
            font_family.clone(),
            font_weight,
            font_data.clone(),
            600.,
            200.,
            "SDF Text",
        );

        self.atlas_msdf = self.create_text::<AtlasMsdf>(
            font_family,
            font_weight,
            font_data.clone(),
            600.,
            200.,
            "MSDF Text",
        );

        match Shader::try_new(
            include_str!("glsl/text_alpha.glsl.vert"),
            include_str!("glsl/text_alpha.glsl.frag"),
        ) {
            Ok(mut shader) => {
                let buffer = Buffer::new()
                    .add_attribute("a_position", 2, GL_FLOAT, false, 16, 0)
                    .add_attribute("a_textureCoords", 2, GL_FLOAT, false, 16, 8);
                shader.add_buffer(buffer);
                self.shaders.insert("text-alpha", shader);
            }
            Err(error) => panic!("ERROR: {error}"),
        };

        match Shader::try_new(
            include_str!("glsl/sdf.glsl.vert"),
            include_str!("glsl/sdf.glsl.frag"),
        ) {
            Ok(mut shader) => {
                let buffer = Buffer::new()
                    .add_attribute("a_position", 2, GL_FLOAT, false, 16, 0)
                    .add_attribute("a_textureCoords", 2, GL_FLOAT, false, 16, 8);
                shader.add_buffer(buffer);
                self.shaders.insert("text-sdf", shader);
            }
            Err(error) => panic!("ERROR: {error}"),
        };
    }

    fn frame(&mut self, ctx: &mut xs_core::AppContext) {
        glViewport(0, 0, ctx.screen.width as _, ctx.screen.height as _);
        glClear(GL_COLOR_BUFFER_BIT);
        glClearColor(0.5, 0.5, 0.5, 1.0);

        glDisable(GL_DEPTH_TEST);
        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

        // Draw alpha text
        let shader_alpha = self.shaders.get_mut("text-alpha").unwrap();
        render_text(
            ctx,
            shader_alpha,
            100.0,
            100.0,
            &self.atlas_alpha,
            Color::rgba(0.0, 0.0, 1.0, 1.0),
        );

        // Draw SDF text
        let shader_sdf = self.shaders.get_mut("text-sdf").unwrap();
        render_text(
            ctx,
            shader_sdf,
            100.0,
            300.0,
            &self.atlas_sdf,
            Color::rgba(0.0, 1.0, 0.0, 1.0),
        );

        // Draw MSDF text
        render_text(
            ctx,
            shader_sdf,
            100.0,
            500.0,
            &self.atlas_msdf,
            Color::rgba(1.0, 0.0, 0.0, 1.0),
        );
    }
}

pub fn main() {
    let text_properties = TextProperties {
        font_family: "sans-serif".to_string(),
        font_size: 82,
        font_weight: 400,
        font_style: xs_core::FontStyle::Normal,
    };

    let app = TextCanvas {
        text_properties,

        shaders: Default::default(),
        atlas_alpha: Default::default(),
        atlas_sdf: Default::default(),
        atlas_msdf: Default::default(),
    };
    Canvas::new().run(app);
}
