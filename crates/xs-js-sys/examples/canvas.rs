use xs_js_sys::{Canvas, Closure, Document, EventEmitter, EventResize, document, window};

pub fn main() {
    let window = window();
    let doc = document();
    let canvas = doc.query_selector("canvas");
    window.add_event_listener(
        "resize",
        Closure::new(move |_event: EventResize| {
            xs_js_sys::log(&format!(
                "canvas resized: {}x{}",
                canvas.width(),
                canvas.height()
            ))
        }),
    );
}
