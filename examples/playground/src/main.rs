use xs::{
    AnimationDirection::Alternate, Application, Color, Context, ParentElement, Render, Spring, div,
};

struct App;
impl Render for App {
    fn render(&self, ctx: &mut Context) -> impl xs::IntoElement {
        let state = ctx.use_state(|_, _| 600.);
        let value = *state.read();

        div()
            .testid("root")
            .width(ctx.app_context.screen.width as _)
            .height(ctx.app_context.screen.height as _)
            .background_color(Color::GRAY)
            .child(
                div()
                    .absolute()
                    .top(100.)
                    .left(100.)
                    .width(100.)
                    .height(100.)
                    .background(Color::RED)
                    .with_animation(
                        "box1",
                        Spring::new(0.0).repeat().direction(Alternate),
                        value,
                        |this, delta| {
                            eprintln!("delta box1: {delta}");
                            this.left(100. + delta)
                        },
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top(250.)
                    .left(700.)
                    .width(100.)
                    .height(100.)
                    .background(Color::BLUE)
                    .with_animation(
                        "box2",
                        Spring::new(700.0).repeat().direction(Alternate),
                        100.,
                        |this, delta| {
                            eprintln!("delta box2: {delta}");
                            this.left(delta)
                        },
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
