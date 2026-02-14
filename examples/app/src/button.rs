use std::any::TypeId;

use xs::{AppContext, Color, Element, FocusHandle, GRAY, NavigationNode, Render, WHITE, div};


pub struct Button {
    color: Color,
}

impl Button {
    pub fn new(color: Color) -> Self {
        Button { color }
    }
}

impl Render for Button {
    fn render(&self, ctx: &mut AppContext) -> Box<dyn Element> {
        let nav_node = ctx
            .use_state::<FocusHandle>(|ctx| ctx.nav.create_handle(NavigationNode::new("button")));
        div()
            .id("button")
            .width(200.)
            .height(40.)
            .margin_top(16.)
            .background_color(self.color)
            .border(if nav_node.has_focus() { WHITE } else { GRAY }, 2.)
            .into()
    }
    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}

pub fn button(color: Color) -> Button {
    Button::new(color)
}
