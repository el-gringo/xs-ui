use super::*;
use xs_backend::AppContext;
use xs_layout::{Color, colors::{BLUE, GRAY, GREEN, RED, WHITE}};
use xs_primitive2::{Element, Mount, div};

struct App {
    selected_index: i32,
}

impl Mount for App {
    fn mount() -> App {
        App { selected_index: 0 }
    }

    fn update(&mut self, ctx: &mut AppContext) -> Element {
        if let Some(key_down) = ctx.on_key_down() {
            match key_down {
                103 => {
                    self.selected_index = if self.selected_index > 0 {
                        self.selected_index - 1
                    } else {
                        2
                    } % 3;
                }
                108 => {
                    self.selected_index = (self.selected_index + 1) % 3;
                }
                _ => {}
            }
        }
        div()
            .width(300.)
            .column()
            .background_color(Color::rgba(0.128, 0.128, 0.128, 0.128))
            .child(button(RED, self.selected_index == 0))
            .child(button(GREEN, self.selected_index == 1))
            .child(button(BLUE, self.selected_index == 2))
    }
}

pub fn button(color: Color, selected: bool) -> Element {
    div()
        .width(200.)
        .heigth(40.)
        .margin_top(16.)
        .margin_left(16.)
        .background_color(color)
        .border(if selected { WHITE } else { GRAY }, 2.)
}

pub fn main() {
    let mut backend = Backend::new(1280, 720);

    backend.run(|_ctx| {
        let root = App::mount();
        Box::new(root)
    });
}
