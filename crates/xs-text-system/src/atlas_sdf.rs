use std::collections::HashMap;

use xs_core::{ImageFormat, Rectangle};
use xs_text_system_glyph::{Font, FontRef, PxScale, ScaleFont};

use super::font_atlas::{gather_glyphs, Atlas, AtlasData, GlyphInfo};

pub struct AtlasSdf {
    pub atlas: AtlasData,
}

impl Atlas for AtlasSdf {
    fn data(&self) -> &AtlasData {
        &self.atlas
    }

    fn generate(font: &FontRef<'_>, font_size: f32, padding: f32) -> (Self, ImageFormat, Vec<u8>) {
        let scale = PxScale::from(font_size);
        let font = font.into_scaled(scale);
        let space_width = if let Some(space_glyph) = font.outline_glyph(font.scaled_glyph(' ')) {
            let bounds = space_glyph.px_bounds();
            bounds.max.x - bounds.min.x
        } else {
            font_size
        };
        let ascent = font.ascent();

        let glyphs = gather_glyphs(&font);
        let glyph_count = glyphs.len();
        let atlas_size = (glyph_count as f32).sqrt().ceil();
        let atlas_width = (atlas_size + padding) * font_size;
        let atlas_height = (atlas_size + padding) * font_size;
        let mut bitmap = vec![0; (atlas_width * atlas_height * 3.0) as usize];
        let mut glyph_map = HashMap::new();
        for row in 0..atlas_size as _ {
            for col in 0..atlas_size as _ {
                if let Some((_, glyph)) = glyphs.get(col + row * atlas_size as usize) {
                    let bounds = glyph.px_bounds();
                    let atlas_x = col as f32 * font_size;
                    let atlas_y = row as f32 * font_size;
                    let width = bounds.width() + padding * 2.0;
                    let height = bounds.height() + padding * 2.0;
                    let mut is_covered = vec![false; width as usize * height as usize];
                    glyph.draw(|x, y, v| {
                        let px = (x as f32 + padding) as usize;
                        let py = (y as f32 + padding) as usize;
                        if px < width as usize && py < height as usize {
                            is_covered[py * width as usize + px] = v >= 0.5;
                        }
                    });
                    for py in 0..height as usize {
                        for px in 0..width as usize {
                            let interior = is_covered[py * width as usize + px];
                            let mut min_dist = f32::INFINITY;
                            for ty in 0..height as usize {
                                for tx in 0..width as usize {
                                    let other_interior = is_covered[ty * width as usize + tx];
                                    if interior != other_interior {
                                        let dist = ((px as f32 - tx as f32).powi(2)
                                            + (py as f32 - ty as f32).powi(2))
                                        .sqrt();
                                        min_dist = min_dist.min(dist);
                                    }
                                }
                            }
                            if min_dist == f32::INFINITY {
                                min_dist = 0.0;
                            }
                            let signed_dist = if interior { min_dist } else { -min_dist };
                            let normalized = (signed_dist / padding + 1.0) * 0.5 * 255.0;
                            let clamped = normalized.clamp(0.0, 255.0) as u8;

                            let atlas_px = (atlas_x + px as f32) as usize;
                            let atlas_py = (atlas_y + py as f32) as usize;
                            let idx = (atlas_py * atlas_width as usize + atlas_px) * 3;
                            bitmap[idx] = clamped;
                            bitmap[idx + 1] = clamped;
                            bitmap[idx + 2] = clamped;
                        }
                    }
                    glyph_map.insert(
                        glyph.glyph().id,
                        GlyphInfo {
                            atlas_x: atlas_x + padding,
                            atlas_y: atlas_y + padding,
                            bounds: Rectangle::new(
                                bounds.min.x,
                                bounds.min.y,
                                bounds.width(),
                                bounds.height(),
                            ),
                        },
                    );
                }
            }
        }

        (
            AtlasSdf {
                atlas: AtlasData {
                    width: atlas_width,
                    height: atlas_height,
                    size: atlas_size as _,
                    scale: font_size,
                    spacing: space_width,
                    ascent,
                    v_advance: font.height() + font.line_gap(),
                    padding,
                    glyphs: glyph_map,
                },
            },
            ImageFormat::Rgb,
            bitmap,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xs_text_system_glyph::FontRef;

    fn get_test_font() -> FontRef<'static> {
        let font_data = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");
        FontRef::try_from_slice(Box::leak(font_data.to_vec().into_boxed_slice())).unwrap()
    }

    #[test]
    fn test_sdf_atlas_has_content() {
        let font = get_test_font();
        let (atlas, bitmap) = AtlasSdf::generate(&font, 32.0, 4.0);

        let mut non_zero = 0;
        for &byte in &bitmap {
            if byte > 0 {
                non_zero += 1;
            }
        }

        println!(
            "Atlas {}x{}, non-zero bytes: {}/{}",
            atlas.atlas.width,
            atlas.atlas.height,
            non_zero,
            bitmap.len()
        );

        assert!(non_zero > 1000, "Atlas should have significant content");
    }

    #[test]
    fn test_sdf_rendering_glyph() {
        let padding = 4.0;
        let font_size = 32.0;
        let font = get_test_font();
        let (atlas, bitmap) = AtlasSdf::generate(&font, font_size, padding);

        let mut actual = String::from("");
        let char_to_render = 'G';
        let glyph_id = font.glyph_id(char_to_render);
        if let Some(info) = atlas.atlas.glyphs.get(&glyph_id) {
            let x = info.atlas_x as usize;
            let y = info.atlas_y as usize;
            let w = info.bounds.width as usize;
            let h = info.bounds.height as usize;
            let mapping = b"@#x+=:-. "; // The approximation of greyscale
            let mapping_scale = (mapping.len() - 1) as f32;

            for py in y..y + h {
                let mut row = String::new();
                for px in x..x + w {
                    let idx = (py * atlas.atlas.width as usize + px) * 3;
                    let v = bitmap[idx];
                    let alpha = v as f32 / 255.;
                    let index = ((1.0 - alpha) * mapping_scale + 0.5) as usize;
                    let c = mapping[index.clamp(0, mapping.len() - 1)];
                    row.push(c as char);
                }
                if !row.is_empty() {
                    actual = actual + row.trim_end() + "\n";
                }
            }
        }
        let expected = "

        .........
       .---------..
      .-:::::::::--.
     .-::+++++++:::-.
    .-::++++::++++:-.
   .-::+++::::::++::-.
   .-:+++::----::++:-.
   .-:++::-....-::::-.
   .-:++:-.  ...----.
   .-:++:-. .-------.
  .-::++:-..-:::::::-.
  .-:+++:-..-:+++++:-.
  .-::++:-..-:+++++:-.
   .-:++:-..-::::++:-.
   .-:++:-. .---:++:-.
   .-:++::-....-:++:-.
   .-:+++::----::++:-.
   .-::+++::::::+++:-.
    .-::+++++++++++:-.
     .-::+++++++:++:-.
      .-::::::::::::-.
       .------------.
        ............
          ";
        assert_eq!(actual.trim_end(), expected.trim_end())
    }
}
