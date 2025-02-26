use xs_layout::{BoxConstraints, Color, Layout, LayoutTree};
use xs_math::{Rectangle, Vector2};

use crate::Shader;
use crate::WebGl;


pub struct RendererWebGL {
    shaders: Shader
}

impl RendererWebGL {
    pub fn new() -> Self {
        let shaders = Shader::new();
        Self {
            shaders
        }
    }

    pub fn render(&mut self, widget_tree: Box<dyn Layout>) {
        WebGl::clear(Color::black());

        let mut tree = LayoutTree::new();
        let (width, height) = WebGl::screen_size();
        let viewport = Vector2::new(width as f32, height as f32);
        self.shaders.set_viewport(viewport);

        let constraints = BoxConstraints {
            min: Vector2::new(0.0, 0.0),
            max: Vector2::new(width as f32, height as f32),
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
}
