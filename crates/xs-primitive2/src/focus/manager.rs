use std::rc::Rc;

use crate::{Element, Mount, div, render::Renderable};

struct FocusManager {
    child: Option<Rc<Element>>
}

impl Mount for FocusManager {
    fn mount() -> Self where Self: Sized {
        FocusManager { child: None }
    }

    fn update(&mut self, ctx: &mut xs_backend_core::AppContext) -> Element {
        if let Some(key_down) = ctx.on_key_down() {
            let root = self.child.clone().expect("FocusManager must have one child");
            match key_down {
                103 => {
                }
                108 => {

                }
        }
        div()
    }
}
