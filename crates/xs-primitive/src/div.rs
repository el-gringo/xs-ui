use std::{any::TypeId, rc::Rc};

use xs_core::{AppContext, Context, Element, Message, Render};
use xs_layout::{
    AlignItems, BorderSide, Borders, BoxConstraints, Color, EdgeInsets, FlexDirection, FlexWrap,
    Image, JustifyContent, Layout, LayoutTree, Material, Positioned, SizedLayoutBox, Style,
    StyleDisplay, StylePosition, View,
};

pub struct DivLayout(pub Style, pub Vec<View>, pub Option<Material>);

impl Layout for DivLayout {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let mut style = self.0.clone();
        style.children = self.1.clone();
        style.material = self.2.clone();
        style.layout(tree, constraints)
    }

    fn flex_grow(&self) -> f32 {
        self.0.flex_grow
    }

    fn flex_shrink(&self) -> f32 {
        self.0.flex_shrink
    }

    fn flex_basis(&self) -> f32 {
        self.0.flex_basis
    }

    fn align_self(&self) -> AlignItems {
        self.0.align_self
    }

    fn min_main(&self) -> f32 {
        self.0.min_width.unwrap_or(0.0)
    }

    fn max_main(&self) -> f32 {
        self.0.max_width.unwrap_or(f32::INFINITY)
    }

    fn min_cross(&self) -> f32 {
        self.0.min_height.unwrap_or(0.0)
    }

    fn max_cross(&self) -> f32 {
        self.0.max_height.unwrap_or(f32::INFINITY)
    }

    fn margin(&self) -> EdgeInsets {
        self.0.margin
    }
}

#[derive(Default, Clone)]
pub struct Div {
    pub id: Option<&'static str>,
    pub style: Style,
    pub text: &'static str,
    pub children: Vec<Rc<Box<dyn Render>>>,
}

impl Div {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn id(mut self, id: &'static str) -> Self {
        self.id = Some(id);
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

    //
    // COLOR
    //
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

    pub fn grow(mut self, grow: f32) -> Self {
        self.style.flex_grow = grow;
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

    //
    // OTHER
    //
    pub fn child(mut self, child: Box<dyn Render>) -> Self {
        self.children.push(Rc::new(child));
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Rc<Box<dyn Render>>>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn text(mut self, text: &'static str) -> Self {
        self.text = text;
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
        self.id.unwrap_or("div")
    }

    fn children(&self) -> &[Rc<Box<dyn Render>>] {
        &self.children
    }

    fn paint(&self, app_context: &mut AppContext, children: &[View]) -> View {
        let image = if let Some(src) = self.style.background_image.clone() {
            app_context
                .textures
                .entry(src.clone())
                .or_insert_with(|| {
                    app_context.messages.push(Message::LoadImage(src));
                    None
                })
                .clone()
        } else {
            Option::<Image>::None
        };

        let material = Material {
            borders: self.style.border,
            fill: self.style.background_color,
            image,
        };

        match self.style.position {
            StylePosition::Absolute => {
                let mut style = self.style.clone();
                style.position = StylePosition::Static;
                style.children = children.to_vec();
                style.material = Some(material.clone());
                Rc::new(Positioned {
                    position: (style.left.unwrap_or(0.), style.top.unwrap_or(0.)).into(),
                    child: Rc::new(DivLayout(style, vec![], Some(material.clone()))),
                })
            }
            StylePosition::Static => {
                let mut style = self.style.clone();
                style.material = Some(material.clone());
                Rc::new(DivLayout(style, children.to_vec(), Some(material.clone())))
            }
        }
    }
}

impl From<Div> for Box<dyn Element> {
    fn from(value: Div) -> Self {
        Box::new(value)
    }
}

impl Into<Box<dyn Render>> for Div {
    fn into(self) -> Box<dyn Render> {
        Box::new(self)
    }
}

impl Into<Rc<Box<dyn Render>>> for Div {
    fn into(self) -> Rc<Box<dyn Render>> {
        Rc::new(Box::new(self))
    }
}

pub fn div() -> Div {
    Div::new()
}
