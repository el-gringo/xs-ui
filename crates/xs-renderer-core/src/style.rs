use xs_layout::{
    Axis, BorderSide, Borders, Color, CrossAxisAlignment, EdgeInsets, MainAxisAlignment,
};

pub enum StyleDisplay {
    Flex,
    Static,
    Inline
}

impl Default for StyleDisplay {
    fn default() -> Self {
        Self::Flex
    }
}

pub enum StylePosition {
    Static,
    Absolute,
}

impl Default for StylePosition {
    fn default() -> Self {
        Self::Static
    }
}

pub struct Style {
    pub background_color: Color,

    pub border: Borders,
    pub border_top: Option<BorderSide>,
    pub border_right: Option<BorderSide>,
    pub border_bottom: Option<BorderSide>,
    pub border_left: Option<BorderSide>,

    pub margin: EdgeInsets,
    pub margin_top: Option<f32>,
    pub margin_right: Option<f32>,
    pub margin_bottom: Option<f32>,
    pub margin_left: Option<f32>,

    pub padding: EdgeInsets,
    pub padding_top: Option<f32>,
    pub padding_right: Option<f32>,
    pub padding_bottom: Option<f32>,
    pub padding_left: Option<f32>,

    pub position: StylePosition,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,

    pub display: StyleDisplay,
    pub flex_direction: Axis,
    pub flex_grow: bool,
    pub flex_shrink: bool,
    pub justify_content: MainAxisAlignment,
    pub align_items: CrossAxisAlignment,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background_color: Default::default(),
            border: Default::default(),
            border_top: Default::default(),
            border_right: Default::default(),
            border_bottom: Default::default(),
            border_left: Default::default(),
            margin: Default::default(),
            margin_top: Default::default(),
            margin_right: Default::default(),
            margin_bottom: Default::default(),
            margin_left: Default::default(),
            padding: Default::default(),
            padding_top: Default::default(),
            padding_right: Default::default(),
            padding_bottom: Default::default(),
            padding_left: Default::default(),
            display: Default::default(),
            position: Default::default(),
            top: Default::default(),
            right: Default::default(),
            bottom: Default::default(),
            left: Default::default(),
            width: Default::default(),
            height: Default::default(),
            flex_direction: Default::default(),
            flex_grow: Default::default(),
            flex_shrink: Default::default(),
            justify_content: Default::default(),
            align_items: Default::default(),
        }
    }
}

impl Style {
    pub fn prepare(&mut self) {
        if self.border_top.is_some() {
            self.border.top = self.border_top;
        }
        if self.border_right.is_some() {
            self.border.right = self.border_right;
        }
        if self.border_bottom.is_some() {
            self.border.bottom = self.border_bottom;
        }
        if self.border_left.is_some() {
            self.border.left = self.border_left;
        }

        if self.margin_top.is_some() {
            self.margin.top = self.margin_top.unwrap();
        }
        if self.margin_right.is_some() {
            self.margin.right = self.margin_right.unwrap();
        }
        if self.margin_bottom.is_some() {
            self.margin.bottom = self.margin_bottom.unwrap();
        }
        if self.margin_left.is_some() {
            self.margin.left = self.margin_left.unwrap();
        }

        if self.padding_top.is_some() {
            self.padding.top = self.padding_top.unwrap();
        }
        if self.padding_right.is_some() {
            self.padding.right = self.padding_right.unwrap();
        }
        if self.padding_bottom.is_some() {
            self.padding.bottom = self.padding_bottom.unwrap();
        }
        if self.padding_left.is_some() {
            self.padding.left = self.padding_left.unwrap();
        }
    }
}
