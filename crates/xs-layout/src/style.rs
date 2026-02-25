use crate::{
    AlignContent, AlignItems, BorderSide, Borders, BoxConstraints, Color, EdgeInsets,
    FlexDirection, FlexWrap, Image, JustifyContent, Layout, LayoutTree, Material, SizedLayoutBox,
    View,
};
use std::rc::Rc;
use xs_math::Vector2;

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
    pub background_image: Option<&'static str>,

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

    pub children: Vec<View>,
    pub material: Option<Material>,
}

impl Style {
    pub fn to_material(&self) -> Material {
        Material {
            borders: self.border,
            fill: self.background_color,
            image: None,
        }
    }

    pub fn to_view(self) -> View
    where
        Self: 'static,
    {
        struct StyleWrapper(Style);

        impl Layout for StyleWrapper {
            fn layout(
                &self,
                tree: &mut LayoutTree,
                constraints: &BoxConstraints,
            ) -> SizedLayoutBox {
                self.0.layout(tree, constraints)
            }

            fn flex_grow(&self) -> f32 {
                self.0.flex_grow()
            }
            fn flex_shrink(&self) -> f32 {
                self.0.flex_shrink()
            }
            fn flex_basis(&self) -> f32 {
                self.0.flex_basis()
            }
            fn align_self(&self) -> AlignItems {
                self.0.align_self()
            }
            fn min_main(&self) -> f32 {
                self.0.min_main()
            }
            fn max_main(&self) -> f32 {
                self.0.max_main()
            }
            fn min_cross(&self) -> f32 {
                self.0.min_cross()
            }
            fn max_cross(&self) -> f32 {
                self.0.max_cross()
            }
            fn margin(&self) -> EdgeInsets {
                self.0.margin()
            }
        }

        Rc::new(StyleWrapper(self)) as View
    }

    pub fn row() -> Style {
        Style {
            flex_direction: FlexDirection::Row,
            ..Default::default()
        }
    }

    pub fn column() -> Style {
        Style {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Borders, View};
    use std::rc::Rc;
    use xs_math::Vector2;

    #[test]
    fn style_as_view() {
        let style = Style {
            width: Some(100.0),
            height: Some(50.0),
            background_color: Color::rgba(1.0, 0.0, 0.0, 1.0),
            border: Borders::none(),
            background_image: None,
            children: vec![],
            material: None,
            ..Default::default()
        };

        let view = style.to_view();
        let mut tree = LayoutTree::new();
        let result = view.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(200.0, 100.0)),
        );

        assert_eq!(result.size.x, 100.0);
        assert_eq!(result.size.y, 50.0);
        assert!(result.material.is_some());
    }

    #[test]
    fn style_fills_parent_when_no_size() {
        let style = Style {
            background_color: Color::rgba(1.0, 0.0, 0.0, 1.0),
            border: Borders::none(),
            background_image: None,
            children: vec![],
            material: None,
            ..Default::default()
        };

        let view = style.to_view();
        let mut tree = LayoutTree::new();
        let result = view.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(200.0, 100.0)),
        );

        assert_eq!(result.size.x, 200.0);
        assert_eq!(result.size.y, 100.0);
    }
}
