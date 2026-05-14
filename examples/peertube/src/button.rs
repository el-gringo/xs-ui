use std::{any::TypeId, rc::Rc};

use xs::{Color, Context, Element, NavigationNode, Render, div, text};

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
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let nav_node = ctx.focus_handle(NavigationNode::new().id("button"));
        div()
            .testid("button")
            .justify_content(xs::JustifyContent::Center)
            .align_items(xs::AlignItems::Center)
            .width(200.)
            .height(50.)
            .margin_top(16.)
            .border_radius_all(8.)
            .background_color(theme::ORANGE4)
            .color(Color::BLACK)
            .border(
                if nav_node.has_focus() {
                    theme::BRIGHT_RED
                } else {
                    Color::TRANSPARENT
                },
                4.,
            )
            .font_size(40)
            .child(text(self.text))
            .into()
    }
    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}

impl From<Button> for Rc<dyn Render> {
    fn from(value: Button) -> Self {
        Rc::new(value)
    }
}

pub fn button(text: &'static str) -> Button {
    Button::new(text)
}
