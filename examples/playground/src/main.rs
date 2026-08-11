use std::time::Duration;

use xs::{
    AnimationDirection::Alternate, Application, Color, Context, Easing, ParentElement, Render,
    Spring, div,
};

struct App;
impl Render for App {
    fn render(&self, ctx: &mut Context) -> impl xs::IntoElement {
        div()
            .testid("root")
            .width(ctx.app_context.screen.width as _)
            .height(ctx.app_context.screen.height as _)
            .background_color(Color::GRAY)
            .child(
                div()
                    .absolute()
                    .top(50.)
                    .width(100.)
                    .height(100.)
                    .background(Color::RED)
                    .with_animation(
                        "spring-forward",
                        Spring::new(600.).repeat(u32::MAX).direction(Alternate),
                        |this, delta| this.left(100. + delta),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top(200.)
                    .width(100.)
                    .height(100.)
                    .background(Color::BLUE)
                    .with_animation(
                        "spring-backward",
                        Spring::new(600.).repeat(u32::MAX).direction(Alternate),
                        |this, delta| {
                            eprintln!("delta box2: {delta}");
                            this.left(700. - delta)
                        },
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top(350.)
                    .width(100.)
                    .height(100.)
                    .background(Color::GREEN)
                    .with_animation(
                        "easing-forward",
                        Easing::new(Duration::from_millis(600))
                            .repeat(u32::MAX)
                            .direction(Alternate),
                        |this, delta| this.left(100. + delta * 600.),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top(500.)
                    .width(100.)
                    .height(100.)
                    .background(Color::YELLOW)
                    .with_animation(
                        "easing-backward",
                        Easing::new(Duration::from_millis(600))
                            .repeat(u32::MAX)
                            .direction(Alternate),
                        |this, delta| this.left(700. - delta * 600.),
                    ),
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
