use std::any::TypeId;

use xs::{
    Application, BLACK, BLUE, Context, Element, GRAY, GREEN, ORANGE, RED, Render, WHITE, YELLOW,
    div, text,
};

struct MyApp;
impl Render for MyApp {
    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        div()
            .id("root")
            .width(ctx.app_context.screen.width as _)
            .height(ctx.app_context.screen.height as _)
            .column()
            .justify_content(xs::JustifyContent::Center)
            .align_items(xs::AlignItems::Center)
            .background_color(GRAY)
            .child(
                div()
                    .id("body")
                    .column()
                    .background_color(RED)
                    .width(200.)
                    .height(300.)
                    .child(
                        div()
                            .id("increment")
                            .justify_content(xs::JustifyContent::Center)
                            .align_items(xs::AlignItems::Center)
                            .background_color(BLACK)
                            .color(WHITE)
                            .height(50.)
                            .font_size(36)
                            .child(text("Inc")),
                    )
                    .child(
                        div()
                            .id("inner")
                            .justify_content(xs::JustifyContent::Center)
                            .align_items(xs::AlignItems::Center)
                            .background_color(WHITE)
                            .color(BLACK)
                            .height(200.)
                            .font_size(64)
                            .child(text("0")),
                    )
                    .child(
                        div()
                            .id("decrement")
                            .justify_content(xs::JustifyContent::Center)
                            .align_items(xs::AlignItems::Center)
                            .background_color(BLACK)
                            .color(WHITE)
                            .height(50.)
                            .font_size(36)
                            .child(text("Dec")),
                    ),
            )
            .into()
    }
}

pub fn main() {
    Application::new()
        .id("app")
        .title("App")
        .width(1440)
        .height(810)
        .run(|_ctx| MyApp);
}
