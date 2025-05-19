use std::collections::HashMap;

use xs_gl_sys::{
    glClear,
    glClearColor,
    glViewport,
    load_gl_funcs,
    GL_COLOR_BUFFER_BIT
};
use xs_layout::{
    colors::BLACK,
    BoxConstraints,
    Color,
    Layout,
    LayoutBox,
    LayoutTree,
};
use xs_math::Vector2;
use xs_renderer_core::Renderer;

use crate::mesh::Mesh;
use crate::shader::{
    ShaderGlsl,
    ShaderGlslFormat::{Float1, Float2, Float4, Int1, Int2},
};

const VERTEX_SHADER: &str = include_str!("glsl/default.vert");
const FRAGMENT_SHADER: &str = include_str!("glsl/default.frag");

pub struct RendererOpenGl {
    width: i32,
    height: i32,
    shaders: HashMap<String, ShaderGlsl>,
}

impl RendererOpenGl {
    pub fn new(width: i32, height: i32) -> Self {
        let shaders = HashMap::new();
        Self {
            width,
            height,
            shaders,
        }
    }
}

impl Renderer for RendererOpenGl {
    fn init(
        &mut self,
        egl_get_proc_address: unsafe extern "C" fn(*const i8) -> Option<unsafe extern "C" fn()>,
    ) {
        const BACKGROUND: Color = BLACK;
        unsafe {
            load_gl_funcs(|proc| match std::ffi::CString::new(proc) {
                Ok(name) => egl_get_proc_address(name.as_ptr() as _),
                _ => None,
            });
            let mut uniforms = vec![
                ("u_viewport".to_string(), (Int2, 1)),
                ("u_color".to_string(), (Float4, 1)),
                ("u_rect_min".to_string(), (Float2, 1)),
                ("u_rect_max".to_string(), (Float2, 1)),
            ];
            for index in 0..4 {
                let prefix = format!("u_borders[{}]", index);
                uniforms.push((format!("{}.enabled", prefix), (Int1, 1 as i32)));
                uniforms.push((format!("{}.width", prefix), (Float1, 1 as i32)));
                uniforms.push((format!("{}.color", prefix), (Float4, 1 as i32)));
            }
            let default_shader = ShaderGlsl::new(
                VERTEX_SHADER,
                FRAGMENT_SHADER,
                uniforms,
                vec![("a_position".to_string(), (Float2, 1))],
            );
            self.shaders.insert("default".into(), default_shader);
            glViewport(0, 0, self.width, self.height);
            glClearColor(BACKGROUND.r, BACKGROUND.g, BACKGROUND.b, BACKGROUND.a);
        }
    }

    fn draw_frame(&mut self, widget_tree: Box<dyn Layout>) {
        if let Some(shader) = self.shaders.get_mut("default") {
            unsafe {
                glClear(GL_COLOR_BUFFER_BIT);
            }
            let mut tree = LayoutTree::new();
            let constraints = BoxConstraints {
                min: Vector2::new(0.0, 0.0),
                max: Vector2::new(self.width as f32, self.height as f32),
            };
            let root_sbox = widget_tree.layout(&mut tree, &constraints);
            let root_lbox = LayoutBox::from_child(root_sbox, (0.0, 0.0));
            let root_id = tree.insert(root_lbox);
            tree.set_root(Some(root_id));
            for (_, child, offset) in tree.iter() {
                let min =
                    offset + Vector2::new(child.bounds.x, child.bounds.y) + child.margin.min();
                let max = min + Vector2::new(child.bounds.width, child.bounds.height)
                    - child.margin.max();
                if let Some(material) = &child.material {
                    let vertices = &[
                        min.x, min.y, 0.0, min.x, max.y, 0.0, max.x, min.y, 0.0, min.x, max.y, 0.0,
                        max.x, min.y, 0.0, max.x, max.y, 0.0,
                    ];
                    let mut mesh = Mesh::new(vertices);
                    mesh.begin();
                    let color = material.fill;
                    shader.use_program();
                    shader.set_uniform_2f("u_viewport", self.width as f32, self.height as f32);
                    shader.set_uniform_4f("u_color", color.r, color.g, color.b, color.a);
                    shader.set_uniform_2f("u_rect_min", min.x, min.y);
                    shader.set_uniform_2f("u_rect_max", max.x, max.y);
                    for index in 0..4 {
                        let prefix = "u_borders[".to_owned() + index.to_string().as_str() + "]";
                        let border = match index {
                            0 => material.borders.top,
                            1 => material.borders.bottom,
                            2 => material.borders.left,
                            3 => material.borders.right,
                            _ => None,
                        };
                        if let Some(border) = border {
                            let name = prefix.clone() + ".enabled";
                            shader.set_uniform_1i(&name, 1);
                            let name = prefix.clone() + ".width";
                            shader.set_uniform_1f(&name, border.width);
                            let name = prefix.clone() + ".color";
                            shader.set_uniform_4f(
                                &name,
                                border.color.r,
                                border.color.g,
                                border.color.b,
                                border.color.a,
                            );
                        } else {
                            let name = prefix.clone() + ".enabled";
                            shader.set_uniform_1i(&name, 0);
                        }
                    }
                    shader.apply_attributes();
                    mesh.end();
                }
            }
        }
    }
}
