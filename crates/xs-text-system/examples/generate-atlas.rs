use std::{fs::File, io::Write};

use xs_text_system::FontAtlas;
use xs_text_system_glyph::{Font, FontRef, Glyph, PxScale, ScaleFont};

pub fn main() {
    let font_size = 48;
    let binary_font = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");
    let font = FontRef::try_from_slice(binary_font).unwrap();
    let (font_atlas, atlas) = FontAtlas::generate(&font, font_size as _);
    let width = font_atlas.width;
    let height = font_atlas.height;
    let filename = "atlas.pgm";
    let mut o = File::create(filename).unwrap();
    let _ = o.write(format!("P5\n{} {}\n255\n", width, height).as_bytes());
    if let Ok(size) = o.write(&atlas) {
        println!("Generated: text.pgn ({width}x{height}) {size}");
    }
}
