use std::{fs::File, io::Read};

use xs_core::{TextProperties, TextSystem, TextVertices};
use xs_text_system::AtlasAlpha;

const TEXT: &str = r"Un text avec plusieurs mots
et un retour à la ligne";

struct Cli {
    text: String,
    max_width: f32,
    font: Vec<u8>,
    font_size: u16,
    font_weight: u16,
}

impl Cli {
    pub fn parse() -> Self {
        let mut args = std::env::args().skip(1);
        let mut text = TEXT.to_string();
        let mut max_width = 200.;
        let mut font = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf").to_vec();
        let mut font_size = 45;
        let mut font_weight = 400;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--text" => args
                    .next()
                    .map(|value| text = value)
                    .expect("Need valid arguments for --text"),
                "--max-width" => args
                    .next()
                    .map(|value| max_width = value.parse::<f32>().expect("Need valid width number"))
                    .expect("Need argument for --max-width"),
                "--font" => args
                    .next()
                    .map(|value| {
                        font = {
                            let mut buf = Vec::new();
                            let _ = File::open(value).unwrap().read(&mut buf);
                            buf
                        }
                    })
                    .expect("Need a valid path for --font"),
                "--font-size" => args
                    .next()
                    .map(|value| font_size = value.parse::<u16>().expect("Need valid font size"))
                    .expect("Need argument for --font-size"),
                "--font-weight" => args
                    .next()
                    .map(|value| {
                        font_weight = value.parse::<u16>().expect("Need valid font weight")
                    })
                    .expect("Need argument for --font-weight"),
                _ => panic!("Invalid argument: {}", arg),
            }
        }
        Cli {
            text,
            max_width,
            font,
            font_size,
            font_weight,
        }
    }
}

fn main() {
    let Cli {
        font,
        text,
        max_width,
        font_size,
        font_weight,
    } = Cli::parse();
    let mut text_system: xs_text_system::TextSystem<AtlasAlpha> = xs_text_system::TextSystem::new();
    let font_family = "no-name";
    let font_style = xs_core::FontStyle::Normal;
    text_system.load_font(font_family.to_string(), font_weight, font);

    let text_properties =
        TextProperties::new(font_family.into(), font_size, font_weight, font_style);
    let TextVertices {
        vertices,
        width,
        height,
        ..
    } = text_system.render_text(&text, &text_properties, max_width, 200.);

    println!(
        "Text rendered: {}x{} with {} vertices",
        width,
        height,
        vertices.len() / 4
    );
}
