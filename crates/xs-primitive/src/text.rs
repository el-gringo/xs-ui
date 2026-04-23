use std::{any::TypeId, rc::Rc};

use xs_core::{Context, Element, Render};
use xs_layout::{BoxConstraints, Layout, LayoutTree, Material, SizedLayoutBox, Vector2, View};

pub struct Text(pub String);

impl Render for Text {
    fn tag(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let text_vertices = ctx.app_context.text_system.render_text(
            &self.0,
            &ctx.text_properties,
            ctx.constraints.max.x,
            ctx.constraints.max.y,
        );
        Box::new(TextElement {
            width: text_vertices.width,
            height: text_vertices.height,
            material: Some(Material::Text(
                ctx.color,
                ctx.text_properties.clone(),
                text_vertices.clone(),
            )),
        })
    }
}

pub struct TextElement {
    width: f32,
    height: f32,
    material: Option<Material>,
}

impl Element for TextElement {
    fn paint(&self, _app_context: &mut xs_core::AppContext, _views: &[View]) -> View {
        Rc::new(TextElement {
            width: self.width,
            height: self.height,
            material: self.material.clone(),
        })
    }
}

impl Layout for TextElement {
    fn layout(&self, _tree: &mut LayoutTree, _constraints: &BoxConstraints) -> SizedLayoutBox {
        SizedLayoutBox {
            size: Vector2::new(self.width, self.height),
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
