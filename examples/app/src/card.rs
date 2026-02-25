use std::any::TypeId;

use xs::{AlignItems, Context, Element, FocusHandle, JustifyContent, NavigationNode, RED, Render, TRANSPARENT, WHITE, div};

use crate::theme::GRAY3;

pub struct Card {
    id: &'static str,
    thumbnail: &'static str,
}

impl Card {
    pub fn new() -> Self {
        Self {
            id: "card",
            thumbnail: "",
        }
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
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let nav_node =
            ctx.use_state::<FocusHandle>(|ctx| ctx.focus_handle(NavigationNode::new(self.id)));
        div()
            .column()
            .width(320.)
            .height(240.)
            .background_color(WHITE)
            .justify_content(JustifyContent::FlexStart)
            .align_items(AlignItems::Center)
            .border(if nav_node.has_focus() { RED } else { TRANSPARENT }, 4.)
            .child(
                div()
                    .width(280.)
                    .height(157.)
                    .margin_top(8.)
                    .background_color(GRAY3)
                    .into(),
            )
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}

impl From<Card> for Box<dyn Render> {
    fn from(val: Card) -> Self {
        Box::new(val)
    }
}

pub fn card() -> Card {
    Card::new()
}
