use xs_layout::{
    Axis, BorderSide, Borders, Color, CrossAxisAlignment, EdgeInsets, MainAxisAlignment
};

#[derive(Clone, Copy)]
pub enum StyleDisplay {
    Flex,
    Static,
    Inline,
}

impl Default for StyleDisplay {
    fn default() -> Self {
        Self::Flex
    }
}

#[derive(Clone, Copy)]
pub enum StylePosition {
    Static,
    Absolute,
}

impl Default for StylePosition {
    fn default() -> Self {
        Self::Static
    }
}

#[derive(Clone, Copy)]
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
