use std::{ptr::null_mut, sync::mpsc::Sender, time::Instant};
use xs_http::HttpClient;
use zune_image::codecs::qoi::zune_core::{colorspace::ColorSpace, options::DecoderOptions};

use xs_backend_wayland_client::{
    WL_COMPOSITOR_CREATE_SURFACE, wl_proxy, wl_request_constructor, wl_surface,
};
use xs_core::{AppConfig, AppContext, Backend, BackendResponse, EventLoop, set_app_context};

use crate::{
    config::WaylandConfig, context::WaylandContext, egl::create_egl_window,
    registry::create_registry, seat::create_seat, xdg::create_xdg,
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

#[derive(Default)]
pub struct BackendWayland {
    elapsed: u128,
}

impl BackendWayland {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Backend for BackendWayland {
    fn run(
        &mut self,
        app: &AppConfig,
        mut app_context: AppContext,
        mut event_loop: Box<dyn EventLoop>,
    ) {
        let config = WaylandConfig {
            app_id: app.id,
            window_title: app.title,
            window_width: app.width,
            window_height: app.height,
            fullscreen: false,
        };
        let mut ctx = WaylandContext::new(&mut app_context);
        //let mut message_dispatcher = WaylandDispatcher;
        let app_id = app.id;
        let app_title = app.title;
        connect(&mut ctx);
        create_registry(&mut ctx);
        create_seat(&mut ctx);
        create_surface(&mut ctx);
        create_xdg(&mut ctx, app_id, app_title);
        create_egl_window(&mut ctx, config.window_width, config.window_height);
        event_loop.initialize(ctx.app_context);
        set_app_context(ctx.app_context);
        let start = Instant::now();
        while ctx.dispatch_pending() != -1 && ctx.app_context.running {
            self.elapsed = start.elapsed().as_millis();
            event_loop.frame(ctx.app_context);
            ctx.swap_buffers();
            //std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }

    fn performance_now(&self) -> u128 {
        self.elapsed
    }
}

pub fn load_image(src: String, tx: Sender<BackendResponse>) {
    std::thread::spawn(move || {
        let http = HttpClient::new();
        http.get(
            src.clone(),
            move |response| match zune_image::image::Image::read(
                std::io::Cursor::new(response.body),
                DecoderOptions::default(),
            ) {
                Ok(mut image) => {
                    let (width, height) = image.dimensions();
                    let width = width as u32;
                    let height = height as u32;
                    let _ = image.convert_color(ColorSpace::RGBA);
                    let bytes = image.flatten_to_u8().first().unwrap().clone();
                    let _ = tx.send(BackendResponse::ImageLoaded(
                        src.clone(),
                        -1,
                        bytes,
                        width,
                        height,
                    ));
                }
                Err(error) => {
                    eprintln!("Backend::load_image error: {:?}", error);
                }
            },
        );
    });
}
