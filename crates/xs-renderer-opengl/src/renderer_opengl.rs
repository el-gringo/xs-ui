use std::collections::HashMap;

use xs_core::Renderer;
use xs_gl::{
    GL_ARRAY_BUFFER, GL_BLEND, GL_COLOR_BUFFER_BIT, GL_DEPTH_TEST, GL_DYNAMIC_DRAW, GL_FALSE,
    GL_FLOAT, GL_LINEAR, GL_LINEAR_MIPMAP_LINEAR, GL_ONE_MINUS_SRC_ALPHA, GL_SRC_ALPHA,
    GL_STATIC_DRAW, GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_TEXTURE_MIN_FILTER, GL_TEXTURE_WRAP_S,
    GL_TEXTURE_WRAP_T, GL_TEXTURE0, GL_TRIANGLES, GL_UNSIGNED_BYTE, glActiveTexture, glBindBuffer,
    glBindTexture, glBlendFunc, glBufferData, glClear, glClearColor, glDisable, glDrawArrays,
    glEnable, glEnableVertexAttribArray, glFlush, glGenerateMipmap, glTexImage2DByBytes,
    glTexImage2DByPointer, glTexParameteri, glVertexAttribPointer, glViewport, load_texture,
};
use xs_layout::{
    BoxConstraints, LayoutBox, LayoutTree,
    Material::{Quad, Text},
    View,
};
use xs_math::Vector2;

use crate::shader::{
    ShaderGlsl,
    ShaderGlslFormat::{Float2, Float4, Int1, Int2},
};

const DEFAULT_VERTEX_SHADER: &str = include_str!("glsl/default.vert");
const DEFAUL_TFRAGMENT_SHADER: &str = include_str!("glsl/default.frag");
const TEXT_FRAGMENT_SHADER: &str = include_str!("glsl/text.frag");

pub struct RendererOpenGl {
    width: i32,
    height: i32,
    shaders: HashMap<String, ShaderGlsl>,
    fonts: HashMap<(&'static str, u16), (u32, f32)>,
}

impl RendererOpenGl {
    pub fn new(width: i32, height: i32) -> Self {
        let shaders = Default::default();
        Self {
            width,
            height,
            shaders,
            fonts: Default::default(),
        }
    }

    pub fn init_shader_default(&mut self) {
        let uniforms = vec![
            ("u_viewport".to_string(), (Int2, 1)),
            ("u_rect_min".to_string(), (Float2, 1)),
            ("u_rect_max".to_string(), (Float2, 1)),
            ("u_color".to_string(), (Float4, 1)),
            ("u_texture_enabled".to_string(), (Int1, 1)),
            ("u_texture".to_string(), (Int1, 1)),
            ("u_border_width".to_string(), (Float4, 1)),
            ("u_border_color".to_string(), (Float4, 1)),
        ];
        match ShaderGlsl::new(
            DEFAULT_VERTEX_SHADER,
            DEFAUL_TFRAGMENT_SHADER,
            uniforms,
            vec![
                ("a_position".to_string(), (Float2, 1)),
                ("a_texture_coordinate".to_string(), (Float2, 1)),
            ],
        ) {
            Ok(shader) => {
                self.shaders.insert("default".into(), shader);
            }
            Err(error) => panic!("Error in default shader: {}", error),
        }
    }

