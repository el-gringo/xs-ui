use std::any::TypeId;

use xs::{
    Application, BLACK, BLUE, Color, Context, Div, Element, GRAY, GREEN, ORANGE, RED, Render,
    YELLOW, div,
};

fn tile(color: Color) -> Div {
    div()
        .width(50.)
        .height(50.)
        .background_color(color)
        .border(BLUE, 8.)
}

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
                    .background_color(BLACK)
                    .border(RED, 8.)
                    .width(200.)
                    .height(300.)
                    .child(tile(RED))
                    .child(tile(GREEN))
                    .child(tile(BLUE))
                    .child(tile(YELLOW))
                    .child(tile(ORANGE).margin_left(16.).grow(0., 1.)),
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
