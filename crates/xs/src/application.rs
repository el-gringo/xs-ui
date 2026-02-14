use std::str;

use xs_backend_core::{App, Backend, Render};
use xs_renderer::Renderer;

pub struct Application(App);
impl Application {
    pub fn new() -> Self {
        Self(App {
            id: "app",
            title: "App",
            width: 960,
            height: 540,
            backend: None,
        })
    }

    pub fn id(mut self, id: &'static str) -> Self {
        self.0.id = id;
        self
    }

    pub fn title(mut self, title: &'static str) -> Self {
        self.0.title = title;
        self
    }

    pub fn height(mut self, height: i32) -> Self {
        self.0.height = height;
        self
    }

    pub fn width(mut self, width: i32) -> Self {
        self.0.width = width;
        self
    }

    pub fn run(self, root: Box<dyn Render>) {
        let renderer = Box::new(Renderer::new(self.0.width, self.0.height));
        let mut backend = xs_backend::Backend::new(
            self.0.id,
            self.0.title,
            renderer,
        );
        backend.run(root);
    }
}
