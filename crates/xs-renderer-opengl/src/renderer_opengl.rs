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
    Material::{Quad, SimpleQuad, Text, Vector},
};
use xs_math::{Rectangle, Vector2};

use crate::render_quad::{QUAD_VERTEX_FLOATS, SIMPLE_QUAD_VERTEX_FLOATS};
use crate::render_text::TEXT_VERTEX_FLOATS;
use crate::render_vector::DRAWN_STROKE_VERTEX_FLOATS;

/// The shader programs a frame batches into, and the slot each one owns in
/// [`Frame`]. The order is the order the arenas are uploaded in.
const SH_QUAD: usize = 0;
const SH_SIMPLE: usize = 1;
const SH_TEXT: usize = 2;
const SH_STROKE: usize = 3;
const SHADER_COUNT: usize = 4;

const SHADER_NAMES: [&str; SHADER_COUNT] = ["quad", "simple_quad", "text_alpha", "vector_stroke"];
const SHADER_FLOATS: [usize; SHADER_COUNT] = [
    QUAD_VERTEX_FLOATS,
    SIMPLE_QUAD_VERTEX_FLOATS,
    TEXT_VERTEX_FLOATS,
    DRAWN_STROKE_VERTEX_FLOATS,
];

/// Geometry written since this shader was last flushed: a texture and the
/// span of floats it occupies in that shader's arena.
struct Batch {
    texture: u32,
    first: usize,
    len: usize,
}

/// One `glDrawArrays`, in the order the frame issues them.
struct DrawCall {
    shader: usize,
    texture: u32,
    /// First vertex in that shader's arena, which is what `glDrawArrays` wants.
    first: i32,
    count: i32,
}

/// Everything one frame accumulates, kept on the renderer so the vectors keep
/// their capacity between frames.
///
/// The arenas are the reason this exists. Batching by texture alone still means
/// a separate `glBufferData` per batch, and peertube's 73 draws per frame were
/// 73 of them - measured on a Mali-TDVX at 7.8us each, which is more than the
/// draws themselves cost. Since the vertex data for a whole frame is known
/// before any of it is drawn, each arena is uploaded once and the draws then
/// index into it.
#[derive(Default)]
struct Frame {
    vertices: [Vec<f32>; SHADER_COUNT],
    pending: [Vec<Batch>; SHADER_COUNT],
    /// Texture -> slot in `pending`, so a run of same-textured nodes merges into
    /// one draw instead of one each.
    indices: [HashMap<u32, usize>; SHADER_COUNT],
    draws: Vec<DrawCall>,
    /// The one shader with an unrecorded run, if any.
    ///
    /// This is what makes the recorded draws come out in tree order. A batch is
    /// only turned into a draw call when something flushes it, so without an order
    /// to flush by, geometry queued between two of another shader's is recorded
    /// after both and painted underneath them.
    ///
    /// One, not several: a shader's run is closed the moment another shader is
    /// added, so at most one can be open. Closing a run on *any* change of shader
    /// is also the rule that fixed the vector fill losing to the panel background
    /// under it, which is in a different shader and is queued by the node visited
    /// *before* the fill.
    current: Option<usize>,
}

