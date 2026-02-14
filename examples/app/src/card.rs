use std::any::TypeId;

use xs::{AppContext, Element, FocusHandle, GRAY, NavigationNode, RED, Render, TRANSPARENT, WHITE, div};

use crate::theme::{GRAY3, GRAY4};

pub struct Card {
    id: &'static str,
    thumbnail: &'static str,
}

impl Card {
    pub fn new() -> Self {
        Self { id: "card", thumbnail: "" }
    }

    pub fn id(mut self, id: &'static str) -> Self {
        self.id = id;
        self
    }

    pub fn thumbnail(mut self, thumbnail: &'static str) -> Self {
        self.thumbnail = thumbnail;
        self
    }
}

impl Render for Card {
    fn render(&self, ctx: &mut AppContext) -> Box<dyn Element> {
        let nav_node =
            ctx.use_state::<FocusHandle>(|ctx| ctx.nav.create_handle(NavigationNode::new(self.id)));
        div()
            .column()
            .width(260.)
            .height(180.)
            .margin_left(32.)
            .background_color(WHITE)
            .justify_content(xs::MainAxisAlignment::Start)
            .align_items(xs::CrossAxisAlignment::Center)
            .border(if nav_node.has_focus() { RED } else { TRANSPARENT }, 4.)
            .child(
                div()
                    .width(233.)
                    .height(122.)
                    .margin_top(16.)
                    .background_color(GRAY3)
                    .background_image(self.thumbnail)
                    .into()
            )
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}


impl Into<Box<dyn Render>> for Card {
    fn into(self) -> Box<dyn Render> {
        Box::new(self)
    }
}

pub fn card() -> Card {
    Card::new()
}
