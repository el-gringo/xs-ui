use xs_layout::{TextProperties, TextVertices};

use crate::ImageFormat;

pub trait TextSystem {
    fn load_font(
        &mut self,
        font_family: &'static str,
        font_weight: u16,
        font_size: f32,
        padding: f32,
        binary_font: Vec<u8>,
    ) -> (Vec<u8>, ImageFormat, u32, u32);
    fn render_text(
        &self,
        text: &str,
        text_properties: &TextProperties,
        max_width: f32,
        max_height: f32,
    ) -> TextVertices;
}

pub struct TextSystemDummy;
impl TextSystem for TextSystemDummy {
    fn load_font(
        &mut self,
        _font_family: &'static str,
        _font_weight: u16,
        _font_size: f32,
        _padding: f32,
        _binary_font: Vec<u8>,
    ) -> (Vec<u8>, ImageFormat, u32, u32) {
        (Vec::new(), ImageFormat::Alpha, 0, 0)
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
