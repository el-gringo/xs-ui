use std::any::TypeId;

use crate::{AppContext, Element};

pub trait Render {
    fn id(&self) -> &'static str {
        "render"
    }
    fn tag(&self) -> TypeId;
    fn render(&self, ctx: &mut AppContext) -> Box<dyn Element>;
}
