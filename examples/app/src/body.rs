use std::{any::TypeId, rc::Rc};

use xs::{Context, Element, FocusHandle, NavigationNode, Render, div};

use crate::theme::{GRAY1, GRAY4};

pub struct Body {
    pub children: Vec<Rc<Box<dyn Render>>>,
}

const TITLE: &str = "PeerTube";

impl Body {
    pub fn child(mut self, child: Box<dyn Render>) -> Self {
        self.children.push(Rc::new(child));
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Rc<Box<dyn Render>>>) -> Self {
        self.children.extend(children);
        self
    }
}

impl Render for Body {
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let _ = ctx.use_state::<FocusHandle>(|ctx, _| {
            ctx.focus_handle(NavigationNode::new("body").horizontal())
        });
        div()
            .id("body")
            .column()
            .grow(1.)
            .background_color(GRAY1)
            .child(
                div()
                    .height(50.)
                    .background_color(GRAY4)
                    .into(),
            )
            .child(div().grow(1.).children(self.children.clone()).into())
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}

impl From<Body> for Box<dyn Render> {
    fn from(val: Body) -> Self {
        Box::new(val)
    }
}

pub fn body() -> Body {
    Body {
        children: Vec::new(),
    }
}
