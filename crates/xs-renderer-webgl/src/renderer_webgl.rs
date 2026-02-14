use std::collections::HashMap;

use xs_layout::colors::TRANSPARENT;
use xs_layout::{BoxConstraints, Image, Layout, LayoutTree};
use xs_math::{Rectangle, Vector2};
use xs_renderer_core::Renderer;

use crate::glsl::StandardShader;
use crate::webgl::{self, load_texture};

pub struct RendererWebGl {
    width: i32,
    height: i32,
    shaders: StandardShader,
    textures: HashMap<Image, usize>
}

impl RendererWebGl {
    pub fn new(width: i32, height: i32) -> Self {
        let shaders = StandardShader::new();
        let textures = HashMap::new();
        Self {
            width,
            height,
            shaders,
            textures
        }
    }

    pub fn load_texture(&mut self, image: &Image, width: i32, height: i32) -> usize {
        let texture_index = self.textures.len();
        *self.textures.entry(image.clone()).or_insert_with(|| {
            load_texture(image, texture_index, width, height);
            texture_index
        })
    }
}

impl Renderer for RendererWebGl {
    fn init(&mut self) {}

    fn draw_frame(&mut self, widget_tree: Box<dyn Layout>) {
        webgl::clear(TRANSPARENT);

        let mut tree = LayoutTree::new();
        let viewport = Vector2::new(self.width as f32, self.height as f32);
        self.shaders.set_viewport(viewport);
        let constraints = BoxConstraints {
            min: Vector2::new(0.0, 0.0),
            max: viewport,
        };
        let root_sbox = widget_tree.layout(&mut tree, &constraints);
        let root_lbox = xs_layout::LayoutBox::from_child(root_sbox, (0.0, 0.0));
        let root_id = tree.insert(root_lbox);
        tree.set_root(Some(root_id));
        for (_, child, offset) in tree.iter() {
            let min = offset + Vector2::new(child.bounds.x, child.bounds.y) + child.margin.min();
            let max = Vector2::new(child.bounds.width, child.bounds.height) - child.margin.max();
            if let Some(material) = &child.material {
                let rect = Rectangle::new(min.x, min.y, max.x, max.y);
                let texture = if let Some(image) = material.image.clone() {
                    Some(self.load_texture(&image, rect.width as i32, rect.height as i32))
                } else {
                    None
                };
                self.shaders.paint_rect(rect, &material, texture);
            }
        }
    }

    fn screen_width(&self) -> i32 {
        self.width
    }

    fn screen_height(&self) -> i32 {
        self.height
    }
}
