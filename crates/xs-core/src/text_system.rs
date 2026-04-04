use xs_layout::{TextProperties, TextVertices};

pub trait TextSystem {
    fn load_font(
        &mut self,
        font_family: &'static str,
        font_weight: u16,
        binary_font: Vec<u8>,
    ) -> (Vec<u8>, u32, u32);
    fn render_text(
        &self,
        text: &str,
        text_properties: &TextProperties,
        constraints: &(f32, f32),
    ) -> TextVertices;
}

pub struct TextSystemDummy;
impl TextSystem for TextSystemDummy {
    fn load_font(
        &mut self,
        _font_family: &'static str,
        _font_weight: u16,
        _binary_font: Vec<u8>,
    ) -> (Vec<u8>, u32, u32) {
        (Vec::new(), 0, 0)
    }

    fn render_text(
        &self,
        _text: &str,
        _text_properties: &TextProperties,
        _constraints: &(f32, f32),
    ) -> TextVertices {
        Default::default()
    }
}
