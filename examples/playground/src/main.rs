use xs::{Application, Color, Context, ParentElement, Render, Spring, div};

struct App;
impl Render for App {
    fn render(&self, ctx: &mut Context) -> impl xs::IntoElement {
        let target = if ((ctx.app_context.now as f32) % 2000.) < 1000. {
            600.
        } else {
            0.
        };
        let animated_offset = ctx.animate(Spring::new(0.0), target);

        div()
            .testid("root")
            .width(ctx.app_context.screen.width as _)
            .height(ctx.app_context.screen.height as _)
            .background_color(Color::GRAY)
            .child(
                div()
                    .absolute()
                    .left(100. + animated_offset)
                    .top(100.)
                    .width(100.)
                    .height(100.)
                    .background(Color::RED),
            )
            .child(
                div()
                    .absolute()
                    .left(700. - animated_offset)
                    .top(250.)
                    .width(100.)
                    .height(100.)
                    .background(Color::BLUE),
            )
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
