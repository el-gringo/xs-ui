use crate::{
    tree::{LayoutBox, LayoutTree},
    AlignItems, BoxConstraints, EdgeInsets, FlexWrap, JustifyContent, Layout, LayoutBoxId,
    Material, SizedLayoutBox, Style,
};
use xs_math::{Rectangle, Vector2};

struct Item {
    size: Vector2,
    flex_grow: f32,
    min_main: f32,
    max_main: f32,
    children: Vec<LayoutBoxId>,
    material: Option<Material>,
    margin: EdgeInsets,
}

pub trait FlexItemLayout {
    fn flex_grow(&self) -> f32;
    fn flex_shrink(&self) -> f32;
    fn flex_basis(&self) -> f32;
    fn align_self(&self) -> AlignItems;
    fn min_main(&self) -> f32;
    fn max_main(&self) -> f32;
    fn min_cross(&self) -> f32;
    fn max_cross(&self) -> f32;
}

impl Layout for Style {
    fn flex_grow(&self) -> f32 {
        self.flex_grow
    }

    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let main_is_x = self.is_row();
        let is_wrap = matches!(self.flex_wrap, FlexWrap::Wrap | FlexWrap::WrapReverse);
        let is_reverse = self.is_reverse();
        let gap = if self.is_row() {
            self.columns_gap
        } else {
            self.row_gap
        }
        .max(self.gap);

        let w = self.width.unwrap_or(constraints.max.x);
        let h = self.height.unwrap_or(constraints.max.y);
        eprintln!(
            "DEBUG flex: w={}, h={}, explicit_h={:?}, is_column={}, children={}",
            w,
            h,
            self.height,
            self.is_row(),
            self.children.len()
        );
        let pw = self.padding.left + self.padding.right + self.border.total_width();
        let ph = self.padding.top + self.padding.bottom + self.border.total_height();
        let inner = Vector2::new((w - pw).max(0.0), (h - ph).max(0.0));
        let container_main = if main_is_x { inner.x } else { inner.y };
        let inner_constraints = BoxConstraints::from_max(inner);

        let mut items: Vec<Item> = self
            .children
            .iter()
            .map(|child| {
                let child_constraints = if child.flex_grow() > 0.0 {
                    if main_is_x {
                        BoxConstraints::from_max(Vector2::new(0.0, inner.y))
                    } else {
                        BoxConstraints::from_max(Vector2::new(inner.x, 0.0))
                    }
                } else {
                    inner_constraints.clone()
                };
                let layout = child.layout(tree, &child_constraints);
                Item {
                    size: layout.size,
                    flex_grow: child.flex_grow(),
                    min_main: child.min_main(),
                    max_main: child.max_main(),
                    children: layout.children,
                    material: layout.material,
                    margin: child.margin(),
                }
            })
            .collect();

        if is_reverse {
            items.reverse();
        }

        let mut lines: Vec<Vec<Item>> = if is_wrap {
            let mut lines = Vec::new();
            let mut current = Vec::new();
            let mut current_main = 0.0f32;
            for item in items.drain(..) {
                let item_main = if main_is_x { item.size.x } else { item.size.y };
                if !current.is_empty() && current_main + gap + item_main > container_main {
                    lines.push(current);
                    current = Vec::new();
                    current_main = 0.0;
                }
                if !current.is_empty() {
                    current_main += gap;
                }
                current_main += item_main;
                current.push(item);
            }
            if !current.is_empty() {
                lines.push(current);
            }
            lines
        } else {
            vec![items]
        };

        let total_main: f32 = lines
            .iter()
            .map(|l| {
                l.iter()
                    .map(|i| if main_is_x { i.size.x } else { i.size.y })
                    .sum::<f32>()
                    + gap * (l.len().saturating_sub(1)) as f32
            })
            .sum();

