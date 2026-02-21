use std::any::TypeId;

use crate::{Application, element::Element};

pub trait Render {
    fn id(&self) -> &'static str {
        "render"
    }
    fn tag(&self) -> TypeId;
    fn render(&self, ctx: &mut Application) -> Box<dyn Element>;
}
