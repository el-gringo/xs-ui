use xs::{
    Application, Color, Context, IntoElement, KeyCode, NavigationNode, ParentElement, Render, div,
    text,
};

struct Counter;
impl Render for Counter {
    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        let state = ctx.use_state(|_, _| 0);
        let mut handle = ctx.focus_handle(NavigationNode::new().vertical().id("counter"));
        let s = state.clone();
        handle.on_key_down(move |key_code| {
            let state = s.clone();
            let mut value = *state.read();
            match key_code {
                KeyCode::Up => {
                    value += 1;
                }
                KeyCode::Down => {
                    value -= 1;
                }
                _ => {}
            };
            state.update(|| value);
            false
        });
        let value = state.read().to_string();
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
                            .child(text(value)),
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
                            .child(text("Dec")),
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
