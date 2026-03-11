use std::collections::HashMap;

use xs_core::Renderer;
use xs_gl_sys::{
    GL_ALPHA, GL_BLEND, GL_CLAMP_TO_EDGE, GL_COLOR_BUFFER_BIT, GL_DEPTH_TEST, GL_LINEAR,
    GL_ONE_MINUS_SRC_ALPHA, GL_RGBA, GL_SRC_ALPHA, GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER,
    GL_TEXTURE_MIN_FILTER, GL_TEXTURE_WRAP_S, GL_TEXTURE_WRAP_T, GL_UNPACK_ALIGNMENT,
    GL_UNSIGNED_BYTE, glBindTexture, glBlendFunc, glClear, glClearColor,
    glDisable, glEnable, glFlush, glGenTextures, glGenerateMipmap, glPixelStorei,
    glTexImage2DByBytes, glTexImage2DByPointer, glTexParameteri, glViewport,
};
use xs_layout::{
    BoxConstraints, LayoutBox, LayoutTree,
    Material::{Quad, Text},
    View,
};
use xs_math::Vector2;

use crate::mesh::Mesh;
use crate::shader::{
    ShaderGlsl,
    ShaderGlslFormat::{Float1, Float2, Float4, Int1, Int2},
};

const VERTEX_SHADER_DEFAULT: &str = include_str!("glsl/default.vert");
const FRAGMENT_SHADER_DEFAULT: &str = include_str!("glsl/default.frag");
const FRAGMENT_SHADER_TEXT: &str = include_str!("glsl/text.frag");

fn is_power_of_2(value: i32) -> bool {
    (value & (value - 1)) == 0
}

pub struct RendererOpenGl {
    width: i32,
    height: i32,
    shaders: HashMap<String, ShaderGlsl>
}

impl RendererOpenGl {
    pub fn new(width: i32, height: i32) -> Self {
        let shaders = Default::default();
        Self {
            width,
            height,
            shaders
        }
    }

    pub fn init_shader_default(&mut self) {
        let mut uniforms = vec![
            ("u_viewport".to_string(), (Int2, 1)),
            ("u_color".to_string(), (Float4, 1)),
            ("u_rect_min".to_string(), (Float2, 1)),
            ("u_rect_max".to_string(), (Float2, 1)),
            ("u_texture_enabled".to_string(), (Float2, 1)),
            ("u_texture".to_string(), (Int1, 1)),
        ];
        for index in 0..4 {
            let prefix = format!("u_borders[{}]", index);
            uniforms.push((format!("{}.enabled", prefix), (Int1, 1)));
            uniforms.push((format!("{}.width", prefix), (Float1, 1)));
            uniforms.push((format!("{}.color", prefix), (Float4, 1)));
        }
        let shader = ShaderGlsl::new(
            VERTEX_SHADER_DEFAULT,
            FRAGMENT_SHADER_DEFAULT,
            uniforms,
            vec![("a_position".to_string(), (Float2, 1))],
        );
        self.shaders.insert("default".into(), shader);
    }

    pub fn init_shader_text(&mut self) {
        let uniforms = vec![
            ("u_viewport".to_string(), (Int2, 1)),
            ("u_color".to_string(), (Float4, 1)),
            ("u_rect_min".to_string(), (Float2, 1)),
            ("u_rect_max".to_string(), (Float2, 1)),
            ("u_texture".to_string(), (Int1, 1)),
        ];
        let shader = ShaderGlsl::new(
            VERTEX_SHADER_DEFAULT,
            FRAGMENT_SHADER_TEXT,
            uniforms,
            vec![("a_position".to_string(), (Float2, 1))],
        );
        self.shaders.insert("text".into(), shader);
    }
}

impl Renderer for RendererOpenGl {
    fn init(&mut self) {
        self.init_shader_default();
        self.init_shader_text();
    }

    fn screen_width(&self) -> i32 {
        self.width
    }

    fn screen_height(&self) -> i32 {
        self.height
    }

