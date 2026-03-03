use crate::{
    AlignContent, AlignItems, BorderSide, Borders, Color, EdgeInsets, FlexDirection, FlexWrap,
    JustifyContent, colors::BLACK,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl Default for FontStyle {
    fn default() -> Self {
        FontStyle::Normal
    }
}

#[derive(Clone, Debug)]
pub struct TextPropertiesInner {
    pub color: Color,
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: u32,
    pub font_style: FontStyle,
}

impl Default for TextPropertiesInner {
    fn default() -> Self {
        Self {
            color: BLACK,
            font_family: String::from("sans-serif"),
            font_size: 16.,
            font_weight: 400,
            font_style: FontStyle::Normal
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct TextProperties(pub Option<TextPropertiesInner>);


impl TextProperties {
    pub fn color(&mut self, color: Color) {
        self.update_field(|prop| prop.color = color);
    }

    pub fn font_family(&mut self, font_family: String) {
        self.update_field(|prop| prop.font_family = font_family);
    }

    pub fn font_size(&mut self, font_size: f32) {
        self.update_field(|prop| prop.font_size = font_size);
    }

    pub fn font_weight(&mut self, font_weight: u32) {
        self.update_field(|prop| prop.font_weight = font_weight);
    }

    pub fn unwrap_or_default(&self) -> TextPropertiesInner {
        self.0.clone().unwrap_or_default()
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn replace(&mut self, new_text_properties: TextProperties) {
        self.0 = new_text_properties.0.clone();
    }

    fn update_field(&mut self, callback: impl FnOnce(&mut TextPropertiesInner)) {
        match self.0.as_mut() {
            Some(mut v) => callback(&mut v),
            None => {
                let v = Default::default();
                self.0.replace(v);
                self.update_field(callback);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum StyleDisplay {
    Flex,
    Block,
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
    pub text_properties: TextProperties,
}
