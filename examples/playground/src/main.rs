use std::any::TypeId;

use xs::{Application, Color, Context, Element, Gradient, Render, div};

struct App;
impl Render for App {
    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        div()
            .testid("root")
            .width(ctx.app_context.screen.width as _)
            .height(ctx.app_context.screen.height as _)
            .column()
            .justify_content(xs::JustifyContent::Center)
            .align_items(xs::AlignItems::Center)
            .background(Color::GRAY)
            .child(
                div()
                    .testid("body")
                    .column()
                    .background(Gradient::linear(Color::RED, Color::BLUE))
                    .border(Color::WHITE, 4.)
                    .width(400.)
                    .height(500.)
                    .child(
                        div()
                            .background(Gradient::angled(Color::GREEN, Color::YELLOW, 1.57))
                            .width(200.)
                            .height(100.)
                            .margin_left(16.)
                            .margin_top(16.),
                    )
                    .child(
                        div()
                            .background(Gradient::linear(Color::ORANGE, Color::PURPLE))
                            .width(200.)
                            .height(100.)
                            .margin_left(16.)
                            .margin_top(16.),
                    )
                    .child(
                        div()
                            .background_color(Color::WHITE)
                            .width(200.)
                            .height(50.)
                            .margin_left(16.)
                            .margin_top(16.),
                    ),
            )
            .into()
    }
}

pub fn main() {
    Application::new()
        .id("playground")
        .title("Playground")
        .run(|_ctx| App);
}