    fn draw_frame(&mut self, widget_tree: &View) {
        let shader_default = self.shaders.get("default").unwrap();
        let shader_text = self.shaders.get("text").unwrap();

        glViewport(0, 0, self.width, self.height);
        glClear(GL_COLOR_BUFFER_BIT);
        glClearColor(0., 0., 0., 0.);
        //glColorMask(true, true, true, false);
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
            let size = Vector2::new(child.bounds.width, child.bounds.height);
            let min = offset + origin + child.margin.min();
            let max = offset + origin + size - child.margin.max();
            if child.material.is_some() || child.text_material.is_some() {
                let vertices = &[
                    min.x, min.y, 0.0, min.x, max.y, 0.0, max.x, min.y, 0.0, min.x, max.y, 0.0,
                    max.x, min.y, 0.0, max.x, max.y, 0.0,
                ];
                let mut mesh = Mesh::new(vertices);
                mesh.begin();
                if let Some(material) = &child.material {
                    match material {
                        Quad(borders, color, image) => {
                            shader_default.use_program();
                            shader_default.set_uniform_2f(
                                "u_viewport",
                                self.width as f32,
                                self.height as f32,
                            );
                            shader_default
                                .set_uniform_4f("u_color", color.r, color.g, color.b, color.a);
                            shader_default.set_uniform_2f("u_rect_min", min.x, min.y);
                            shader_default.set_uniform_2f("u_rect_max", max.x, max.y);
                            for index in 0..4 {
                                let prefix =
                                    "u_borders[".to_owned() + index.to_string().as_str() + "]";
                                let border = match index {
                                    0 => borders.top,
                                    1 => borders.bottom,
                                    2 => borders.left,
                                    3 => borders.right,
                                    _ => None,
                                };
                                if let Some(border) = border {
                                    let name = prefix.clone() + ".enabled";
                                    shader_default.set_uniform_1i(&name, 1);
                                    let name = prefix.clone() + ".width";
                                    shader_default.set_uniform_1f(&name, border.width);
                                    let name = prefix.clone() + ".color";
                                    shader_default.set_uniform_4f(
                                        &name,
                                        border.color.r,
                                        border.color.g,
                                        border.color.b,
                                        border.color.a,
                                    );
                                } else {
                                    let name = prefix.clone() + ".enabled";
                                    shader_default.set_uniform_1i(&name, 0);
                                }
                            }
                            if let Some(image) = image.clone() {
                                //glActiveTexture(image.texture_unit);
                                glBindTexture(GL_TEXTURE_2D, image.texture);
                                shader_default.set_uniform_1i("u_texture_enabled", 1);
                                shader_default.set_uniform_1i("u_texture", 0);
                            } else {
                                shader_default.set_uniform_1i("u_texture_enabled", 0);
                            }
                            shader_default.apply_attributes();
                        }
                        Text(color, image) => {
                            shader_text.use_program();
                            shader_text.set_uniform_2f(
                                "u_viewport",
                                self.width as f32,
                                self.height as f32,
                            );
                            shader_text
                                .set_uniform_4f("u_color", color.r, color.g, color.b, color.a);
                            shader_text.set_uniform_2f("u_rect_min", min.x, min.y);
                            shader_text.set_uniform_2f("u_rect_max", max.x, max.y);
                            if let Some(image) = image.clone() {
                                //glActiveTexture(image.texture_unit);
                                glBindTexture(GL_TEXTURE_2D, image.texture);
                                shader_text.set_uniform_1i("u_texture", 0);
                            }
                            shader_text.apply_attributes();
                        }
                    }
                }
                mesh.end();
            }
        }
        glFlush();
    }

    fn load_texture(
        &mut self,
        bytes: &[u8],
        image_handler: i32,
        width: u32,
        height: u32,
        alpha: bool,
    ) -> u32 {
        let internal_format = if alpha { GL_ALPHA } else { GL_RGBA };
        let format = if alpha { GL_ALPHA } else { GL_RGBA };
        let texture = glGenTextures(1)[0];
        glBindTexture(GL_TEXTURE_2D, texture);
        glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
        if is_power_of_2(width as _) && is_power_of_2(height as _) {
            glGenerateMipmap(GL_TEXTURE_2D);
        } else {
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S as _, GL_CLAMP_TO_EDGE as _);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T as _, GL_CLAMP_TO_EDGE as _);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as _);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as _);
        }
        if image_handler > 0 {
            glTexImage2DByPointer(
                GL_TEXTURE_2D,
                0,
                internal_format as _,
                width as _,
                height as _,
                0,
                format as _,
                GL_UNSIGNED_BYTE,
                image_handler as _,
            );
        } else if !bytes.is_empty() {
            glTexImage2DByBytes(
                GL_TEXTURE_2D,
                0,
                internal_format as _,
                width as _,
                height as _,
                0,
                format as _,
                GL_UNSIGNED_BYTE,
                bytes,
            );
        }
        texture
    }
}
