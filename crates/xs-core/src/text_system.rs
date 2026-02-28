pub trait TextSystem {
    fn load_font(&mut self, font_family: &'static str, binary_font: &'static [u8]);
    fn render_text(
        &self,
        text: &str,
        font_family: &str,
        font_size: f32,
    ) -> Result<(Vec<u8>, u32, u32), &'static str>;
}

// pub struct TextSystemDummy;
// impl TextSystem for TextSystemDummy  {
//     fn load_font(&mut self, _font_family: &'static str, _binary_font: &'static [u8]) {}

//     fn render_text(
//         &self,
//         _text: &str,
//         _font_family: &str,
//         _font_size: f32,
//     ) -> Result<(Vec<u8>, u32, u32), &'static str> {
//         Err("TextSystemDummy::render_text")
//     }
// }
