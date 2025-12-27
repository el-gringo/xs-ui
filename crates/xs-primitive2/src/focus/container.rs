use crate::{Mount, div, focus::focusable::Focusable};

enum FocusableContainerLayout {
    Horizontal,
    Verical,
}

pub struct FocusableContainer {
    layout: FocusableContainerLayout,
    selected_index: i32,
    children: Vec<FocusableContainer>,
    is_in_focus_path: bool
}

use FocusableContainerLayout::*;

impl Mount for FocusableContainer {
    fn mount() -> Self
    where
        Self: Sized,
    {
        FocusableContainer {
            layout: Horizontal,
            selected_index: 0,
            children: Vec::new(),
            is_in_focus_path: true
        }
    }

    fn update(&mut self, ctx: &mut xs_backend_core::AppContext) -> crate::Element {
        if let Some(key_down) = ctx.on_key_down() {
            match self.layout {
                Verical => match key_down {
                    103 => {
                        self.selected_index = if self.selected_index > 0 {
                            self.selected_index - 1
                        } else {
                            2
                        } % self.children.len() as i32;
                    }
                    108 => {
                        self.selected_index = (self.selected_index + 1) % 3;
                    },
                    _ => {}
                },
                Horizontal => {}
            }
        }

        match self.layout {
            Horizontal => {
                div()
                    .row()
                    .grow()
            }
            Verical => {
                div()
                    .column()
                    .grow()
            }
        }
    }
}

impl Focusable for FocusableContainer {
    fn on_focus<F>(&mut self, callback: F) where F: Fn() {
        self.is_in_focus_path = true;
        callback();
    }

    fn on_blur<F>(&mut self, callback: F) where F: Fn() {
        self.is_in_focus_path = false;
        callback();
    }
}
