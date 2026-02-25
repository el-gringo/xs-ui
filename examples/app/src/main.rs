#![allow(dead_code)]

mod body;
mod button;
mod card;
mod grid;
mod sidebar;
mod theme;

use std::any::TypeId;

use xs::{
    AlignItems, Application, Element, FocusHandle, NavigationNode, ORANGE, Render, div, run
};

use crate::{
    body::body,
    card::card,
    sidebar::sidebar,
    theme::GRAY4,
};

static IMAGE1: &str = "https://video.monsieurbidouille.fr/lazy-static/thumbnails/812ca374-a473-4cb8-ae0b-eebdd8a7ccfe.jpg";
static IMAGE2: &str = "https://video.monsieurbidouille.fr/lazy-static/thumbnails/440cf8a3-1389-4475-9b65-6e1dba693235.jpg";
static IMAGE3: &str = "https://video.monsieurbidouille.fr/lazy-static/thumbnails/4a8eb065-deca-47c7-a574-048a63e6b668.jpg";

struct MyApp;
impl Render for MyApp {
    fn render(&self, app: &mut Application) -> Box<dyn Element> {
        let _ = app.ctx.use_state::<FocusHandle>(|ctx| {
            ctx.nav
                .create_handle(NavigationNode::new("app").horizontal())
        });
        div()
            .id("app")
            .row()
            .width(app.width as f32)
            .height(app.height as f32)
            .justify_content(xs::JustifyContent::FlexStart)
            .align_items(AlignItems::FlexStart)
            //.background_color(YELLOW)
            .child(
                sidebar()
                    //.buttons([button(RED), button(GREEN), button(BLUE)].into())
                    .into(),
            )
            .child(
                body()
                    .children([
                        div().height(50.).background_color(GRAY4).into(),
                        div()
                            .background_color(ORANGE)
                            .column()
                            .children([
                                // div()
                                //     .height(50.)
                                //     .text("PeerTube")
                                //     .into(),
                                div()
                                    .row()
                                    .padding_top(64.)
                                    .gap(32.)
                                    .child(card().id("card1").thumbnail(IMAGE1).into())
                                    .child(card().id("card2").thumbnail(IMAGE2).into())
                                    .child(card().id("card3").thumbnail(IMAGE3).into())
                                    .into(),
                            ])
                            .into(),
                    ])
                    .into(),
            )
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}

pub fn main() {
    run(
        Application::new()
            .id("app")
            .title("App")
            .width(1440)
            .height(810),
        Box::new(MyApp {}),
    );
}
