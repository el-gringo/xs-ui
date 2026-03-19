use std::{any::TypeId, rc::Rc};

use xs::{
    AlignItems, BLUE, Context, Element, FocusHandle, GREEN, JustifyContent, NavigationNode, RED,
    Render, WHITE, div, text,
};

use crate::{
    button::{Button, button},
    theme::{GRAY2, GRAY4},
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
        let _ = ctx.use_state::<FocusHandle>(|ctx, _| {
            let node = NavigationNode::new("sidebar").vertical();
            ctx.focus_handle(node)
        });

        div()
            .id("sidebar")
            .column()
            .width(300.)
            .justify_content(xs::JustifyContent::FlexStart)
            .child(
                div()
                    .id("sidebar-header")
                    .row()
                    .height(50.)
                    .background_color(GRAY4)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .color(WHITE)
                    .font_size(45)
                    .child(text("PeerTube")),
            )
            .child(
                div()
                    .id("sidebar-body")
                    .column()
                    .background_color(GRAY2)
                    .align_items(AlignItems::Center)
                    .child(button(RED))
                    .child(button(GREEN))
                    .child(button(BLUE)),
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
