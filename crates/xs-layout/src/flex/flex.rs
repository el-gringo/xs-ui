use crate::{
    FlexDirection, Material,
    Style, View,
};

#[derive(Default, Clone)]
pub struct Flex {
    pub style: Style,
    pub children: Vec<View>,
    pub material: Option<Material>,
    pub text_material: Option<Material>,
}

impl Flex {
    pub fn row() -> Flex {
        Flex {
            style: Style {
                flex_direction: FlexDirection::Row,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn column() -> Flex {
        Flex {
            style: Style {
                flex_direction: FlexDirection::Row,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn is_row(&self) -> bool {
        matches!(
            self.style.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        )
    }

    pub fn is_reverse(&self) -> bool {
        matches!(
            self.style.flex_direction,
            FlexDirection::ColumnReverse | FlexDirection::RowReverse
        )
    }
}
