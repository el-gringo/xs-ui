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
    AlignItems, Application, Context, Element, FocusHandle, NavigationNode, ORANGE, Render, div,
    run,
};

use crate::{body::body, card::card, grid::grid, sidebar::sidebar, theme::GRAY4};

struct MyApp;
impl Render for MyApp {
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let (_, videos) = ctx.use_state::<(FocusHandle, Vec<api::peertube::Video>)>(|ctx| {
            let focus_handle = ctx.focus_handle(NavigationNode::new("app").horizontal());
            let videos = api::peertube::PeerTubeClient::get_videos();
            (focus_handle, videos)
        });
        div()
            .id("app")
            .row()
            // .width(ctx.app.width as f32)
            // .height(ctx.app.height as f32)
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
                                    .child(
                                        grid()
                                            .items(videos.to_vec())
                                            .columns(3)
                                            .cell_width(320.)
                                            .cell_height(240.)
                                            .gap(16.)
                                            .cell_renderer(|_index, video| {
                                                card()
                                                    .thumbnail(
                                                        video.thumbnail_url().unwrap_or_default(),
                                                    )
                                                    .title(video.name.clone())
                                                    .into()
                                            })
                                            .into(),
                                    )
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