        let free_space = (container_main - total_main).max(0.0);
        let total_grow: f32 = lines
            .iter()
            .flat_map(|l| l.iter())
            .map(|i| i.flex_grow)
            .sum();
        if total_grow > 0.0 && free_space > 0.0 {
            for line in &mut lines {
                for item in line.iter_mut() {
                    if item.flex_grow > 0.0 {
                        let grow = free_space * item.flex_grow / total_grow;
                        let current = if main_is_x { item.size.x } else { item.size.y };
                        let new_main = (current + grow).clamp(item.min_main, item.max_main);
                        if main_is_x {
                            item.size.x = new_main
                        } else {
                            item.size.y = new_main
                        };
                    }
                }
            }
        }

        let mut child_ids = Vec::new();
        let padding_min = self.padding.min();
        let justify_offset = match self.justify_content {
            JustifyContent::Center => (container_main - total_main) / 2.0,
            JustifyContent::FlexEnd => (container_main - total_main).max(0.0),
            _ => 0.0,
        };
        let mut main_pos = padding_min.x + padding_min.y + justify_offset;

        for line in &lines {
            let line_main: f32 = line
                .iter()
                .map(|i| if main_is_x { i.size.x } else { i.size.y })
                .sum();
            let _line_cross = line
                .iter()
                .map(|i| if main_is_x { i.size.y } else { i.size.x })
                .fold(0.0f32, |a, b| a.max(b));

            let start_offset = match self.justify_content {
                JustifyContent::Center => (container_main - line_main) / 2.0,
                JustifyContent::FlexEnd => (container_main - line_main).max(0.0),
                _ => 0.0,
            };

            let container_cross = if main_is_x { inner.y } else { inner.x };

            let mut cursor = main_pos + start_offset;
            for item in line {
                let item_main = if main_is_x { item.size.x } else { item.size.y };
                let item_cross = if main_is_x { item.size.y } else { item.size.x };

                let margin_main_start = if main_is_x {
                    item.margin.left
                } else {
                    item.margin.top
                };
                let margin_main_end = if main_is_x {
                    item.margin.right
                } else {
                    item.margin.bottom
                };
                let margin_cross_start = if main_is_x {
                    item.margin.top
                } else {
                    item.margin.left
                };
                let margin_cross_end = if main_is_x {
                    item.margin.bottom
                } else {
                    item.margin.right
                };

                let cross_offset = match self.align_items {
                    AlignItems::Center => {
                        (container_cross - item_cross - margin_cross_start - margin_cross_end) / 2.0
                    }
                    AlignItems::FlexEnd => {
                        (container_cross - item_cross - margin_cross_start - margin_cross_end)
                            .max(0.0)
                    }
                    _ => 0.0,
                };

                let px = if main_is_x {
                    cursor + margin_main_start
                } else {
                    padding_min.x + cross_offset + margin_cross_start
                };
                let py = if main_is_x {
                    padding_min.y + cross_offset + margin_cross_start
                } else {
                    cursor + margin_main_start
                };

                let box_id = tree.insert(LayoutBox {
                    bounds: Rectangle::new(px, py, item.size.x, item.size.y),
                    margin: item.margin,
                    children: item.children.clone(),
                    material: item.material.clone(),
                });
                child_ids.push(box_id);
                cursor += item_main + margin_main_start + margin_main_end + gap;
            }
            main_pos += line_main + gap;
        }

        if is_reverse {
            child_ids.reverse();
        }

        let content_w: f32 = lines
            .iter()
            .flat_map(|l| l.iter())
            .map(|i| i.size.x)
            .fold(0.0f32, |a, b| a.max(b));
        let content_h: f32 = lines
            .iter()
            .map(|l| {
                l.iter().map(|i| i.size.y).sum::<f32>() + gap * (l.len().saturating_sub(1)) as f32
            })
            .sum();

        let final_w = self.width.unwrap_or(w);
        let final_h = self.height.unwrap_or(h);

