use std::{any::TypeId, rc::Rc};

use crate::style::{Style, StyleDisplay, StylePosition};
use xs_backend_core::{AppContext, Element, Render};
use xs_layout::{
    Axis, BorderSide, Borders, Color, Container, CrossAxisAlignment, EdgeInsets, Flex, MainAxisAlignment, Positioned, View
};

#[derive(Default, Clone)]
pub struct Div {
    pub id: Option<&'static str>,
    pub style: Style,
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
    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.style.padding = padding;
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

    pub fn background_image(mut self, src: &'static str) -> Self {
        self.style.background_image = Some(src);
        self
    }

    //
    // FLEX
    //
    pub fn row(mut self) -> Self {
        self.style.flex_direction = Axis::Row;
        self
    }

    pub fn column(mut self) -> Self {
        self.style.flex_direction = Axis::Column;
        self
    }

    pub fn justify_content(mut self, value: MainAxisAlignment) -> Self {
        self.style.justify_content = value;
        self
    }

    pub fn align_items(mut self, value: CrossAxisAlignment) -> Self {
        self.style.align_items = value;
        self
    }

    pub fn grow(mut self) -> Self {
        self.style.flex_grow = true;
        self
    }

    pub fn shrink(mut self) -> Self {
        self.style.flex_shrink = true;
        self
    }

    //
    // OTHER
    //
    pub fn child(mut self, child: Box<dyn Render>) -> Self {
        self.children.push(Rc::new(child));
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Box<dyn Render>>) -> Self {
        let iterator = children.into_iter().map(|child| Rc::new(child));
        self.children.extend(iterator);
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
        TypeId::of::<Div>()
    }

    fn render(&self, _ctx: &mut AppContext) -> Box<dyn Element> {
        Box::new(self.clone())
    }
}


impl Element for Div {
    fn id(&self) -> &'static str {
        self.id.unwrap_or("div")
    }

    fn children(&self) -> Vec<Rc<Box<dyn Render>>> {
        self.children.iter().map(|child| child.clone()).collect()
    }

    fn paint(&self, ctx: &mut AppContext, children: Vec<View>) -> View {
        let style = self.style;
        let child: Option<View> = if children.len() > 0 {
            Some(match style.display {
                StyleDisplay::Flex => Box::new(Flex {
                    axis: style.flex_direction,
                    // main_axis_size: if style.flex_grow {
                    //     MainAxisSize::Max
                    // } else {
                    //     MainAxisSize::Min
                    // },
                    main_axis_alignment: style.justify_content,
                    cross_axis_alignment: style.align_items,
                    children,
                    ..Flex::default()
                }),
                StyleDisplay::Static => todo!(),
                StyleDisplay::Inline => todo!(),
            })
        } else {
            None
        };

        let image = if style.background_image.is_some() {
            let src = style.background_image.unwrap();
            let ptr = src.as_bytes().as_ptr() as u32;
            if ctx.ressources.get(&ptr).is_none() {
                ctx.load_image(src);
            }
            ctx.ressources.entry(ptr).or_insert(None).clone()
        } else {
            None
        };
        let container = Box::new(Container {
            image,
            color: style.background_color,
            width: style.width,
            height: style.height,
            margin: style.margin,
            borders: style.border,
            padding: style.padding,
            child,
            ..Container::default()
        });
        let result = match style.position {
            StylePosition::Absolute => Box::new(Positioned {
                position: (style.left.unwrap_or(0.), style.top.unwrap_or(0.)).into(),
                child: container,
            }),
            StylePosition::Static => container as View,
        };
        result
    }
}

impl From<Div> for Box<dyn Element> {
    fn from(value: Div) -> Self {
        Box::new(value)
    }
}

// impl Into<Box<dyn Element>> for Div {
//     fn into(self) -> Box<dyn Element> {
//         Box::new(self)
//     }
// }

impl Into<Box<dyn Render>> for Div  {
    fn into(self) -> Box<dyn Render> {
        Box::new(self)
    }
}

pub fn div() -> Div {
    Div::new()
}
