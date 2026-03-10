use xs_core::{AppContext, Application, Backend, Render, Renderer, TextSystem};

pub fn run(app: Application, root: Box<dyn Render>) {
    let mut text_system = Box::new(xs_text_system::TextSystem::new());
    let font_family = "sans-serif";
    let data = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");
    text_system.load_font(font_family, data);

    let app_context = AppContext::new();
    let mut backend = Box::new(xs_backend::Backend::new(text_system));
    let renderer =
        Box::new(xs_renderer::Renderer::new(app.width, app.height)) as Box<dyn Renderer>;
    backend.run(app, app_context, renderer, root);
}
