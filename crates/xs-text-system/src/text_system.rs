use std::collections::HashMap;

use xs_core::{ImageFormat, TextProperties, TextVertices};
pub use xs_text_system_glyph::{point, FontRef, PxScale, PxScaleFont, ScaleFont};

use crate::{layout_paragraph::layout_paragraph, Atlas};

pub struct TextSystem<A: Atlas> {
    fonts: HashMap<(&'static str, u16), (FontRef<'static>, A)>,
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

    pub fn load_font(
        &mut self,
        font_family: &'static str,
        font_weight: u16,
        font_size: f32,
        padding: f32,
        binary_font: Vec<u8>,
    ) -> (Vec<u8>, ImageFormat, u32, u32) {
        let font = FontRef::try_from_slice(Box::leak(binary_font.into_boxed_slice())).unwrap();
        let (font_atlas, format, atlas_data) = A::generate(&font, font_size, padding);
        let width = font_atlas.width();
        let height = font_atlas.height();
        self.fonts
            .insert((font_family, font_weight), (font, font_atlas));
        (atlas_data, format, width as _, height as _)
    }

    pub fn render_text(
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
            .get(&(font_family.as_str(), *font_weight))
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

pub fn make_vertices_for_string<A: Atlas>(
    font: &FontRef<'static>,
    font_atlas: &A,
    text_properties: &TextProperties,
    text: &str,
    max_width: f32,
    _max_height: f32,
) -> TextVertices {
    let scale = PxScale::from(text_properties.font_size as f32);
    let scaled_font = PxScaleFont {
        font: font.clone(),
        scale,
    };

    let mut glyphs = Vec::new();
    layout_paragraph(&scaled_font, point(0.0, 0.0), max_width, text, &mut glyphs);

    let font_scale = text_properties.font_size as f32 / font_atlas.scale();
    let distance_range = font_scale * font_atlas.padding() as f32;

    let ascent = scaled_font.ascent();
    let _descent = scaled_font.descent();

    let mut min_x = f32::INFINITY;
    let mut max_x_pos = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y_pos = f32::NEG_INFINITY;
    let atlas_width = font_atlas.width();
    let atlas_height = font_atlas.height();
    let padding = font_atlas.padding() as f32;

    let mut vertices = Vec::new();
    for glyph in &glyphs {
        if let Some(glyph_info) = font_atlas.glyph_info(&glyph.id) {
            let glyph_width = glyph_info.bounds.width * font_scale;
            let glyph_height = glyph_info.bounds.height * font_scale;
            let x = glyph.position.x;
            let y = glyph.position.y - ascent + glyph_info.bounds.y * font_scale;

            let x2 = x + glyph_width;
            let y2 = y + glyph_height;

            let u1 = (glyph_info.atlas_x - padding) / atlas_width;
            let u2 = (glyph_info.atlas_x - padding + glyph_info.bounds.width + padding * 2.0)
                / atlas_width;
            let v1 = (glyph_info.atlas_y - padding) / atlas_height;
            let v2 = (glyph_info.atlas_y - padding + glyph_info.bounds.height + padding * 2.0)
                / atlas_height;

            let mut glyph_vertices = [0.; 24];
            glyph_vertices[0] = x;
            glyph_vertices[1] = y;
            glyph_vertices[2] = u1;
            glyph_vertices[3] = v1;

            glyph_vertices[4] = x2;
            glyph_vertices[5] = y;
            glyph_vertices[6] = u2;
            glyph_vertices[7] = v1;

            glyph_vertices[8] = x;
            glyph_vertices[9] = y2;
            glyph_vertices[10] = u1;
            glyph_vertices[11] = v2;

            glyph_vertices[12] = x2;
            glyph_vertices[13] = y;
            glyph_vertices[14] = u2;
            glyph_vertices[15] = v1;

            glyph_vertices[16] = x2;
            glyph_vertices[17] = y2;
            glyph_vertices[18] = u2;
            glyph_vertices[19] = v2;

            glyph_vertices[20] = x;
            glyph_vertices[21] = y2;
            glyph_vertices[22] = u1;
            glyph_vertices[23] = v2;
            vertices.extend(glyph_vertices);

            min_x = min_x.min(x);
            max_x_pos = max_x_pos.max(x2);
            min_y = min_y.min(y);
            max_y_pos = max_y_pos.max(y2);
        }
    }

    let width = if max_x_pos.is_finite() {
        max_x_pos - min_x
    } else {
        0.0
    };
    let height = if max_y_pos.is_finite() {
        max_y_pos - min_y
    } else {
        0.0
    };

    TextVertices {
        vertices,
        width,
        height,
        font_scale,
        distance_range,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AtlasSdf;
    use xs_core::TextProperties;

    #[test]
    fn test_render_text_texture_coordinates() {
        let font_data = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");
        let font =
            FontRef::try_from_slice(Box::leak(font_data.to_vec().into_boxed_slice())).unwrap();
        let (atlas, _format, _) = AtlasSdf::generate(&font, 32.0, 4.0);

        let char_to_render = 'X';
        let glyph_id = font.glyph_id(char_to_render);

        let glyph_info = atlas
            .atlas
            .glyphs
            .get(&glyph_id)
            .expect("'X' should be in atlas");

        let atlas_width = atlas.atlas.width as f32;
        let atlas_height = atlas.atlas.height as f32;

        let expected_u1 = glyph_info.x / atlas_width;
        let expected_v1 = glyph_info.y / atlas_height;
        let expected_u2 = (glyph_info.x + glyph_info.width) / atlas_width;
        let expected_v2 = (glyph_info.y + glyph_info.height) / atlas_height;

        let mut text_system: TextSystem<AtlasSdf> = TextSystem::new();
        text_system.load_font("test", 400, 32.0, 4.0, font_data.to_vec());

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
                "V{} u={:.6} should be between {:.6} and {:.6}",
                i,
                u,
                expected_u1,
                expected_u2
            );
            assert!(
                v >= expected_v1 - 0.001 && v <= expected_v2 + 0.001,
                "V{} v={:.6} should be between {:.6} and {:.6}",
                i,
                v,
                expected_v1,
                expected_v2
            );
        }
    }
}
