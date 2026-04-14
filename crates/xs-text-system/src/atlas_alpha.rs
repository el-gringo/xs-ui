use std::collections::HashMap;

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use xs_core::{ImageFormat, Rectangle};

use super::font_atlas::{Atlas, AtlasData, GlyphInfo, gather_glyphs};

pub struct AtlasAlpha {
    pub atlas: AtlasData,
}

impl Atlas for AtlasAlpha {
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
                        glyph.glyph().id,
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

        (
            AtlasAlpha {
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
            ImageFormat::Alpha,
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
    fn test_alpha_atlas_generation() {
        let font = get_test_font();
        let (atlas, _format, bitmap) = AtlasAlpha::generate(&font, 32.0, 0.0);

        assert!(atlas.atlas.width > 0.0);
        assert!(atlas.atlas.height > 0.0);
        assert!(!atlas.atlas.glyphs.is_empty());

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
    fn test_alpha_distribution() {
        let font = get_test_font();
        let (atlas, _format, bitmap) = AtlasAlpha::generate(&font, 32.0, 0.0);

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
