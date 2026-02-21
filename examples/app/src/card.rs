use std::any::TypeId;

use xs::{Context, Element, Render, WHITE, div};

use crate::theme::GRAY3;

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
    fn render(&self, _ctx: &mut Context) -> Box<dyn Element> {
        // let nav_node =
        //     ctx.use_state::<FocusHandle>(|ctx| ctx.nav.create_handle(NavigationNode::new(self.id)));
        div()
            .column()
            .width(320.)
            .height(240.)
            .background_color(WHITE)
            .justify_content(xs::MainAxisAlignment::Start)
            .align_items(xs::CrossAxisAlignment::Center)
            //.border(if nav_node.has_focus() { RED } else { TRANSPARENT }, 4.)
            .child(
                div()
                    .width(280.)
                    .height(157.)
                    .margin_top(8.)
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
