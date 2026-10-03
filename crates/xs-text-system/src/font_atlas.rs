use xs_core::{AtlasInfo, AtlasOptions, GlyphInfo};
use xs_font::{Font, RasterizedGlyph, ScaledFont};

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

/// Scales a font for a `font_size`.
///
/// `font_size` is a line height in pixels here, not an em, and that is not an
/// accident to be tidied up: `ab_glyph`, which this crate used to go through,
/// divided its `PxScale` by `ascent - descent` rather than by `units_per_em`, so
/// a nominal 32px was a 32px line whose cap height was 698/1300 * 32 = 17.2px.
/// Scaling by the em instead would grow every glyph in the app by 30%, so the
/// conversion lives here, once, rather than in each atlas.
#[must_use]
pub fn scaled<'a>(font: &'a Font, font_size: f32) -> ScaledFont<'a> {
    font.scaled(font.px_per_em_for_line_height(font_size))
}

/// Rasterizes the glyphs the atlases pack: printable ASCII and Latin-1.
///
/// Glyphs with nothing to draw, spaces among them, are left out. Every atlas is
/// a square grid over whatever this returns, so adding one changes the atlas
/// size and the texture coordinates of everything after it.
pub fn gather_glyphs(font: &ScaledFont<'_>) -> Vec<(u16, RasterizedGlyph)> {
    let mut glyphs = Vec::new();
    for range in [0x020..0x07e, 0x0a0..0x17f] {
        for code in range {
            let Some(c) = char::from_u32(code) else {
                continue;
            };
            let Some(id) = font.glyph_index(c) else {
                continue;
            };
            if let Some(raster) = font.outline(id) {
                glyphs.push((id, raster));
            }
        }
    }
    glyphs
}

/// The horizontal space a space character takes, falling back to `font_size`.
///
/// The atlas stores this as its spacing and nothing else reads it back, so the
/// fallback only matters for a font whose space has no outline at all.
#[must_use]
pub fn space_width(font: &ScaledFont<'_>, font_size: f32) -> f32 {
    font.glyph_index(' ')
        .and_then(|id| font.outline(id))
        .map_or(font_size, |raster| {
            let bounds = raster.pixel_bounds();
            bounds.max.x - bounds.min.x
        })
}
