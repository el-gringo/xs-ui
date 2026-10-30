mod api;
mod body;
mod button;
mod card;
mod icons;
mod load;
mod sidebar;
mod splash;
mod theme;

use xs::{
    AlignItems, Application, Color, Context, Div, Easing, JustifyContent, NavigationNode, Overflow,
    ParentElement, Render, div, grid, text,
};

use crate::{
    api::peertube::Instance,
    body::body,
    card::card,
    load::{Loaded, Loader},
    sidebar::{COLLAPSE, sidebar},
    splash::{failed, splash},
};

const VIDEO_COUNT: u32 = 100;

struct Peertube;

impl Render for Peertube {
    fn render(&self, ctx: &mut Context) -> impl xs::IntoElement {
        let loaded = ctx.use_state(|_| Loaded::default());

        let _fetch = ctx.use_state(|_| Loader::new(loaded.clone()).load(VIDEO_COUNT));

        let root = ctx.focus_handle_with_key("root", NavigationNode::horizontal().id("root"));
        let sidebar_focus =
            ctx.focus_handle_with_key("sidebar", NavigationNode::vertical().id("sidebar"));
        ctx.nav_id = root.id;

        let data = loaded.read().clone();

        if let Some(error) = &data.error {
            return failed(error);
        }
        let Some(interface) = data.opened() else {
            return splash(ctx);
        };

        let slide = ctx.animate(
            Easing::new().ease_out().duration(COLLAPSE),
            if sidebar_focus.has_focus() { 0. } else { 1. },
        );

        div()
            .testid("app")
            .row()
            .justify_content(JustifyContent::FlexStart)
            .align_items(AlignItems::FlexStart)
            .background(theme::BACK_950)
            .child(
                sidebar(sidebar_focus.clone())
                    .categories(interface.categories)
                    .slide(slide),
            )
            .child(
                body().slide(slide).children([
                    title(interface.instance),
                    div()
                        .padding_top(48.)
                        .padding_left(72.)
                        .padding_right(72.)
                        .overflow(Overflow::Hidden)
                        .grow(1., 1.)
                        .justify_content(JustifyContent::Center)
                        .child(
                            grid()
                                .items(interface.videos)
                                .columns(5)
                                .rows(3)
                                .cell_width(320.)
                                .cell_height(320.)
                                .gap(12.)
                                .cell_renderer(|_index, video, _ctx| -> xs::AnyElement {
                                    card()
                                        .thumbnail(video.thumbnail_url())
                                        .title(video.name.clone())
                                        .into()
                                }),
                        ),
                ]),
            )
    }
}

fn title(instance: Instance) -> Div {
    div()
        .testid("body-title")
        .column()
        .height(NAME_HEIGHT + SUBTITLE_HEIGHT)
        .width(1140.)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .child(
            div()
                .height(NAME_HEIGHT)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center)
                .font_size(60)
                .color(Color::WHITE)
                .child(text(instance.name)),
        )
        .child(
            div()
                .height(SUBTITLE_HEIGHT)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center)
                .font_size(26)
                .color(theme::NEUTRAL_300)
                .child(text(instance.short_description)),
        )
}

const NAME_HEIGHT: f32 = 70.;
const SUBTITLE_HEIGHT: f32 = 36.;

pub fn main() {
    Application::new()
        .id("peertube")
        .title("Peertube")
        .run(|_ctx| Peertube);
}