        let final_w = final_w
            .max(self.min_width.unwrap_or(0.0))
            .min(self.max_width.unwrap_or(f32::INFINITY));
        let final_h = final_h
            .max(self.min_height.unwrap_or(0.0))
            .min(self.max_height.unwrap_or(f32::INFINITY));

        tree.insert(LayoutBox {
            bounds: Rectangle::new(0.0, 0.0, final_w, final_h),
            margin: self.margin,
            children: child_ids.clone(),
            material: self.material.clone(),
        });

        if self.height.is_some() {
            eprintln!(
                "DEBUG flex: final_h={}, children_y positions will be calculated",
                final_h
            );
        }

        SizedLayoutBox {
            margin: self.margin,
            size: Vector2::new(final_w, final_h),
            children: child_ids,
            material: self.material.clone().or_else(|| {
                Some(Material {
                    borders: self.border,
                    fill: self.background_color,
                    image: None,
                })
            }),
        }
    }
    fn margin(&self) -> EdgeInsets {
        self.margin
    }
}

impl FlexItemLayout for Style {
    fn flex_grow(&self) -> f32 {
        self.flex_grow
    }
    fn flex_shrink(&self) -> f32 {
        self.flex_shrink
    }
    fn flex_basis(&self) -> f32 {
        self.flex_basis
    }
    fn align_self(&self) -> AlignItems {
        self.align_self
    }
    fn min_main(&self) -> f32 {
        if self.is_row() {
            self.min_width.unwrap_or(0.0)
        } else {
            self.min_height.unwrap_or(0.0)
        }
    }
    fn max_main(&self) -> f32 {
        if self.is_row() {
            self.max_width.unwrap_or(f32::INFINITY)
        } else {
            self.max_height.unwrap_or(f32::INFINITY)
        }
    }
    fn min_cross(&self) -> f32 {
        if self.is_row() {
            self.min_height.unwrap_or(0.0)
        } else {
            self.min_width.unwrap_or(0.0)
        }
    }
    fn max_cross(&self) -> f32 {
        if self.is_row() {
            self.max_height.unwrap_or(f32::INFINITY)
        } else {
            self.max_width.unwrap_or(f32::INFINITY)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Borders, Color};
    use std::rc::Rc;

    fn child(w: f32, h: f32) -> Style {
        Style {
            width: Some(w),
            height: Some(h),
            material: Some(Material {
                borders: Borders::none(),
                fill: Color::rgba(1.0, 0.0, 0.0, 1.0),
                image: None,
            }),
            ..Default::default()
        }
    }

    fn child_with_margin(w: f32, h: f32, margin: EdgeInsets) -> Style {
        Style {
            width: Some(w),
            height: Some(h),
            margin,
            material: Some(Material {
                borders: Borders::none(),
                fill: Color::rgba(1.0, 0.0, 0.0, 1.0),
                image: None,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn empty_flex_fills_parent() {
        let flex = Style::row();
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(100.0, 100.0)),
        );
        assert_eq!(result.size.x, 100.0);
        assert_eq!(result.size.y, 100.0);
    }

    #[test]
    fn row_with_explicit_size() {
        let mut flex = Style::row();
        flex.width = Some(200.0);
        flex.height = Some(100.0);
        flex.children.push(Rc::new(child(50.0, 50.0)));
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        assert_eq!(result.size.x, 200.0);
        assert_eq!(result.size.y, 100.0);
    }

    #[test]
    fn column_with_explicit_size() {
        let mut flex = Style::column();
        flex.width = Some(100.0);
        flex.height = Some(200.0);
        flex.children.push(Rc::new(child(50.0, 50.0)));
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        assert_eq!(result.size.x, 100.0);
        assert_eq!(result.size.y, 200.0);
    }

    #[test]
    fn column_children_positioned_vertically() {
        let mut flex = Style::column();
        flex.width = Some(100.0);
        flex.height = Some(200.0);
        flex.children.push(Rc::new(child(50.0, 50.0)));
        flex.children.push(Rc::new(child(50.0, 50.0)));
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();
        assert_eq!(c1.bounds.x, 0.0);
        assert_eq!(c1.bounds.y, 0.0);
        assert_eq!(c1.bounds.width, 50.0);
        assert_eq!(c1.bounds.height, 50.0);
        assert_eq!(c2.bounds.x, 0.0);
        assert_eq!(c2.bounds.y, 50.0);
        assert_eq!(c2.bounds.width, 50.0);
        assert_eq!(c2.bounds.height, 50.0);
    }

    #[test]
    fn flex_wrap_creates_new_line() {
        let mut flex = Style::row();
        flex.flex_wrap = FlexWrap::Wrap;
        flex.width = Some(100.0);
        flex.height = Some(200.0);
        flex.children.push(Rc::new(child(60.0, 50.0)));
        flex.children.push(Rc::new(child(60.0, 50.0)));
        let mut tree = LayoutTree::new();
        let _ = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        assert!(tree.boxes.len() > 2);
    }

    #[test]
    fn child_has_material_from_layout() {
        let mut flex = Style::row();
        flex.width = Some(100.0);
        flex.height = Some(50.0);
        flex.children.push(Rc::new(child(50.0, 30.0)));
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c = tree.get(result.children[0]).unwrap();
        assert!(c.material.is_some());
    }

    #[test]
    fn nested_flex_layout() {
        let mut outer = Style::column();
        outer.width = Some(200.0);
        outer.height = Some(200.0);

        let mut inner = Style::row();
        inner.width = Some(100.0);
        inner.height = Some(50.0);
        inner.children.push(Rc::new(child(30.0, 20.0)));
        inner.children.push(Rc::new(child(30.0, 20.0)));

        outer.children.push(Rc::new(inner));
        outer.children.push(Rc::new(child(50.0, 30.0)));

        let mut tree = LayoutTree::new();
        let result = outer.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(1920.0, 1080.0)),
        );

        assert_eq!(result.size.x, 200.0);
        assert_eq!(result.size.y, 200.0);
        assert!(result.children.len() >= 2);

        let inner_box = tree.get(result.children[0]).unwrap();
        assert_eq!(inner_box.bounds.x, 0.0);
        assert_eq!(inner_box.bounds.y, 0.0);
        assert_eq!(inner_box.bounds.width, 100.0);
        assert_eq!(inner_box.bounds.height, 50.0);
    }

    #[test]
    fn nested_flex_child_count() {
        let mut outer = Style::row();
        outer.width = Some(200.0);
        outer.height = Some(100.0);

        let mut inner = Style::column();
        inner.width = Some(80.0);
        inner.height = Some(80.0);
        inner.children.push(Rc::new(child(20.0, 20.0)));
        inner.children.push(Rc::new(child(20.0, 20.0)));
        inner.children.push(Rc::new(child(20.0, 20.0)));

        outer.children.push(Rc::new(inner));
        outer.children.push(Rc::new(child(40.0, 40.0)));

        let mut tree = LayoutTree::new();
        let result = outer.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );

        let outer_box = tree.get(result.children[0]).unwrap();

        assert_eq!(
            outer_box.children.len(),
            3,
            "inner column should have 3 children"
        );
        assert_eq!(result.children.len(), 2, "outer row should have 2 children");

        println!("Tree boxes count: {}", tree.boxes.len());
        println!("Outer children: {:?}", result.children);
        println!("Inner column children: {:?}", outer_box.children);
    }

