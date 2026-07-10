use crate::{Context, element::IntoElement};

pub trait Render {
    fn id(&self) -> &'static str {
        "render"
    }
    fn render(&self, ctx: &mut Context) -> impl IntoElement;
}
