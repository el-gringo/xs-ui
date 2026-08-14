use std::sync::{
    atomic::Ordering,
    mpsc::{Receiver, Sender, channel},
};

use xs_backend::load_image;
use xs_core::{
    AppConfig, AppContext, Backend, BackendRequest, BackendResponse, EventLoop, ImageFormat,
    IntoElement, PENDING_UPDATE, Render, Renderer,
};
use xs_layout::Image;
use xs_text_system::{AtlasAlpha, TextSystem};

/// Temporary per-second max-duration profiler for the frame phases. Logs one
/// line per second; remove once the slow frames are tracked down.
#[cfg(target_arch = "wasm32")]
mod frame_profile {
    use std::sync::atomic::{AtomicU32, Ordering};

    static BE: AtomicU32 = AtomicU32::new(0);
    static BET: AtomicU32 = AtomicU32::new(0);
    static LAY: AtomicU32 = AtomicU32::new(0);
    static REN: AtomicU32 = AtomicU32::new(0);
    static TOT: AtomicU32 = AtomicU32::new(0);
    static LAST: AtomicU32 = AtomicU32::new(0);

    fn bump(max: &AtomicU32, v: u32) {
        let mut cur = max.load(Ordering::Relaxed);
        while v > cur {
            match max.compare_exchange(cur, v, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(actual) => cur = actual,
            }
        }
    }

    pub fn record(be: u32, bet: u32, lay: u32, ren: u32, tot: u32) {
        bump(&BE, be);
        bump(&BET, bet);
        bump(&LAY, lay);
        bump(&REN, ren);
        bump(&TOT, tot);
        let now = xs_js_sys::performance_now();
        let last = LAST.load(Ordering::Relaxed);
        if last == 0 {
            LAST.store(now, Ordering::Relaxed);
        } else if now.wrapping_sub(last) >= 1_000_000 {
            let msg = format!(
                "MAX be={}us bet={}us lay={}us ren={}us tot={}us",
                BE.load(Ordering::Relaxed),
                BET.load(Ordering::Relaxed),
                LAY.load(Ordering::Relaxed),
                REN.load(Ordering::Relaxed),
                TOT.load(Ordering::Relaxed),
            );
            xs_js_sys::log(&msg);
            let items = xs_layout::layout_prof::ITEMS_US.load(Ordering::Relaxed);
            let place = xs_layout::layout_prof::PLACE_US.load(Ordering::Relaxed);
            let frames = xs_layout::layout_prof::FRAMES.load(Ordering::Relaxed);
            if frames > 0 {
                xs_js_sys::log(&format!(
                    "  flex items={}us place={}us frames={}",
                    items,
                    place,
                    frames
                ));
                xs_layout::layout_prof::reset();
            }
            BE.store(0, Ordering::Relaxed);
            BET.store(0, Ordering::Relaxed);
            LAY.store(0, Ordering::Relaxed);
            REN.store(0, Ordering::Relaxed);
            TOT.store(0, Ordering::Relaxed);
            LAST.store(now, Ordering::Relaxed);
        }
    }
}

#[derive(Default)]
pub struct Application(AppConfig);

struct AppEventLoop<R: Render> {
    renderer: Box<dyn Renderer>,
    root: R,
    tx: Sender<BackendResponse>,
    rx: std::rc::Rc<Receiver<BackendResponse>>,
}

impl<R: Render> AppEventLoop<R> {
    pub fn new(renderer: Box<dyn Renderer>, root: R) -> Self {
        let (tx, rx) = channel();
        let rx = std::rc::Rc::new(rx);
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
                    let texture = self.renderer.load_texture(
                        &bytes,
                        image_handle,
                        width,
                        height,
                        ImageFormat::Rgba,
                    );
                    let image = Image::new(width, height, texture, bytes, ImageFormat::Rgba);
                    app_context.add_texture(src, image);
                }
            }
        }
    }
}

impl<R: Render + 'static> EventLoop for AppEventLoop<R> {
    fn initialize(&mut self, ctx: &mut AppContext) {
        self.renderer.init(ctx);
        let font_family = "sans-serif";
        let font_data =
            include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf").to_vec();
        let atlas_info = ctx
            .text_system
            .load_font(font_family.into(), 400, font_data);
        self.renderer.load_font_atlas(font_family, 400, atlas_info);
    }

    fn frame(&mut self, ctx: &mut AppContext) -> bool {
        #[cfg(target_arch = "wasm32")]
        let frame_start = xs_js_sys::performance_now();
        self.read_messages(ctx);
        if PENDING_UPDATE.swap(false, Ordering::Relaxed) {
            ctx.dirty = true;
        }
        if !ctx.dirty {
            return false;
        }
        ctx.dirty = false;

        let constraints = xs_core::BoxConstraints {
            min: xs_layout::Vector2::new(0.0, 0.0),
            max: xs_layout::Vector2::new(ctx.screen.width as f32, ctx.screen.height as f32),
        };
        let mut node_ctx = xs_core::Context::new(
            ctx,
            (std::any::TypeId::of::<R>(), 0, 0),
            ctx.current_nav_node_id,
            xs_core::TextProperties::default(),
            xs_core::Color::rgba(0.0, 0.0, 0.0, 1.0),
            constraints,
        );
        #[cfg(target_arch = "wasm32")]
        let t0 = xs_js_sys::performance_now();
        let root_element = self.root.render(&mut node_ctx).into_element();
        #[cfg(target_arch = "wasm32")]
        let build_ms = t0.wrapping_sub(frame_start).min(u32::MAX / 2);
        let root_nav_id = node_ctx.nav_id;
        drop(node_ctx);
        ctx.current_nav_node_id = root_nav_id;
        #[cfg(target_arch = "wasm32")]
        let t1 = xs_js_sys::performance_now();
        let widget_tree = self.renderer.build_element_tree(ctx, root_element);
        #[cfg(target_arch = "wasm32")]
        let build_tree_ms = t1.wrapping_sub(t0).min(u32::MAX / 2);
        #[cfg(target_arch = "wasm32")]
        let t2 = xs_js_sys::performance_now();
        let layout_tree =
            self.renderer
                .layout(&widget_tree, ctx.screen.width as _, ctx.screen.height as _);
        #[cfg(target_arch = "wasm32")]
        let layout_ms = t2.wrapping_sub(t1).min(u32::MAX / 2);
        #[cfg(target_arch = "wasm32")]
        {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static LOGGED: AtomicUsize = AtomicUsize::new(0);
            if LOGGED.load(Ordering::Relaxed) < 3 {
                LOGGED.fetch_add(1, Ordering::Relaxed);
                xs_js_sys::log(&format!("layout boxes: {}", layout_tree.boxes.len()));
            }
        }
        #[cfg(target_arch = "wasm32")]
        let t3 = xs_js_sys::performance_now();
        self.renderer.render(ctx, &layout_tree);
        #[cfg(target_arch = "wasm32")]
        {
            let render_ms = t3.wrapping_sub(t2).min(u32::MAX / 2);
            let total_ms = t3.wrapping_sub(frame_start).min(u32::MAX / 2);
            frame_profile::record(build_ms, build_tree_ms, layout_ms, render_ms, total_ms);
        }

        true
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
        let renderer = Box::new(xs_renderer::Renderer::new());
        let root = on_initialize(&mut app_context);
        let mut backend = xs_backend::Backend::new();
        let event_loop = Box::new(AppEventLoop::new(renderer, root));
        backend.run(&self.0, app_context, event_loop);
    }
}
