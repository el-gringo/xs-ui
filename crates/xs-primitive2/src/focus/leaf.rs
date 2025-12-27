use crate::{Mount, div};

struct FocusableLeaf {
    has_focus: bool
}

impl FocusableLeaf {
    fn on_focus<F>(&mut self, callback: F) where F: Fn() {
        self.has_focus = true;
        callback();
    }

    fn on_blur<F>(&mut self, callback: F) where F: Fn() {
        self.has_focus = false;
        callback();
    }
}

impl Mount for FocusableLeaf {
    fn mount() -> Self
    where
        Self: Sized,
    {
    }

    fn update(&mut self, ctx: &mut xs_backend_core::AppContext) -> crate::Element {
        div()
    }
}
