use std::collections::HashMap;

use xs_canvas::Canvas;
use xs_core::{EventLoop, TextProperties, TextVertices};
use xs_gl::{
    GL_BLEND, GL_COLOR_BUFFER_BIT, GL_DEPTH_TEST, GL_FLOAT, GL_ONE, GL_ONE_MINUS_SRC_ALPHA,
    GL_TEXTURE_2D, GL_TEXTURE0, GL_TRIANGLES, ShaderGlSl, glActiveTexture, glBindTexture,
    glBlendFunc, glClear, glClearColor, glDisable, glDrawArrays, glEnable, glViewport,
    load_texture,
};
use xs_text_system::{Atlas, AtlasAlpha, AtlasMsdf, AtlasSdf, TextSystem};
use xs_text_system_glyph::FontRef;

const APP_WIDTH: i32 = 1440;
const APP_HEIGHT: i32 = 810;

struct TextCanvas {
    text_properties: TextProperties,
    shaders: HashMap<&'static str, ShaderGlSl>,
    alpha_atlas_texture: u32,
    alpha_text_vertices: TextVertices,
    sdf_atlas_texture: u32,
    sdf_text_vertices: TextVertices,
    msdf_atlas_texture: u32,
    msdf_text_vertices: TextVertices,
}

impl EventLoop for TextCanvas {
    fn initialize(&mut self, _ctx: &mut xs_core::AppContext) {
        let font_family = "sans-serif";
        let font_data =
            include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf").to_vec();

        let atlas_font_size = 42.;

        let font =
            FontRef::try_from_slice(Box::leak(font_data.clone().into_boxed_slice())).unwrap();
        let (alpha_atlas, alpha_format, alpha_atlas_data) =
            AtlasAlpha::generate(&font, atlas_font_size, 0.0);
        let (sdf_atlas, sdf_format, sdf_atlas_data) =
            AtlasSdf::generate(&font, atlas_font_size, 4.0);
        let (msdf_atlas, msdf_format, msdf_atlas_data) =
            AtlasMsdf::generate(&font, atlas_font_size, 4.0);

        self.alpha_atlas_texture = load_texture(
            &alpha_atlas_data,
            -1,
            alpha_atlas.width() as _,
            alpha_atlas.height() as _,
            alpha_format,
        );
        self.sdf_atlas_texture = load_texture(
            &sdf_atlas_data,
            -1,
            sdf_atlas.atlas.width as _,
            sdf_atlas.atlas.height as _,
            sdf_format,
        );
        self.msdf_atlas_texture = load_texture(
            &msdf_atlas_data,
            -1,
            msdf_atlas.atlas.width as _,
            msdf_atlas.atlas.height as _,
            msdf_format,
        );

        let mut alpha_text_system = TextSystem::<AtlasAlpha>::new();
        alpha_text_system.load_font(font_family, 400, atlas_font_size, 0.0, font_data.clone());

        let mut sdf_text_system = TextSystem::<AtlasSdf>::new();
        sdf_text_system.load_font(font_family, 400, atlas_font_size, 4., font_data.clone());

        let mut msdf_text_system = TextSystem::<AtlasMsdf>::new();
        msdf_text_system.load_font(font_family, 400, atlas_font_size, 4., font_data.clone());

        self.text_properties.font_size = 64;

        self.alpha_text_vertices =
            alpha_text_system.render_text("Alpha text", &self.text_properties, 600., 200.);

        self.sdf_text_vertices =
            sdf_text_system.render_text("SDF text", &self.text_properties, 600., 200.);

        self.msdf_text_vertices =
            msdf_text_system.render_text("MSDF text", &self.text_properties, 600., 200.);

        match ShaderGlSl::try_new(
            include_str!("glsl/text_alpha.glsl.vert"),
            include_str!("glsl/text_alpha.glsl.frag"),
            &[
                "u_resolution",
                "u_pixelRatio",
                "u_transform",
                "u_texture",
                "u_color",
            ],
            &[&[
                ("a_position", 2, GL_FLOAT, false, 16, 0),
                ("a_textureCoords", 2, GL_FLOAT, false, 16, 8),
            ]],
        ) {
            Ok(shader) => {
                self.shaders.insert("text_alpha", shader);
            }
            Err(error) => panic!("ERROR: {error}"),
        };

        match ShaderGlSl::try_new(
            include_str!("glsl/sdf.glsl.vert"),
            include_str!("glsl/sdf.glsl.frag"),
            &[
                "u_resolution",
                "u_pixelRatio",
                "u_transform",
                "u_size",
                "u_distanceRange",
                "u_texture",
                "u_color",
            ],
            &[&[
                ("a_position", 2, GL_FLOAT, false, 16, 0),
                ("a_textureCoords", 2, GL_FLOAT, false, 16, 8),
            ]],
        ) {
            Ok(shader) => {
                self.shaders.insert("text_sdf", shader);
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

        // Draw alpha text
        let vertices = self.alpha_text_vertices.vertices.clone();
        let shader = self.shaders.get("text_alpha").unwrap();
        shader.use_program();
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, self.alpha_atlas_texture);
        shader.uniform_2f("u_resolution", APP_WIDTH as _, APP_HEIGHT as _);
        shader.uniform_1f("u_pixelRatio", 1.0);
        shader.uniform_matrix_3fv(
            "u_transform",
            false,
            &[1., 0., 0., 0., 1., 0., 100., 100., 1.], // Translate to (100, 100)
        );
        shader.uniform_1i("u_texture", 0);
        shader.uniform_4f("u_color", 0.0, 0.0, 1.0, 1.0);

        let num_vertices = vertices.len() / 4;
        shader.attribute_buffer_data("a_position", &vertices);
        glDrawArrays(GL_TRIANGLES, 0, num_vertices as _);

        // Draw SDF text
        let vertices = self.sdf_text_vertices.vertices.clone();
        let shader = self.shaders.get("text_sdf").unwrap();
        shader.use_program();
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, self.sdf_atlas_texture);
        shader.uniform_2f("u_resolution", APP_WIDTH as _, APP_HEIGHT as _);
        shader.uniform_1f("u_pixelRatio", 1.0);
        shader.uniform_matrix_3fv(
            "u_transform",
            false,
            &[1., 0., 0., 0., 1., 0., 100., 400., 1.], // Translate to (100, 100)
        );
        let sdf_dist_range = self.sdf_text_vertices.distance_range * 2.0;
        shader.uniform_1i("u_texture", 0);
        shader.uniform_1f("u_size", self.sdf_text_vertices.font_scale);
        shader.uniform_1f("u_distanceRange", sdf_dist_range);
        shader.uniform_4f("u_color", 1.0, 0.0, 0.0, 1.0); // Bright red
        let num_vertices = vertices.len() / 4;
        shader.attribute_buffer_data("a_position", &vertices);
        glDrawArrays(GL_TRIANGLES, 0, num_vertices as _);

        // Draw MSDF text
        shader.use_program();
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, self.msdf_atlas_texture);
        shader.uniform_2f("u_resolution", APP_WIDTH as _, APP_HEIGHT as _);
        shader.uniform_1f("u_pixelRatio", 1.0);
        shader.uniform_matrix_3fv(
            "u_transform",
            false,
            &[1., 0., 0., 0., 1., 0., 100., 600., 1.], // Translate to (100, 100)
        );
        let msdf_dist_range = self.msdf_text_vertices.distance_range * 2.0;
        let msdf_vertices = self.msdf_text_vertices.vertices.clone();
        shader.uniform_1i("u_texture", 0);
        shader.uniform_1f("u_size", self.msdf_text_vertices.font_scale);
        shader.uniform_1f("u_distanceRange", msdf_dist_range);
        shader.uniform_4f("u_color", 1.0, 0.0, 0.0, 1.0); // Bright red
        let num_vertices = msdf_vertices.len() / 4;
        shader.attribute_buffer_data("a_position", &msdf_vertices);
        glDrawArrays(GL_TRIANGLES, 0, num_vertices as _);

        shader.uniform_4f("u_color", 0.0, 1.0, 0.0, 1.0); // Bright red
        shader.uniform_matrix_3fv(
            "u_transform",
            false,
            &[1., 0., 0., 0., 1., 0., 100., 600., 1.], // Translate to (100, 100)
        );
        glDrawArrays(GL_TRIANGLES, 0, num_vertices as _);
    }
}

pub fn main() {
    let text_properties = TextProperties {
        font_family: "sans-serif".to_string(),
        font_size: 42,
        font_weight: 400,
        font_style: xs_core::FontStyle::Normal,
    };

    let app = TextCanvas {
        text_properties,
        alpha_atlas_texture: 0,
        sdf_atlas_texture: 0,
        msdf_atlas_texture: 0,
        alpha_text_vertices: TextVertices::default(),
        sdf_text_vertices: TextVertices::default(),
        msdf_text_vertices: TextVertices::default(),
        shaders: Default::default(),
    };
    Canvas::new().run(app);
}
