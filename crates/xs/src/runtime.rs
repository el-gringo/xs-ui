use xs_core::{Application, Backend, BackendDummy, Render, Renderer, RendererDummy, TextSystem};

pub struct Runtime {
    app: Application,
    backend: Box<dyn Backend>,
    renderer: Box<dyn Renderer>,
}

impl Runtime {
    pub fn new(app: Application) -> Self {
        Self {
            app,
            backend: Box::new(BackendDummy),
            renderer: Box::new(RendererDummy),
        }
    }

    pub fn run(&mut self, root: Box<dyn Render>) {
        let mut text_system = Box::new(xs_text_system::TextSystem::new());
        let font_family = "IBMPlexSans Regular";
        let data = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");
        text_system.load_font(font_family, data);
        self.backend = Box::new(xs_backend::Backend::new());
        self.renderer = Box::new(xs_renderer::Renderer::new(
            self.app.width,
            self.app.height,
            text_system,
        ));
        self.backend.run(&mut self.app, &mut self.renderer, root);
    }
}

pub fn run(app: Application, root: Box<dyn Render>) {
    Runtime::new(app).run(root);
}
