use xs::{AlignItems, Context, JustifyContent, NavigationNode, ParentElement, Render, div, text};

use crate::{button::button, theme};

pub struct Sidebar;

impl Render for Sidebar {
    fn render(&self, ctx: &mut Context) -> impl xs::IntoElement {
        ctx.focus_handle(NavigationNode::new().vertical().id("sidebar"));

        div()
            .testid("sidebar")
            .column()
            .width(300.)
            .justify_content(xs::JustifyContent::FlexStart)
            .background_color(theme::BACK_900)
            .child(
                div()
                    .testid("sidebar-header")
                    .row()
                    .height(75.)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .color(theme::NEUTRAL_300)
                    .font_size(40)
                    .child(text("PeerTube")),
            )
            .child(
                div()
                    .testid("sidebar-body")
                    .column()
                    .align_items(AlignItems::Center)
                    .padding_top(48.)
                    .width(300.)
                    .gap(16.)
                    .grow(1., 1.)
                    .child(button("Movies").id("button-movies"))
                    .child(button("Series").id("button-shows"))
                    .child(button("Settings").id("button-settings")),
            )
    }
}

pub fn sidebar() -> Sidebar {
    Sidebar
}
