
use std::{any::TypeId, rc::Rc};

use xs_core::{Context, Element, Message, Render};
use xs_layout::{FixedSize, Material, View};

pub struct Text(pub String);

impl Render for Text {
    fn tag(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let image = ctx.app_context
                .texts
                .entry(self.0.to_string())
                .or_insert_with(|| {
                    ctx.app_context.messages.push(Message::Text(
                        self.0.to_string(),
                        ctx.text_properties.clone()
                    ));
                    None
                })
                .clone();
        let (width, height) = match &image {
            Some(img) => (img.width as f32, img.height as f32),
            None => (0., 0.),
        };
        let material = Material::Text(ctx.text_properties.unwrap_or_default().color, image);
        Box::new(TextElement(FixedSize {
            width,
            height,
            material: material.clone()
        }))
    }
}


struct TextElement(pub FixedSize);

impl Element for TextElement {
    fn paint(&self, _app_context: &mut xs_core::AppContext, _views: &[xs_layout::View]) -> View {
        Rc::new(self.0.clone())
    }
}

impl Into<Box<dyn Render>> for Text {
    fn into(self) -> Box<dyn Render> {
        Box::new(self)
    }
}

impl Into<Rc<Box<dyn Render>>> for Text {
    fn into(self) -> Rc<Box<dyn Render>> {
        Rc::new(Box::new(self))
    }
}

impl From<Text> for String {
    fn from(value: Text) -> Self {
        value.0
    }
}

pub fn text(text: String) -> Text {
    Text(text)
}
