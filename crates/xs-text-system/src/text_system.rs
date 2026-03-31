use std::collections::HashMap;

use xs_core::{Renderer, TextProperties, TextVertices};
pub use xs_text_system_glyph::{Font, FontRef, PxScale, PxScaleFont, ScaleFont, point};

use crate::{FontAtlas, layout_paragraph::layout_paragraph};

#[derive(Default)]
pub struct TextSystem {
    fonts: HashMap<(&'static str, u16), (FontRef<'static>, FontAtlas, u32)>,
}

impl TextSystem {
    pub fn new() -> Self {
        Default::default()
    }
}

impl xs_core::TextSystem for TextSystem {
    fn load_font(
        &mut self,
        font_family: &'static str,
        font_weight: u16,
        binary_font: Vec<u8>,
        renderer: &mut Box<dyn Renderer>,
    ) {
        let font = FontRef::try_from_slice(Box::leak(binary_font.into_boxed_slice())).unwrap();
        let (font_atlas, atlas_data) = FontAtlas::generate(&font, 48.);
        let atlas_texture = renderer.load_texture(
            &atlas_data,
            -1,
            font_atlas.width as _,
            font_atlas.height as _,
            true,
        );
        self.fonts.insert(
            (font_family, font_weight),
            (font, font_atlas, atlas_texture),
        );
    }

    fn render_text(
        &self,
        text: &str,
        text_properties: &TextProperties,
        constraints: &(f32, f32),
    ) -> TextVertices {
        let (font, font_atlas, atlas_texture) = self
            .fonts
            .get(&(
                text_properties.font_family.as_str(),
                text_properties.font_weight,
            ))
            .unwrap();
        make_vertices_for_string(
            font,
            font_atlas,
            atlas_texture,
            text_properties,
            text,
            constraints,
        )
    }
}

pub fn make_vertices_for_string(
    font: &FontRef<'static>,
    font_atlas: &FontAtlas,
    atlas_texture: &u32,
    text_properties: &TextProperties,
    text: &str,
    constraints: &(f32, f32),
) -> TextVertices {
    let scale = PxScale::from(text_properties.font_size as f32);
    let scaled_font = PxScaleFont {
        font: font.clone(),
        scale,
    };

    let max_width = constraints.0;

    let mut glyphs = Vec::new();
    layout_paragraph(&scaled_font, point(0.0, 0.0), max_width, text, &mut glyphs);

    let max_x = font_atlas.width as f32;
    let max_y = font_atlas.height as f32;

    let atlas_font_size = 48.0;
    let scale_factor = text_properties.font_size as f32 / atlas_font_size;

    let ascent = scaled_font.ascent();
    let _descent = scaled_font.descent();

    let mut min_x = f32::INFINITY;
    let mut max_x_pos = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y_pos = f32::NEG_INFINITY;

    let num_vertices = glyphs.len() * 6;
    let mut positions = vec![0.; (num_vertices * 2) as usize];
    let mut texcoords = vec![0.; (num_vertices * 2) as usize];
    let mut offset = 0;

    for glyph in &glyphs {
        let glyph_id = glyph.id.0;
        let letter = text.chars().find(|c| font.glyph_id(*c).0 == glyph_id);
        if let Some(c) = letter {
            if let Some(glyph_info) = font_atlas.glyphs.get(&c) {
                let glyph_width = font_atlas.scale * scale_factor;
                let glyph_height = font_atlas.scale * scale_factor;
                let x = glyph.position.x;
                let y = glyph.position.y - ascent;

                let x2 = x + glyph_width;
                let y2 = y + glyph_height;

                let u1 = glyph_info.x / max_x;
                let v1 = (glyph_info.y + font_atlas.scale - 1.) / max_y;
                let u2 = (glyph_info.x + font_atlas.scale - 1.) / max_x;
                let v2 = glyph_info.y / max_y;

                positions[offset + 0] = x;
                positions[offset + 1] = y;
                texcoords[offset + 0] = u1;
                texcoords[offset + 1] = v1;

                positions[offset + 2] = x2;
                positions[offset + 3] = y;
                texcoords[offset + 2] = u2;
                texcoords[offset + 3] = v1;

                positions[offset + 4] = x;
                positions[offset + 5] = y2;
                texcoords[offset + 4] = u1;
                texcoords[offset + 5] = v2;

                positions[offset + 6] = x;
                positions[offset + 7] = y2;
                texcoords[offset + 6] = u1;
                texcoords[offset + 7] = v2;

                positions[offset + 8] = x2;
                positions[offset + 9] = y;
                texcoords[offset + 8] = u2;
                texcoords[offset + 9] = v1;

                positions[offset + 10] = x2;
                positions[offset + 11] = y2;
                texcoords[offset + 10] = u2;
                texcoords[offset + 11] = v2;

                offset += 12;

                min_x = min_x.min(x);
                max_x_pos = max_x_pos.max(x2);
                min_y = min_y.min(y);
                max_y_pos = max_y_pos.max(y2);
            }
        }
    }

    let width = if max_x_pos.is_finite() {
        max_x_pos - min_x
    } else {
        0.0
    };
    let height = if max_y_pos.is_finite() {
        max_y_pos - min_y
    } else {
        0.0
    };

    TextVertices {
        position: (positions, offset),
        atlas_texture: *atlas_texture,
        texture_coordinate: (texcoords, offset),
        num_vertices: offset / 2,
        width,
        height,
    }
}
