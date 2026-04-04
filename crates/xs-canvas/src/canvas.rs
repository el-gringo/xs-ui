use xs_core::{AppConfig, AppContext, Backend, EventLoop, TextSystemDummy};

pub struct CanvasEventLoop(Box<dyn Fn(&mut AppContext)>);
impl EventLoop for CanvasEventLoop {
    fn initialize(&mut self, _ctx: &mut AppContext) {}

    fn frame(&mut self, ctx: &mut AppContext) {
        self.0(ctx);
    }
}

#[derive(Default)]
pub struct Canvas(AppConfig);

impl Canvas {
    pub fn new() -> Self {
        Self(Default::default())
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

    pub fn run<I, F>(&self, on_initialize: I)
    where
        I: 'static + Fn(&mut AppContext) -> F,
        F: 'static + Fn(&mut AppContext),
    {
        let text_system = Box::new(TextSystemDummy);
        let mut app_context = AppContext::new(text_system);
        let mut backend = Box::new(xs_backend::Backend::new());
        let on_render = on_initialize(&mut app_context);
        let event_loop = Box::new(CanvasEventLoop(Box::new(on_render)));
        backend.run(&self.0, app_context, event_loop);
    }
}
