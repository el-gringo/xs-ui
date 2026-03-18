use xs_layout::{TextProperties, TextVertices};

use crate::Renderer;

pub trait TextSystem {
    fn load_font(
        &mut self,
        font_family: &'static str,
        font_weight: u16,
        binary_font: &'static [u8],
        renderer: &mut Box<dyn Renderer>,
    );
    fn render_text(&self, text: &str, text_properties: &TextProperties) -> (TextVertices, u32, u32, u32);
}
