use std::collections::HashMap;

pub use xs_font::Font;

use xs_core::{AtlasInfo, AtlasOptions, TextProperties, TextVertices};

use crate::{Atlas, glyph::make_vertices_for_string};

pub struct TextSystem<A: Atlas> {
    fonts: HashMap<(String, u16), (Font, A)>,
}

impl<A: Atlas> Default for TextSystem<A> {
    fn default() -> Self {
        Self {
            fonts: Default::default(),
        }
    }
}

impl<A: Atlas> TextSystem<A> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<A: Atlas> xs_core::TextSystem for TextSystem<A> {
    fn load_font_with_options(
        &mut self,
        font_family: String,
        font_weight: u16,
        binary_font: Vec<u8>,
        atlas_options: Option<AtlasOptions>,
    ) -> &AtlasInfo {
        // The font keeps the bytes, so this no longer has to leak a copy of them
        // for the lifetime of the process.
        let font = Font::parse(binary_font.clone()).expect("could not read the font");
        let font_atlas = A::generate_with_options(&binary_font, atlas_options);
        let atlas_key = (font_family, font_weight);
        self.fonts.insert(atlas_key.clone(), (font, font_atlas));
        self.fonts.get(&atlas_key).unwrap().1.info()
    }

    fn render_text(
        &self,
        text: &str,
        text_properties: &TextProperties,
        max_width: f32,
        max_height: f32,
    ) -> TextVertices {
        let TextProperties {
            font_family,
            font_weight,
            ..
        } = text_properties;
        let (font, font_atlas) = self
            .fonts
            .get(&(font_family.clone(), *font_weight))
            .unwrap();
        make_vertices_for_string(
            font,
            font_atlas,
            text_properties,
            text,
            max_width,
            max_height,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AtlasSdf;
    use xs_core::{AtlasOptions, TextProperties, TextSystem as _};

    static FONT_DATA: &[u8] = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");

    #[test]
    fn test_render_text_texture_coordinates() {
        let font = Font::parse(FONT_DATA.to_vec()).unwrap();
        let atlas = AtlasSdf::generate_with_options(
            FONT_DATA,
            Some(AtlasOptions {
                font_size: 32.0,
                padding: 4.0,
            }),
        );

        let char_to_render = 'X';
        let glyph_id = font.glyph_index(char_to_render).unwrap();
        let glyph_info = atlas
            .atlas
            .glyphs
            .get(&glyph_id)
            .expect("'X' should be in atlas");

        let atlas_width = atlas.atlas.image.width as f32;
        let atlas_height = atlas.atlas.image.height as f32;

        // The sampled rectangle is the glyph plus the padding ring around it,
        // which is what the distance field stores and what the shader reads.
        let padding = 4.0;
        let expected_u1 = (glyph_info.atlas_x - padding) / atlas_width;
        let expected_v1 = (glyph_info.atlas_y - padding) / atlas_height;
        let expected_u2 =
            (glyph_info.atlas_x - padding + glyph_info.bounds.width + padding * 2.0) / atlas_width;
        let expected_v2 = (glyph_info.atlas_y - padding + glyph_info.bounds.height + padding * 2.0)
            / atlas_height;

        let mut text_system: TextSystem<AtlasSdf> = TextSystem::new();
        text_system.load_font_with_options(
            "test".to_string(),
            400,
            FONT_DATA.to_vec(),
            Some(AtlasOptions {
                font_size: 32.0,
                padding: 4.0,
            }),
        );

        let text_props = TextProperties {
            font_family: "test".to_string(),
            font_size: 32,
            font_weight: 400,
            font_style: xs_core::FontStyle::Normal,
        };

        let vertices =
            text_system.render_text(&char_to_render.to_string(), &text_props, 200.0, 100.0);

        assert!(
            vertices.vertices.len() >= 24,
            "Should have at least 6 vertices (2 triangles)"
        );

        for i in 0..(vertices.vertices.len() / 4) {
            let idx = i * 4;
            let u = vertices.vertices[idx + 2];
            let v = vertices.vertices[idx + 3];

            assert!(
                u >= expected_u1 - 0.001 && u <= expected_u2 + 0.001,
                "V{i} u={u:.6} should be between {expected_u1:.6} and {expected_u2:.6}"
            );
            assert!(
                v >= expected_v1 - 0.001 && v <= expected_v2 + 0.001,
                "V{i} v={v:.6} should be between {expected_v1:.6} and {expected_v2:.6}"
            );
        }
    }
}
