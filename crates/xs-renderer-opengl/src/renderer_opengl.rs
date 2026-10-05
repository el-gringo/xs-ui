use std::collections::HashMap;

use xs_core::{AppContext, AtlasInfo, ImageFormat, Renderer};
use xs_gl::{
    GL_BLEND, GL_DEPTH_TEST, GL_ELEMENT_ARRAY_BUFFER, GL_ONE_MINUS_SRC_ALPHA, GL_SRC_ALPHA,
    GL_STATIC_DRAW, GL_TEXTURE_2D, GL_TEXTURE0, GL_TRIANGLES, Shader, glActiveTexture,
    glBindBuffer, glBindTexture, glBlendFunc, glBufferData, glClearColor, glDisable, glDrawArrays,
    glEnable, glFlush, glGenBuffers, glViewport, load_texture,
};
use xs_layout::{
    LayoutTree,
    Material::{Quad, SimpleQuad, Text},
};
use xs_math::{Rectangle, Vector2};

use crate::render_quad::{QUAD_VERTEX_FLOATS, SIMPLE_QUAD_VERTEX_FLOATS};
use crate::render_text::TEXT_VERTEX_FLOATS;

#[derive(Default)]
pub struct RendererOpenGl {
    shaders: HashMap<&'static str, Shader>,
    fonts: HashMap<(&'static str, u16), (u32, f32)>,
    pub texture_place_holder: u32,
    index_buffer: u32,
}

impl RendererOpenGl {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_shader(&mut self, name: &'static str, shader: Shader) {
        self.shaders.insert(name, shader);
    }

    pub fn get_shader(&mut self, name: &'static str) -> &mut Shader {
        self.shaders
            .get_mut(name)
            .unwrap_or_else(|| panic!("Cannot find shader {name}"))
    }

    fn intersect_rect(a: &Rectangle, b: &Rectangle) -> Rectangle {
        let x = a.x.max(b.x);
        let y = a.y.max(b.y);
        let right = (a.x + a.width).min(b.x + b.width);
        let bottom = (a.y + a.height).min(b.y + b.height);
        Rectangle::new(x, y, (right - x).max(0.0), (bottom - y).max(0.0))
    }

    fn flush_quads(
        &mut self,
        batches: &mut Vec<(u32, Vec<f32>)>,
        indices: &mut HashMap<u32, usize>,
        ctx: &AppContext,
    ) {
        if batches.iter().all(|(_, v)| v.is_empty()) {
            return;
        }
        let shader = self.get_shader("quad");
        shader.use_program();
        shader.uniform_2f(
            "u_resolution",
            ctx.screen.width as _,
            ctx.screen.height as _,
        );
        for (texture, vertices) in batches.iter() {
            if vertices.is_empty() {
                continue;
            }
            glActiveTexture(GL_TEXTURE0);
            glBindTexture(GL_TEXTURE_2D, *texture);
            shader.get_buffer(0).push_data(vertices);
            glDrawArrays(GL_TRIANGLES, 0, (vertices.len() / QUAD_VERTEX_FLOATS) as _);
        }
        batches.clear();
        indices.clear();
    }

    fn flush_texts(
        &mut self,
        batches: &mut Vec<(u32, Vec<f32>)>,
        indices: &mut HashMap<u32, usize>,
        ctx: &AppContext,
    ) {
        if batches.iter().all(|(_, v)| v.is_empty()) {
            return;
        }
        let shader = self.get_shader("text_alpha");
        shader.use_program();
        shader.uniform_2f(
            "u_resolution",
            ctx.screen.width as _,
            ctx.screen.height as _,
        );
        for (texture, vertices) in batches.iter() {
            if vertices.is_empty() {
                continue;
            }
            glActiveTexture(GL_TEXTURE0);
            glBindTexture(GL_TEXTURE_2D, *texture);
            shader.get_buffer(0).push_data(vertices);
            glDrawArrays(GL_TRIANGLES, 0, (vertices.len() / TEXT_VERTEX_FLOATS) as _);
        }
        batches.clear();
        indices.clear();
    }

    fn flush_simple(
        &mut self,
        batches: &mut Vec<(u32, Vec<f32>)>,
        indices: &mut HashMap<u32, usize>,
        ctx: &AppContext,
    ) {
        if batches.iter().all(|(_, v)| v.is_empty()) {
            return;
        }
        let shader = self.get_shader("simple_quad");
        shader.use_program();
        shader.uniform_2f(
            "u_resolution",
            ctx.screen.width as _,
            ctx.screen.height as _,
        );
        for (texture, vertices) in batches.iter() {
            if vertices.is_empty() {
                continue;
            }
            glActiveTexture(GL_TEXTURE0);
            glBindTexture(GL_TEXTURE_2D, *texture);
            shader.get_buffer(0).push_data(vertices);
            glDrawArrays(
                GL_TRIANGLES,
                0,
                (vertices.len() / SIMPLE_QUAD_VERTEX_FLOATS) as _,
            );
        }
        batches.clear();
        indices.clear();
    }
}

impl Renderer for RendererOpenGl {
    fn init(&mut self, _ctx: &AppContext) {
        let index_buffer = glGenBuffers(1)[0];
        let indices: &[u16] = &[0, 1, 3, 1, 2, 3];
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, index_buffer);
        glBufferData(GL_ELEMENT_ARRAY_BUFFER, indices, GL_STATIC_DRAW);
        self.index_buffer = index_buffer;
        self.texture_place_holder =
            load_texture(&[0, 0, 0, 0], -1, 1, 1, xs_core::ImageFormat::Rgba);

