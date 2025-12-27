use xs_layout::{
    Axis, BorderSide, Borders, Color, CrossAxisAlignment, EdgeInsets, MainAxisAlignment,
};

use crate::style::{Style, StylePosition};

pub struct Element {
    pub style: Style,
    pub children: Vec<Element>,
}

impl Element {
    pub fn new() -> Self {
        Self {
            style: Style::default(),
            children: Vec::new(),
        }
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

    pub fn heigth(mut self, size: f32) -> Self {
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
    pub fn child(mut self, child: Self) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: Vec<Self>) -> Self {
        self.children.extend(children);
        self
    }
}