    #[test]
    fn justify_content_center_row() {
        let mut flex = Style::row();
        flex.width = Some(200.0);
        flex.height = Some(100.0);
        flex.justify_content = JustifyContent::Center;
        flex.children.push(Rc::new(child(50.0, 50.0)));
        flex.children.push(Rc::new(child(50.0, 50.0)));
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();
        assert_eq!(c1.bounds.x, 100.0);
        assert_eq!(c1.bounds.y, 0.0);
        assert_eq!(c1.bounds.width, 50.0);
        assert_eq!(c1.bounds.height, 50.0);
        assert_eq!(c2.bounds.x, 150.0);
        assert_eq!(c2.bounds.y, 0.0);
        assert_eq!(c2.bounds.width, 50.0);
        assert_eq!(c2.bounds.height, 50.0);
    }

    #[test]
    fn justify_content_flex_end_row() {
        let mut flex = Style::row();
        flex.width = Some(200.0);
        flex.height = Some(100.0);
        flex.justify_content = JustifyContent::FlexEnd;
        flex.children.push(Rc::new(child(50.0, 50.0)));
        flex.children.push(Rc::new(child(50.0, 50.0)));
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();
        assert_eq!(c1.bounds.x, 200.0);
        assert_eq!(c1.bounds.y, 0.0);
        assert_eq!(c1.bounds.width, 50.0);
        assert_eq!(c1.bounds.height, 50.0);
        assert_eq!(c2.bounds.x, 250.0);
        assert_eq!(c2.bounds.y, 0.0);
        assert_eq!(c2.bounds.width, 50.0);
        assert_eq!(c2.bounds.height, 50.0);
    }

