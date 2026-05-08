use std::{any::TypeId, rc::Rc};

use xs_core::{AppContext, Context, Element, Render, TextProperties};
use xs_layout::{
    AlignItems, BorderRadius, BorderSide, Borders, BoxShadow, Color, EdgeInsets, Flex,
    FlexDirection, FlexWrap, JustifyContent, Material, Positioned, Style, StyleDisplay,
    StylePosition, View,
};

#[derive(Default, Clone)]
pub struct Div {
    pub testid: Option<&'static str>,
    pub style: Style,
    pub children: Vec<Rc<dyn Render>>,
}

impl Div {
    pub fn new() -> Self {
        Default::default()
    }

    /// An id that is only used for debug and tests
    pub fn testid(mut self, testid: &'static str) -> Self {
        self.testid = Some(testid);
        self
    }

    //
    // POSITION
    //
    pub fn absolute(mut self) -> Self {
        self.style.position = StylePosition::Absolute;
        self
    }

    pub fn top(mut self, value: f32) -> Self {
        self.style.top = Some(value);
        self
    }

    pub fn right(mut self, value: f32) -> Self {
        self.style.right = Some(value);
        self
    }

    pub fn bottom(mut self, value: f32) -> Self {
        self.style.bottom = Some(value);
        self
    }

    pub fn left(mut self, value: f32) -> Self {
        self.style.left = Some(value);
        self
    }

    pub fn width(mut self, size: f32) -> Self {
        self.style.width = Some(size);
        self
    }

    pub fn height(mut self, size: f32) -> Self {
        self.style.height = Some(size);
        self
    }

    //
    // MARGIN
    //
    pub fn margin(mut self, margin: EdgeInsets) -> Self {
        self.style.margin = margin;
        self
    }

    pub fn margin_top(mut self, size: f32) -> Self {
        self.style.margin.top = size;
        self
    }

    pub fn margin_right(mut self, size: f32) -> Self {
        self.style.margin.right = size;
        self
    }

    pub fn margin_bottom(mut self, size: f32) -> Self {
        self.style.margin.bottom = size;
        self
    }

    pub fn margin_left(mut self, size: f32) -> Self {
        self.style.margin.left = size;
        self
    }

    //
    // PADDING
    //
    pub fn padding(mut self, size: f32) -> Self {
        self.style.padding = EdgeInsets::all(size);
        self
    }

    pub fn padding_top(mut self, size: f32) -> Self {
        self.style.padding.top = size;
        self
    }

    pub fn padding_right(mut self, size: f32) -> Self {
        self.style.padding.right = size;
        self
    }

    pub fn padding_bottom(mut self, size: f32) -> Self {
        self.style.padding.bottom = size;
        self
    }

    pub fn padding_left(mut self, size: f32) -> Self {
        self.style.padding.left = size;
        self
    }

