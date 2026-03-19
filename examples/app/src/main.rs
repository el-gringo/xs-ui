#![allow(dead_code)]

mod api;
mod body;
mod button;
mod card;
mod grid;
mod sidebar;
mod theme;

use std::{any::TypeId, rc::Rc};

use xs::{
    AlignItems, Application, Context, Element, FocusHandle, NavigationNode, ORANGE, RED, Render,
    WHITE, div, run, text,
};

use crate::{
    api::peertube::Video, body::body, card::card, grid::grid, sidebar::sidebar, theme::GRAY4,
};

const PEERTUBE_API_URL: &str = "https://video.monsieurbidouille.fr/api/v1";

struct MyApp;
impl Render for MyApp {
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let _ = ctx.use_state::<FocusHandle>(|ctx, _| {
            ctx.focus_handle(NavigationNode::new("app").horizontal())
        });
        let videos = ctx.use_state::<Vec<Video>>(|_ctx, mutator| {
            let videos = Vec::new();
            let client = api::peertube::PeerTubeClient::new();
            client.get_videos(0, 50, mutator);
            videos
        });
        div()
            .id("app")
            .row()
            .justify_content(xs::JustifyContent::FlexStart)
            .align_items(AlignItems::FlexStart)
            .child(
                sidebar(), //.buttons([button(RED), button(GREEN), button(BLUE)].into())
            )
            .child(
                body().children([
                    div().height(50.).background_color(GRAY4),
                    div().background_color(ORANGE).column().children([
                        div().height(50.).color(WHITE).text("Monsieur Bidouille"),
                        div().row().padding_top(64.).gap(32.).child(
                            grid()
                                .items(videos.clone())
                                .columns(3)
                                .cell_width(320.)
                                .cell_height(240.)
                                .gap(16.)
                                .cell_renderer(|_index, video| {
                                    card()
                                        .thumbnail(video.thumbnail_url())
                                        .title(video.name.clone())
                                        .into()
                                }),
                        ),
                    ]),
                ]),
            )
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}

struct MyApp2;
impl Render for MyApp2 {
    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
    fn render(&self, _ctx: &mut Context) -> Box<dyn Element> {
        div()
            .background_color(RED)
            .child(
                div()
                    .absolute()
                    .justify_content(xs::JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .children([
                        div()
                            .width(100.)
                            .height(50.)
                            .background_color(WHITE)
                            .font_size(45)
                            .child(text("TEST")),
                        div()
                            .width(100.)
                            .height(50.)
                            .background_color(WHITE)
                            .font_size(30)
                            .child(text("TEST")),
                    ]),
            )
            .into()
    }
}

pub fn main() {
    run(
        Application::new()
            .id("app")
            .title("App")
            .width(1440)
            .height(810),
        Rc::new(MyApp {}),
    );
}
