use std::{ptr::null_mut, sync::mpsc::Sender, time::Instant};

use xs_backend_wayland_client::{
    WL_COMPOSITOR_CREATE_SURFACE, wl_proxy, wl_request_constructor, wl_surface, wl_surface_listener,
};
use xs_core::{AppConfig, AppContext, Backend, BackendResponse, EventLoop};

use crate::{
    context::WaylandContext, egl::create_egl_window, registry::create_registry, seat::create_seat,
    xdg::create_xdg,
};

pub fn connect(ctx: &mut WaylandContext) {
    unsafe {
        ctx.wl_display = (ctx.client.wl_display_connect)(null_mut());
        if ctx.wl_display.is_null() {
            panic!("Unable to connect to Wayland display");
        }
    }
}

/// `wl_surface.preferred_buffer_scale`: how many buffer pixels this surface
/// would like per surface-local pixel.
///
/// This is the scale that matters, and the protocol says to prefer it over
/// `wl_output.scale`: it is a property of *this* surface rather than of the
/// output, and a compositor is required to keep it current. It is the native
/// counterpart of the browser reading `devicePixelRatio` once per frame.
unsafe extern "C" fn surface_handle_preferred_buffer_scale(
    data: *mut std::ffi::c_void,
    _surface: *mut wl_surface,
    factor: std::ffi::c_int,
) {
    unsafe {
        assert!(!data.is_null());
        let ctx: &mut WaylandContext = &mut *(data as *mut _);
        ctx.apply_scale(factor as f32, true);
    }
}

static SURFACE_LISTENER: wl_surface_listener = {
    let mut listener = wl_surface_listener::dummy();
    listener.preferred_buffer_scale = surface_handle_preferred_buffer_scale;
    listener
};

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
        (ctx.client.wl_proxy_add_listener)(
            ctx.wl_surface as _,
            &raw const SURFACE_LISTENER as *const _ as _,
            ctx as *mut _ as _,
        );
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
        // The declared size is a surface-local one, which is what `Screen` holds
        // and what the compositor's configure will replace. It is set here so
        // the egl window has something to be created with before that configure
        // arrives - on a compositor that never sends one, the app would
        // otherwise have a zero-sized viewport.
        app_context.screen.width = app.width.max(0) as u32;
        app_context.screen.height = app.height.max(0) as u32;
        let mut ctx = WaylandContext::new(&mut app_context);
        let app_id = app.id;
        let app_title = app.title;
        connect(&mut ctx);
        create_registry(&mut ctx);
        // The listener has to be attached *before* anything dispatches, and
        // this is the first call that would: the compositor sends
        // `wl_seat.capabilities` as soon as the seat is bound inside
        // `create_registry`'s round trip, so the event is already queued. A
        // round trip here would consume it with no listener on the proxy, which
        // discards it - and the compositor only resends `capabilities` when the
        // set changes, so `wl_keyboard` would never be created and no key
        // would ever reach the app.
        create_seat(&mut ctx);
        // Safe to dispatch now that both listeners are in place. The output's
        // geometry, mode and scale are only delivered to a proxy created
        // during the registry round trip on the *next* one, so the scale is
        // still unknown until this returns - and it is read two lines down, to
        // size the egl window.
        ctx.round_trip();
        create_surface(&mut ctx);
        // Announced before the first commit, so the very buffer the egl window
        // is sized for is the one the compositor expects to receive.
        ctx.set_buffer_scale(ctx.app_context.screen.scale);
        create_xdg(&mut ctx, app_id, app_title);
        create_egl_window(&mut ctx);
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
