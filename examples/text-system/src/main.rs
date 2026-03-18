use xs_core::{Renderer, RendererDummy, TextProperties, TextSystem};

const TEXT: &str = r"Un text avec plusieurs mots
et un retour à la ligne";

fn main() {
    let mut text_system = xs_text_system::TextSystem::new();

    let font_family = "sans-serif";
    let font_weight = 400;
    let data = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");

    // Use a boxed RendererDummy to satisfy the trait
    let mut renderer: Box<dyn Renderer> = Box::new(RendererDummy);
    text_system.load_font(font_family, font_weight, data, &mut renderer);

    let mut text_properties = TextProperties::default();
    text_properties.font_size = 45;
    text_properties.max_width = Some(200.0);
    let (vertices, width, height, _) = text_system.render_text(TEXT, &text_properties);
    println!(
        "Text rendered: {}x{} with {} vertices",
        width, height, vertices.num_vertices
    );
}
