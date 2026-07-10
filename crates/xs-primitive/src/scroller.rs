use xs_animation::Spring;
use xs_core::{AnyElement, Context, IntoElement, ParentElement, Render};

use crate::{animate, div};

pub struct Scroller {
    offset_top: f32,
    children: Vec<AnyElement>,
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
}

impl Render for Scroller {
    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        let animated_offset = animate(ctx, Spring::new(0.0), self.offset_top);

        div()
            .absolute()
            .top(-animated_offset)
            .children(self.children.clone())
    }
}

impl ParentElement for Scroller {
    fn child(mut self, child: impl Into<AnyElement>) -> Self {
        self.children.push(child.into());
        self
    }

    fn children(
        mut self,
        children: impl IntoIterator<Item = impl Into<AnyElement>>,
    ) -> Self {
        self.children
            .extend(children.into_iter().map(|child| child.into()));
        self
    }
}

pub fn scroller() -> Scroller {
    Scroller::new()
}
