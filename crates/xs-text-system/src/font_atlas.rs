use std::collections::HashMap;

use ab_glyph::{Font, FontRef, GlyphId, OutlinedGlyph, ScaleFont};
use xs_core::{ImageFormat, Rectangle};

#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    pub atlas_x: f32,
    pub atlas_y: f32,
    pub bounds: Rectangle,
}

#[derive(Debug, Clone)]
pub struct AtlasData {
    pub size: u8,
    pub scale: f32,
    pub ascent: f32,
    pub v_advance: f32,
    pub spacing: f32,
    pub padding: f32,
    pub glyphs: HashMap<GlyphId, GlyphInfo>,
    pub height: f32,
    pub width: f32,
}

pub trait Atlas: Sized {
    fn data(&self) -> &AtlasData;
    fn width(&self) -> f32 {
        self.data().width
    }
    fn height(&self) -> f32 {
        self.data().height
    }
    fn scale(&self) -> f32 {
        self.data().scale
    }
    fn padding(&self) -> f32 {
        self.data().padding
    }
    fn glyph_info(&self, glyph_id: &GlyphId) -> Option<&GlyphInfo> {
        self.data().glyphs.get(glyph_id)
    }

    fn generate(font: &FontRef<'_>, font_size: f32, padding: f32) -> (Self, ImageFormat, Vec<u8>);
}

pub fn gather_glyphs<F: ScaleFont<impl Font>>(font: &F) -> Vec<(char, OutlinedGlyph)> {
    let mut glyphs = Vec::new();
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
    glyphs
        .into_iter()
        .filter_map(|(c, g)| font.outline_glyph(g).map(|g| (c, g)))
        .collect()
}
