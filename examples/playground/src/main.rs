use std::any::TypeId;

use xs::{Application, Color, Context, Element, Render, Spring, animate, div};

struct App;
impl Render for App {
    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let target = if ((ctx.app_context.now as f32) % 2000.) < 1000. {
            600.
        } else {
            0.
        };
        let animated_offset = animate(ctx, Spring::new(0.0), target);

        div()
            .testid("root")
            .width(ctx.app_context.screen.width as _)
            .height(ctx.app_context.screen.height as _)
            .wrap()
            .child(
                div()
                    .absolute()
                    .left(100. + animated_offset)
                    .top(100.)
                    .width(100.)
                    .height(100.)
                    .background(Color::RED),
            )
            .into()
    }
}

pub fn main() {
    xs_log::init();
    xs_log::info!("started !");
    Application::new()
        .id("playground")
        .title("Playground")
        .run(|_ctx| App);
}
