use std::collections::HashMap;

use xs_core::{AtlasInfo, AtlasOptions, GlyphInfo, Image, ImageFormat, Rectangle};
use xs_font::Font;

use super::font_atlas::{Atlas, gather_glyphs, scaled, space_width};

pub struct AtlasMsdf {
    pub atlas: AtlasInfo,
}

impl Atlas for AtlasMsdf {
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
        let mut bitmap = vec![0u8; (atlas_width * atlas_height * 3.0) as usize];
        let mut glyph_map = HashMap::new();

        for row in 0..atlas_size as _ {
            for col in 0..atlas_size as _ {
                if let Some((id, glyph)) = glyphs.get(col + row * atlas_size as usize) {
                    let bounds = glyph.pixel_bounds();
                    let local_atlas_x = col as f32 * font_size;
                    let local_atlas_y = row as f32 * font_size;
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
                                    let other = is_covered[ty * width as usize + tx];
                                    if interior != other {
                                        let dist = ((px as f32 - tx as f32).powi(2)
                                            + (py as f32 - ty as f32).powi(2))
                                        .sqrt();
                                        min_dist = min_dist.min(dist);
                                    }
                                }
                            }
                            if min_dist == f32::INFINITY {
                                min_dist = padding;
                            }

                            let signed_dist = if interior { min_dist } else { -min_dist };
                            let normalized = (signed_dist / padding + 1.0) * 0.5 * 255.0;
                            let clamped = normalized.clamp(0.0, 255.0) as u8;

                            let atlas_px = (local_atlas_x + px as f32) as usize;
                            let atlas_py = (local_atlas_y + py as f32) as usize;
                            let idx = (atlas_py * atlas_width as usize + atlas_px) * 3;
                            bitmap[idx] = clamped;
                            bitmap[idx + 1] = clamped;
                            bitmap[idx + 2] = clamped;
                        }
                    }

                    glyph_map.insert(
                        *id,
                        GlyphInfo {
                            atlas_x: local_atlas_x + padding,
                            atlas_y: local_atlas_y + padding,
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

        AtlasMsdf {
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
