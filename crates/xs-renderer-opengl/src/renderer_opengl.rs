use std::collections::HashMap;

use xs_core::{AppContext, ImageFormat, Renderer};
use xs_gl::{
    GL_BLEND, GL_COLOR_BUFFER_BIT, GL_DEPTH_TEST, GL_ELEMENT_ARRAY_BUFFER, GL_ONE_MINUS_SRC_ALPHA,
    GL_SRC_ALPHA, GL_STATIC_DRAW, Shader, glBindBuffer, glBlendFunc, glBufferData, glClear,
    glClearColor, glDisable, glEnable, glFlush, glGenBuffers, glViewport, load_texture,
};
use xs_layout::{
    BoxConstraints, LayoutBox, LayoutTree,
    Material::{Quad, Text},
    View,
};
use xs_math::{Rectangle, Vector2};

pub struct RendererOpenGl {
    width: i32,
    height: i32,
    shaders: HashMap<&'static str, Shader>,
    fonts: HashMap<(&'static str, u16), (u32, f32)>,
    pub texture_place_holder: u32,
    index_buffer: u32,
}

impl RendererOpenGl {
    pub fn new(width: i32, height: i32) -> Self {
        let shaders = Default::default();
        Self {
            width,
            height,
            shaders,
            texture_place_holder: 0,
            fonts: Default::default(),
            index_buffer: 0,
        }
    }

    pub fn add_shader(&mut self, name: &'static str, shader: Shader) {
        self.shaders.insert(name, shader);
    }

    pub fn get_shader(&mut self, name: &'static str) -> &mut Shader {
        self.shaders
            .get_mut(name)
            .unwrap_or_else(|| panic!("Cannot find shader {name}"))
    }
}

impl Renderer for RendererOpenGl {
    fn init(&mut self) {
        let index_buffer = glGenBuffers(1)[0];
        let indices: &[u16] = &[0, 1, 3, 1, 2, 3];
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, index_buffer);
        glBufferData(GL_ELEMENT_ARRAY_BUFFER, indices, GL_STATIC_DRAW);
        self.index_buffer = index_buffer;
        self.texture_place_holder =
            load_texture(&[0, 0, 0, 0], -1, 1, 1, xs_core::ImageFormat::Rgba);
        self.initialize_quad_shader();
        self.initialize_text_shader();
    }

    fn draw_frame(&mut self, ctx: &AppContext, widget_tree: &View) {
        glViewport(0, 0, ctx.screen.width as _, ctx.screen.height as _);
        glClear(GL_COLOR_BUFFER_BIT);
        glClearColor(0., 0., 0., 0.);

        glEnable(GL_BLEND);
        glDisable(GL_DEPTH_TEST);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

        let mut tree = LayoutTree::new();
        let constraints = BoxConstraints {
            min: Vector2::new(0.0, 0.0),
            max: Vector2::new(self.width as f32, self.height as f32),
        };
        let root_sbox = widget_tree.layout(&mut tree, &constraints);
        let root_lbox = LayoutBox::from_child(root_sbox, (0.0, 0.0));
        let root_id = tree.insert(root_lbox.clone());
        tree.set_root(Some(root_id));
        for (_, _, _, _, child, offset) in tree.iter() {
            let origin = Vector2::new(child.bounds.x, child.bounds.y);
            let min = offset + origin + child.margin.min();
            let quad = Rectangle::new(min.x, min.y, child.bounds.width, child.bounds.height);
            if (child.material.is_some() || child.text_material.is_some())
                && let Some(material) = &child.material
            {
                match material {
                    Quad(style, image) => {
                        self.render_quad(ctx, &quad, style, image);
                    }
                    Text(color, text_properties, text_vertices) => {
                        let (atlas_texture, _atlas_font_size) = self
                            .fonts
                            .get(&(&text_properties.font_family, text_properties.font_weight))
                            .unwrap();
                        self.render_text(ctx, &quad, color, text_vertices, *atlas_texture);
                    }
                }
            }
        }
        glFlush();
    }

    fn load_font_atlas(
        &mut self,
        font_family: &'static str,
        font_weight: u16,
        font_size: f32,
        atlas_data: &[u8],
        width: u32,
        height: u32,
        image_format: ImageFormat,
    ) {
        let texture = self.load_texture(atlas_data, -1, width, height, image_format);
        self.fonts
            .insert((font_family, font_weight), (texture, font_size));
    }

    fn load_texture(
        &mut self,
        bytes: &[u8],
        image_handler: i32,
        width: u32,
        height: u32,
        image_format: ImageFormat,
    ) -> u32 {
        load_texture(bytes, image_handler, width, height, image_format)
    }
}
