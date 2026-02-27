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

impl Item {
    fn main(&self, is_row: bool) -> f32 {
        if is_row {
            self.size.x
        } else {
            self.size.y
        }
    }

    fn cross(&self, is_row: bool) -> f32 {
        if is_row {
            self.size.y
        } else {
            self.size.x
        }
    }

    fn set_main(&mut self, is_row: bool, value: f32) {
        if is_row {
            self.size.x = value;
        } else {
            self.size.y = value;
        }
    }

    fn main_margin(&self, is_row: bool) -> (f32, f32) {
        if is_row {
            (self.margin.left, self.margin.right)
        } else {
            (self.margin.top, self.margin.bottom)
        }
    }

    fn cross_margin(&self, is_row: bool) -> (f32, f32) {
        if is_row {
            (self.margin.top, self.margin.bottom)
        } else {
            (self.margin.left, self.margin.right)
        }
    }
}

fn justify_offset(container_main: f32, content_main: f32, justify: JustifyContent) -> f32 {
    match justify {
        JustifyContent::Center => (container_main - content_main) / 2.0,
        JustifyContent::FlexEnd => (container_main - content_main).max(0.0),
        _ => 0.0,
    }
}

fn align_offset(
    container_cross: f32,
    item_cross: f32,
    margins: (f32, f32),
    align: AlignItems,
) -> f32 {
    match align {
        AlignItems::Center => (container_cross - item_cross - margins.0 - margins.1) / 2.0,
        AlignItems::FlexEnd => (container_cross - item_cross - margins.0 - margins.1).max(0.0),
        _ => 0.0,
    }
}

impl Layout for Style {
    fn flex_grow(&self) -> f32 {
        self.flex_grow
    }

    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let main_is_x = self.is_row();
        let is_wrap = matches!(self.flex_wrap, FlexWrap::Wrap | FlexWrap::WrapReverse);
        let is_reverse = self.is_reverse();
        let gap = if main_is_x {
            self.columns_gap.max(self.gap)
        } else {
            self.row_gap.max(self.gap)
        };

