mod api;
mod body;
mod button;
mod card;
mod sidebar;
mod theme;

use xs::{
    AlignItems, Application, Color, Context, JustifyContent, NavigationNode, Overflow,
    ParentElement, Render, div, grid, text,
};

use crate::{api::peertube::PeerTubeClient, body::body, card::card, sidebar::sidebar};

struct Peertube;
impl Render for Peertube {
    fn render(&self, ctx: &mut Context) -> impl xs::IntoElement {
        ctx.focus_handle(NavigationNode::new().horizontal().id("root"));
        let videos = ctx.use_state(|_, set_videos| {
            let client = PeerTubeClient::new();
            client.get_videos(0, 100, move |new_videos| {
                set_videos.update(|| new_videos);
            });
            Vec::new()
        });

        let count = videos.read().clone().len();
        div()
            .testid("app")
            .row()
            .justify_content(xs::JustifyContent::FlexStart)
            .align_items(AlignItems::FlexStart)
            .background(theme::BACK_950)
            .child(sidebar())
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
                        .when(count == 0, |this| {
                            this.child(
                                div()
                                    .font_size(32)
                                    .color(Color::WHITE)
                                    .justify_content(JustifyContent::Center)
                                    .child(text("LOADING...")),
                            )
                        })
                        .when(count > 0, |this| {
                            this.child(
                                grid()
                                    .items(videos.read().clone())
                                    .columns(3)
                                    .rows(3)
                                    .cell_width(320.)
                                    .cell_height(320.)
                                    .gap(48.)
                                    .cell_renderer(|index, video, _ctx| -> xs::AnyElement {
                                        card()
                                            .id(format!("card-{index}").leak())
                                            .thumbnail(video.thumbnail_url())
                                            .title(video.name.clone())
                                            .into()
                                    }),
                            )
                        }),
                ]),
            )
    }
}

pub fn main() {
    Application::new()
        .id("peertube")
        .title("Peertube")
        .run(|_ctx| Peertube);
}
