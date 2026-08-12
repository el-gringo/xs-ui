use std::rc::Rc;

use xs_animation::Animate;

use crate::div::{Div, DivAnimation};

pub trait WithAnimation {
    fn with_animation<A: Animate + 'static>(
        self,
        id: &'static str,
        animate: A,
        callback: impl Fn(Div, f32) -> Div + 'static,
    ) -> Self;
}

impl WithAnimation for Div {
    fn with_animation<A: Animate + 'static>(
        mut self,
        id: &'static str,
        animate: A,
        callback: impl Fn(Div, f32) -> Div + 'static,
    ) -> Self {
        self.animation = Some(DivAnimation {
            id,
            animate: Box::new(animate),
            callback: Rc::new(callback),
        });
        self
    }
}
