use std::time::Duration;

use xs::{AnimationDirection::Alternate, Application, KeyCode, NavigationNode, prelude::*};

use crate::CounterState::{Decrement, Increment};

#[derive(PartialEq, Clone, Copy)]
enum CounterState {
    None,
    Increment,
    Decrement,
}

struct Counter;
impl Render for Counter {
    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        let state = ctx.use_state(|_| CounterState::None);
        let state_value = ctx.use_state(|_| 0);
        let mut handle = ctx.focus_handle(NavigationNode::new().vertical().id("counter"));

        let s = state.clone();
        let v = state_value.clone();
        handle.on_key_down(move |key_code| {
            let v = v.clone();
            let s = s.clone();
            let mut value = v.get();
            match key_code {
                KeyCode::Up => {
                    value += 1;
                    s.set(Increment);


                }
                KeyCode::Down => {
                    value -= 1;
                    s.set(Decrement);


                }
                _ => {}
            };
            v.set(value);
            false
        });
        let value = state_value.get();
        let state = state.get();
        div()
            .testid("root")
            .width(ctx.app_context.screen.width as _)
            .height(ctx.app_context.screen.height as _)
            .column()
            .justify_content(xs::JustifyContent::Center)
            .align_items(xs::AlignItems::Center)
            .background_color(Color::GRAY)
            .child(
                div()
                    .testid("body")
                    .column()
                    .width(200.)
                    .height(300.)
                    .child(
                        div()
                            .testid("increment")
                            .justify_content(xs::JustifyContent::Center)
                            .align_items(xs::AlignItems::Center)
                            .background_color(Color::BLACK)
                            .border_radius(16., 16., 0., 0.)
                            .color(Color::WHITE)
                            .height(75.)
                            .font_size(36)
                            .with_animation(
                                "increment",
                                Easing::new().duration(Duration::from_millis(if Increment == state {
                                    100
                                } else {
                                    0
                                }))
                                .repeat(1)
                                .direction(Alternate),
                                |this, delta| {
                                    this.background_color(Color::rgba(delta, delta, delta, 1.0))
                                },
                            )
                            .child(text("Inc")),
                    )
                    .child(
                        div()
                            .testid("inner")
                            .justify_content(xs::JustifyContent::Center)
                            .align_items(xs::AlignItems::Center)
                            .background_color(Color::WHITE)
                            .color(Color::BLACK)
                            .height(200.)
                            .font_size(64)
                            .child(text(value.to_string())),
                    )
                    .child(
                        div()
                            .testid("decrement")
                            .justify_content(xs::JustifyContent::Center)
                            .align_items(xs::AlignItems::Center)
                            .background_color(Color::BLACK)
                            .border_radius(0., 0., 16., 16.)
                            .color(Color::WHITE)
                            .height(75.)
                            .font_size(36)
                            .child(text("Dec"))
                            .with_animation(
                                "increment",
                                Easing::new().duration(Duration::from_millis(if Decrement == state {
                                    100
                                } else {
                                    0
                                }))
                                .repeat(1)
                                .direction(Alternate),
                                |this, delta| {
                                    this.background_color(Color::rgba(delta, delta, delta, 1.0))
                                },
                            ),
                    ),
            )
    }
}

pub fn main() {
    Application::new()
        .id("counter")
        .title("Counter")
        .run(|_ctx| Counter);
}
