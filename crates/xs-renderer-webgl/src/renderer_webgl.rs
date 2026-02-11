use xs_layout::colors::TRANSPARENT;
use xs_layout::{BoxConstraints, Layout, LayoutTree};
use xs_math::{Rectangle, Vector2};
use xs_renderer_core::Renderer;

use crate::glsl::StandardShader;
use crate::webgl;

pub struct RendererWebGl {
    width: i32,
    height: i32,
    shaders: StandardShader,
}

impl RendererWebGl {
    pub fn new(width: i32, height: i32) -> Self {
        let shaders = StandardShader::new();
        Self {
            width,
            height,
            shaders,
        }
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
                //log(&format!("render {} {} {} {}", min.x, min.y, max.x, max.y));
                self.shaders.paint_rect(rect, &material);
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
