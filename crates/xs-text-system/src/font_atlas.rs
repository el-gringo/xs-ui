use ab_glyph::{Font, OutlinedGlyph, ScaleFont};
use xs_core::{AtlasInfo, AtlasOptions, GlyphInfo};

pub trait Atlas: Sized {
    fn info(&self) -> &AtlasInfo;
    fn width(&self) -> f32 {
        self.info().image.width as _
    }
    fn height(&self) -> f32 {
        self.info().image.height as _
    }
    fn scale(&self) -> f32 {
        self.info().scale
    }
    fn padding(&self) -> f32 {
        self.info().padding
    }
    fn glyph_info(&self, glyph_id: &u16) -> Option<&GlyphInfo> {
        self.info().glyphs.get(glyph_id)
    }

    fn generate_with_options(font_data: &[u8], atlas_options: Option<AtlasOptions>) -> Self;
    fn generate(font_data: &[u8]) -> Self {
        Self::generate_with_options(font_data, None)
    }
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
