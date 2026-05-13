#![allow(dead_code)]

mod api;
mod body;
mod button;
mod card;
mod grid;
mod scroller;
mod sidebar;
mod theme;

use std::any::TypeId;

use xs::{
    AlignItems, Application, Color, Context, Element, Gradient, JustifyContent, NavigationNode,
    Overflow, Render, div, text,
};

use crate::{api::peertube::PeerTubeClient, body::body, card::card, grid::grid, sidebar::sidebar};

const PEERTUBE_API_URL: &str = "https://video.monsieurbidouille.fr/api/v1";

struct Peertube;
impl Render for Peertube {
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        ctx.focus_handle(NavigationNode::new().horizontal().id("root"));
        let videos = ctx.use_state(|_| Vec::new());
        let _ = ctx.use_state::<bool>(|_| {
            let set_videos = videos.clone();
            let client = PeerTubeClient::new();
            client.get_videos(0, 20, move |new_videos| {
                set_videos.update(|| new_videos);
            });
            true
        });

        div()
            .testid("app")
            .row()
            .justify_content(xs::JustifyContent::FlexStart)
            .align_items(AlignItems::FlexStart)
            .background(Gradient::angled(theme::ORANGE1, theme::ORANGE5, 0.5))
            .child(
                sidebar(), //.buttons([button(RED), button(GREEN), button(BLUE)].into())
            )
            .child(
                body().children([
                    div()
                        .height(80.)
                        .width(1140.)
                        .color(Color::WHITE)
                        .justify_content(JustifyContent::Center)
                        .align_items(AlignItems::Center)
                        .font_size(60)
                        .child(text("Monsieur Bidouille")),
                    div()
                        .width(1140.)
                        .padding_top(64.)
                        .overflow(Overflow::Hidden)
                        .child(
                            grid()
                                .items(videos.read().clone())
                                .columns(3)
                                .rows(4)
                                .cell_width(320.)
                                .cell_height(240.)
                                .gap(48.)
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
        .run(|_ctx| Peertube);
}
