use std::{any::TypeId, rc::Rc};

use xs::{Context, Element, FocusHandle, NavigationNode, Render, div};

use crate::theme::GRAY1;

pub struct Body {
    pub children: Vec<Rc<dyn Render>>,
}

impl Body {
    pub fn child(mut self, child: impl Into<Rc<dyn Render>>) -> Self {
        self.children.push(child.into());
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

impl Render for Body {
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let _ = ctx.use_state::<FocusHandle>(|ctx, _| {
            ctx.focus_handle(NavigationNode::new("body").horizontal())
        });
        div()
            .testid("body")
            .column()
            .background_color(GRAY1)
            .children(self.children.clone())
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}

impl From<Body> for Rc<dyn Render> {
    fn from(val: Body) -> Self {
        Rc::new(val)
    }
}

pub fn body() -> Body {
    Body {
        children: Vec::new(),
    }
}
