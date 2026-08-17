use std::time::Duration;

use xs::{AnimationDirection::Alternate, Application, prelude::*};

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
                    .left(100.)
                    .top(50.)
                    .width(1105.)
                    .height(2105.)
                    .background(Color::rgba(0.1, 0.1, 0.1, 1.))
                    .with_animation(
                        "spring-forward",
                        //Spring::new(1500.).repeat(u32::MAX).direction(Alternate),
                        Easing::new()
                            .repeat(u32::MAX)
                            .direction(Alternate)
                            .duration(Duration::from_millis(1000)),
                        |this, delta| this.top(100. + delta * 500.),
                    )
                    .wrap()
                    //.gap(5.)
                    .padding(5.)
                    .children((0..50).map(|_| {
                        div()
                            .wrap()
                            .width(100.)
                            .height(100.)
                            .background(Color::RED)
                            .border_radius_all(5.)
                            .border(Color::WHITE, 2.)
                            .gap(5.)
                            .margin(5.)
                            .child(div().height(10.).background(Color::BLUE))
                            .child(div().height(10.).background(Color::GREEN))
                            .child(div().height(10.).background(Color::YELLOW))
                    })),
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
