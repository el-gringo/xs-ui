use std::{
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
};

use xs_backend::load_image;
use xs_core::{
    AppConfig, AppContext, Backend, BackendRequest, BackendResponse, EventLoop, Render, Renderer,
};
use xs_layout::Image;
use xs_text_system::{AtlasAlpha, TextSystem};

#[derive(Default)]
pub struct Application(AppConfig);

struct AppEventLoop {
    renderer: Box<dyn Renderer>,
    root: Rc<dyn Render>,
    tx: Sender<BackendResponse>,
    rx: Rc<Receiver<BackendResponse>>,
}

impl AppEventLoop {
    pub fn new(renderer: Box<dyn Renderer>, root: Rc<dyn Render>) -> Self {
        let (tx, rx) = channel();
        let rx = Rc::new(rx);
        Self {
            renderer,
            root,
            rx,
            tx,
        }
    }
    pub fn read_messages(&mut self, app_context: &mut AppContext) {
        app_context.dispatch();
        while let Some(message) = app_context.messages.pop() {
            match message {
                BackendRequest::LoadImage(src) => {
                    load_image(src, self.tx.clone());
                }
            }
        }
        if let Ok(message) = self.rx.try_recv() {
            match message {
                BackendResponse::ImageLoaded(src, image_handle, bytes, width, height) => {
                    let texture =
                        self.renderer
                            .load_texture(&bytes, image_handle, width, height, false);
                    let image = Image::new(width, height, texture, bytes, false);
                    app_context.textures.insert(src.clone(), Some(image));
                }
            }
        }
    }
}

impl EventLoop for AppEventLoop {
    fn initialize(&mut self, ctx: &mut AppContext) {
        self.renderer.init();
        let font_family = "sans-serif";
        let font_size = 42.;
        let font_data =
            include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf").to_vec();
        let (atlas_data, width, height) =
            ctx.text_system
                .load_font(font_family, 400, font_size, font_data);
        self.renderer
            .load_font_atlas(font_family, 400, font_size, &atlas_data, width, height);
    }

    fn frame(&mut self, ctx: &mut AppContext) {
        self.read_messages(ctx);
        let widget_tree = self.renderer.render(ctx, self.root.clone());
        self.renderer.draw_frame(&widget_tree);
    }
}

impl Application {
    pub fn new() -> Self {
        Default::default()
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

    pub fn run<I, R>(&self, on_initialize: I)
    where
        I: 'static + FnOnce(&mut AppContext) -> R,
        R: 'static + Render,
    {
        let text_system = Box::new(TextSystem::<AtlasAlpha>::new());
        let mut app_context = AppContext::new(text_system);
        let renderer = Box::new(xs_renderer::Renderer::new(self.0.width, self.0.height));
        let root = Rc::new(on_initialize(&mut app_context));
        let event_loop = Box::new(AppEventLoop::new(renderer, root));
        let mut backend = xs_backend::Backend::new();
        backend.run(&self.0, app_context, event_loop);
    }
}