    //
    // BORDER
    //
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.style.border = Borders::all(color, width);
        self
    }

    pub fn border_top(mut self, color: Color, width: f32) -> Self {
        self.style.border.top = Some(BorderSide::new(color, width));
        self
    }

    pub fn border_right(mut self, color: Color, width: f32) -> Self {
        self.style.border.right = Some(BorderSide::new(color, width));
        self
    }

    pub fn border_bottom(mut self, color: Color, width: f32) -> Self {
        self.style.border.bottom = Some(BorderSide::new(color, width));
        self
    }

    pub fn border_left(mut self, color: Color, width: f32) -> Self {
        self.style.border.left = Some(BorderSide::new(color, width));
        self
    }

    pub fn border_radius(
        mut self,
        top_left: f32,
        top_right: f32,
        bottom_right: f32,
        bottom_left: f32,
    ) -> Self {
        self.style.border_radius =
            BorderRadius::new(top_left, top_right, bottom_right, bottom_left);
        self
    }

    pub fn border_radius_all(mut self, radius: f32) -> Self {
        self.style.border_radius = BorderRadius::new(radius, radius, radius, radius);
        self
    }

    pub fn border_top_left_radius(mut self, radius: f32) -> Self {
        self.style.border_top_left_radius = radius;
        self
    }

    pub fn border_top_right_radius(mut self, radius: f32) -> Self {
        self.style.border_top_right_radius = radius;
        self
    }

    pub fn border_bottom_right_radius(mut self, radius: f32) -> Self {
        self.style.border_bottom_right_radius = radius;
        self
    }

    pub fn border_bottom_left_radius(mut self, radius: f32) -> Self {
        self.style.border_bottom_left_radius = radius;
        self
    }

    //
    // BOX SHADOW
    //
    pub fn box_shadow(
        mut self,
        offset_left: f32,
        offset_top: f32,
        blur: f32,
        spread: f32,
        color: Color,
    ) -> Self {
        self.style.box_shadow = BoxShadow::new(offset_left, offset_top, blur, spread, color);
        self
    }

    //
    // COLOR
    //
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.style.color = color.into();
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.style.background_color = color;
        self
    }

    pub fn background_image(mut self, image: String) -> Self {
        self.style.background_image = Some(image);
        self
    }

    //
    // FLEX
    //
    pub fn row(mut self) -> Self {
        self.style.display = StyleDisplay::Flex;
        self.style.flex_direction = FlexDirection::Row;
        self
    }

    pub fn column(mut self) -> Self {
        self.style.display = StyleDisplay::Flex;
        self.style.flex_direction = FlexDirection::Column;
        self
    }

    pub fn justify_content(mut self, value: JustifyContent) -> Self {
        self.style.justify_content = value;
        self
    }

    pub fn align_items(mut self, value: AlignItems) -> Self {
        self.style.align_items = value;
        self
    }

    pub fn grow(mut self, main: f32, cross: f32) -> Self {
        self.style.flex_grow = [main, cross];
        self
    }

    pub fn shrink(mut self, shrink: f32) -> Self {
        self.style.flex_shrink = shrink;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap = gap;
        self
    }

    pub fn wrap(mut self) -> Self {
        self.style.flex_wrap = FlexWrap::Wrap;
        self
    }

    pub fn font_family(mut self, font_family: impl Into<String>) -> Self {
        self.update_text_properties(|text_properties| {
            text_properties.font_family = font_family.into()
        });
        self
    }

    pub fn font_size(mut self, font_size: u16) -> Self {
        self.update_text_properties(|text_properties| text_properties.font_size = font_size);
        self
    }

    pub fn font_weight(mut self, font_weight: u16) -> Self {
        self.update_text_properties(|text_properties| text_properties.font_weight = font_weight);
        self
    }

    fn update_text_properties(&mut self, mutator: impl FnOnce(&mut TextProperties)) {
        match self.style.text_properties.as_mut() {
            Some(v) => mutator(v),
            None => {
                let v = Default::default();
                self.style.text_properties.replace(v);
                self.update_text_properties(mutator);
            }
        }
    }

    //
    // OTHER
    //
    pub fn child(mut self, child: impl Into<Rc<dyn Render>>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn children(
        mut self,
        children: impl IntoIterator<Item = impl Into<Rc<dyn Render>>>,
    ) -> Self {
        self.children
            .extend(children.into_iter().map(|child| child.into()));
        self
    }
}

// impl Element for Div {
//     fn id(&self) -> &'static str {
//         self.id.unwrap_or("div")
//     }

//     fn children(&self) -> Vec<Rc<Box<dyn Render>>> {
//         self.children.iter().map(|child| child.clone()).collect()
//     }

//     fn paint(&self, ctx: &mut AppContext, views: Vec<View>) -> View {
//         self.style.paint(ctx, views)
//     }
// }

impl Render for Div {
    fn tag(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn render(&self, _ctx: &mut Context) -> Box<dyn Element> {
        Box::new(self.clone())
    }
}

impl Element for Div {
    fn id(&self) -> &'static str {
        self.testid.unwrap_or("div")
    }

    fn children(&self) -> &[Rc<dyn Render>] {
        &self.children
    }

    fn post_render(&self, ctx: &mut Context) {
        if let Some(text_properties) = self.style.text_properties.clone() {
            ctx.text_properties = text_properties;
        }
        ctx.color = self.style.color;
        if let Some(w) = self.style.width {
            ctx.constraints.max.x = w;
        };
        if let Some(h) = self.style.height {
            ctx.constraints.max.y = h;
        };
    }

    fn paint(&self, app_context: &mut AppContext, children: &[View]) -> View {
        let image = app_context.load_image(self.style.background_image.clone());
        let material = Material::Quad(Rc::new(self.style.clone()), image);
        match self.style.position {
            StylePosition::Absolute => {
                let mut style = self.style.clone();
                style.position = StylePosition::Static;
                let material = Some(material);
                Rc::new(Positioned {
                    position: (style.left.unwrap_or(0.), style.top.unwrap_or(0.)).into(),
                    child: Rc::new(Flex {
                        style,
                        material,
                        text_material: None,
                        children: children.to_vec(),
                    }),
                })
            }
            StylePosition::Static => {
                let style = self.style.clone();
                let material = Some(material);
                Rc::new(Flex {
                    style,
                    material,
                    text_material: None,
                    children: children.to_vec(),
                })
            }
        }
    }
}

impl From<Div> for Box<dyn Element> {
    fn from(value: Div) -> Self {
        Box::new(value)
    }
}

impl From<Div> for Rc<dyn Render> {
    fn from(value: Div) -> Self {
        Rc::new(value)
    }
}

pub fn div() -> Div {
    Div::new()
}