    #[test]
    fn justify_content_flex_end_column() {
        let mut flex = Style::column();
        flex.width = Some(100.0);
        flex.height = Some(200.0);
        flex.justify_content = JustifyContent::FlexEnd;
        flex.children.push(Rc::new(child(50.0, 50.0)));
        flex.children.push(Rc::new(child(50.0, 50.0)));
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();
        assert_eq!(c1.bounds.x, 0.0);
        assert_eq!(c1.bounds.y, 200.0);
        assert_eq!(c1.bounds.width, 50.0);
        assert_eq!(c1.bounds.height, 50.0);
        assert_eq!(c2.bounds.x, 0.0);
        assert_eq!(c2.bounds.y, 250.0);
        assert_eq!(c2.bounds.width, 50.0);
        assert_eq!(c2.bounds.height, 50.0);
    }

    #[test]
    fn align_items_center_row() {
        let mut flex = Style::row();
        flex.width = Some(200.0);
        flex.height = Some(100.0);
        flex.align_items = AlignItems::Center;
        flex.children.push(Rc::new(child(50.0, 30.0)));
        flex.children.push(Rc::new(child(50.0, 40.0)));
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();
        assert_eq!(c1.bounds.x, 0.0);
        assert_eq!(c1.bounds.y, 35.0);
        assert_eq!(c1.bounds.width, 50.0);
        assert_eq!(c1.bounds.height, 30.0);
        assert_eq!(c2.bounds.x, 50.0);
        assert_eq!(c2.bounds.y, 30.0);
        assert_eq!(c2.bounds.width, 50.0);
        assert_eq!(c2.bounds.height, 40.0);
    }

    #[test]
    fn align_items_flex_end_row() {
        let mut flex = Style::row();
        flex.width = Some(200.0);
        flex.height = Some(100.0);
        flex.align_items = AlignItems::FlexEnd;
        flex.children.push(Rc::new(child(50.0, 30.0)));
        flex.children.push(Rc::new(child(50.0, 40.0)));
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();
        assert_eq!(c1.bounds.x, 0.0);
        assert_eq!(c1.bounds.y, 70.0);
        assert_eq!(c1.bounds.width, 50.0);
        assert_eq!(c1.bounds.height, 30.0);
        assert_eq!(c2.bounds.x, 50.0);
        assert_eq!(c2.bounds.y, 60.0);
        assert_eq!(c2.bounds.width, 50.0);
        assert_eq!(c2.bounds.height, 40.0);
    }

