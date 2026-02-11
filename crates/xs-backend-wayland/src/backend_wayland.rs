use std::{ptr::null_mut, rc::Rc};

use xs_backend_core::{Backend, Render};
use xs_backend_wayland_client::{
    WL_COMPOSITOR_CREATE_SURFACE, wl_proxy, wl_request_constructor, wl_surface,
};
use xs_renderer_core::Renderer;

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
    id: &'static str,
    title: &'static str,
    renderer: Box<dyn Renderer>,
}

impl BackendWayland {
    pub fn new(id: &'static str, title: &'static str, renderer: Box<dyn Renderer>) -> Self {
        Self {
            id,
            title,
            renderer,
        }
    }

    pub fn log(&self, _message: &str) {}
}

impl Backend for BackendWayland {
    fn run(&mut self, root: Box<dyn Render>) {
        let config = WaylandConfig {
            app_id: self.id,
            window_title: self.title,
            window_width: self.renderer.screen_width(),
            window_height: self.renderer.screen_height(),
            fullscreen: false,
        };
        let mut ctx = WaylandContext::new();
        connect(&mut ctx);
        create_registry(&mut ctx);
        create_seat(&mut ctx);
        create_surface(&mut ctx);
        create_egl_window(&mut ctx, config.window_width, config.window_height);
        create_xdg(&mut ctx);
        set_app_id(&mut ctx, self.id.into());
        set_title(&mut ctx, self.title.into());
        self.renderer.init();
        let root = Rc::new(root);
        while ctx.dispatch_pending() != -1 && ctx.app_context.running {
            ctx.app_context.dispatch();
            let frame = ctx.app_context.render(root.clone(), 0, 0);
            self.renderer.draw_frame(frame);
            ctx.swap_buffers();
        }
    }
}
