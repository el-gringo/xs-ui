use xs_core::{AppConfig, AppContext, Backend, EventLoop, TextSystemDummy};

#[derive(Default)]
pub struct Canvas {
    app_config: AppConfig,
}

impl Canvas {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn id(mut self, id: &'static str) -> Self {
        self.app_config.id = id;
        self
    }

    pub fn title(mut self, title: &'static str) -> Self {
        self.app_config.title = title;
        self
    }

    pub fn height(mut self, height: i32) -> Self {
        self.app_config.height = height;
        self
    }

    pub fn width(mut self, width: i32) -> Self {
        self.app_config.width = width;
        self
    }

    pub fn run<E>(&self, event_loop: E)
    where
        E: EventLoop + 'static,
    {
        let text_system = Box::new(TextSystemDummy);
        let app_context = AppContext::new(text_system);
        let mut backend = Box::new(xs_backend::Backend::new());
        backend.run(&self.app_config, app_context, Box::new(event_loop));
    }
}
