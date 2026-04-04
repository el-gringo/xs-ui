use std::{fs::File, io::Read};

use xs_core::{Renderer, RendererDummy, TextProperties, TextSystem, TextVertices};

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
    let mut text_system = xs_text_system::TextSystem::new();
    // Use a boxed RendererDummy to satisfy the trait
    let mut renderer: Box<dyn Renderer> = Box::new(RendererDummy);
    let font_family = "no-name";
    text_system.load_font(font_family, font_weight, font, &mut renderer);

    let mut text_properties = TextProperties::default();
    text_properties.font_family = font_family.into();
    text_properties.font_size = font_size;
    text_properties.font_weight = font_weight;
    let constraints = (max_width, 200.);
    let TextVertices {
        num_vertices,
        width,
        height,
        ..
    } = text_system.render_text(&text, &text_properties, &constraints);

    println!(
        "Text rendered: {}x{} with {} vertices",
        width, height, num_vertices
    );
}
