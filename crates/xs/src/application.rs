use xs_core::{AppContext, Application, Backend, BackendDummy, Render, RendererDummy, TextSystem};
use xs_renderer::Renderer;

pub trait Runnable {
    fn run(&mut self, root: Box<dyn Render>);
}

impl Runnable for Application {
    fn run(&mut self, root: Box<dyn Render>) {
        let mut text_system = Box::new(xs_text_system::TextSystem::new());
        let font_family = "IBMPlexSans Regular";
        let data = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");
        text_system.load_font(font_family, data);
        self.ctx.renderer = Box::new(Renderer::new(self.width, self.height, text_system));
        self.ctx.backend = Box::new(xs_backend::Backend);
        self.ctx.backend.run(self, root);
    }
}