        let w = self.width.unwrap_or(constraints.max.x);
        let h = self.height.unwrap_or(constraints.max.y);
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
                let item_main = item.main(main_is_x);
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
                l.iter().map(|i| i.main(main_is_x)).sum::<f32>()
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
            lines
                .iter_mut()
                .flat_map(|line| line.iter_mut())
                .filter(|i| i.flex_grow > 0.0)
                .for_each(|item| {
                    let grow = free_space * item.flex_grow / total_grow;
                    let current = item.main(main_is_x);
                    let new_main = (current + grow).clamp(item.min_main, item.max_main);
                    item.set_main(main_is_x, new_main);
                });
        }

        let mut child_ids = Vec::new();
        let padding_min = self.padding.min();
        let justify_base = justify_offset(container_main, total_main, self.justify_content);
        let container_cross = if main_is_x { inner.y } else { inner.x };

        let mut main_pos = padding_min.x + padding_min.y + justify_base;
        let mut cross_pos = if main_is_x {
            padding_min.y
        } else {
            padding_min.x
        };

        for (line_idx, line) in lines.iter().enumerate() {
            let line_main: f32 = line.iter().map(|i| i.main(main_is_x)).sum();
            let line_cross: f32 = line
                .iter()
                .map(|i| i.cross(main_is_x))
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0);
            let start_offset = justify_offset(container_main, line_main, self.justify_content);

            let line_cross_start = cross_pos;

            // For wrapped lines, reset main position to start of container (not accumulated)
            let line_main_start = if line_idx == 0 {
                padding_min.x + padding_min.y + justify_base
            } else {
                padding_min.x + padding_min.y
            };

            let (line_ids, _new_cursor) = line.iter().fold(
                (Vec::new(), line_main_start + start_offset),
                |(mut ids, cursor), item| {
                    let (margin_main_start, margin_main_end) = item.main_margin(main_is_x);
                    let (margin_cross_start, margin_cross_end) = item.cross_margin(main_is_x);
                    let cross_offset = align_offset(
                        container_cross,
                        item.cross(main_is_x),
                        (margin_cross_start, margin_cross_end),
                        self.align_items,
                    );

                    let px = if main_is_x {
                        cursor + margin_main_start
                    } else {
                        cross_pos + cross_offset + margin_cross_start
                    };
                    let py = if main_is_x {
                        line_cross_start + cross_offset + margin_cross_start
                    } else {
                        cursor + margin_main_start
                    };

                    let box_id = tree.insert(LayoutBox {
                        bounds: Rectangle::new(px, py, item.size.x, item.size.y),
                        margin: item.margin,
                        children: item.children.clone(),
                        material: item.material.clone(),
                    });
                    ids.push(box_id);
                    (
                        ids,
                        cursor + item.main(main_is_x) + margin_main_start + margin_main_end + gap,
                    )
                },
            );
            child_ids.extend(line_ids);
            main_pos += line_main + gap;
            cross_pos += line_cross + gap;
        }

        if is_reverse {
            child_ids.reverse();
        }

        let final_w = self
            .width
            .unwrap_or(w)
            .max(self.min_width.unwrap_or(0.0))
            .min(self.max_width.unwrap_or(f32::INFINITY));
        let final_h = self
            .height
            .unwrap_or(h)
            .max(self.min_height.unwrap_or(0.0))
            .min(self.max_height.unwrap_or(f32::INFINITY));

        tree.insert(LayoutBox {
            bounds: Rectangle::new(0.0, 0.0, final_w, final_h),
            margin: self.margin,
            children: child_ids.clone(),
            material: self.material.clone(),
        });

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
    fn flex_wrap_items_on_multiple_lines() {
        let mut flex = Style::row();
        flex.flex_wrap = FlexWrap::Wrap;
        flex.width = Some(100.0);
        flex.height = Some(200.0);

        // Three items of 60px each - should wrap to 2 lines (2 + 1)
        flex.children.push(Rc::new(child(60.0, 50.0)));
        flex.children.push(Rc::new(child(60.0, 50.0)));
        flex.children.push(Rc::new(child(60.0, 50.0)));

        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );

        // First item on line 0 (x=0, y=0)
        let c1 = tree.get(result.children[0]).unwrap();
        assert_eq!(c1.bounds.x, 0.0, "first item x should be 0");
        assert_eq!(c1.bounds.y, 0.0, "first item y should be 0");

        // Second item wraps to line 1 (x=0, y=50)
        let c2 = tree.get(result.children[1]).unwrap();
        assert_eq!(c2.bounds.x, 0.0, "second item x should be 0 (new line)");
        assert_eq!(
            c2.bounds.y, 50.0,
            "second item y should be 50 (on second line)"
        );

        // Third item wraps to line 2 (x=0, y=100)
        let c3 = tree.get(result.children[2]).unwrap();
        assert_eq!(c3.bounds.x, 0.0, "third item x should be 0 (new line)");
        assert!(
            c3.bounds.y > c1.bounds.y,
            "third item should be on second line (y: {} > {})",
            c3.bounds.y,
            c1.bounds.y
        );
    }

    #[test]
    fn flex_wrap_with_gap() {
        let mut flex = Style::row();
        flex.flex_wrap = FlexWrap::Wrap;
        flex.width = Some(130.0);
        flex.height = Some(200.0);
        flex.gap = 10.0;

        // Two items of 60px each + 10px gap = 130px - should fit exactly
        flex.children.push(Rc::new(child(60.0, 50.0)));
        flex.children.push(Rc::new(child(60.0, 50.0)));

        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );

        // Both items should be on same line
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();

        // c2 should be 70px to the right of c1 (60 + 10 gap)
        assert!(
            (c2.bounds.x - c1.bounds.x - 70.0).abs() < 1.0,
            "gap between items should be 10px"
        );
    }

    #[test]
    fn flex_no_wrap_keeps_on_one_line() {
        let mut flex = Style::row();
        flex.flex_wrap = FlexWrap::NoWrap;
        flex.width = Some(100.0);
        flex.height = Some(200.0);

        // Two items that would normally wrap should stay on one line
        flex.children.push(Rc::new(child(60.0, 50.0)));
        flex.children.push(Rc::new(child(60.0, 50.0)));

        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );

        // Both items should be on same line (same y)
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();

        assert_eq!(
            c1.bounds.y, c2.bounds.y,
            "items should be on same line when no wrap"
        );
    }

    #[test]
    fn flex_wrap_column_layout() {
        let mut flex = Style::column();
        flex.flex_wrap = FlexWrap::Wrap;
        flex.width = Some(200.0);
        flex.height = Some(100.0);

        // Two items stacked vertically should wrap to new column
        flex.children.push(Rc::new(child(50.0, 60.0)));
        flex.children.push(Rc::new(child(50.0, 60.0)));

        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );

        // Items should be on different columns (different x) since they wrapped
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();

        assert_ne!(
            c1.bounds.x, c2.bounds.x,
            "items should be on different columns after wrap"
        );
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
