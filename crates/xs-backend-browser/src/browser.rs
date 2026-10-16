use std::{cell::RefCell, rc::Rc, sync::mpsc::Sender};

use crate::{bindings, browser_event};

use xs_core::{AppConfig, AppContext, Backend, BackendResponse, EventLoop};
use xs_js_sys::{
    Canvas, Closure, Element, Event, EventEmitter, EventKeyDown, Node, Window, device_pixel_ratio,
    document, request_animation_frame, window,
};
use xs_platform_event::KeyEvent;

#[derive(Default)]
pub struct BackendBrowser;

impl BackendBrowser {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Backend for BackendBrowser {
    fn run(
        &mut self,
        app: &AppConfig,
        mut app_context: AppContext,
        mut event_loop: Box<dyn EventLoop>,
    ) {
        // The page has no canvas: the backend owns it, the html only styles it.
        // It has to be in the document before the event loop initializes, the
        // renderer creates its gl resources there.
        let doc = document();
        let canvas = doc.create_element("canvas");
        // Asking for the context is also how the app says which canvas it
        // renders into: the gl bindings read `app.gl` from then on. It has to
        // happen before the event loop initializes, the renderer creates its
        // gl resources there.
        let _ = canvas.get_context("webgl2");
        canvas.set_attribute("id", "glcanvas");
        canvas.set_attribute("tabindex", "1");
        doc.query_selector("body").append_child(&canvas);

        let width = app.width as u32;
        let height = app.height as u32;
        // The css size stays the logical one the app was declared at, so the
        // layout is the same on every display; the drawing buffer is that many
        // times larger so the GPU rasterises into device pixels rather than into
        // a 1080p image the compositor then stretches. See [`resize_canvas`].
        canvas.set_style("width", &format!("{width}px"));
        canvas.set_style("height", &format!("{height}px"));
        app_context.screen.width = width;
        app_context.screen.height = height;
        app_context.screen.set_scale(device_pixel_ratio());
        resize_canvas(&*canvas, width, height);
        // The canvas is what receives the key events, so keep the focus on it.
        canvas.focus();
        canvas.add_event_listener(
            "blur",
            Closure::new(|_event: Event| {
                document().query_selector("canvas").focus();
            }),
        );

        event_loop.initialize(&mut app_context);
        let app_context = Rc::new(RefCell::new(app_context));

        let app_context_clone = app_context.clone();
        canvas.add_event_listener(
            "keydown",
            Closure::new(move |event: EventKeyDown| {
                let code = browser_event::_translate_code(event.code());
                let mods = browser_event::_translate_mod(event.modifiers());
                app_context_clone.borrow_mut().key_down = Some(KeyEvent::new(code, mods))
            }),
        );

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::new(move |now| {
            let mut app_context = app_context.borrow_mut();
            app_context.now = now as _;
            // The canvas is pinned to the app's own logical size, so a window
            // resize does not move it. The one thing that does change the
            // drawing buffer is the pixel ratio, and it moves when the window
            // lands on another display, when the user zooms, and when a
            // debugger overrides the device metrics.
            //
            // This is read every frame rather than taken from a `resize`
            // event, because that event turns out not to be reliable for it:
            // changing only the ratio, with the viewport the same size, fires
            // `resize` in some cases and not others, and a drawing buffer left
            // at the old ratio is a canvas that is silently stretched or
            // letterboxed. Reading one property is far cheaper than the frame it
            // is read at the top of, and it cannot be missed.
            //
            // The buffer has to be rebuilt from scratch when it changes, since
            // setting `canvas.width` clears it - hence the `dirty`, without
            // which the app would sit on a blank canvas until something else
            // happened to repaint.
            let scale = device_pixel_ratio();
            if (app_context.screen.scale - scale).abs() > f32::EPSILON {
                app_context.screen.set_scale(scale);
                let (width, height) = (app_context.screen.width, app_context.screen.height);
                resize_canvas(&*canvas, width, height);
                app_context.dirty = true;
            }
            event_loop.frame(&mut app_context);
            request_animation_frame(f.borrow_mut().as_mut().unwrap());
        }));
        request_animation_frame(g.borrow_mut().as_mut().unwrap());
    }

    #[cfg(feature = "image")]
    fn load_image(&mut self, src: String, tx: Sender<BackendResponse>) {
        load_image(src, tx);
    }
}

/// Resizes the drawing buffer to the display's pixel ratio.
///
/// The css size is left alone: that is the logical size the app laid out
/// against, and the browser is what scales it to the panel. Only the drawing
/// buffer has to follow the ratio, because a canvas is a bitmap - at 2x a
/// 1920x1080 buffer shown across 1920x1080 css pixels would be an upscale, and
/// every edge, glyph and rounded corner in it would be half a device pixel wide.
fn resize_canvas(canvas: &impl Canvas, width: u32, height: u32) {
    let scale = device_pixel_ratio();
    let scaled = |value: u32| (value as f32 * scale).round().max(1.0) as u32;
    canvas.set_width(scaled(width));
    canvas.set_height(scaled(height));
}

pub fn load_image(src: String, tx: Sender<BackendResponse>) {
    bindings::load_image(src.clone(), move |(handle, width, height)| {
        let bytes = vec![];
        let _ = tx.send(BackendResponse::ImageLoaded(
            src.clone(),
            handle as _,
            bytes,
            width,
            height,
        ));
    });
}

pub fn reload() {
    let win = window();
    win.reload();
}
