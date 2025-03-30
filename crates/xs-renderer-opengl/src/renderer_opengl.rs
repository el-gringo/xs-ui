use std::collections::HashMap;

use xs_gl_sys::{glClear, glClearColor, glViewport, GL_COLOR_BUFFER_BIT};
use xs_layout::{BoxConstraints, Color, Layout, LayoutBox, LayoutTree};
use xs_math::{Vector2, Vector3};
use xs_renderer_core::Renderer;

use crate::mesh::Mesh;
use crate::shader::{ShaderGlsl, ShaderGlslFormat};

const VERTEX_SHADER: &str = include_str!("glsl/default.vert");
const FRAGMENT_SHADER: &str = include_str!("glsl/default.frag");

pub struct RendererOpenGl {
    width: i32,
    height: i32,
    shaders: HashMap<String, ShaderGlsl>
}

impl RendererOpenGl {
    pub fn new(width: i32, height: i32) -> Self {
        let shaders = HashMap::new();
        Self { width, height, shaders }
    }
}

impl Renderer for RendererOpenGl {
    fn init(&mut self, _egl_get_proc_address: unsafe extern "C" fn(*const i8) -> Option<unsafe extern "C" fn()>) {
        /*
        unsafe {
            gl::load_gl_funcs(|proc| {
                match std::ffi::CString::new(proc) {
                    Ok(name) => egl_get_proc_address(name.as_ptr() as _),
                    _ => None
                }
            });
        }
        */
        let color = Color::black();
        unsafe {
            let default_shader = ShaderGlsl::new(
                VERTEX_SHADER,
                FRAGMENT_SHADER,
                vec![
                    ("u_viewport", (ShaderGlslFormat::Float2, 1)),
                    ("u_color", (ShaderGlslFormat::Float4, 1)),
                    ("u_rect_min", (ShaderGlslFormat::Float2, 1)),
                    ("u_rect_max", (ShaderGlslFormat::Float2, 1))
                ],
                vec![
                    ("a_position", (ShaderGlslFormat::Float2, 1))
                ]
            );
            self.shaders.insert("default".into(), default_shader);
            glViewport(0, 0, self.width, self.height);
            glClearColor(color.r, color.g, color.b, 1.0);
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
                let min = offset + Vector2::new(child.bounds.x, child.bounds.y) + child.margin.min();
                let max = min + Vector2::new(child.bounds.width, child.bounds.height) - child.margin.max();
                if let Some(material) = &child.material {
                    let vertices = [
                        Vector3::new(min.x, min.y, 0.0),
                        Vector3::new(min.x, max.y, 0.0),
                        Vector3::new(max.x, min.y, 0.0),
                        Vector3::new(min.x, max.y, 0.0),
                        Vector3::new(max.x, min.y, 0.0),
                        Vector3::new(max.x, max.y, 0.0),
                    ];
                    let mut mesh = Mesh::new(&vertices);
                    let color = material.fill;
                    mesh.draw(
                        shader,
                        vec![
                            ("u_viewport", [self.width, self.height].as_ptr() as _),
                            ("u_color", color.to_array().as_ptr() as _),
                            ("u_rect_min", [min.x, min.y].as_ptr() as _),
                            ("u_rect_max", [max.x, max.y].as_ptr() as _)
                        ]
                    );
                }
            }
        }
    }
}
