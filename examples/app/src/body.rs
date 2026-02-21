use std::{any::TypeId, rc::Rc};

use xs::{Context, Element, FocusHandle, NavigationNode, Render, WHITE, div};

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
    fn render(&self, _ctx: &mut Context) -> Box<dyn Element> {
        // let _ = ctx.use_state::<FocusHandle>(|ctx| {
        //     ctx.nav
        //         .create_handle(NavigationNode::new("body").horizontal())
        // });
        div()
            .id("body")
            .column()
            .children(self.children.clone())
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}

impl Into<Box<dyn Render>> for Body {
    fn into(self) -> Box<dyn Render> {
        Box::new(self)
    }
}

pub fn body() -> Body {
    Body {
        children: Vec::new(),
    }
}
