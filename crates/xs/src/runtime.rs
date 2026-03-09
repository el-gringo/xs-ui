use xs_core::{AppContext, Application, Backend, Render, Renderer, TextSystem, set_app_context};

pub fn run(mut app: Application, root: Box<dyn Render>) {
    let mut text_system = Box::new(xs_text_system::TextSystem::new());
    let font_family = "sans-serif";
    let data = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");
    text_system.load_font(font_family, data);

    let mut app_context = AppContext::new();
    let mut backend = Box::new(xs_backend::Backend::new(text_system));
    let mut renderer =
        Box::new(xs_renderer::Renderer::new(app.width, app.height)) as Box<dyn Renderer>;
    set_app_context(&mut app_context);
    backend.run(&mut app, &mut app_context, &mut renderer, root);
}
