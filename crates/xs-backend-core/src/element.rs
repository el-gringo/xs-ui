use std::rc::Rc;

use crate::{AppContext, Render};
use xs_layout::View;

pub trait Element {
    fn id(&self) -> &'static str {
        "unknown"
    }
    fn children(&self) -> Vec<Rc<Box<dyn Render>>> {
        Vec::new()
    }
    fn paint(&self, ctx: &mut AppContext, views: Vec<View>) -> View;
}

// pub trait Element: 'static + IntoElement {
//     fn id(&self) -> &'static str {
//         ""
//     }

//     fn children(&self) -> Vec<AnyElement> {
//         Vec::new()
//     }

//     fn paint(&self, ctx: &mut AppContext, views: Vec<View>) -> View;

//     fn into_any(self) -> AnyElement {
//         AnyElement::new(self)
//     }
// }

// pub trait IntoElement: Sized {
//     /// The specific type of element into which the implementing type is converted.
//     /// Useful for converting other types into elements automatically, like Strings
//     type Element: Element;

//     /// Convert self into a type that implements [`Element`].
//     fn into_element(self) -> Self::Element;

//     /// Convert self into a dynamically-typed [`AnyElement`].
//     fn into_any_element(self) -> AnyElement {
//         self.into_element().into_any()
//     }
// }

// pub struct Drawable<E: Element> {
//     /// The drawn element.
//     pub element: E,
// }

// impl<E: Element> Drawable<E> {
//     pub(crate) fn new(element: E) -> Self {
//         Drawable { element }
//     }
// }

// trait ElementObject {
//     fn inner_element(&mut self) -> &mut dyn Any;
//     fn paint(&mut self);
// }

// impl<E> ElementObject for Drawable<E>
// where
//     E: Element
// {
//     fn inner_element(&mut self) -> &mut dyn Any {
//         &mut self.element
//     }

//     fn paint(&mut self) {
//         //Drawable::paint(self);
//     }
// }

// pub struct AnyElement(Box<dyn ElementObject>);
// impl AnyElement {
//     pub(crate) fn new<E>(element: E) -> Self
//     where
//         E: 'static + Element
//     {
//         Self(Box::new(Drawable::new(element)))
//     }

//     /// Attempt to downcast a reference to the boxed element to a specific type.
//     pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
//         self.0.inner_element().downcast_mut::<T>()
//     }
// }