    #[test]
    fn align_items_center_column() {
        let mut flex = Style::column();
        flex.width = Some(100.0);
        flex.height = Some(200.0);
        flex.align_items = AlignItems::Center;
        flex.children.push(Rc::new(child(30.0, 50.0)));
        flex.children.push(Rc::new(child(40.0, 50.0)));
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();
        assert_eq!(c1.bounds.x, 35.0);
        assert_eq!(c1.bounds.y, 0.0);
        assert_eq!(c1.bounds.width, 30.0);
        assert_eq!(c1.bounds.height, 50.0);
        assert_eq!(c2.bounds.x, 30.0);
        assert_eq!(c2.bounds.y, 50.0);
        assert_eq!(c2.bounds.width, 40.0);
        assert_eq!(c2.bounds.height, 50.0);
    }

    #[test]
    fn align_items_flex_end_column() {
        let mut flex = Style::column();
        flex.width = Some(100.0);
        flex.height = Some(200.0);
        flex.align_items = AlignItems::FlexEnd;
        flex.children.push(Rc::new(child(30.0, 50.0)));
        flex.children.push(Rc::new(child(40.0, 50.0)));
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();
        assert_eq!(c1.bounds.x, 70.0);
        assert_eq!(c1.bounds.y, 0.0);
        assert_eq!(c1.bounds.width, 30.0);
        assert_eq!(c1.bounds.height, 50.0);
        assert_eq!(c2.bounds.x, 60.0);
        assert_eq!(c2.bounds.y, 50.0);
        assert_eq!(c2.bounds.width, 40.0);
        assert_eq!(c2.bounds.height, 50.0);
    }

    #[test]
    fn column_with_centered_items() {
        let mut column = Style::column();
        column.width = Some(300.0);
        column.height = Some(300.0);
        column.align_items = AlignItems::Center;
        column.children.push(Rc::new(child(150.0, 50.0)));
        column.children.push(Rc::new(child(150.0, 50.0)));
        column.children.push(Rc::new(child(150.0, 50.0)));
        let mut tree = LayoutTree::new();
        let result = column.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();
        let c3 = tree.get(result.children[2]).unwrap();
        assert_eq!(c1.bounds.x, 75.0);
        assert_eq!(c1.bounds.y, 0.0);
        assert_eq!(c1.bounds.width, 150.0);
        assert_eq!(c1.bounds.height, 50.0);
        assert_eq!(c2.bounds.x, 75.0);
        assert_eq!(c2.bounds.y, 50.0);
        assert_eq!(c2.bounds.width, 150.0);
        assert_eq!(c2.bounds.height, 50.0);
        assert_eq!(c3.bounds.x, 75.0);
        assert_eq!(c3.bounds.y, 100.0);
        assert_eq!(c3.bounds.width, 150.0);
        assert_eq!(c3.bounds.height, 50.0);
    }

    #[test]
    fn row_with_margin_on_children() {
        let mut flex = Style::row();
        flex.width = Some(200.0);
        flex.height = Some(100.0);
        flex.children.push(Rc::new(child_with_margin(
            50.0,
            50.0,
            EdgeInsets {
                top: 0.0,
                bottom: 0.0,
                left: 10.0,
                right: 0.0,
            },
        )));
        flex.children.push(Rc::new(child_with_margin(
            50.0,
            50.0,
            EdgeInsets {
                top: 0.0,
                bottom: 0.0,
                left: 0.0,
                right: 10.0,
            },
        )));
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();
        assert_eq!(c1.bounds.x, 10.0);
        assert_eq!(c1.bounds.y, 0.0);
        assert_eq!(c1.bounds.width, 50.0);
        assert_eq!(c1.bounds.height, 50.0);
        assert_eq!(c1.margin.left, 10.0);
        assert_eq!(c2.bounds.x, 60.0);
        assert_eq!(c2.bounds.y, 0.0);
        assert_eq!(c2.bounds.width, 50.0);
        assert_eq!(c2.bounds.height, 50.0);
        assert_eq!(c2.margin.right, 10.0);
    }

