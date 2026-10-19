use xs::{AnyElement, Context, ParentElement, Render, div};

use crate::sidebar::SLIDE;

pub struct Body {
    pub children: Vec<AnyElement>,
    pub slide: f32,
}

impl Body {
    pub fn children(mut self, children: impl IntoIterator<Item = impl Into<AnyElement>>) -> Self {
        self.children
            .extend(children.into_iter().map(|child| child.into()));
        self
    }

    pub fn slide(mut self, slide: f32) -> Self {
        self.slide = slide;
        self
    }
}

impl Render for Body {
    fn render(&self, _ctx: &mut Context) -> impl xs::IntoElement {
        div()
            .testid("body")
            .column()
            .translate_xy(-SLIDE * self.slide, 0.)
            .children(self.children.clone())
    }
}

pub fn body() -> Body {
    Body {
        children: Vec::new(),
        slide: 0.,
    }
}
