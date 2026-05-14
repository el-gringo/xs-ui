use std::any::TypeId;

use xs::{Application, Color, Context, Div, Element, Render, div};

fn row() -> Div {
    div().absolute().grow(1.0, 0.).height(100.)
}

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
            .child(row().testid("row1").top(100.).background(Color::RED))
            .child(row().testid("row2").top(210.).background(Color::GREEN))
            .child(row().testid("row3").top(320.).background(Color::BLUE))
            .child(row().testid("row4").top(320.).background(Color::ORANGE))
            .child(row().testid("row5").top(320.).background(Color::YELLOW))
            .into()
    }
}

pub fn main() {
    xs_log::init();
    log::info!("started !");
    Application::new()
        .id("playground")
        .title("Playground")
        .run(|_ctx| App);
}
