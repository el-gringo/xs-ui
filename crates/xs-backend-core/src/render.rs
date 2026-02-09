use crate::{AppContext, Element};

pub trait Render {
    fn render(&self, ctx: &mut AppContext) -> Box<dyn Element>;
}
