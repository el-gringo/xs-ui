use ab_glyph::{FontRef, Glyph, PxScale, PxScaleFont, ScaleFont, point};
use xs_core::{AtlasInfo, GlyphInfo, TextProperties, TextVertices};

use crate::{Atlas, layout_paragraph::layout_paragraph};

fn draw_glyph(
    glyph: &Glyph,
    glyph_info: &GlyphInfo,
    font_scale: f32,
    atlas_info: &AtlasInfo,
    descent: f32,
) -> ([f32; 24], f32, f32, f32, f32) {
    let padding = atlas_info.padding;
    let atlas_width = atlas_info.image.width as f32;
    let atlas_height = atlas_info.image.height as f32;
    let glyph_width = glyph_info.bounds.width * font_scale;
    let glyph_height = glyph_info.bounds.height * font_scale;
    let x = glyph.position.x + glyph_info.bounds.x * font_scale;
    let y = glyph.position.y + (glyph_info.bounds.y + descent) * font_scale;

    let x2 = x + glyph_width;
    let y2 = y + glyph_height;

    let u1 = (glyph_info.atlas_x - padding) / atlas_width;
    let u2 = (glyph_info.atlas_x - padding + glyph_info.bounds.width + padding * 2.0) / atlas_width;
    let v1 = (glyph_info.atlas_y - padding) / atlas_height;
    let v2 =
        (glyph_info.atlas_y - padding + glyph_info.bounds.height + padding * 2.0) / atlas_height;

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
    (glyph_vertices, x, x2, y, y2)
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
    let distance_range = font_scale * font_atlas.padding();

    let _ascent = scaled_font.ascent();
    let descent = scaled_font.descent();

    let mut min_x = f32::INFINITY;
    let mut max_x_pos = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y_pos = f32::NEG_INFINITY;
    let atlas_info = font_atlas.info();

    let mut vertices = Vec::new();
    for glyph in &glyphs {
        if let Some(glyph_info) = font_atlas.glyph_info(&glyph.id.0) {
            let (glyph_vertices, x, x2, y, y2) =
                draw_glyph(glyph, glyph_info, font_scale, atlas_info, descent);
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
