use std::any::TypeId;

use xs::{AlignItems, Application, BLUE, Element, FocusHandle, GREEN, NavigationNode, RED, Render, div};

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
    fn render(&self, app: &mut Application) -> Box<dyn Element> {
        let _ = app.ctx.use_state::<FocusHandle>(|ctx| {
            let node = NavigationNode::new("sidebar").vertical();
            ctx.nav.create_handle(node)
        });

        div()
            .id("sidebar")
            .column()
            .width(300.)
            .justify_content(xs::JustifyContent::FlexStart)
            .child(
                div()
                    .id("sidebar-header")
                    .height(50.)
                    .background_color(GRAY4)
                    .into(),
            )
            .child(
                div()
                    .id("sidebar-body")
                    .column()
                    .background_color(GRAY2)
                    .align_items(AlignItems::Center)
                    .child(Box::new(button(RED)))
                    .child(Box::new(button(GREEN)))
                    .child(Box::new(button(BLUE)))
                    .into()
            )
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Sidebar>()
    }
}

impl From<Sidebar> for Box<dyn Render> {
    fn from(val: Sidebar) -> Self {
        Box::new(val)
    }
}

pub fn sidebar() -> Sidebar {
    Sidebar::new()
}
