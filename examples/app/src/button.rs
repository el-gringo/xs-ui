use std::any::TypeId;

use xs::{div, Color, Context, Element, FocusHandle, NavigationNode, Render, GRAY, WHITE};

pub struct Button {
    color: Color,
}

impl Button {
    pub fn new(color: Color) -> Self {
        Button { color }
    }
}

impl Render for Button {
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let nav_node =
            ctx.use_state::<FocusHandle>(|ctx, _| ctx.focus_handle(NavigationNode::new("button")));
        div()
            .id("button")
            .width(200.)
            .height(50.)
            .margin_top(16.)
            .background_color(self.color)
            .border(if nav_node.has_focus() { WHITE } else { GRAY }, 2.)
            .into()
    }
    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}

impl From<Button> for Box<dyn Render> {
    fn from(value: Button) -> Self {
        Box::new(value)
    }
}

pub fn button(color: Color) -> Button {
    Button::new(color)
}
