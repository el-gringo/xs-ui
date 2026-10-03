use std::collections::HashMap;

use xs_core::{AtlasInfo, AtlasOptions, GlyphInfo, Image, ImageFormat, Rectangle};
use xs_font::Font;

use super::font_atlas::{Atlas, gather_glyphs, scaled, space_width};

pub struct AtlasSdf {
    pub atlas: AtlasInfo,
}

impl Atlas for AtlasSdf {
    fn info(&self) -> &AtlasInfo {
        &self.atlas
    }

    fn generate_with_options(font_data: &[u8], atlas_options: Option<AtlasOptions>) -> Self {
        let font = &Font::from_slice(font_data).expect("could not read the font");
        let (font_size, padding) =
            atlas_options.map_or((42., 4.0), |options| (options.font_size, options.padding));
        let scaled_font = scaled(font, font_size);
        let space = space_width(&scaled_font, font_size);
        let ascent = scaled_font.ascent();

        let glyphs = gather_glyphs(&scaled_font);
        let glyph_count = glyphs.len();
        let atlas_size = (glyph_count as f32).sqrt().ceil();
        let atlas_width = (atlas_size + padding) * font_size;
        let atlas_height = (atlas_size + padding) * font_size;
        let mut bitmap = vec![0; (atlas_width * atlas_height * 3.0) as usize];
        let mut glyph_map = HashMap::new();
        for row in 0..atlas_size as _ {
            for col in 0..atlas_size as _ {
                if let Some((id, glyph)) = glyphs.get(col + row * atlas_size as usize) {
                    let bounds = glyph.pixel_bounds();
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
                        *id,
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

        let image = Image::new(
            atlas_width as _,
            atlas_height as _,
            0,
            bitmap,
            ImageFormat::Rgb,
        );

        AtlasSdf {
            atlas: AtlasInfo {
                image,
                size: atlas_size as _,
                scale: font_size,
                spacing: space,
                ascent,
                v_advance: scaled_font.height() + scaled_font.line_gap(),
                padding,
                glyphs: glyph_map,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xs_core::AtlasOptions;

    static FONT_DATA: &[u8] = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");

    fn generate(font_size: f32, padding: f32) -> AtlasSdf {
        AtlasSdf::generate_with_options(FONT_DATA, Some(AtlasOptions { font_size, padding }))
    }

    #[test]
    fn test_sdf_atlas_has_content() {
        let atlas = generate(32.0, 4.0);
        let bitmap = &atlas.atlas.image.bytes;

        let non_zero = bitmap.iter().filter(|b| **b > 0).count();
        println!(
            "Atlas {}x{}, non-zero bytes: {non_zero}/{}",
            atlas.atlas.image.width,
            atlas.atlas.image.height,
            bitmap.len()
        );

        assert!(non_zero > 1000, "Atlas should have significant content");
    }

    #[test]
    fn test_sdf_rendering_glyph() {
        let padding = 4.0;
        let font_size = 32.0;
        let atlas = generate(font_size, padding);
        let bitmap = &atlas.atlas.image.bytes;
        let atlas_width = atlas.atlas.image.width as usize;

        let char_to_render = 'G';
        let glyph_id = Font::from_slice(FONT_DATA)
            .unwrap()
            .glyph_index(char_to_render)
            .unwrap();
        let info = atlas
            .atlas
            .glyphs
            .get(&glyph_id)
            .expect("'G' should be in atlas");

        // The padding ring is part of the distance field, and it is what the
        // shader samples, so read the whole padded block rather than starting at
        // the glyph origin.
        let x = (info.atlas_x - padding) as usize;
        let y = (info.atlas_y - padding) as usize;
        let w = info.bounds.width as usize + padding as usize * 2;
        let h = info.bounds.height as usize + padding as usize * 2;
        let mapping = b"@#x+=:-. "; // The approximation of greyscale
        let mapping_scale = (mapping.len() - 1) as f32;

        let mut actual = String::new();
        for py in y..y + h {
            let mut row = String::new();
            for px in x..x + w {
                let idx = (py * atlas_width + px) * 3;
                let alpha = bitmap[idx] as f32 / 255.;
                let index = ((1.0 - alpha) * mapping_scale + 0.5) as usize;
                let c = mapping[index.clamp(0, mapping.len() - 1)];
                row.push(c as char);
            }
            if !row.trim_end().is_empty() {
                actual = actual + row.trim_end() + "\n";
            }
        }
        // A regression guard on the distance field rather than a picture of the
        // letter. Every row of it was checked against what this crate produced
        // before `ab_glyph` was replaced, and matched. The art is flush left on
        // purpose: it is compared with `trim`, so the indentation of this
        // literal does not become part of the comparison.
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
        assert_eq!(actual.trim(), expected.trim())
    }
}
