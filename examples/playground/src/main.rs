use std::any::TypeId;

use xs::{
    Application, BLACK, BLUE, BorderRadius, BoxShadow, Context, Element, GRAY, GREEN, ORANGE, RED,
    Render, WHITE, YELLOW, div, text,
};

struct Counter;
impl Render for Counter {
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
            .background_color(GRAY)
            .child(
                div()
                    .testid("body")
                    .column()
                    .background_color(RED)
                    .width(200.)
                    .height(300.)
                    .child(
                        div()
                            .testid("button")
                            .column()
                            .justify_content(xs::JustifyContent::Center)
                            .align_items(xs::AlignItems::Center)
                            .background_color(BLUE)
                            .color(WHITE)
                            .height(100.)
                            .font_size(60)
                            .margin_top(30.)
                            .border_radius(20., 20., 20., 0.)
                            .box_shadow(10., 10., 0., 10., BLACK)
                            .child(
                                //text("text")
                                div()
                                    .margin_top(16.)
                                    .height(50.)
                                    .justify_content(xs::JustifyContent::Center)
                                    .align_items(xs::AlignItems::Center)
                                    .child(text("text")),
                            ),
                    ),
            )
            .into()
    }
}

pub fn main() {
    Application::new()
        .id("playground")
        .title("Playground")
        .width(1440)
        .height(810)
        .run(|_ctx| Counter);
}
