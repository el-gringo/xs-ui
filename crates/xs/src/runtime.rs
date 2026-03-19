use std::rc::Rc;

use xs_core::{AppContext, Application, Backend, Render, Renderer};

pub fn run(app: Application, root: Rc<dyn Render>) {
    let renderer = Box::new(xs_renderer::Renderer::new(app.width, app.height)) as Box<dyn Renderer>;
    let text_system = Box::new(xs_text_system::TextSystem::new());
    let app_context = AppContext::new();
    let mut backend = Box::new(xs_backend::Backend::new());
    backend.run(app, app_context, renderer, text_system, root);
}
