use std::{fs::File, io::Write};

use xs_core::{TextProperties, TextSystem};

//const TEXT: &str = "This is ab_glyph rendered into a png!";
const TEXT: &str = "PeerTube";

fn main() {
    let mut text_system = xs_text_system::TextSystem::new();

    let font_family = "sans-serif";
    let data = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");
    text_system.load_font(font_family, data);

    let text_properties = Default::default();
    if let Ok((data, width, height)) = text_system.render_text(TEXT, &text_properties) {
        // Save the image to a png file
        let mut o = File::create("text.pgm").unwrap();
        let _ = o.write(format!("P5\n{} {}\n255\n", width, height).as_bytes());
        if let Ok(size) = o.write(&data) {
            println!("Generated: text.pgn ({}x{}) {}", width, height, size);
        }
    }
}
