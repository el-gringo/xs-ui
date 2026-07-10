mod api;
mod body;
mod button;
mod card;
mod sidebar;
mod theme;

use std::any::TypeId;

use xs::{
    AlignItems, Application, Color, Context, Element, JustifyContent, NavigationNode, Overflow,
    Render, div, grid, text,
};

use crate::{
    api::peertube::{PeerTubeClient, Video},
    body::body,
    card::card,
    sidebar::sidebar,
};

struct Peertube;
impl Render for Peertube {
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        ctx.focus_handle(NavigationNode::new().horizontal().id("root"));
        let videos = ctx.use_state(|_, set_videos| {
            let client = PeerTubeClient::new();
            client.get_videos(0, 100, move |new_videos| {
                set_videos.update(|| new_videos);
            });
            Vec::new()
        });

        div()
            .testid("app")
            .row()
            .justify_content(xs::JustifyContent::FlexStart)
            .align_items(AlignItems::FlexStart)
            .background(theme::BACK_950)
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
                        .padding_top(48.)
                        .overflow(Overflow::Hidden)
                        .width(1200.)
                        .justify_content(JustifyContent::Center)
                        .child(
                            grid()
                                .items(videos.read().clone())
                                .columns(3)
                                .rows(3)
                                .cell_width(320.)
                                .cell_height(320.)
                                .gap(48.)
                                .cell_renderer(|_index, video| {
                                    //eprintln!("{}", video.name.clone());
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
