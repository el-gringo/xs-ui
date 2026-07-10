use xs::{AnyElement, Context, ParentElement, Render, div};

pub struct Body {
    pub children: Vec<AnyElement>,
}

impl Body {
    pub fn children(mut self, children: impl IntoIterator<Item = impl Into<AnyElement>>) -> Self {
        self.children
            .extend(children.into_iter().map(|child| child.into()));
        self
    }
}

impl Render for Body {
    fn render(&self, _ctx: &mut Context) -> impl xs::IntoElement {
        div()
            .testid("body")
            .column()
            .children(self.children.clone())
    }
}

pub fn body() -> Body {
    Body {
        children: Vec::new(),
    }
}
