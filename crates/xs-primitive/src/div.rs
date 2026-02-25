use std::{any::TypeId, rc::Rc};

use xs_core::{Application, Context, Element, Render};
use xs_layout::{
    AlignItems, BorderSide, Borders, BoxConstraints, Color, EdgeInsets, FlexDirection,
    JustifyContent, Layout, LayoutTree, Material, Positioned, SizedLayoutBox, Style, StyleDisplay,
    StylePosition, View,
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

    fn paint(&self, _app: &mut Application, children: &[View]) -> View {
        let material = Material {
            borders: self.style.border,
            fill: self.style.background_color,
            image: None,
        };

        match self.style.position {
            StylePosition::Absolute => {
                let mut style = self.style.clone();
                style.position = StylePosition::Static;
                style.children = children.to_vec();
                style.material = Some(material);
                Rc::new(Positioned {
                    position: (style.left.unwrap_or(0.), style.top.unwrap_or(0.)).into(),
                    child: Rc::new(DivLayout(style, vec![], None)),
                })
            }
            StylePosition::Static => {
                let mut style = self.style.clone();
                style.material = Some(material);
                Rc::new(DivLayout(style, children.to_vec(), None))
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

#[cfg(test)]
mod tests {
    use super::*;
    use xs_core::{Renderer, RendererDummy};
    use xs_layout::{BoxConstraints, LayoutBox, LayoutTree, Vector2};

    fn build_app(app: &Application) -> Rc<Box<dyn Render>> {
        let sidebar_header: Rc<Box<dyn Render>> = div()
            .row()
            .height(60.)
            .background_color(Color::rgba(0.9, 0.9, 0.9, 1.0))
            .into();

        let btn1: Rc<Box<dyn Render>> = div()
            .width(218.)
            .height(40.)
            .background_color(Color::rgba(0.8, 0.8, 0.8, 1.0))
            .border(Color::rgba(0.5, 0.5, 0.5, 1.0), 1.0)
            .into();

        let btn2: Rc<Box<dyn Render>> = div()
            .width(218.)
            .height(40.)
            .background_color(Color::rgba(0.8, 0.8, 0.8, 1.0))
            .border(Color::rgba(0.5, 0.5, 0.5, 1.0), 1.0)
            .into();

        let btn3: Rc<Box<dyn Render>> = div()
            .width(218.)
            .height(40.)
            .background_color(Color::rgba(0.8, 0.8, 0.8, 1.0))
            .border(Color::rgba(0.5, 0.5, 0.5, 1.0), 1.0)
            .into();

        let sidebar_body: Rc<Box<dyn Render>> = div()
            .column()
            .padding(16.)
            .justify_content(JustifyContent::Center)
            .children(vec![btn1, btn2, btn3])
            .into();

        let sidebar: Rc<Box<dyn Render>> = div()
            .column()
            .width(250.)
            .height(900.)
            .background_color(Color::rgba(0.85, 0.85, 0.85, 1.0))
            .children(vec![sidebar_header, sidebar_body])
            .into();

        let app_header: Rc<Box<dyn Render>> = div()
            .row()
            .height(60.)
            .background_color(Color::rgba(0.95, 0.95, 0.95, 1.0))
            .into();

        let card1: Rc<Box<dyn Render>> = div()
            .width(280.)
            .height(200.)
            .background_color(Color::rgba(1.0, 1.0, 1.0, 1.0))
            .border(Color::rgba(0.7, 0.7, 0.7, 1.0), 1.0)
            .into();

        let card2: Rc<Box<dyn Render>> = div()
            .width(280.)
            .height(200.)
            .background_color(Color::rgba(1.0, 1.0, 1.0, 1.0))
            .border(Color::rgba(0.7, 0.7, 0.7, 1.0), 1.0)
            .into();

        let card3: Rc<Box<dyn Render>> = div()
            .width(280.)
            .height(200.)
            .background_color(Color::rgba(1.0, 1.0, 1.0, 1.0))
            .border(Color::rgba(0.7, 0.7, 0.7, 1.0), 1.0)
            .into();

        let card4: Rc<Box<dyn Render>> = div()
            .width(280.)
            .height(200.)
            .background_color(Color::rgba(1.0, 1.0, 1.0, 1.0))
            .border(Color::rgba(0.7, 0.7, 0.7, 1.0), 1.0)
            .into();

        let cards_grid: Rc<Box<dyn Render>> = div()
            .row()
            .padding(24.)
            .children(vec![card1, card2, card3, card4])
            .into();

        let app_body: Rc<Box<dyn Render>> = div()
            .column()
            .grow(1.0)
            .children(vec![app_header, cards_grid])
            .into();

        div()
            .width(app.width as f32)
            .height(app.height as f32)
            .row()
            .children(vec![sidebar, app_body])
            .into()
    }

    #[test]
    fn div_real_world_layout() {
        let mut renderer = RendererDummy;
        let mut app = Application::new().width(1440).height(900);
        let widget = build_app(&app);
        let widget_tree = renderer.render(&mut app, widget);
        let constraints = BoxConstraints {
            min: Vector2::new(0.0, 0.0),
            max: Vector2::new(app.width as f32, app.height as f32),
        };
        let mut tree = LayoutTree::new();
        let result = widget_tree.layout(&mut tree, &constraints);
        let root_sbox = LayoutBox::from_child(result.clone(), (0., 0.));
        let root_id = tree.insert(root_sbox);
        tree.set_root(Some(root_id));

        let sidebar_box = tree.get(result.children[0]).unwrap();
        let app_body_box = tree.get(result.children[1]).unwrap();
        let _app_header = tree.get(app_body_box.children[0]).unwrap();

        // Test sidebar - should have correct height now!
        assert_eq!(sidebar_box.bounds.x, 0.0);
        assert_eq!(sidebar_box.bounds.y, 0.0);
        assert_eq!(
            sidebar_box.bounds.width, 250.0,
            "sidebar width should be 250"
        );
        assert_eq!(
            sidebar_box.bounds.height, 900.0,
            "sidebar height should be 900"
        );

        // Test sidebar material/color
        assert!(
            sidebar_box.material.is_some(),
            "sidebar should have material"
        );
        let sidebar_fill = &sidebar_box.material.as_ref().unwrap().fill;
        assert_eq!(sidebar_fill.r, 0.85, "sidebar should have gray fill");

        // Test app_body - should fill remaining space
        assert_eq!(app_body_box.bounds.x, 250.0);
        assert_eq!(app_body_box.bounds.y, 0.0);
        assert_eq!(
            app_body_box.bounds.width, 1190.0,
            "app body width should be 1190"
        );
        assert_eq!(
            app_body_box.bounds.height, 900.0,
            "app body height should be 900"
        );
    }
}
