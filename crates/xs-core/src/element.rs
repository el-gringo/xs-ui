use std::{any::TypeId, rc::Rc};

use xs_layout::{DummyContainer, View};

use crate::{Application, Context, render::Render};

pub trait Element {
    fn id(&self) -> &'static str {
        "unknown"
    }
    fn children(&self) -> &[Rc<Box<dyn Render>>] {
        &[]
    }
    fn paint(&self, ctx: &mut Application, views: &[View]) -> View;
}

pub struct RootElement;
impl Element for RootElement {
    fn paint(&self, _ctx: &mut Application, _views: &[View]) -> View {
        Rc::new(DummyContainer {})
    }
}

impl Render for RootElement {
    fn render(&self, _ctx: &mut Context) -> Box<dyn Element> {
        Box::new(RootElement {})
    }
    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<RootElement>()
    }
}
