use std::{any::TypeId, rc::Rc};

use xs::{AlignItems, Color, Context, Element, NavigationNode, Render, div, text};

use crate::theme;

pub struct Card {
    id: Option<&'static str>,
    thumbnail: String,
    title: String,
    on_focus: Option<Rc<dyn Fn() + 'static>>,
}

impl Card {
    pub fn new() -> Self {
        Self {
            id: None,
            thumbnail: String::new(),
            title: String::new(),
            on_focus: None,
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
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let mut focus_handle = ctx.focus_handle(NavigationNode::new().id("grid-cell"));

        if let Some(callback) = &self.on_focus {
            let cb = callback.clone();
            focus_handle.on_focus(move || cb());
        }

        div()
            .column()
            .width(320.)
            .height(270.)
            .background_color(Color::WHITE)
            .border_radius_all(8.)
            .box_shadow(4., 4., 8., 0., Color::BLACK)
            .border(
                if focus_handle.has_focus() {
                    theme::BRIGHT_RED
                } else {
                    Color::WHITE
                },
                4.,
            )
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
                div().width(280.).height(60.).margin_top(8.).child(
                    div()
                        .font_size(24)
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
