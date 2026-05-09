use std::{any::TypeId, rc::Rc};

use xs::{AlignItems, Color, Context, Element, Render, div, text};

use crate::theme;

pub struct Card {
    id: Option<&'static str>,
    thumbnail: String,
    title: String,
}

impl Card {
    pub fn new() -> Self {
        Self {
            id: None,
            thumbnail: String::new(),
            title: String::new(),
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

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
}

impl Render for Card {
    fn render(&self, _ctx: &mut Context) -> Box<dyn Element> {
        div()
            .column()
            .width(320.)
            .height(240.)
            .background_color(Color::WHITE)
            .align_items(AlignItems::Center)
            .child(
                div()
                    .width(280.)
                    .height(157.)
                    .margin_top(8.)
                    .background_color(theme::GRAY3)
                    .background_image(self.thumbnail.clone()),
            )
            .child(
                div().width(280.).height(60.).margin_top(16.).child(
                    div()
                        .font_size(26)
                        .color(theme::GRAY3)
                        .child(text(self.title.clone())),
                ),
            )
            .into()
    }

    fn tag(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl From<Card> for Rc<dyn Render> {
    fn from(val: Card) -> Self {
        Rc::new(val)
    }
}

pub fn card() -> Card {
    Card::new()
}