#[derive(Default)]
pub struct RendererOpenGl {
    shaders: HashMap<&'static str, Shader>,
    fonts: HashMap<(&'static str, u16), (u32, f32)>,
    pub texture_place_holder: u32,
    index_buffer: u32,
    frame: Frame,
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

    /// Runs `emit` into the given shader's arena and records the span as a
    /// batch of `texture`, merging into the run already collected for that
    /// texture if there is one.
    ///
    /// A shader's run ends when a *different* shader's begins, and every run ended
    /// so far is recorded before the new one starts.
    ///
    /// Continuing an existing run just appends, which is where the batching comes
    /// from: a screen of plain quads is one run of one shader and one draw. This
    /// bites hardest on a vector fill, because a fill shares the solid-colour
    /// arena while the background it has to sit on top of is in the decorated one -
    /// and a `Div`'s own material rides the box its *parent* made for it, which the
    /// tree visits before the `Div`'s children. The panel's background and the
    /// vector inside it are therefore two different nodes, in that order, and a fill
    /// recorded lazily loses to the background that was queued before it.
    fn add(
        &self,
        frame: &mut Frame,
        shader: usize,
        texture: u32,
        emit: impl FnOnce(&mut Vec<f32>),
    ) {
        if frame.current != Some(shader) {
            if let Some(queued) = frame.current.take() {
                self.flush(frame, queued);
            }
            frame.current = Some(shader);
        }
        let arena = &mut frame.vertices[shader];
        let first = arena.len();
        emit(arena);
        let len = arena.len() - first;
        if len == 0 {
            return;
        }
        match frame.indices[shader].get(&texture) {
            // Merging is only valid when this geometry starts exactly where that
            // batch ends. `first + len` is what a draw call reads, so adding the
            // length regardless would make one batch span whatever another
            // texture's vertices were appended in between - and a plain
            // background sitting between two others that share a texture is the
            // case, since backgrounds and vector fills both draw from the
            // placeholder. A batch that cannot be extended starts its own, and
            // leaves the one it was going to extend alone.
            Some(&slot)
                if frame.pending[shader][slot].first + frame.pending[shader][slot].len == first =>
            {
                frame.pending[shader][slot].len += len;
            }
            _ => {
                frame.pending[shader].push(Batch {
                    texture,
                    first,
                    len,
                });
                frame.indices[shader].insert(texture, frame.pending[shader].len() - 1);
            }
        }
    }

    /// Ends the current run of batches for `shader`: they are recorded as draw
    /// calls, in order, and the next node starts a fresh run.
    fn flush(&self, frame: &mut Frame, shader: usize) {
        let floats = SHADER_FLOATS[shader];
        for batch in frame.pending[shader].drain(..) {
            frame.draws.push(DrawCall {
                shader,
                texture: batch.texture,
                first: (batch.first / floats) as i32,
                count: (batch.len / floats) as i32,
            });
        }
        frame.indices[shader].clear();
    }

    /// Uploads each arena once and replays the recorded draws.
    fn submit(&mut self, frame: &mut Frame, ctx: &AppContext) {
        let draws = std::mem::take(&mut frame.draws);
        for (shader, name) in SHADER_NAMES.iter().enumerate() {
            let vertices = &frame.vertices[shader];
            if !vertices.is_empty() {
                self.get_shader(name).get_buffer(0).push_data(vertices);
            }
        }

        let mut bound_shader = usize::MAX;
        let mut bound_texture = u32::MAX;
        // Device pixels, because that is what the vertices are in: a style is
        // scaled by `screen.scale` before it reaches layout, so the drawing
        // buffer and the geometry share one space. The viewport below reads the
        // same value and the two have to agree or everything lands off centre.
        let (width, height) = ctx.screen.framebuffer_size();
        for draw in &draws {
            // `use_program` rebinds the vertex attributes and `u_resolution` is
            // the same for every node of a frame, so both are done once per
            // program rather than once per batch as they were.
            if draw.shader != bound_shader {
                let shader = self.get_shader(SHADER_NAMES[draw.shader]);
                shader.use_program();
                shader.uniform_2f("u_resolution", width as _, height as _);
                bound_shader = draw.shader;
                bound_texture = u32::MAX;
            }
            if draw.texture != bound_texture {
                glActiveTexture(GL_TEXTURE0);
                glBindTexture(GL_TEXTURE_2D, draw.texture);
                bound_texture = draw.texture;
            }
            glDrawArrays(GL_TRIANGLES, draw.first, draw.count);
        }

        for arena in &mut frame.vertices {
            arena.clear();
        }
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
        self.initialize_vector_stroke_shader();
    }

    fn present(&mut self) {
        glFlush();
    }

    fn render(&mut self, ctx: &AppContext, layout_tree: &LayoutTree) {
        let (width, height) = ctx.screen.framebuffer_size();
        glViewport(0, 0, width as _, height as _);

        let node_count = layout_tree.boxes.len();
        let mut clip_rects: Vec<Option<Rectangle>> = vec![None; node_count];

        // All three material types are batched independently and drawn in tree
        // order. A batch is flushed whenever the material type changes so that
        // SimpleQuads, Quads and Texts interleave correctly.
        //
        // Flushing only ends a run of batches now - it records draw calls and
        // touches no GL. Nothing reaches the driver until `submit`, which
        // uploads each shader's arena once and replays the recorded draws.
        let mut frame = std::mem::take(&mut self.frame);

        for (_, parent_id, _, child_id, child, offset, transform) in layout_tree.iter() {
            let origin = Vector2::new(child.bounds.x, child.bounds.y);
            let min = offset + origin + child.margin.min();
            let quad = Rectangle::new(min.x, min.y, child.bounds.width, child.bounds.height);

            // A stored clip is already in screen coordinates, so the only thing
            // that still needs transforming is this node's own box. Intersecting
            // with the parent's clip before transforming it - which is what this
            // used to do, transforming the result as well - applied the composed
            // transform a second time to every inherited clip, so a descendant
            // with a translate of its own was clipped against a rectangle shifted
            // by that same translate and could draw straight out of its
            // ancestor's `overflow: hidden`.
            let parent_clip = clip_rects[parent_id];
            let clip = if let Some(Quad(props)) = &child.material
                && props.overflow == xs_layout::Overflow::Hidden
            {
                let box_clip = Rectangle::new(min.x, min.y, quad.width, quad.height);
                // The clip is stored axis-aligned, so a rotated ancestor's
                // overflow region becomes the box around its transformed
                // corners. That is conservative - it can clip slightly less than
                // the exact rotated shape, never more - and it keeps the clip
                // test a cheap four-way compare in every shader instead of a full
                // inverse transform.
                let box_clip = transform.transform_rect_bounds(box_clip);
                Some(match parent_clip {
                    Some(p) => Self::intersect_rect(&p, &box_clip),
                    None => box_clip,
                })
            } else {
                parent_clip
            };
            clip_rects[child_id] = clip;

            if child.material.is_some()
                && let Some(material) = &child.material
            {
                match material {
                    Quad(props) => {
                        let texture = props
                            .background_image
                            .as_ref()
                            .map_or(self.texture_place_holder, |img| img.texture);
                        self.add(&mut frame, SH_QUAD, texture, |out| {
                            self.push_quad(&quad, props, clip, &transform, out);
                        });
                    }
                    SimpleQuad(color, texture) => {
                        let tex = if *texture == 0 {
                            self.texture_place_holder
                        } else {
                            *texture
                        };
                        // A quad with no alpha and no texture of its own blends
                        // every pixel it covers to exactly what was already
                        // there, so it can be dropped whole. Layout produces a
                        // lot of them: peertube's card wraps its title in two
                        // `background_color(Color::TRANSPARENT)` divs, which is
                        // 34k pixels of pure overdraw per card, 500k a frame.
                        // The frame is fill-rate bound on the TV GPU (measured:
                        // frame time scales with the drawing buffer while the
                        // JS cost does not), so this is worth more than the
                        // draw call it saves.
                        let draws_nothing = color.a <= 0.0 && *texture == 0;
                        if !draws_nothing {
                            self.add(&mut frame, SH_SIMPLE, tex, |out| {
                                self.push_simple_quad(&quad, *color, clip, &transform, out);
                            });
                        }
                    }
                    Text(color, text_properties, text_vertices) => {
                        let (atlas_texture, _atlas_font_size) = self
                            .fonts
                            .get(&(&text_properties.font_family, text_properties.font_weight))
                            .unwrap();
                        self.add(&mut frame, SH_TEXT, *atlas_texture, |out| {
                            self.push_text(&quad, color, text_vertices, clip, &transform, out);
                        });
                    }
                    Vector(geometry) => {
                        if geometry.is_empty() {
                            continue;
                        }
                        // The fill goes in with the plain solid-colour quads -
                        // that is the whole reason it needs no shader of its own -
                        // and the placeholder, not 0. A fill samples no texture, but
                        // it still goes through the simple-quad fragment shader,
                        // which blends by the texture's alpha; texture object 0 is
                        // *no* texture, and sampling one is undefined - it comes back
                        // opaque black on most drivers, which paints every fill in
                        // the app black.
                        if !geometry.fill_vertices.is_empty() {
                            self.add(&mut frame, SH_SIMPLE, self.texture_place_holder, |out| {
                                self.push_vector_fill(min, clip, geometry, &transform, out);
                            });
                        }
                        if !geometry.stroke_vertices.is_empty() {
                            self.add(&mut frame, SH_STROKE, 0, |out| {
                                self.push_vector_stroke(
                                    min,
                                    transform.scale_factor(),
                                    clip,
                                    geometry,
                                    &transform,
                                    out,
                                );
                            });
                        }
                    }
                }
            }
        }

        if let Some(queued) = frame.current.take() {
            self.flush(&mut frame, queued);
        }
        self.submit(&mut frame, ctx);
        self.frame = frame;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_quad::SIMPLE_QUAD_VERTEX_FLOATS;

    /// One simple quad *vertex*, in the 12 floats `push_simple_quad` actually
    /// writes: position, texture coordinate, colour, clip rect.
    ///
    /// The stride is not a formality. A helper that wrote 11 floats per vertex
    /// drops a vertex per quad here and the only symptom is a `count` one short.
    fn vertex(vec: &mut Vec<f32>, x: f32) {
        let before = vec.len();
        vec.extend_from_slice(&[
            x, 0.0, // position
            0.0, 0.0, // texture coordinate
            1.0, 1.0, 1.0, 1.0, // colour
            0.0, 0.0, 1920.0, 1080.0, // clip rect
        ]);
        assert_eq!(vec.len() - before, SIMPLE_QUAD_VERTEX_FLOATS);
    }

    /// A batch is `first`/`len` over one arena, so merging two spans that are not
    /// adjacent hands one draw call geometry that belongs to whatever was queued in
    /// between. Textures A -> B -> A used to produce a single draw for A covering
    /// all three, and B was never drawn at all.
    #[test]
    fn a_batch_never_spans_another_textures_vertices() {
        let renderer = RendererOpenGl::default();
        let mut frame = Frame::default();
        let a = 7;
        let b = 9;
        for (texture, x) in [(a, 0.0), (b, 100.0), (a, 200.0)] {
            renderer.add(&mut frame, SH_SIMPLE, texture, |arena| {
                vertex(arena, x);
            });
        }
        if let Some(queued) = frame.current.take() {
            renderer.flush(&mut frame, queued);
        }

        let batches = &frame.pending[SH_SIMPLE];
        let mut spans: Vec<(u32, i32, i32)> = frame
            .draws
            .iter()
            .filter(|d| d.shader == SH_SIMPLE)
            .map(|d| (d.texture, d.first, d.first + d.count))
            .collect();
        // By position, which is the order they are drawn in.
        spans.sort_unstable_by_key(|span| span.1);

        // Two draws for A, one for B, and no draw covers another's vertices.
        assert_eq!(spans.len(), 3, "{spans:?}");
        assert_eq!(spans[0], (a, 0, 1));
        assert_eq!(spans[1], (b, 1, 2));
        assert_eq!(spans[2], (a, 2, 3));
        assert!(batches.is_empty(), "everything was flushed");
    }

    /// Adjacent same-textured geometry is the whole point of the batching: it is
    /// one draw instead of one each.
    #[test]
    fn adjacent_same_textured_geometry_stays_one_draw() {
        let renderer = RendererOpenGl::default();
        let mut frame = Frame::default();
        for x in [0.0, 100.0, 200.0] {
            renderer.add(&mut frame, SH_SIMPLE, 7, |arena| {
                vertex(arena, x);
            });
        }
        if let Some(queued) = frame.current.take() {
            renderer.flush(&mut frame, queued);
        }
        let draws: Vec<_> = frame
            .draws
            .iter()
            .filter(|d| d.shader == SH_SIMPLE)
            .collect();
        assert_eq!(draws.len(), 1);
        // `first`/`count` are vertices, not floats, and the stride is the
        // vertex's own size - which is the other half of the reason the old 14 for
        // a 12 float vertex was dead weight rather than a visual bug.
        assert_eq!(draws[0].count, 3);
    }

    /// The reason the flush happens on a shader *change* rather than when a
    /// shader's geometry happens to end: a vector fill rides the simple quad arena
    /// while the panel it sits on rides the decorated one, and the panel's node is
    /// visited first. The fill has to be painted second.
    #[test]
    fn a_shader_change_flushes_the_run_that_was_open() {
        let renderer = RendererOpenGl::default();
        let mut frame = Frame::default();
        renderer.add(&mut frame, SH_QUAD, 0, |arena| {
            vertex(arena, 0.0);
        });
        assert!(frame.draws.is_empty(), "the run is still open");
        renderer.add(&mut frame, SH_SIMPLE, 0, |arena| {
            vertex(arena, 0.0);
        });
        let quad_draws = frame.draws.iter().filter(|d| d.shader == SH_QUAD).count();
        assert_eq!(quad_draws, 1, "the quad run was recorded before the fill's");
    }
}
