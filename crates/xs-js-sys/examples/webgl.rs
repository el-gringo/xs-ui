use std::{cell::RefCell, rc::Rc};

use xs_gl_sys::{GL_COLOR_BUFFER_BIT, glClear, glClearColor, glViewport};
use xs_js_sys::{Canvas, Closure, Element, Node, document, log, request_animation_frame};

pub fn main() {
    log("webgl example: starting");

    // The page has no canvas, the app creates the one it needs.
    let doc = document();
    let canvas = doc.create_element("canvas");
    // Getting the context also attaches it to the app, the gl bindings read
    // `app.gl` from then on.
    let context = canvas.get_context("webgl2");
    canvas.set_attribute("id", "glcanvas");
    canvas.set_attribute("tabindex", "1");
    doc.query_selector("body").append_child(&canvas);

    // One drawing buffer pixel per css pixel, whatever the window is.
    let width = canvas.offset_width();
    let height = canvas.offset_height();
    canvas.set_width(width);
    canvas.set_height(height);
    canvas.focus();
    log(&format!("canvas {width}x{height}"));

    log(&format!("webgl2 context: {}", context.idx));

    // The drawing buffer is dropped after every compositing pass, so the clear
    // has to be repeated on each frame to stay on screen.
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::new(move |_now: u32| {
        glViewport(0, 0, width as i32, height as i32);
        glClearColor(0.1, 0.35, 0.6, 1.0);
        glClear(GL_COLOR_BUFFER_BIT);
        request_animation_frame(f.borrow_mut().as_mut().unwrap());
    }));
    request_animation_frame(g.borrow_mut().as_mut().unwrap());
}
