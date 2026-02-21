use std::{any::TypeId, ops::Deref};

use xs_backend_core::{AppContext, Element, Render};
use xs_layout::{Alignment, Borders, Container, DummyContainer, EdgeInsets, colors::WHITE};

struct MyString(&'static str);
impl Deref for MyString {
    type Target = &'static str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Element for MyString {
    fn paint(&self, ctx: &mut AppContext, _views: Vec<xs_layout::View>) -> xs_layout::View {
        if let Ok((data, width, height)) =
            ctx.text_system
                .render_text(self, "IBMPlexSans Regular", 12.4)
        {
            Box::new(Container {
                width: Some(width as f32),
                height: Some(height as f32),
                alignment: Alignment::top_left(),
                padding: EdgeInsets::zero(),
                borders: Borders::none(),
                margin: EdgeInsets::zero(),
                color: WHITE,
                image: None,
                child: None,
            })
        } else {
            Box::new(DummyContainer)
        }
    }
}

impl Render for MyString {
    fn tag(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn render(&self, _ctx: &mut AppContext) -> Box<dyn Element> {
        Box::new(self.clone())
    }
}
