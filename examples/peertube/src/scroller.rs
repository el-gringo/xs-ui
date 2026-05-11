use std::rc::Rc;

use xs::{Render, div};

#[derive(Default)]
pub struct Scroller {
    offset_top: f32,
    children: Vec<Rc<dyn Render>>,
}

impl Scroller {
    pub fn new() -> Self {
        Default::default()
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

    fn render(&self, _ctx: &mut xs::Context) -> Box<dyn xs::Element> {
        div()
            .absolute()
            .top(-self.offset_top)
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
