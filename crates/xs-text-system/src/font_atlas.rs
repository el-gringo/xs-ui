use std::collections::HashMap;

use xs_text_system_glyph::{Font, FontRef, OutlinedGlyph, PxScale, Rect, ScaleFont};

pub struct GlyphInfo {
    pub x: f32,
    pub y: f32,
    pub bounds: Rect,
}

pub struct FontAtlas {
    pub size: u8,
    pub scale: f32,
    pub ascent: f32,
    pub spacing: f32,
    pub glyphs: HashMap<char, GlyphInfo>,
    pub height: f32,
    pub width: f32
}

impl FontAtlas {
    pub fn generate(font: &FontRef<'_>, font_size: f32) -> (Self, Vec<u8>) {
        let scale = PxScale::from(font_size);
        let font = font.into_scaled(scale);
        let mut glyphs = Vec::new();
        let space_width = if let Some(space_glyph) = font.outline_glyph(font.scaled_glyph(' ')) {
            let bounds = space_glyph.px_bounds();
            bounds.max.x - bounds.min.x
        } else {
            font_size
        };
        for code in 0x020..0x07e {
            if let Some(c) = char::from_u32(code) {
                let glyph = font.scaled_glyph(c);
                glyphs.push((c, glyph));
            }
        }
        for code in 0x0a0..0x17f {
            if let Some(c) = char::from_u32(code) {
                let glyph = font.scaled_glyph(c);
                glyphs.push((c, glyph));
            }
        }
        let glyphs: Vec<(char, OutlinedGlyph)> = glyphs
            .into_iter()
            .filter_map(|(c, g)| match font.outline_glyph(g) {
                Some(g) => Some((c, g)),
                None => None,
            })
            .collect();
        let glyph_count = glyphs.len();
        let atlas_size = (glyph_count as f32).sqrt().ceil();
        let width = atlas_size * font_size;
        let height = atlas_size * (font_size + 1.);
        let mut bitmap = vec![0; (width * height) as usize];
        let mut font_atlas = Self {
            size: atlas_size as _,
            width: width,
            height: height,
            scale: font_size,
            spacing: space_width,
            ascent: font.ascent(),
            glyphs: Default::default(),
        };
        for row in 0..atlas_size as i32 {
            for col in 0..atlas_size as i32 {
                if let Some((c, glyph)) = glyphs.get((col + row * atlas_size as i32) as usize) {
                    let bounds = glyph.px_bounds();
                    let left = col as f32 * font_size;
                    let top = row as f32 * font_size;
                    font_atlas.glyphs.insert(*c, GlyphInfo { x: left, y: top, bounds });
                    glyph.draw(|x, y, v| {
                        let x = x as f32 + left + bounds.min.x;
                        let y = y as f32 + top + bounds.min.y + font_size;
                        let index = (x + y * width) as usize;
                        bitmap[index] = (v * 255.) as u8;
                    });
                }
            }
        }
        (font_atlas, bitmap)
    }
}
