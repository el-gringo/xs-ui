use std::{ptr::null_mut, rc::Rc};

use xs_backend_wayland_client::{
    WL_COMPOSITOR_CREATE_SURFACE, wl_proxy, wl_request_constructor, wl_surface,
};
use xs_core::{Application, Backend, Render, Renderer};

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

pub struct BackendWayland;

impl BackendWayland {
    pub fn new() -> Self {
        Self
    }
}

impl Backend for BackendWayland {
    fn run(
        &mut self,
        app: &mut Application,
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
        let mut ctx = WaylandContext::new(app);
        let app_id = ctx.app.id;
        let app_title = ctx.app.title;
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
        while ctx.dispatch_pending() != -1 && ctx.app.ctx.running {
            ctx.app.ctx.dispatch();
            let widget_tree = renderer.render(ctx.app, root.clone());
            renderer.draw_frame(&widget_tree);
            ctx.app.ctx.nav.tree.borrow().debug(0, 0);
            ctx.swap_buffers();
            //std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }

    fn load_image(&mut self, _src: &'static str, _ptr: u32) {
        todo!()
    }

    fn new_image(&mut self, _array: Vec<u8>, _width: u32) -> u32 {
        todo!()
    }
}
