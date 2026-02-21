use std::{collections::HashMap, usize};

pub use xs_text_system_glyph::{Font, FontRef, PxScale, ScaleFont, point};

use crate::layout_paragraph::layout_paragraph;

pub struct TextSystem {
    fonts: HashMap<&'static str, FontRef<'static>>,
}

impl TextSystem {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }
}

impl xs_core::TextSystem for TextSystem {
    fn load_font(&mut self, font_family: &'static str, binary_font: &'static [u8]) {
        let font = FontRef::try_from_slice(&binary_font).unwrap();
        self.fonts.insert(font_family, font);
    }

    fn render_text(
        &self,
        text: &str,
        font_family: &str,
        font_size: f32,
    ) -> Result<(Vec<u8>, u32, u32), &'static str> {
        let font = self.fonts.get(font_family).unwrap();

        // The font size to use
        let scale = PxScale::from(font_size);
        let scaled_font = font.into_scaled(scale);

        let mut glyphs = Vec::new();
        layout_paragraph(&scaled_font, point(20.0, 20.0), 9999.0, text, &mut glyphs);

        // to work out the exact size needed for the drawn glyphs we need to outline
        // them and use their `px_bounds` which hold the coords of their render bounds.
        let outlined: Vec<_> = glyphs
            .into_iter()
            // Note: not all layout glyphs have outlines (e.g. " ")
            .filter_map(|g| font.outline_glyph(g))
            .collect();

        // combine px_bounds to get min bounding coords for the entire layout
        if let Some(all_px_bounds) = outlined
            .iter()
            .map(|g| g.px_bounds())
            .reduce(|mut b, next| {
                b.min.x = b.min.x.min(next.min.x);
                b.max.x = b.max.x.max(next.max.x);
                b.min.y = b.min.y.min(next.min.y);
                b.max.y = b.max.y.max(next.max.y);
                b
            })
        {
            let width = all_px_bounds.width() as u32;
            let height = all_px_bounds.height() as u32;

            // create a new rgba image using the combined px bound width and height
            let mut pixels = vec![0; (width * height * 4) as usize];

            // Loop through the glyphs in the text, positing each one on a line
            for glyph in outlined {
                let bounds = glyph.px_bounds();
                // calc top/left ords in "image space"
                // image-x=0 means the *left most pixel*, equivalent to
                // px_bounds.min.x which *may be non-zero* (and similarly with y)
                // so `- px_bounds.min` converts the left-most/top-most to 0
                let img_left = (bounds.min.x - all_px_bounds.min.x) as u32;
                let img_top = (bounds.min.y - all_px_bounds.min.y) as u32;
                // Draw the glyph into the image per-pixel by using the draw closure
                glyph.draw(|x, y, v| {
                    // save the coverage alpha
                    let pos = (img_left + x + (img_top + y) * width) as usize;
                    pixels[pos] = 255;
                    pixels[pos + 1] = 255;
                    pixels[pos + 2] = 255;
                    pixels[pos + 3] = (v * 255.) as u8;
                });
            }
            return Ok((pixels, width, height));
        } else {
            Err("No outlined glyphs?")
        }
    }
}
