use xs::{Color, Context, NavigationNode, ParentElement, Render, div, text};

use crate::theme;

pub struct Button {
    text: &'static str,
}

impl Button {
    pub fn new(text: &'static str) -> Self {
        Button { text }
    }
}

impl Render for Button {
    fn render(&self, ctx: &mut Context) -> impl xs::IntoElement {
        let nav_node = ctx.focus_handle(NavigationNode::new().id("button"));
        div()
            .testid("button")
            .justify_content(xs::JustifyContent::Center)
            .align_items(xs::AlignItems::Center)
            .width(200.)
            .height(50.)
            .margin_top(16.)
            .background_color(theme::BACK_900)
            .color(theme::NEUTRAL_300)
            .border_left(Color::TRANSPARENT, 4.)
            .when(nav_node.has_focus(), |this| {
                this.color(theme::ACCENT_400)
                    .border_left(theme::ACCENT_400, 4.)
            })
            .font_size(32)
            .child(text(self.text))
    }
}

pub fn button(text: &'static str) -> Button {
    Button::new(text)
}
