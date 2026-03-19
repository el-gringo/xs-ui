use std::{any::TypeId, rc::Rc};

use xs_core::{BackendRequest, Context, Element, Render};
use xs_layout::{BoxConstraints, Layout, LayoutTree, Material, SizedLayoutBox, Vector2, View};

pub struct Text(pub String);

impl Render for Text {
    fn tag(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let entry = ctx
            .app_context
            .texts
            .entry((self.0.clone(), ctx.text_properties.clone()));

        let width;
        let height;
        let material;

        match entry {
            std::collections::hash_map::Entry::Occupied(o) => {
                let opt = o.get();
                match opt {
                    &Some((ref tv, texture_id)) => {
                        width = tv.width;
                        height = tv.height;
                        material = Some(Material::Text(
                            ctx.text_properties.color,
                            (tv.clone(), texture_id),
                        ));
                    }
                    &None => {
                        width = 0.0;
                        height = 0.0;
                        material = None;
                    }
                }
            }
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert(None);
                ctx.app_context.messages.push(BackendRequest::Text(
                    self.0.clone(),
                    ctx.text_properties.clone(),
                ));
                width = 0.0;
                height = 0.0;
                material = None;
            }
        }

        Box::new(TextElement {
            width,
            height,
            material,
            max_width: ctx.text_properties.max_width,
        })
    }
}

pub struct TextElement {
    width: f32,
    height: f32,
    material: Option<Material>,
    max_width: Option<f32>,
}

impl Element for TextElement {
    fn paint(&self, _app_context: &mut xs_core::AppContext, _views: &[View]) -> View {
        Rc::new(TextElement {
            width: self.width,
            height: self.height,
            material: self.material.clone(),
            max_width: self.max_width,
        })
    }
}

impl Layout for TextElement {
    fn layout(&self, _tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let max_w = self.max_width.unwrap_or(constraints.max.x);
        let width = max_w.min(self.width);
        let height = constraints.max.y.min(self.height);
        SizedLayoutBox {
            size: Vector2::new(width, height),
            margin: Default::default(),
            children: vec![],
            material: self.material.clone(),
            text_material: None,
        }
    }
}

impl From<Text> for Rc<dyn Render> {
    fn from(value: Text) -> Self {
        Rc::new(value)
    }
}

impl From<Text> for String {
    fn from(value: Text) -> Self {
        value.0
    }
}

pub fn text(text: impl Into<String>) -> Text {
    Text(text.into())
}
