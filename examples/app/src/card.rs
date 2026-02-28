use std::any::TypeId;

use xs::{
    AlignItems, Context, Element, FocusHandle, JustifyContent, NavigationNode, RED, Render,
    TRANSPARENT, WHITE, div,
};

use crate::theme::GRAY3;

pub struct Card {
    id: Option<&'static str>,
    thumbnail: String,
}

impl Card {
    pub fn new() -> Self {
        Self {
            id: None,
            thumbnail: String::new(),
        }
    }

    pub fn id(mut self, id: &'static str) -> Self {
        self.id = Some(id);
        self
    }

    pub fn thumbnail(mut self, thumbnail: impl Into<String>) -> Self {
        self.thumbnail = thumbnail.into();
        self
    }
}

impl Render for Card {
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let id = self.id.unwrap_or("card");
        let nav_node =
            ctx.use_state::<FocusHandle>(|ctx| ctx.focus_handle(NavigationNode::new(id)));
        div()
            .column()
            .width(320.)
            .height(240.)
            .background_color(WHITE)
            .justify_content(JustifyContent::FlexStart)
            .align_items(AlignItems::Center)
            .border(
                if nav_node.has_focus() {
                    RED
                } else {
                    TRANSPARENT
                },
                4.,
            )
            .child(
                div()
                    .width(280.)
                    .height(157.)
                    .margin_top(8.)
                    .background_color(GRAY3)
                    .background_image(self.thumbnail.clone())
                    .into(),
            )
            .into()
    }

    fn tag(&self) -> TypeId {
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
