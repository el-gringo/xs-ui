use std::rc::Rc;

use xs_js_sys::{Closure, EventEmitter, EventKeyDown, EventResize, Node, document, log, window};

pub fn main() {
    let window = window();
    let doc = document();
    let canvas = Rc::new(doc.query_selector("canvas"));
    let canvas_clone = canvas.clone();
    window.add_event_listener(
        "resize",
        Closure::new(move |_event: EventResize| {
            let canvas_clone = canvas_clone.clone();
            xs_js_sys::log(&format!(
                "canvas resized: {}x{}",
                canvas_clone.offset_width(),
                canvas_clone.offset_height()
            ))
        }),
    );
    canvas.clone().add_event_listener(
        "keydown",
        Closure::new(|event: EventKeyDown| {
            log(&format!(
                "canvas keydown: key_code={}, key={}, shift={}, ctrl={}, alt={}, meta={}",
                event.key_code(),
                event.key(),
                event.shift(),
                event.ctrl(),
                event.alt(),
                event.meta()
            ));
        }),
    );
}
