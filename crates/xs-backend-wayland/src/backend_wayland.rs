use std::{ptr::null_mut, sync::mpsc::Sender, time::Instant};

use xs_backend_wayland_client::{
    WL_COMPOSITOR_CREATE_SURFACE, wl_proxy, wl_request_constructor, wl_surface,
};
use xs_core::{AppConfig, AppContext, Backend, BackendResponse, EventLoop};

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
        let app_id = app.id;
        let app_title = app.title;
        connect(&mut ctx);
        create_registry(&mut ctx);
        create_seat(&mut ctx);
        create_surface(&mut ctx);
        create_xdg(&mut ctx, app_id, app_title);
        create_egl_window(&mut ctx, config.window_width, config.window_height);
        event_loop.initialize(ctx.app_context);
        let start = Instant::now();
        ctx.swap_buffers();
        ctx.swap_buffers();
        while ctx.app_context.running {
            ctx.app_context.now = (Instant::now() - start).as_millis();
            self.elapsed = start.elapsed().as_millis();
            while ctx.dispatch_pending() > 0 {}
            let rendered = event_loop.frame(ctx.app_context);
            if rendered {
                ctx.swap_buffers();
            } else {
                ctx.dispatch();
            }
        }
    }

    /// Not used yet, this will need to adapt event loop to e able to pass messages to the backend
    fn load_image(&mut self, src: String, tx: Sender<BackendResponse>) {
        load_image(src, tx);
    }
}

#[cfg(not(feature = "image"))]
pub fn load_image(_src: String, _tx: Sender<BackendResponse>) {}

#[cfg(feature = "image")]
pub fn load_image(src: String, tx: Sender<BackendResponse>) {
    use xs_http::HttpClient;
    use xs_image::{ColorSpace, Image, ImageOptions};

    let http = HttpClient::new();
    http.get(src.clone(), move |response| {
        // `xs_image` sniffs the container itself, so no `Content-Type` and no
        // file extension are needed to pick a codec. RGBA is requested because
        // `Application::read_messages` hands the samples straight to
        // `glTexImage2D` as `GL_RGBA`, so the byte count has to be
        // `width * height * 4`.
        match Image::read(
            response.body.as_slice(),
            ImageOptions::default().color_space(ColorSpace::Rgba),
        ) {
            Ok(image) => {
                let (width, height) = image.dimensions();
                let bytes = image.into_buffer().into_vec();
                let _ = tx.send(BackendResponse::ImageLoaded(
                    src.clone(),
                    -1,
                    bytes,
                    width,
                    height,
                ));
            }
            Err(error) => {
                eprintln!("Backend::load_image error: {}", error);
            }
        }
    });
}

pub fn reload() {
    eprintln!("backend_wayland::reload() not implemented");
}
