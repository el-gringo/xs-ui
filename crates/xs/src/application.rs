use xs_backend_core::{BackendDummy, Render};

pub struct Application {
    id: &'static str,
    title: &'static str,
    width: i32,
    height: i32,
    backend: Box<dyn xs_backend_core::Backend>
}

impl Application {
    pub fn new() -> Self {
        Application {
            id: "app",
            title: "App",
            width: 960,
            height: 540,
            backend: Box::new(BackendDummy {}) as Box<dyn xs_backend_core::Backend>
        }
    }

    pub fn id(mut self, id: &'static str) -> Self {
        self.id = id;
        self
    }

    pub fn title(mut self, title: &'static str) -> Self {
        self.title = title;
        self
    }

    pub fn height(mut self, height: i32) -> Self {
        self.height = height;
        self
    }

    pub fn width(mut self, width: i32) -> Self {
        self.width = width;
        self
    }

    pub fn run(mut self, root: Box<dyn Render>) {
        let renderer = Box::new(xs_renderer::Renderer::new(self.width, self.height));
        let backend = xs_backend::Backend::new(self.id, self.title, renderer);
        self.backend = Box::new(backend);
        self.backend.run(root);
    }
}
