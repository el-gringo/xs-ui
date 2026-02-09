use std::{ptr::null_mut};

use xs_backend_core::Render;
use xs_backend_wayland_client::{
    WL_COMPOSITOR_CREATE_SURFACE, wl_proxy, wl_request_constructor, wl_surface,
};
use xs_renderer_core::Renderer;
use xs_renderer_opengl::RendererOpenGl;

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
    config: WaylandConfig,
}

impl BackendWayland {
    pub fn new(width: i32, height: i32) -> Self {
        let config = WaylandConfig {
            window_title: String::from("Test"),
            window_width: width,
            window_height: height,
            fullscreen: false,
        };
        Self { config }
    }

    pub fn run<F>(&mut self, mut create_root: F)
    where
        F: FnMut() -> Box<dyn Render>,
    {
        let width = self.config.window_width;
        let height = self.config.window_height;
        let mut renderer = RendererOpenGl::new(width, height);
        let mut ctx = WaylandContext::new();
        connect(&mut ctx);
        create_registry(&mut ctx);
        create_seat(&mut ctx);
        create_surface(&mut ctx);
        create_egl_window(
            &mut ctx,
            self.config.window_width,
            self.config.window_height,
        );
        create_xdg(&mut ctx);
        set_app_id(&mut ctx, "app".into());
        set_title(&mut ctx, "Title".into());
        renderer.init();
        while ctx.dispatch_pending() != -1 && ctx.app_context.running {
            ctx.app_context.dispatch();
            let root = create_root();
            let frame = ctx.app_context.render(root.into(), 0, 0);
            renderer.draw_frame(frame);
            ctx.swap_buffers();
        }
    }

    pub fn log(&self, _message: &str) {}
}
