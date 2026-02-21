use std::any::TypeId;

use xs::{Context, BLUE, Element, FocusHandle, GREEN, NavigationNode, RED, Render, div};

use crate::{
    button::{Button, button},
    theme::{GRAY2, GRAY3},
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
    fn render(&self, _ctx: &mut Context) -> Box<dyn Element> {
        // let _ = ctx.use_state::<FocusHandle>(|ctx| {
        //     let node = NavigationNode::new("sidebar").vertical();
        //     ctx.nav.create_handle(node)
        // });

        div()
            .id("sidebar")
            .width(300.)
            .column()
            .align_items(xs::CrossAxisAlignment::Center)
            .background_color(GRAY2)
            .child(
                div()
                    .id("sidebar-header")
                    .width(300.)
                    .height(50.)
                    .background_color(GRAY3)
                    .margin_bottom(32.)
                    .into(),
            )
            .child(Box::new(button(RED)))
            .child(Box::new(button(GREEN)))
            .child(Box::new(button(BLUE)))
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Sidebar>()
    }
}

impl Into<Box<dyn Render>> for Sidebar {
    fn into(self) -> Box<dyn Render> {
        Box::new(self)
    }
}

pub fn sidebar() -> Sidebar {
    Sidebar::new()
}
