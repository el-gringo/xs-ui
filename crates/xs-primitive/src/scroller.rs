use std::rc::Rc;

use xs_animation::Spring;
use xs_core::{Context, Element, Render};

use crate::{animate, div};

pub struct Scroller {
    offset_top: f32,
    children: Vec<Rc<dyn Render>>,
}

impl Scroller {
    pub fn new() -> Self {
        Self {
            offset_top: 0.0,
            children: Vec::new(),
        }
    }

    pub fn scroll_to(mut self, offset_top: f32) -> Self {
        self.offset_top = offset_top;
        self
    }

    pub fn children(
        mut self,
        children: impl IntoIterator<Item = impl Into<Rc<dyn Render>>>,
    ) -> Self {
        self.children
            .extend(children.into_iter().map(|child| child.into()));
        self
    }
}

impl Render for Scroller {
    fn tag(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }

    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let animated_offset = animate(ctx, Spring::new(0.0), self.offset_top);

        div()
            .absolute()
            .top(-animated_offset)
            .children(self.children.clone())
            .into()
    }
}

impl From<Scroller> for Rc<dyn Render> {
    fn from(value: Scroller) -> Self {
        Rc::new(value)
    }
}

pub fn scroller() -> Scroller {
    Scroller::new()
}
