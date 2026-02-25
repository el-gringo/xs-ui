use std::any::TypeId;

use crate::{Context, element::Element};

pub trait Render {
    fn id(&self) -> &'static str {
        "render"
    }
    fn tag(&self) -> TypeId;
    fn render(&self, app: &mut Context) -> Box<dyn Element>;
}
