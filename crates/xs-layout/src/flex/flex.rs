use crate::{
    AlignContent, AlignItems, Borders, EdgeInsets, FlexDirection, FlexWrap, JustifyContent,
    Material, View,
};

#[derive(Default)]
pub struct Flex {
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
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub margin: EdgeInsets,
    pub padding: EdgeInsets,
    pub borders: Borders,
    pub material: Option<Material>,
    pub children: Vec<View>,
}

impl Flex {
    pub fn new() -> Self {
        Flex::default()
    }

    pub fn row() -> Self {
        Flex {
            flex_direction: FlexDirection::Row,
            ..Default::default()
        }
    }

    pub fn column() -> Self {
        Flex {
            flex_direction: FlexDirection::Column,
            ..Default::default()
        }
    }

    pub fn is_row(&self) -> bool {
        matches!(
            self.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        )
    }

    pub fn is_reverse(&self) -> bool {
        matches!(
            self.flex_direction,
            FlexDirection::ColumnReverse | FlexDirection::RowReverse
        )
    }
}
