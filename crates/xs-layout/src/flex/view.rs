use crate::{FlexDirection, Material, Style, View};

#[derive(Default, Clone)]
pub struct Flex {
    pub style: Style,
    pub children: Vec<View>,
    pub material: Option<Material>,
}

impl Flex {
    #[must_use]
    pub fn row() -> Flex {
        Flex {
            style: Style {
                flex_direction: FlexDirection::Row,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[must_use]
    pub fn column() -> Flex {
        Flex {
            style: Style {
                flex_direction: FlexDirection::Row,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[must_use]
    pub fn is_row(&self) -> bool {
        matches!(
            self.style.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        )
    }

    #[must_use]
    pub fn is_reverse(&self) -> bool {
        matches!(
            self.style.flex_direction,
            FlexDirection::ColumnReverse | FlexDirection::RowReverse
        )
    }
}
