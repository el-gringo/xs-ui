use std::{any::TypeId, rc::Rc};

use xs::{Color, Context, Element, FocusHandle, GRAY, NavigationNode, Render, WHITE, div, text};

pub struct Button {
    text: &'static str,
    background_color: Color,
}

impl Button {
    pub fn new(text: &'static str, background_color: Color) -> Self {
        Button {
            text,
            background_color,
        }
    }
}

impl Render for Button {
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let nav_node =
            ctx.use_state::<FocusHandle>(|ctx, _| ctx.focus_handle(NavigationNode::new("button")));
        div()
            .id("button")
            .justify_content(xs::JustifyContent::Center)
            .width(200.)
            .height(50.)
            .margin_top(16.)
            //.background_color(self.background_color)
            .color(WHITE)
            .border(if nav_node.has_focus() { WHITE } else { GRAY }, 4.)
            .font_size(32)
            .child(text(self.text))
            .into()
    }
    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}

impl From<Button> for Rc<dyn Render> {
    fn from(value: Button) -> Self {
        Rc::new(value)
    }
}

pub fn button(text: &'static str, color: Color) -> Button {
    Button::new(text, color)
}
