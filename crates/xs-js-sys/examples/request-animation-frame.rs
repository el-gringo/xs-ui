use std::{cell::RefCell, rc::Rc};

use xs_js_sys::{Closure, log, request_animation_frame};

pub fn main() {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::new(move |now| {
        log(&format!("frame: {now}"));
        request_animation_frame(f.borrow_mut().as_mut().unwrap());
    }));
    request_animation_frame(g.borrow_mut().as_mut().unwrap());
}
