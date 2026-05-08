#![allow(dead_code)]

mod api;
mod body;
mod button;
mod card;
mod grid;
mod sidebar;
mod theme;

use std::any::TypeId;

use xs::{
    AlignItems, Application, Color, Context, Element, FocusHandle, JustifyContent, NavigationNode,
    Render, div, text,
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
            .testid("app")
            .row()
            .justify_content(xs::JustifyContent::FlexStart)
            .align_items(AlignItems::FlexStart)
            .child(
                sidebar(), //.buttons([button(RED), button(GREEN), button(BLUE)].into())
            )
            .child(
                body().children([
                    div()
                        .height(80.)
                        .width(1140.)
                        .background_color(GRAY4)
                        .color(Color::WHITE)
                        .justify_content(JustifyContent::Center)
                        .align_items(AlignItems::Center)
                        .font_size(60)
                        .child(text("Monsieur Bidouille")),
                    div()
                        .row()
                        .width(1140.)
                        .padding_top(64.)
                        .gap(32.)
                        .justify_content(JustifyContent::Center)
                        .child(
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
            )
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}

pub fn main() {
    Application::new()
        .id("peertube")
        .title("Peertube")
        .run(|_ctx| MyApp);
}
