use xs::{AlignItems, Context, JustifyContent, NavigationNode, ParentElement, Render, div, text};

use crate::{
    button::{Button, button},
    theme,
};

pub struct Sidebar {
    buttons: Vec<Button>,
}

impl Sidebar {
    pub fn new() -> Self {
        Sidebar {
            buttons: Vec::new(),
        }
    }

    pub fn button(mut self, button: Button) -> Self {
        self.buttons.push(button);
        self
    }

    pub fn buttons(mut self, buttons: Vec<Button>) -> Self {
        self.buttons.extend(buttons);
        self
    }
}

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
                    .child(button("Movies"))
                    .child(button("Series"))
                    .child(button("Settings")),
            )
    }
}

pub fn sidebar() -> Sidebar {
    Sidebar::new()
}
