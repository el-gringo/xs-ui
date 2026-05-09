use std::{any::TypeId, rc::Rc};

use xs::{AlignItems, Color, Context, Element, JustifyContent, NavigationNode, Render, div, text};

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
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        ctx.focus_handle(NavigationNode::new().vertical().id("sidebar"));

        div()
            .testid("sidebar")
            .column()
            .width(300.)
            .justify_content(xs::JustifyContent::FlexStart)
            .child(
                div()
                    .testid("sidebar-header")
                    .row()
                    .height(50.)
                    .background_color(theme::GRAY3)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .color(Color::WHITE)
                    .font_size(40)
                    .child(text("PeerTube")),
            )
            .child(
                div()
                    .testid("sidebar-body")
                    .column()
                    .background_color(theme::GRAY2)
                    .align_items(AlignItems::Center)
                    .gap(16.)
                    .child(button("Movies", theme::GRAY4))
                    .child(button("Series", theme::GRAY4))
                    .child(button("Settings", theme::GRAY4)),
            )
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Sidebar>()
    }
}

impl From<Sidebar> for Rc<dyn Render> {
    fn from(val: Sidebar) -> Self {
        Rc::new(val)
    }
}

pub fn sidebar() -> Sidebar {
    Sidebar::new()
}
