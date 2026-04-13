use crate::{
    AlignContent, AlignItems, BorderSide, Borders, Color, EdgeInsets, FlexDirection, FlexWrap,
    JustifyContent,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextProperties {
    pub font_family: String,
    pub font_size: u16,
    pub font_weight: u16,
    pub font_style: FontStyle,
}

impl Default for TextProperties {
    fn default() -> Self {
        Self {
            font_family: String::from("sans-serif"),
            font_size: 16,
            font_weight: 400,
            font_style: FontStyle::Normal,
        }
    }
}

impl TextProperties {
    pub fn new(
        font_family: String,
        font_size: u16,
        font_weight: u16,
        font_style: FontStyle,
    ) -> Self {
        Self {
            font_family,
            font_size,
            font_weight,
            font_style,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum StyleDisplay {
    #[default]
    Flex,
    Block,
    Inline,
}

#[derive(Clone, Copy, Default)]
pub enum StylePosition {
    #[default]
    Static,
    Absolute,
}

#[derive(Clone, Default)]
pub struct Style {
    pub background_color: Color,
    pub background_image: Option<String>,

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
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,

    pub display: StyleDisplay,

    // Flex properties
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub align_content: AlignContent,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,
    pub row_gap: f32,
    pub gap: f32,
    pub columns_gap: f32,
    pub order: u32,
    pub align_self: AlignItems,
    pub flex_basis: f32,
    pub flex_grow: f32,
    pub flex_shrink: f32,

    // Text properties
    pub text_properties: Option<TextProperties>,
}
