use std::{
    ptr::null_mut,
    rc::Rc,
    sync::mpsc::{Receiver, Sender, channel},
    time::Instant,
};
use zune_image::codecs::qoi::zune_core::{colorspace::ColorSpace, options::DecoderOptions};

use xs_backend_wayland_client::{
    WL_COMPOSITOR_CREATE_SURFACE, wl_proxy, wl_request_constructor, wl_surface,
};
use xs_core::{
    AppContext, Application, Backend, BackendMessage, Message, Render, Renderer, TextSystem,
};
use xs_http::HttpClient;
use xs_layout::{Image, TextProperties};

use crate::{
    config::WaylandConfig,
    context::WaylandContext,
    egl::create_egl_window,
    registry::create_registry,
    seat::create_seat,
    xdg::{create_xdg, set_app_id, set_title},
};

pub fn connect(ctx: &mut WaylandContext) {
    unsafe {
        ctx.wl_display = (ctx.client.wl_display_connect)(null_mut());
        if ctx.wl_display.is_null() {
            panic!("Unable to connect to Wayland display");
        }
    }
}

pub fn create_surface(ctx: &mut WaylandContext) -> *mut wl_surface {
    unsafe {
        ctx.wl_surface = wl_request_constructor!(
            ctx.client,
            ctx.wl_compositor,
            WL_COMPOSITOR_CREATE_SURFACE,
            ctx.client.wl_surface_interface
        );
        if ctx.wl_surface.is_null() {
            panic!("failed to create wayland surface");
        }
        ctx.wl_surface
    }
}

pub struct BackendWayland {
    elapsed: u128,
    text_system: Box<dyn TextSystem>,
}

impl BackendWayland {
    pub fn new(text_system: Box<dyn TextSystem>) -> Self {
        Self {
            elapsed: 0,
            text_system,
        }
    }
}

impl Backend for BackendWayland {
    fn run(
        &mut self,
        app: &mut Application,
        app_context: &mut AppContext,
        renderer: &mut Box<dyn Renderer>,
        root: Box<dyn Render>,
    ) {
        let config = WaylandConfig {
            app_id: app.id,
            window_title: app.title,
            window_width: app.width,
            window_height: app.height,
            fullscreen: false,
        };
        let mut ctx = WaylandContext::new(app_context);
        let app_id = app.id;
        let app_title = app.title;
        connect(&mut ctx);
        create_registry(&mut ctx);
        create_seat(&mut ctx);
        create_surface(&mut ctx);
        create_egl_window(&mut ctx, config.window_width, config.window_height);
        create_xdg(&mut ctx);
        set_app_id(&mut ctx, app_id.into());
        set_title(&mut ctx, app_title.into());
        renderer.init();
        let root = Rc::new(root);
        let start = Instant::now();
        let (tx, rx) = channel();
        while ctx.dispatch_pending() != -1 && ctx.app_context.running {
            self.read_messages(&mut ctx.app_context, renderer, tx.clone(), &rx);
            self.elapsed = start.elapsed().as_millis();
            let widget_tree = renderer.render(&mut ctx.app_context, root.clone());
            renderer.draw_frame(&widget_tree);
            ctx.swap_buffers();
            //std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }

    fn read_messages(
        &mut self,
        app_context: &mut AppContext,
        renderer: &mut Box<dyn Renderer>,
        tx: Sender<BackendMessage>,
        rx: &Receiver<BackendMessage>,
    ) {
        app_context.dispatch();
        while let Some(message) = app_context.messages.pop() {
            match message {
                Message::LoadImage(src) => {
                    self.load_image(src, tx.clone());
                }
                Message::Text(text, text_properties) => {
                    eprintln!("Message::Text: {}", text);
                    self.text(text, text_properties, tx.clone());
                }
            }
        }
        if let Ok(message) = rx.try_recv() {
            match message {
                BackendMessage::ImageLoaded(src, bytes, width, height, alpha) => {
                    let texture = renderer.load_texture(&bytes, width, height, alpha);
                    eprintln!("BackendMessage::ImageLoaded: {} {} {}x{} {}", src, bytes.len(), width, height, texture);
                    let image = Image::new(width, height, texture, bytes, alpha);
                    app_context.textures.insert(src, Some(image));
                }
            }
        }
    }

    fn load_image(&mut self, src: String, tx: Sender<BackendMessage>) {
        std::thread::spawn(move || {
            let http = HttpClient::new(src.as_str());
            let response = http.get();
            let bytes = response.body;
            match zune_image::image::Image::read(
                std::io::Cursor::new(bytes),
                DecoderOptions::default(),
            ) {
                Ok(mut image) => {
                    let (width, height) = image.dimensions();
                    let width = width as u32;
                    let height = height as u32;
                    let _ = image.convert_color(ColorSpace::RGBA);
                    let bytes = image.flatten_to_u8().get(0).unwrap().clone();
                    let _ = tx.send(BackendMessage::ImageLoaded(src, bytes, width, height, false));
                }
                Err(error) => {
                    eprintln!("Backend::load_image error: {:?}", error);
                }
            }
        });
    }

    fn new_image(&mut self, _rgba: Vec<u8>, _width: u32, _height: u32) -> u32 {
        0
    }

    fn text(&mut self, text: String, text_properties: TextProperties, tx: Sender<BackendMessage>) {
        if let Ok((bytes, width, height)) =
            self.text_system
                .render_text(&text, &text_properties)
        {
            let _ = tx.send(BackendMessage::ImageLoaded(text, bytes, width, height, true));
        }
    }

    fn performance_now(&self) -> u128 {
        self.elapsed
    }
}