    pub fn init_shader_text(&mut self) {
        let uniforms = vec![
            ("u_viewport".to_string(), (Int2, 1)),
            ("u_rect_min".to_string(), (Float2, 1)),
            ("u_rect_max".to_string(), (Float2, 1)),
            ("u_color".to_string(), (Float4, 1)),
            ("u_texture".to_string(), (Int1, 1)),
            ("u_border_width".to_string(), (Float4, 1)),
            ("u_border_color".to_string(), (Float4, 1)),
        ];
        match ShaderGlsl::new(
            DEFAULT_VERTEX_SHADER,
            TEXT_FRAGMENT_SHADER,
            uniforms,
            vec![
                ("a_position".to_string(), (Float2, 1)),
                ("a_texture_coordinate".to_string(), (Float2, 1)),
            ],
        ) {
            Ok(shader) => {
                self.shaders.insert("text".into(), shader);
            }
            Err(error) => panic!("Error in text shader: {}", error),
        };
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
            if (child.material.is_some() || child.text_material.is_some())
                && let Some(material) = &child.material
            {
                match material {
                    Quad(borders, color, image) => {
                        #[rustfmt::skip]
                            let vertices = &[
                                min.x, min.y,
                                min.x, max.y,
                                max.x, min.y,
                                min.x, max.y,
                                max.x, min.y,
                                max.x, max.y,
                            ];
                        #[rustfmt::skip]
                            let texture_coordinates = &[
                                0.0, 0.0,
                                0.0, 1.0,
                                1.0, 0.0,
                                0.0, 1.0,
                                1.0, 0.0,
                                1.0, 1.0
                            ];
                        shader_default.use_program();

                        let a_position = shader_default.attributes.get("a_position").unwrap();
                        glBindBuffer(GL_ARRAY_BUFFER, a_position.buffer);
                        glBufferData(GL_ARRAY_BUFFER, vertices, GL_STATIC_DRAW);
                        glEnableVertexAttribArray(a_position.location as _);
                        glVertexAttribPointer(
                            a_position.location as _,
                            2,
                            GL_FLOAT,
                            GL_FALSE as _,
                            0,
                            0,
                        );
                        let a_texture_coordinate = shader_default
                            .attributes
                            .get("a_texture_coordinate")
                            .unwrap();
                        glBindBuffer(GL_ARRAY_BUFFER, a_texture_coordinate.buffer);
                        glBufferData(GL_ARRAY_BUFFER, texture_coordinates, GL_STATIC_DRAW);
                        glEnableVertexAttribArray(a_texture_coordinate.location as _);
                        glVertexAttribPointer(
                            a_texture_coordinate.location as _,
                            2,
                            GL_FLOAT,
                            GL_FALSE as _,
                            0,
                            0,
                        );

                        shader_default.set_uniform_2f(
                            "u_viewport",
                            self.width as f32,
                            self.height as f32,
                        );
                        let (r, g, b, a) = color.into();
                        shader_default.set_uniform_4f("u_color", r, g, b, a);
                        shader_default.set_uniform_2f("u_rect_min", min.x, min.y);
                        shader_default.set_uniform_2f("u_rect_max", max.x, max.y);

                        let top = borders.top.map(|b| b.width).unwrap_or(0.0);
                        let right = borders.right.map(|b| b.width).unwrap_or(0.0);
                        let bottom = borders.bottom.map(|b| b.width).unwrap_or(0.0);
                        let left = borders.left.map(|b| b.width).unwrap_or(0.0);
                        shader_default.set_uniform_4f("u_border_width", top, right, bottom, left);

                        // Use top border color (or fallback to main color)
                        let (br, bg, bb, ba) = match borders.top {
                            Some(b) => (&b.color).into(),
                            None => color.into(),
                        };
                        shader_default.set_uniform_4f("u_borderColor", br, bg, bb, ba);

                        if let Some(image) = image.clone() {
                            glActiveTexture(GL_TEXTURE0);
                            glBindTexture(GL_TEXTURE_2D, image.texture);
                            shader_default.set_uniform_1i("u_texture_enabled", 1);
                            shader_default.set_uniform_1i("u_texture", 0);
                        } else {
                            shader_default.set_uniform_1i("u_texture_enabled", 0);
                        }

                        glDrawArrays(GL_TRIANGLES, 0, 6);
                    }
                    Text(color, data) => {
                        let (font_family, font_weight, text_vertices) = data.clone();
                        let (atlas_texture, atlas_font_size) =
                            self.fonts.get(&(&font_family, font_weight)).unwrap();

                        // Position at top-left of container, flip Y for OpenGL
                        let offset_vertices: Vec<f32> = text_vertices
                            .vertices
                            .iter()
                            .enumerate()
                            .map(|(i, v)| if i % 2 == 0 { v + min.x } else { max.y - v })
                            .collect();
                        shader_text.use_program();
                        shader_text.set_uniform_2f(
                            "u_viewport",
                            self.width as f32,
                            self.height as f32,
                        );
                        let (r, g, b, a) = color.into();
                        shader_text.set_uniform_4f("u_color", r, g, b, a);
                        shader_text.set_uniform_2f("u_rect_min", min.x, min.y);
                        shader_text.set_uniform_2f("u_rect_max", max.x, max.y);
                        shader_text.set_uniform_4f("u_borderWidth", 0.0, 0.0, 0.0, 0.0);
                        shader_text.set_uniform_4f("u_borderColor", 0.0, 0.0, 0.0, 0.0);
                        glActiveTexture(GL_TEXTURE0);
                        glBindTexture(GL_TEXTURE_2D, *atlas_texture);
                        shader_text.set_uniform_1i("u_texture", 0);

                        let a_position = shader_text.attributes.get("a_position").unwrap();
                        glBindBuffer(GL_ARRAY_BUFFER, a_position.buffer);
                        glBufferData(GL_ARRAY_BUFFER, &offset_vertices, GL_DYNAMIC_DRAW);
                        glEnableVertexAttribArray(a_position.location as _);
                        glVertexAttribPointer(
                            a_position.location as _,
                            2,
                            GL_FLOAT,
                            GL_FALSE as _,
                            16,
                            0,
                        );
                        let a_texture_coordinate =
                            shader_text.attributes.get("a_texture_coordinate").unwrap();

                        glEnableVertexAttribArray(a_texture_coordinate.location as _);
                        glVertexAttribPointer(
                            a_texture_coordinate.location as _,
                            2,
                            GL_FLOAT,
                            GL_FALSE as _,
                            16,
                            8,
                        );
                        glDrawArrays(GL_TRIANGLES, 0, (text_vertices.vertices.len() / 2) as _);
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
    ) {
        let texture = self.load_texture(atlas_data, -1, width, height, false);
        self.fonts
            .insert((font_family, font_weight), (texture, font_size));
    }

    fn load_texture(
        &mut self,
        bytes: &[u8],
        image_handler: i32,
        width: u32,
        height: u32,
        alpha: bool,
    ) -> u32 {
        load_texture(bytes, image_handler, width, height, alpha)
    }
}
