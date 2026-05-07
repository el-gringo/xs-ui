use std::collections::HashMap;

use xs_layout::{Image, TextProperties, TextVertices};
use xs_math::Rectangle;

type GlyphId = u16;

#[derive(Clone, Copy)]
pub struct GlyphInfo {
    pub atlas_x: f32,
    pub atlas_y: f32,
    pub bounds: Rectangle,
}

pub struct AtlasOptions {
    pub font_size: f32,
    pub padding: f32,
}

#[derive(Default)]
pub struct AtlasInfo {
    pub image: Image,
    pub size: u8,
    pub scale: f32,
    pub ascent: f32,
    pub v_advance: f32,
    pub spacing: f32,
    pub padding: f32,
    pub glyphs: HashMap<GlyphId, GlyphInfo>,
}

pub trait TextSystem {
    fn load_font(
        &mut self,
        font_family: String,
        font_weight: u16,
        binary_font: Vec<u8>,
    ) -> &AtlasInfo {
        self.load_font_with_options(font_family, font_weight, binary_font, None)
    }

    fn load_font_with_options(
        &mut self,
        font_family: String,
        font_weight: u16,
        binary_font: Vec<u8>,
        atlas_options: Option<AtlasOptions>,
    ) -> &AtlasInfo;

    fn render_text(
        &self,
        text: &str,
        text_properties: &TextProperties,
        max_width: f32,
        max_height: f32,
    ) -> TextVertices;
}

#[derive(Default)]
pub struct TextSystemDummy(AtlasInfo);
impl TextSystem for TextSystemDummy {
    fn load_font_with_options(
        &mut self,
        _font_family: String,
        _font_weight: u16,
        _binary_font: Vec<u8>,
        _atlas_options: Option<AtlasOptions>,
    ) -> &AtlasInfo {
        self.0 = AtlasInfo::default();
        &self.0
    }

    fn render_text(
        &self,
        _text: &str,
        _text_properties: &TextProperties,
        _max_width: f32,
        _max_height: f32,
    ) -> TextVertices {
        Default::default()
    }
}
