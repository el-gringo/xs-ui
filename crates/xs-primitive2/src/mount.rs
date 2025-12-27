use xs_backend_core::AppContext;
use xs_layout::View;
use crate::{Element, Render};

pub trait Mount {
    fn mount() -> Self where Self: Sized;
    fn update(&mut self, ctx: &mut AppContext) -> Element;
    fn render(&mut self, ctx: &mut AppContext) -> View {
        self.update(ctx).render()
    }
}
