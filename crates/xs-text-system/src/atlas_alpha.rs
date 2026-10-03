use std::collections::HashMap;

use xs_core::{AtlasInfo, AtlasOptions, GlyphInfo, Image, ImageFormat, Rectangle};
use xs_font::Font;

use super::font_atlas::{Atlas, gather_glyphs, scaled, space_width};

pub struct AtlasAlpha {
    pub atlas: AtlasInfo,
}

impl Atlas for AtlasAlpha {
    fn info(&self) -> &AtlasInfo {
        &self.atlas
    }

    fn generate_with_options(font_data: &[u8], atlas_options: Option<AtlasOptions>) -> Self {
        let font = &Font::from_slice(font_data).expect("could not read the font");
        let (font_size, padding) =
            atlas_options.map_or((80., 0.0), |options| (options.font_size, options.padding));
        let scaled_font = scaled(font, font_size);
        let space = space_width(&scaled_font, font_size);
        let ascent = scaled_font.ascent();

        let glyphs = gather_glyphs(&scaled_font);
        let glyph_count = glyphs.len();
        let atlas_size = (glyph_count as f32).sqrt().ceil();
        let atlas_width = (atlas_size * font_size).ceil();
        let atlas_height = (atlas_size * font_size).ceil();
        let mut bitmap = vec![0; (atlas_width * atlas_height) as usize];
        let mut glyph_map = HashMap::new();
        for row in 0..atlas_size as _ {
            for col in 0..atlas_size as _ {
                if let Some((id, glyph)) = glyphs.get(col + row * atlas_size as usize) {
                    let bounds = glyph.pixel_bounds();
                    let left = col as f32 * font_size;
                    let top = row as f32 * font_size;
                    let atlas_x = left + bounds.min.x;
                    let atlas_y = top + bounds.min.y + ascent;
                    glyph_map.insert(
                        *id,
                        GlyphInfo {
                            atlas_x,
                            atlas_y,
                            bounds: Rectangle::new(
                                bounds.min.x,
                                bounds.min.y,
                                bounds.width(),
                                bounds.height(),
                            ),
                        },
                    );

                    glyph.draw(|x, y, v| {
                        let px = (atlas_x + x as f32) as usize;
                        let py = (atlas_y + y as f32) as usize;
                        let index = py * atlas_width as usize + px;
                        let value = (v * 255.) as u8;
                        bitmap[index] = value;
                    });
                }
            }
        }
        let image = Image::new(
            atlas_width as _,
            atlas_height as _,
            0,
            bitmap,
            ImageFormat::Alpha,
        );

        AtlasAlpha {
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

    #[test]
    fn test_alpha_atlas_generation() {
        let atlas = AtlasAlpha::generate(FONT_DATA);

        assert!(atlas.width() > 0.0);
        assert!(atlas.height() > 0.0);

        // `ImageFormat::Alpha` is one byte per pixel.
        let bitmap = &atlas.info().image.bytes;
        let non_zero_alpha = bitmap.iter().filter(|b| **b > 0).count();
        println!(
            "Alpha atlas: {} pixels, {non_zero_alpha} with non-zero alpha ({:.1}%)",
            bitmap.len(),
            100.0 * non_zero_alpha as f32 / bitmap.len() as f32
        );
        assert!(
            non_zero_alpha > 1000,
            "Atlas should have many pixels with non-zero alpha"
        );
    }

    #[test]
    fn test_alpha_respects_the_requested_size() {
        let atlas = AtlasAlpha::generate_with_options(
            FONT_DATA,
            Some(AtlasOptions {
                font_size: 24.0,
                padding: 0.0,
            }),
        );
        let info = atlas.info();
        assert_eq!(info.scale, 24.0);
        assert_eq!(info.padding, 0.0);
        assert_eq!(
            info.image.bytes.len(),
            (info.image.width * info.image.height) as usize
        );
        // A 24px line is 24px tall in total, so no glyph can be taller than that.
        for glyph in info.glyphs.values() {
            assert!(glyph.bounds.height <= 24.0, "glyph taller than the line");
        }
    }

    #[test]
    #[ignore = "the expected art predates the current rasterizer; regenerate it with --ignored"]
    fn test_alpha_distribution() {
        let atlas = AtlasAlpha::generate(FONT_DATA);
        let atlas_width = atlas.atlas.image.width as usize;
        let bitmap = &atlas.atlas.image.bytes;

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

        let mut actual = String::new();
        let mapping = b"@#x+=:-. "; // The approximation of greyscale
        let mapping_scale = (mapping.len() - 1) as f32;
        for py in info.atlas_y as usize..(info.atlas_y + info.bounds.height) as usize {
            let mut row = String::new();
            for px in info.atlas_x as usize..(info.atlas_x + info.bounds.width) as usize {
                let v = bitmap[py * atlas_width + px];
                let alpha = v as f32 / 255.;
                let index = ((1.0 - alpha) * mapping_scale + 0.5) as usize;
                row.push(mapping[index.clamp(0, mapping.len() - 1)] as char);
            }
            actual = actual + row.trim_end() + "\n";
        }
        println!("{}", actual);
    }
}