    #[test]
    fn real_world_app_layout() {
        let mut sidebar = Style::column();
        sidebar.width = Some(250.0);
        sidebar.material = Some(Material {
            borders: Borders::none(),
            fill: Color::rgba(0.85, 0.85, 0.85, 1.0),
            image: None,
        });

        let mut sidebar_header = Style::row();
        sidebar_header.height = Some(60.0);
        sidebar_header.material = Some(Material {
            borders: Borders::none(),
            fill: Color::rgba(0.9, 0.9, 0.9, 1.0),
            image: None,
        });

        let mut sidebar_body = Style::column();
        sidebar_body.gap = 8.0;
        sidebar_body.padding = EdgeInsets::all(16.0);
        sidebar_body.justify_content = JustifyContent::Center;

        let btn1 = Style {
            width: Some(218.0),
            height: Some(40.0),
            material: Some(Material {
                borders: Borders::all(Color::rgba(0.5, 0.5, 0.5, 1.0), 1.0),
                fill: Color::rgba(0.8, 0.8, 0.8, 1.0),
                image: None,
            }),
            ..Default::default()
        };
        let btn2 = Style {
            width: Some(218.0),
            height: Some(40.0),
            material: Some(Material {
                borders: Borders::all(Color::rgba(0.5, 0.5, 0.5, 1.0), 1.0),
                fill: Color::rgba(0.8, 0.8, 0.8, 1.0),
                image: None,
            }),
            ..Default::default()
        };
        let btn3 = Style {
            width: Some(218.0),
            height: Some(40.0),
            material: Some(Material {
                borders: Borders::all(Color::rgba(0.5, 0.5, 0.5, 1.0), 1.0),
                fill: Color::rgba(0.8, 0.8, 0.8, 1.0),
                image: None,
            }),
            ..Default::default()
        };

        sidebar_body.children.push(Rc::new(btn1));
        sidebar_body.children.push(Rc::new(btn2));
        sidebar_body.children.push(Rc::new(btn3));

        sidebar.children.push(Rc::new(sidebar_header));
        sidebar.children.push(Rc::new(sidebar_body));

        let mut app_body = Style::column();
        app_body.flex_grow = 1.0;

        let mut app_header = Style::row();
        app_header.height = Some(60.0);
        app_header.material = Some(Material {
            borders: Borders::none(),
            fill: Color::rgba(0.95, 0.95, 0.95, 1.0),
            image: None,
        });

        let mut cards_grid = Style::row();
        cards_grid.gap = 16.0;
        cards_grid.padding = EdgeInsets::all(24.0);
        cards_grid.flex_wrap = FlexWrap::Wrap;

        let card1 = Style {
            width: Some(280.0),
            height: Some(200.0),
            material: Some(Material {
                borders: Borders::all(Color::rgba(0.7, 0.7, 0.7, 1.0), 1.0),
                fill: Color::rgba(1.0, 1.0, 1.0, 1.0),
                image: None,
            }),
            ..Default::default()
        };
        let card2 = Style {
            width: Some(280.0),
            height: Some(200.0),
            material: Some(Material {
                borders: Borders::all(Color::rgba(0.7, 0.7, 0.7, 1.0), 1.0),
                fill: Color::rgba(1.0, 1.0, 1.0, 1.0),
                image: None,
            }),
            ..Default::default()
        };
        let card3 = Style {
            width: Some(280.0),
            height: Some(200.0),
            material: Some(Material {
                borders: Borders::all(Color::rgba(0.7, 0.7, 0.7, 1.0), 1.0),
                fill: Color::rgba(1.0, 1.0, 1.0, 1.0),
                image: None,
            }),
            ..Default::default()
        };
        let card4 = Style {
            width: Some(280.0),
            height: Some(200.0),
            material: Some(Material {
                borders: Borders::all(Color::rgba(0.7, 0.7, 0.7, 1.0), 1.0),
                fill: Color::rgba(1.0, 1.0, 1.0, 1.0),
                image: None,
            }),
            ..Default::default()
        };

        cards_grid.children.push(Rc::new(card1));
        cards_grid.children.push(Rc::new(card2));
        cards_grid.children.push(Rc::new(card3));
        cards_grid.children.push(Rc::new(card4));

        app_body.children.push(Rc::new(app_header));
        app_body.children.push(Rc::new(cards_grid));

        let mut main_row = Style::row();
        main_row.children.push(Rc::new(sidebar));
        main_row.children.push(Rc::new(app_body));

        let mut tree = LayoutTree::new();
        let result = main_row.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(1440.0, 900.0)),
        );

        let sidebar_box = tree.get(result.children[0]).unwrap();
        let app_body_box = tree.get(result.children[1]).unwrap();

        assert_eq!(sidebar_box.bounds.x, 0.0);
        assert_eq!(sidebar_box.bounds.y, 0.0);
        assert_eq!(sidebar_box.bounds.width, 250.0);
        assert_eq!(sidebar_box.bounds.height, 900.0);

        assert!(
            sidebar_box.material.is_some(),
            "sidebar should have material"
        );
        let sidebar_fill = &sidebar_box.material.as_ref().unwrap().fill;
        assert_eq!(sidebar_fill.r, 0.85, "sidebar should have gray fill");
        assert_eq!(sidebar_fill.g, 0.85);
        assert_eq!(sidebar_fill.b, 0.85);

        assert_eq!(app_body_box.bounds.x, 250.0);
        assert_eq!(app_body_box.bounds.y, 0.0);
        assert_eq!(app_body_box.bounds.width, 1190.);
        assert_eq!(app_body_box.bounds.height, 900.);

        assert!(
            sidebar_box.children.len() >= 1,
            "sidebar should have children"
        );
        assert!(
            app_body_box.children.len() >= 2,
            "app_body should have children"
        );

        let sidebar_body_box = tree.get(sidebar_box.children[1]).unwrap();
        let btn1_box = tree.get(sidebar_body_box.children[0]).unwrap();
        assert!(btn1_box.material.is_some(), "button should have material");
        assert!(
            btn1_box.material.as_ref().unwrap().borders.top.is_some(),
            "button should have border"
        );

        assert_eq!(btn1_box.bounds.width, 218.0, "button1 width should be 218");
        assert_eq!(btn1_box.bounds.height, 40.0, "button1 height should be 40");

        let btn1_material = btn1_box.material.as_ref().unwrap();
        if let Some(border) = &btn1_material.borders.top {
            assert_eq!(border.width, 1.0, "button1 top border width should be 1.0");
        }

        let btn2_box = tree.get(sidebar_body_box.children[1]).unwrap();
        assert_eq!(btn2_box.bounds.width, 218.0, "button2 width should be 218");
        assert_eq!(btn2_box.bounds.height, 40.0, "button2 height should be 40");

        let btn3_box = tree.get(sidebar_body_box.children[2]).unwrap();
        assert_eq!(btn3_box.bounds.width, 218.0, "button3 width should be 218");
        assert_eq!(btn3_box.bounds.height, 40.0, "button3 height should be 40");

        let cards_grid_box = tree.get(app_body_box.children[1]).unwrap();
        assert_eq!(cards_grid_box.children.len(), 4, "grid should have 4 cards");
        let card1_box = tree.get(cards_grid_box.children[0]).unwrap();
        assert!(card1_box.material.is_some(), "card should have material");
        assert!(
            card1_box.material.as_ref().unwrap().borders.top.is_some(),
            "card should have border"
        );
    }

    #[test]
    fn flex_without_explicit_size_fills_parent() {
        let mut flex = Style::row();
        flex.children.push(Rc::new(child(50.0, 50.0)));
        flex.children.push(Rc::new(child(50.0, 50.0)));

        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(200.0, 100.0)),
        );

        assert_eq!(result.size.x, 200.0);
        assert_eq!(result.size.y, 100.0);
    }
}
