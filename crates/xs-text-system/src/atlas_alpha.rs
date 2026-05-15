use std::collections::HashMap;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use xs_core::{AtlasInfo, AtlasOptions, GlyphInfo, Image, ImageFormat, Rectangle};

use super::font_atlas::{Atlas, gather_glyphs};

pub struct AtlasAlpha {
    pub atlas: AtlasInfo,
}

impl Atlas for AtlasAlpha {
    fn info(&self) -> &AtlasInfo {
        &self.atlas
    }

    fn generate_with_options(font_data: &[u8], atlas_options: Option<AtlasOptions>) -> Self {
        let font = &FontRef::try_from_slice(font_data).unwrap();
        let (font_size, padding) =
            atlas_options.map_or((80., 0.0), |options| (options.font_size, options.padding));
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
        let atlas_width = (atlas_size * font_size).ceil();
        let atlas_height = (atlas_size * font_size).ceil();
        let mut bitmap = vec![0; (atlas_width * atlas_height) as usize];
        let mut glyph_map = HashMap::new();
        for row in 0..atlas_size as _ {
            for col in 0..atlas_size as _ {
                if let Some((_c, glyph)) = glyphs.get(col + row * atlas_size as usize) {
                    let bounds = glyph.px_bounds();
                    let left = col as f32 * font_size;
                    let top = row as f32 * font_size;
                    let atlas_x = left + bounds.min.x;
                    let atlas_y = top + bounds.min.y + ascent;
                    glyph_map.insert(
                        glyph.glyph().id.0,
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
                spacing: space_width,
                ascent,
                v_advance: font.height() + font.line_gap(),
                padding,
                glyphs: glyph_map,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static FONT: &[u8] = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");

    #[test]
    fn test_alpha_atlas_generation() {
        let atlas = AtlasAlpha::generate(FONT);

        assert!(atlas.width() > 0.0);
        assert!(atlas.height() > 0.0);

        let atlas_info = atlas.info();
        let bitmap = &atlas_info.image.bytes;

        let mut non_zero_alpha = 0;
        let total_pixels = bitmap.len() / 3;
        for i in (0..bitmap.len()).step_by(3) {
            if bitmap[i] > 0 {
                non_zero_alpha += 1;
            }
        }
        println!(
            "Alpha atlas: {} pixels, {} with non-zero alpha ({:.1}%)",
            total_pixels,
            non_zero_alpha,
            100.0 * non_zero_alpha as f32 / total_pixels as f32
        );
        assert!(
            non_zero_alpha > 1000,
            "Atlas should have many pixels with non-zero alpha"
        );
    }

    #[test]
    #[ignore = "this require to update this test"]
    fn test_alpha_distribution() {
        let atlas = AtlasAlpha::generate(FONT);

        let atlas_info = atlas.info();
        let bitmap = &atlas_info.image.bytes;

        let mut actual = String::from("");
        let char_to_render = 'G';
        let glyph_id = font.glyph_id(char_to_render);
        if let Some(info) = atlas.atlas.glyphs.get(&glyph_id) {
            let x = info.x as usize;
            let y = info.y as usize;
            let w = info.width as usize;
            let h = info.height as usize;
            let mapping = b"@#x+=:-. "; // The approximation of greyscale
            let mapping_scale = (mapping.len() - 1) as f32;
            for py in y..y + h {
                let mut row = String::new();
                for px in x..x + w {
                    let idx = (py * atlas.atlas.width as usize + px);
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
        println!("{}", actual);
        let expected = "     .:::-
   .x@@@@@#=
  -#@#=:=x@@+
 .#@=     -#@=
 +@+       -@=
 #@.        .
-@#
:@x
=@+
=@+     :#####-
=@+     =@@@@@:
:@x         +@:
-@#         +@:
 #@-        x@:
 =@x       .@@:
  #@x.    -##@:
  .#@@x++#@x+@:
   .=@@@@@+ +@:
      .--
        ";
        assert_eq!(actual.trim_end(), expected.trim_end())
    }
}
