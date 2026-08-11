use std::{any::TypeId, rc::Rc};

use xs_layout::View;

use crate::{AppContext, Context, Render};

pub trait IntoElement {
    fn into_element(self) -> AnyElement;
}

pub struct AnyElement {
    pub(crate) inner: Box<dyn ElementObject>,
}

impl Clone for AnyElement {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone_box(),
        }
    }
}

impl AnyElement {
    pub fn new<T: ElementObject + 'static>(element: T) -> Self {
        Self {
            inner: Box::new(element),
        }
    }
}

impl<T: IntoElement> From<T> for AnyElement {
    fn from(value: T) -> Self {
        value.into_element()
    }
}

pub trait ElementObject {
    fn tag(&self) -> TypeId;
    fn clone_box(&self) -> Box<dyn ElementObject>;
    fn id(&self) -> &'static str {
        "unknown"
    }
    fn children(&self) -> &[AnyElement] {
        &[]
    }
    fn post_render(&mut self, _ctx: &mut Context) {}
    fn paint(&self, app_context: &mut AppContext, views: &[View]) -> View;
    fn resolve(&mut self, _ctx: &mut Context) -> Option<AnyElement> {
        None
    }
}

pub trait ParentElement {
    fn child(self, child: impl Into<AnyElement>) -> Self;
    fn children(self, children: impl IntoIterator<Item = impl Into<AnyElement>>) -> Self;
}

pub struct DeferredRender {
    render_fn: Rc<dyn Fn(&mut Context) -> AnyElement>,
    tag: TypeId,
    render_id: &'static str,
}

impl Clone for DeferredRender {
    fn clone(&self) -> Self {
        Self {
            render_fn: self.render_fn.clone(),
            tag: self.tag,
            render_id: self.render_id,
        }
    }
}

impl<T: Render + 'static> From<T> for DeferredRender {
    fn from(value: T) -> Self {
        let tag = std::any::TypeId::of::<T>();
        let id = Render::id(&value);
        let render = Rc::new(value);
        let render_clone = render.clone();
        Self {
            render_fn: Rc::new(move |ctx: &mut Context| render_clone.render(ctx).into_element()),
            tag,
            render_id: id,
        }
    }
}

impl ElementObject for DeferredRender {
    fn tag(&self) -> TypeId {
        self.tag
    }

    fn clone_box(&self) -> Box<dyn ElementObject> {
        Box::new(self.clone())
    }

    fn id(&self) -> &'static str {
        self.render_id
    }

    fn children(&self) -> &[AnyElement] {
        &[]
    }

    fn paint(&self, _app_context: &mut AppContext, _views: &[View]) -> View {
        unreachable!("DeferredRender should be resolved before paint")
    }

    fn resolve(&mut self, ctx: &mut Context) -> Option<AnyElement> {
        Some((self.render_fn)(ctx))
    }
}

impl<T: Render + 'static> IntoElement for T {
    fn into_element(self) -> AnyElement {
        AnyElement::new(DeferredRender::from(self))
    }
}