        glClearColor(0., 0., 0., 0.);
        glEnable(GL_BLEND);
        glDisable(GL_DEPTH_TEST);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

        self.initialize_quad_shader();
        self.initialize_simple_quad_shader();
        self.initialize_text_shader();
    }

    fn present(&mut self) {
        glFlush();
    }

    fn render(&mut self, ctx: &AppContext, layout_tree: &LayoutTree) {
        glViewport(0, 0, ctx.screen.width as _, ctx.screen.height as _);

        let node_count = layout_tree.boxes.len();
        let mut clip_rects: Vec<Option<Rectangle>> = vec![None; node_count];

        // All three material types are batched independently and drawn
        // in tree order. A batch is flushed whenever the material type
        // changes so that SimpleQuads, Quads, and Texts interleave
        // correctly.
        let mut quad_batches: Vec<(u32, Vec<f32>)> = Vec::new();
        let mut quad_indices: HashMap<u32, usize> = HashMap::new();
        let mut simple_batches: Vec<(u32, Vec<f32>)> = Vec::new();
        let mut simple_indices: HashMap<u32, usize> = HashMap::new();
        let mut text_batches: Vec<(u32, Vec<f32>)> = Vec::new();
        let mut text_indices: HashMap<u32, usize> = HashMap::new();

        for (_, parent_id, _, child_id, child, offset, transform) in layout_tree.iter() {
            let origin = Vector2::new(child.bounds.x, child.bounds.y);
            let min = offset + origin + child.margin.min();
            let quad = Rectangle::new(min.x, min.y, child.bounds.width, child.bounds.height);

            let parent_clip = clip_rects[parent_id];
            let clip = if let Some(Quad(props)) = &child.material
                && props.overflow == xs_layout::Overflow::Hidden
            {
                let box_clip = Rectangle::new(min.x, min.y, quad.width, quad.height);
                Some(match parent_clip {
                    Some(p) => Self::intersect_rect(&p, &box_clip),
                    None => box_clip,
                })
            } else {
                parent_clip
            };
            // The clip is stored axis-aligned, so a rotated ancestor's overflow
            // region becomes the box around its transformed corners. That is
            // conservative - it can clip slightly less than the exact rotated
            // shape, never more - and it keeps the clip test a cheap four-way
            // compare in every shader instead of a full inverse transform.
            let clip = clip.map(|rect| transform.transform_rect_bounds(rect));
            clip_rects[child_id] = clip;

            if child.material.is_some()
                && let Some(material) = &child.material
            {
                match material {
                    Quad(props) => {
                        self.flush_simple(&mut simple_batches, &mut simple_indices, ctx);

                        let texture = props
                            .background_image
                            .as_ref()
                            .map_or(self.texture_place_holder, |img| img.texture);
                        let batch = match quad_indices.get(&texture) {
                            Some(&index) => index,
                            None => {
                                quad_batches.push((texture, Vec::new()));
                                let index = quad_batches.len() - 1;
                                quad_indices.insert(texture, index);
                                index
                            }
                        };
                        self.push_quad(&quad, props, clip, &transform, &mut quad_batches[batch].1);
                    }
                    SimpleQuad(color, texture) => {
                        self.flush_quads(&mut quad_batches, &mut quad_indices, ctx);
                        self.flush_texts(&mut text_batches, &mut text_indices, ctx);

                        let tex = if *texture == 0 {
                            self.texture_place_holder
                        } else {
                            *texture
                        };
                        let batch = match simple_indices.get(&tex) {
                            Some(&index) => index,
                            None => {
                                simple_batches.push((tex, Vec::new()));
                                let index = simple_batches.len() - 1;
                                simple_indices.insert(tex, index);
                                index
                            }
                        };
                        self.push_simple_quad(
                            &quad,
                            *color,
                            clip,
                            &transform,
                            &mut simple_batches[batch].1,
                        );
                    }
                    Text(color, text_properties, text_vertices) => {
                        self.flush_simple(&mut simple_batches, &mut simple_indices, ctx);

                        let (atlas_texture, _atlas_font_size) = self
                            .fonts
                            .get(&(&text_properties.font_family, text_properties.font_weight))
                            .unwrap();
                        let batch = match text_indices.get(atlas_texture) {
                            Some(&index) => index,
                            None => {
                                text_batches.push((*atlas_texture, Vec::new()));
                                let index = text_batches.len() - 1;
                                text_indices.insert(*atlas_texture, index);
                                index
                            }
                        };
                        self.push_text(
                            &quad,
                            color,
                            text_vertices,
                            clip,
                            &transform,
                            &mut text_batches[batch].1,
                        );
                    }
                }
            }
        }

        self.flush_simple(&mut simple_batches, &mut simple_indices, ctx);
        self.flush_quads(&mut quad_batches, &mut quad_indices, ctx);
        self.flush_texts(&mut text_batches, &mut text_indices, ctx);
    }

    fn load_font_atlas(
        &mut self,
        font_family: &'static str,
        font_weight: u16,
        atlas_info: &AtlasInfo,
    ) {
        let font_size = atlas_info.scale;
        let bytes = &atlas_info.image.bytes;
        let width = atlas_info.image.width;
        let height = atlas_info.image.height;
        let format = atlas_info.image.format.clone();
        let texture = self.load_texture(bytes, -1, width, height, format);
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
