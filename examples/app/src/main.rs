#![allow(dead_code)]

mod body;
mod button;
mod card;
mod sidebar;
mod theme;

use std::{any::TypeId, rc::Rc};

use xs::{
    AppContext, Application, BLUE, Element, FocusHandle, GREEN, NavigationNode, RED, Render, div,
};

use crate::{body::body, button::button, card::card, sidebar::sidebar, theme::GRAY3};

static IMAGE1: &str = "https://video.monsieurbidouille.fr/lazy-static/thumbnails/fabcf1f1-5ef0-4551-9a3b-a6ff953bbcd9.jpg";
static IMAGE2: &str = "https://video.monsieurbidouille.fr/lazy-static/thumbnails/440cf8a3-1389-4475-9b65-6e1dba693235.jpg";
static IMAGE3: &str = "https://video.monsieurbidouille.fr/lazy-static/thumbnails/4a8eb065-deca-47c7-a574-048a63e6b668.jpg";

struct MyApp;
impl Render for MyApp {
    fn render(&self, ctx: &mut AppContext) -> Box<dyn Element> {
        let _ = ctx.use_state::<FocusHandle>(|ctx| {
            ctx.nav
                .create_handle(NavigationNode::new("app").horizontal())
        });
        div()
            .id("app")
            .row()
            .child(
                sidebar()
                    .buttons([button(RED), button(GREEN), button(BLUE)].into())
                    .into(),
            )
            .child(
                body()
                    .children([Rc::new(
                        div()
                            .row()
                            .padding_top(64.)
                            //.justify_content(xs::MainAxisAlignment::Center)
                            .child(card().id("card1").thumbnail(IMAGE1).into())
                            .child(card().id("card2").thumbnail(IMAGE2).into())
                            .child(card().id("card3").thumbnail(IMAGE3).into())
                            .into(),
                    )])
                    .into(),
            )
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}

pub fn main() {
    Application::new()
        .id("app")
        .title("App")
        .width(1440)
        .height(810)
        .run(Box::new(MyApp {}));
}
