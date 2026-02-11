use std::rc::Rc;

use crate::{AppContext, Render};
use xs_layout::View;

pub trait Element {
    fn children(&self) -> Vec<Rc<Box<dyn Render>>> {
        Vec::new()
    }

    fn paint(&self, ctx: &mut AppContext, views: Vec<View>) -> View;
}
