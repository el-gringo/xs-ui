use std::{ptr::null_mut, rc::Rc, sync::mpsc::channel, time::Instant};

use xs_backend_wayland_client::{
    WL_COMPOSITOR_CREATE_SURFACE, wl_proxy, wl_request_constructor, wl_surface,
};
use xs_core::{AppContext, Application, Backend, Render, Renderer, TextSystem, set_app_context};

use crate::{
    config::WaylandConfig, context::WaylandContext, egl::create_egl_window,
    registry::create_registry, seat::create_seat, wayland_dispatcher::WaylandDispatcher,
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
        app: Application,
        mut app_context: AppContext,
        mut renderer: Box<dyn Renderer>,
        mut text_system: Box<dyn TextSystem>,
        root: Rc<dyn Render>,
    ) {
        let config = WaylandConfig {
            app_id: app.id,
            window_title: app.title,
            window_width: app.width,
            window_height: app.height,
            fullscreen: false,
        };
        let mut ctx = WaylandContext::new(&mut app_context);
        let mut message_dispatcher = WaylandDispatcher;
        let app_id = app.id;
        let app_title = app.title;
        connect(&mut ctx);
        create_registry(&mut ctx);
        create_seat(&mut ctx);
        create_surface(&mut ctx);
        create_xdg(&mut ctx, app_id, app_title);
        create_egl_window(&mut ctx, config.window_width, config.window_height);
        renderer.init();
        let font_family = "sans-serif";
        let data = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");
        text_system.load_font(font_family, 400, data, &mut renderer);
        set_app_context(ctx.app_context);
        let start = Instant::now();
        let (tx, rx) = channel();
        let rx = Rc::new(rx);
        while ctx.dispatch_pending() != -1 && ctx.app_context.running {
            let rx = rx.clone();
            message_dispatcher.read_messages(
                ctx.app_context,
                &mut renderer,
                &mut text_system,
                tx.clone(),
                rx,
            );
            self.elapsed = start.elapsed().as_millis();
            let widget_tree = renderer.render(ctx.app_context, root.clone());
            renderer.draw_frame(&widget_tree);
            ctx.swap_buffers();
            //std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }

    fn performance_now(&self) -> u128 {
        self.elapsed
    }
}
