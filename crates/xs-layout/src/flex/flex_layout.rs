use crate::{
    AlignItems, BoxConstraints, EdgeInsets, FlexWrap, JustifyContent, Layout, LayoutBoxId,
    Material, SizedLayoutBox, StylePosition,
    flex::flex::Flex,
    tree::{LayoutBox, LayoutTree},
};
use xs_math::{Rectangle, Vector2};

struct FlexItem {
    size: Vector2,
    flex_grow: [f32; 2],
    min_main: f32,
    max_main: f32,
    margin: EdgeInsets,
    padding: EdgeInsets,
    children: Vec<LayoutBoxId>,
    material: Option<Material>,

    is_absolute: bool,
}

impl FlexItem {
    fn main(&self, is_row: bool) -> f32 {
        if is_row { self.size.x } else { self.size.y }
    }

    fn cross(&self, is_row: bool) -> f32 {
        if is_row { self.size.y } else { self.size.x }
    }

    fn set_main(&mut self, is_row: bool, value: f32) {
        if is_row {
            self.size.x = value;
        } else {
            self.size.y = value;
        }
    }

    fn set_cross(&mut self, is_row: bool, value: f32) {
        if is_row {
            self.size.y = value;
        } else {
            self.size.x = value;
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

impl Layout for Flex {
    fn flex_grow(&self) -> [f32; 2] {
        self.style.flex_grow
    }

    fn position(&self) -> StylePosition {
        self.style.position
    }

    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let style = &self.style;
        let main_is_x = self.is_row();
        let is_wrap = matches!(style.flex_wrap, FlexWrap::Wrap | FlexWrap::WrapReverse);
        let is_reverse = self.is_reverse();
        let gap = if main_is_x {
            style.columns_gap.max(style.gap)
        } else {
            style.row_gap.max(style.gap)
        };

        let w = style.width.unwrap_or(constraints.max.x);
        let h = style.height.unwrap_or(constraints.max.y);
        let pw = style.padding.left + style.padding.right + style.border.total_width();
        let ph = style.padding.top + style.padding.bottom + style.border.total_height();
        let inner = Vector2::new((w - pw).max(0.0), (h - ph).max(0.0));
        let container_main = if main_is_x { inner.x } else { inner.y };
        let inner_constraints = BoxConstraints::from_max(inner);

        let mut items: Vec<FlexItem> = self
            .children
            .iter()
            .map(|child| {
                let is_absolute = child.position() == StylePosition::Absolute;
                let growing = !is_absolute && child.flex_grow()[0] > 0.0;
                let layout = child.layout(tree, &inner_constraints);
                let size = if is_absolute {
                    Vector2::new(0.0, 0.0)
                } else if growing {
                    if main_is_x {
                        Vector2::new(0.0, layout.size.y)
                    } else {
                        Vector2::new(layout.size.x, 0.0)
                    }
                } else {
                    layout.size
                };
                FlexItem {
                    size,
                    flex_grow: if is_absolute {
                        [0.0, 0.0]
                    } else {
                        child.flex_grow()
                    },
                    min_main: if is_absolute { 0.0 } else { child.min_main() },
                    max_main: if is_absolute { 0.0 } else { child.max_main() },
                    children: layout.children,
                    material: layout.material,

                    margin: child.margin(),
                    padding: child.padding(),
                    is_absolute,
                }
            })
            .collect();

        if is_reverse {
            items.reverse();
        }

        let mut lines: Vec<Vec<FlexItem>> = if is_wrap {
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
            .map(|i| i.flex_grow[0])
            .sum();

        if total_grow > 0.0 && free_space > 0.0 {
            lines
                .iter_mut()
                .flat_map(|line| line.iter_mut())
                .filter(|i| i.flex_grow[0] > 0.0)
                .for_each(|item| {
                    let grow = free_space * item.flex_grow[0] / total_grow;
                    let current = item.main(main_is_x);
                    let new_main = (current + grow).clamp(item.min_main, item.max_main);
                    item.set_main(main_is_x, new_main);
                });
        }

        let container_cross = if main_is_x { inner.y } else { inner.x };
        let total_cross_before_grow: f32 = lines
            .iter()
            .map(|l| {
                l.iter()
                    .map(|i| i.cross(main_is_x))
                    .fold(0.0f32, |a, b| a.max(b))
            })
            .sum();
        let free_cross = (container_cross - total_cross_before_grow).max(0.0);
        let total_cross_grow: f32 = lines
            .iter()
            .flat_map(|l| l.iter())
            .map(|i| i.flex_grow[1])
            .sum();
        if total_cross_grow > 0.0 && free_cross > 0.0 {
            lines
                .iter_mut()
                .flat_map(|line| line.iter_mut())
                .filter(|i| i.flex_grow[1] > 0.0)
                .for_each(|item| {
                    let grow = free_cross * item.flex_grow[1] / total_cross_grow;
                    let current = item.cross(main_is_x);
                    let cross_padding = if main_is_x {
                        item.padding.top + item.padding.bottom
                    } else {
                        item.padding.left + item.padding.right
                    };
                    let cross_margin = if main_is_x {
                        item.margin.top + item.margin.bottom
                    } else {
                        item.margin.left + item.margin.right
                    };
                    let max_cross = (container_cross - cross_padding - cross_margin).max(0.0);
                    let new_cross = (current + grow).min(max_cross);
                    item.set_cross(main_is_x, new_cross);
                });
        }

        let total_cross: f32 = lines
            .iter()
            .map(|l| {
                l.iter()
                    .map(|i| i.cross(main_is_x))
                    .fold(0.0f32, |a, b| a.max(b))
            })
            .sum();

        let mut child_ids = Vec::new();
        let padding_min = style.padding.min();

        let mut cross_pos = if main_is_x {
            padding_min.y + style.border.min().y
        } else {
            padding_min.x + style.border.min().x
        };

        for (_line_idx, line) in lines.iter().enumerate() {
            let line_cross: f32 = line
                .iter()
                .map(|i| i.cross(main_is_x))
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0);
            let line_cross_start = cross_pos;

            // Per-line justify: each wrapped line is justified independently
            let line_content_main: f32 = line.iter().map(|i| i.main(main_is_x)).sum::<f32>()
                + gap * (line.len().saturating_sub(1)) as f32;
            let line_main_start =
                justify_offset(container_main, line_content_main, style.justify_content);

            let (line_ids, _new_cursor) =
                line.iter()
                    .fold((Vec::new(), line_main_start), |(mut ids, cursor), item| {
                        if item.is_absolute {
                            ids.extend(item.children.clone());
                            return (ids, cursor);
                        }

                        let (margin_main_start, margin_main_end) = item.main_margin(main_is_x);
                        let (margin_cross_start, margin_cross_end) = item.cross_margin(main_is_x);
                        let cross_offset = align_offset(
                            container_cross,
                            item.cross(main_is_x),
                            (margin_cross_start, margin_cross_end),
                            style.align_items,
                        );

                        let px = if main_is_x {
                            cursor + margin_main_start + style.padding.left + style.border.min().x
                        } else {
                            cross_pos + cross_offset
                        };
                        let py = if main_is_x {
                            line_cross_start + cross_offset
                        } else {
                            cursor + margin_main_start + style.padding.top + style.border.min().y
                        };

                        let box_id = tree.insert(LayoutBox {
                            bounds: Rectangle::new(
                                px,
                                py,
                                item.size.x + item.padding.left + item.padding.right,
                                item.size.y + item.padding.top + item.padding.bottom,
                            ),
                            margin: item.margin,
                            children: item.children.clone(),
                            material: item.material.clone(),
                        });
                        ids.push(box_id);
                        (
                            ids,
                            cursor
                                + item.main(main_is_x)
                                + margin_main_start
                                + margin_main_end
                                + gap,
                        )
                    });
            child_ids.extend(line_ids);
            cross_pos += line_cross + gap;
        }

        if is_reverse {
            child_ids.reverse();
        }

        let pw = style.padding.left + style.padding.right + style.border.total_width();
        let ph = style.padding.top + style.padding.bottom + style.border.total_height();
        let content_w = if main_is_x { total_main } else { total_cross };
        let content_h = if main_is_x { total_cross } else { total_main };
        let content_plus_padding_w = content_w + pw;
        let content_plus_padding_h = content_h + ph;
        let has_explicit_width = style.width.is_some();
        let has_explicit_height = style.height.is_some();
        let has_padding = pw > 0.0 || ph > 0.0;
        let final_w = if has_explicit_width {
            style.width.unwrap()
        } else if has_padding {
            content_plus_padding_w
        } else {
            w
        };
        let final_h = if has_explicit_height {
            style.height.unwrap()
        } else if has_padding {
            content_plus_padding_h
        } else {
            h
        };

        tree.insert(LayoutBox {
            bounds: Rectangle::new(0.0, 0.0, final_w, final_h),
            margin: style.margin,
            children: child_ids.clone(),
            material: self.material.clone(),
        });

        SizedLayoutBox {
            margin: style.margin,
            size: Vector2::new(final_w, final_h),
            children: child_ids,
            material: self.material.clone(),
        }
    }
    fn margin(&self) -> EdgeInsets {
        self.style.margin
    }
    fn padding(&self) -> EdgeInsets {
        self.style.padding
    }
}

#[cfg(test)]
mod tests {
use super::*;
use crate::{
    Background, Borders, Color, FlexDirection, QuadProps, Style, View,
};
use std::rc::Rc;

fn flex(width: Option<f32>, height: Option<f32>, children: Vec<View>) -> Flex {
    Flex {
        style: Style {
            width,
            height,
            ..Default::default()
        },
        material: None,
        children,
    }
}

fn child(w: f32, h: f32) -> View {
    let mut flex = flex(Some(w), Some(h), vec![]);
    flex.material = Some(Material::Quad(Rc::new(QuadProps {
        background: Background::Color(Color::rgba(255.0, 0.0, 0.0, 255.0)),
        border: Borders::none(),
        ..Default::default()
    })));
    Rc::new(flex)
}

fn child_with_margin(w: f32, h: f32, margin: EdgeInsets) -> Flex {
    let mut flex = flex(Some(w), Some(h), vec![]);
    flex.style.margin = margin;
    flex.material = Some(Material::Quad(Rc::new(QuadProps {
        background: Background::Color(Color::rgba(255.0, 0.0, 0.0, 255.0)),
        border: Borders::none(),
        ..Default::default()
    })));
    flex
}

fn row(width: Option<f32>, height: Option<f32>, children: Vec<View>) -> Flex {
    let mut flex = flex(width, height, children);
    flex.style.flex_direction = FlexDirection::Row;
    flex
}

fn column(width: Option<f32>, height: Option<f32>, children: Vec<View>) -> Flex {
    let mut flex = flex(width, height, children);
    flex.style.flex_direction = FlexDirection::Column;
    flex
}

fn card(width: Option<f32>, height: Option<f32>) -> Flex {
    let mut flex = flex(width, height, Vec::new());
    flex.material = Some(Material::Quad(Rc::new(QuadProps {
        background: Background::Color(Color::rgba(255.0, 255.0, 255.0, 255.0)),
        border: Borders::all(Color::rgba(178.0, 178.0, 178.0, 255.0), 1.0),
        ..Default::default()
    })));
    flex
}

    #[test]
    fn empty_flex_fills_parent() {
        let flex = row(Some(100.), Some(100.), Vec::new());
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
        let flex = row(Some(200.), Some(100.), vec![child(50.0, 50.0)]);
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
        let flex = column(Some(100.), Some(200.), vec![child(50.0, 50.0)]);
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
        let flex = column(
            Some(200.),
            Some(100.),
            vec![child(50.0, 50.0), child(50.0, 50.0)],
        );
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
        let mut flex = row(
            Some(200.),
            Some(100.),
            vec![child(60.0, 50.0), child(60.0, 50.0)],
        );
        flex.style.flex_wrap = FlexWrap::Wrap;
        let mut tree = LayoutTree::new();
        let _ = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        assert!(tree.boxes.len() > 2);
    }

    #[test]
    fn flex_wrap_items_on_multiple_lines() {
        let mut flex = row(
            Some(100.),
            Some(200.),
            vec![child(60.0, 50.0), child(60.0, 50.0), child(60.0, 50.0)],
        );
        flex.style.flex_wrap = FlexWrap::Wrap;

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
        let mut flex = row(
            Some(130.),
            Some(200.),
            vec![child(60.0, 50.0), child(60.0, 50.0)],
        );
        flex.style.flex_wrap = FlexWrap::Wrap;
        flex.style.gap = 10.0;

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
        let mut flex = row(
            Some(100.),
            Some(200.),
            vec![child(60.0, 50.0), child(60.0, 50.0)],
        );
        flex.style.flex_wrap = FlexWrap::NoWrap;

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
        let mut flex = column(
            Some(200.),
            Some(100.),
            vec![child(50.0, 60.0), child(50.0, 60.0)],
        );
        flex.style.flex_wrap = FlexWrap::Wrap;

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
        let flex = row(
            Some(100.),
            Some(50.),
            vec![child(60.0, 50.0), child(50.0, 30.0)],
        );

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
        let inner = row(Some(100.), Some(50.), vec![child(50.0, 30.0)]);
        let outer = column(
            Some(200.),
            Some(200.),
            vec![Rc::new(inner), child(30.0, 20.0)],
        );

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
        let inner = column(
            Some(80.),
            Some(80.),
            vec![child(20.0, 20.0), child(20.0, 20.0), child(20.0, 20.0)],
        );

        let outer = row(
            Some(200.),
            Some(100.),
            vec![Rc::new(inner), child(40.0, 40.0)],
        );

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
        let mut flex = row(
            Some(200.),
            Some(100.),
            vec![child(50.0, 50.0), child(50.0, 50.0)],
        );
        flex.style.justify_content = JustifyContent::Center;
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();
        assert_eq!(c1.bounds.x, 50.0);
        assert_eq!(c1.bounds.y, 0.0);
        assert_eq!(c1.bounds.width, 50.0);
        assert_eq!(c1.bounds.height, 50.0);
        assert_eq!(c2.bounds.x, 100.0);
        assert_eq!(c2.bounds.y, 0.0);
        assert_eq!(c2.bounds.width, 50.0);
        assert_eq!(c2.bounds.height, 50.0);
    }

    #[test]
    fn justify_content_flex_end_row() {
        let mut flex = row(
            Some(200.),
            Some(100.),
            vec![child(50.0, 50.0), child(50.0, 50.0)],
        );
        flex.style.justify_content = JustifyContent::FlexEnd;
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
    fn justify_content_flex_end_column() {
        let mut flex = column(
            Some(100.),
            Some(200.),
            vec![child(50.0, 50.0), child(50.0, 50.0)],
        );
        flex.style.justify_content = JustifyContent::FlexEnd;
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();
        assert_eq!(c1.bounds.x, 0.0);
        assert_eq!(c1.bounds.y, 100.0);
        assert_eq!(c1.bounds.width, 50.0);
        assert_eq!(c1.bounds.height, 50.0);
        assert_eq!(c2.bounds.x, 0.0);
        assert_eq!(c2.bounds.y, 150.0);
        assert_eq!(c2.bounds.width, 50.0);
        assert_eq!(c2.bounds.height, 50.0);
    }

    #[test]
    fn align_items_center_row() {
        let mut flex = row(
            Some(200.),
            Some(100.),
            vec![child(50.0, 30.0), child(50.0, 40.0)],
        );
        flex.style.align_items = AlignItems::Center;
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
        let mut flex = row(
            Some(200.),
            Some(100.),
            vec![child(50.0, 30.0), child(50.0, 40.0)],
        );
        flex.style.align_items = AlignItems::FlexEnd;
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
        let mut flex = column(
            Some(100.),
            Some(200.),
            vec![child(30.0, 50.0), child(40.0, 50.0)],
        );
        flex.style.align_items = AlignItems::Center;
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
        let mut flex = column(
            Some(100.),
            Some(200.),
            vec![child(30.0, 50.0), child(40.0, 50.0)],
        );
        flex.style.align_items = AlignItems::FlexEnd;
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
        let mut column = column(
            Some(300.),
            Some(300.),
            vec![child(150.0, 50.0), child(150.0, 50.0), child(150.0, 50.0)],
        );
        column.style.align_items = AlignItems::Center;
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
        let flex = row(
            Some(200.),
            Some(100.),
            vec![
                Rc::new(child_with_margin(
                    50.0,
                    50.0,
                    EdgeInsets {
                        top: 0.0,
                        bottom: 0.0,
                        left: 10.0,
                        right: 0.0,
                    },
                )),
                Rc::new(child_with_margin(
                    50.0,
                    50.0,
                    EdgeInsets {
                        top: 0.0,
                        bottom: 0.0,
                        left: 0.0,
                        right: 10.0,
                    },
                )),
            ],
        );
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
    fn row_with_padding_reduces_inner_area() {
        let mut inner = flex(Some(50.0), Some(50.0), vec![]);
        inner.style.padding = EdgeInsets {
            left: 20.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        };

        let mut flex = row(Some(100.), Some(100.), vec![Rc::new(inner)]);
        flex.style.padding = EdgeInsets::all(10.0);
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        assert_eq!(result.size.x, 100.0);
        assert_eq!(result.size.y, 100.0);
        let child_box = tree.get(result.children[0]).unwrap();
        assert_eq!(child_box.bounds.x, 10.0);
        assert_eq!(child_box.bounds.y, 10.0);
        assert_eq!(child_box.bounds.width, 70.0);
        assert_eq!(child_box.bounds.height, 50.0);
    }

    #[test]
    fn column_with_padding_reduces_inner_area() {
        let mut flex = column(Some(100.), Some(100.), vec![child(50.0, 50.0)]);
        flex.style.padding = EdgeInsets::all(10.0);
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        assert_eq!(result.size.x, 100.0);
        assert_eq!(result.size.y, 100.0);
        let child_box = tree.get(result.children[0]).unwrap();
        assert_eq!(child_box.bounds.x, 10.0);
        assert_eq!(child_box.bounds.y, 10.0);
        assert_eq!(child_box.bounds.width, 50.0);
        assert_eq!(child_box.bounds.height, 50.0);
    }

    #[test]
    fn padding_with_unbounded_parent_uses_padding() {
        let mut flex = row(None, None, vec![child(50.0, 50.0)]);
        flex.style.padding = EdgeInsets {
            top: 5.0,
            bottom: 15.0,
            left: 20.0,
            right: 10.0,
        };
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        assert_eq!(result.size.x, 50.0 + 10.0 + 20.0);
        assert_eq!(result.size.y, 50.0 + 5.0 + 15.0);
        let child_box = tree.get(result.children[0]).unwrap();
        assert_eq!(child_box.bounds.x, 20.0);
        assert_eq!(child_box.bounds.y, 5.0);
    }

    #[test]
    fn padding_with_multiple_children() {
        let mut flex = row(
            Some(200.),
            Some(100.),
            vec![child(40.0, 40.0), child(40.0, 40.0)],
        );
        flex.style.padding = EdgeInsets::all(10.0);
        flex.style.gap = 5.0;
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        let c1 = tree.get(result.children[0]).unwrap();
        let c2 = tree.get(result.children[1]).unwrap();
        assert_eq!(c1.bounds.x, 10.0);
        assert_eq!(c1.bounds.y, 10.0);
        assert_eq!(c2.bounds.x, 10.0 + 40.0 + 5.0);
        assert_eq!(c2.bounds.y, 10.0);
    }

    #[test]
    fn flex_without_explicit_size_with_padding() {
        let mut flex = row(None, None, vec![child(50.0, 50.0)]);
        flex.style.padding = EdgeInsets::all(25.0);
        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );
        assert_eq!(result.size.x, 50.0 + 50.0);
        assert_eq!(result.size.y, 50.0 + 50.0);
        let child_box = tree.get(result.children[0]).unwrap();
        assert_eq!(child_box.bounds.x, 25.0);
        assert_eq!(child_box.bounds.y, 25.0);
    }

    #[test]
    fn real_world_app_layout() {
        let mut sidebar_header = row(None, Some(60.0), vec![child(100., 30.)]);
        sidebar_header.style.justify_content = JustifyContent::Center;
        sidebar_header.style.align_items = AlignItems::Center;
        sidebar_header.material = Some(Material::Quad(Rc::new(QuadProps {
            background: Background::Color(Color::rgba(229.0, 229.0, 229.0, 255.0)),
            border: Borders::none(),
            ..Default::default()
        })));

        let mut btn = row(Some(218.0), Some(40.0), vec![]);
        btn.material = Some(Material::Quad(Rc::new(QuadProps {
            background: Background::Color(Color::rgba(204.0, 204.0, 204.0, 255.0)),
            border: Borders::all(Color::rgba(128.0, 128.0, 128.0, 255.0), 1.0),
            ..Default::default()
        })));
        let mut sidebar_body = column(
            None,
            None,
            vec![
                Rc::new(btn.clone()),
                Rc::new(btn.clone()),
                Rc::new(btn.clone()),
            ],
        );
        sidebar_body.style.gap = 8.0;
        sidebar_body.style.padding = EdgeInsets::all(16.0);
        sidebar_body.style.justify_content = JustifyContent::Center;

        let mut sidebar = column(
            Some(250.),
            None,
            vec![Rc::new(sidebar_header), Rc::new(sidebar_body)],
        );
        sidebar.material = Some(Material::Quad(Rc::new(QuadProps {
            background: Background::Color(Color::rgba(217.0, 217.0, 217.0, 255.0)),
            border: Borders::none(),
            ..Default::default()
        })));

        let mut app_header = row(None, Some(60.0), vec![]);
        app_header.material = Some(Material::Quad(Rc::new(QuadProps {
            background: Background::Color(Color::rgba(242.0, 242.0, 242.0, 255.0)),
            border: Borders::none(),
            ..Default::default()
        })));

        let mut card = card(Some(280.0), Some(200.0));
        card.material = Some(Material::Quad(Rc::new(QuadProps {
            background: Background::Color(Color::rgba(255.0, 255.0, 255.0, 255.0)),
            border: Borders::all(Color::rgba(178.0, 178.0, 178.0, 255.0), 1.0),
            ..Default::default()
        })));

        let mut cards_grid = row(
            Some(280.0),
            Some(200.0),
            vec![
                Rc::new(card.clone()),
                Rc::new(card.clone()),
                Rc::new(card.clone()),
                Rc::new(card.clone()),
            ],
        );
        cards_grid.style.gap = 16.0;
        cards_grid.style.padding = EdgeInsets::all(24.0);
        cards_grid.style.flex_wrap = FlexWrap::Wrap;

        let mut app_body = column(None, None, vec![Rc::new(app_header), Rc::new(cards_grid)]);
        app_body.style.flex_grow = [1.0, 0.0];

        let main_row = row(None, None, vec![Rc::new(sidebar), Rc::new(app_body)]);

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

        let sidebar_header_box = tree.get(sidebar_box.children[0]).unwrap();
        let sidebar_header_inner_box = tree.get(sidebar_header_box.children[0]).unwrap();

        assert_eq!(sidebar_header_inner_box.bounds.x, 75.);
        assert_eq!(sidebar_header_inner_box.bounds.y, 15.);
        assert_eq!(sidebar_header_inner_box.bounds.width, 100.);
        assert_eq!(sidebar_header_inner_box.bounds.height, 30.);

        assert!(
            sidebar_box.material.is_some(),
            "sidebar should have material"
        );
        if let Some(Material::Quad(props)) = &sidebar_box.material {
            if let Background::Color(fill) = props.background {
                let expected = 217.0;
                assert_eq!(fill.r, expected, "sidebar should have gray fill");
                assert_eq!(fill.g, expected);
                assert_eq!(fill.b, expected);
            } else {
                panic!("sidebar should have Background::Color");
            }
        }

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
        if let Some(Material::Quad(props)) = &btn1_box.material {
            assert!(props.border.top.is_some(), "button should have border");
        }

        assert_eq!(btn1_box.bounds.width, 218.0, "button1 width should be 218");
        assert_eq!(btn1_box.bounds.height, 40.0, "button1 height should be 40");

        if let Some(Material::Quad(props)) = &btn1_box.material {
            if let Some(border) = &props.border.top {
                assert_eq!(border.width, 1.0, "button1 top border width should be 1.0");
            }
        }

        let btn2_box = tree.get(sidebar_body_box.children[1]).unwrap();
        assert_eq!(btn2_box.bounds.width, 218.0, "button2 width should be 218");
        assert_eq!(btn2_box.bounds.height, 40.0, "button2 height should be 40");

        let btn3_box = tree.get(sidebar_body_box.children[2]).unwrap();
        assert_eq!(btn3_box.bounds.width, 218.0, "button3 width should be 218");
        assert_eq!(btn3_box.bounds.height, 40.0, "button3 height should be 40");

        let cards_grid_box = tree.get(app_body_box.children[1]).unwrap();
        assert_eq!(cards_grid_box.children.len(), 4, "grid should have 4 cards");

        let app_header_box = tree.get(app_body_box.children[0]).unwrap();
        assert_eq!(
            app_header_box.bounds.height, 60.0,
            "app_header height should be 60"
        );

        let card1_box = tree.get(cards_grid_box.children[0]).unwrap();
        assert!(card1_box.material.is_some(), "card should have material");
        if let Some(Material::Quad(props)) = &card1_box.material {
            assert!(props.border.top.is_some(), "card should have border");
        }
    }

    #[test]
    fn flex_without_explicit_size_fills_parent() {
        let flex = row(None, None, vec![child(50.0, 50.0), child(50.0, 50.0)]);

        let mut tree = LayoutTree::new();
        let result = flex.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(200.0, 100.0)),
        );

        assert_eq!(result.size.x, 200.0);
        assert_eq!(result.size.y, 100.0);
    }

    #[test]
    fn flex_grow_child_sees_full_constraint_for_subtree() {
        // Regression test: a Column with flex_grow inside a Row should pass the
        // full container width to its children, not zero. Otherwise, a wrapping
        // Row inside the Column wraps prematurely.
        //
        // Layout:
        //   Row (width=400):
        //     - Fixed child (width=100, height=100)
        //     - Column (flex_grow=1)
        //         +-- Row with Wrap
        //               - Item A (width=80, height=50)
        //               - Item B (width=80, height=50)
        //               - Item C (width=80, height=50)
        //
        // Available width for Column = 400 - 100 = 300
        // All 3 items (240px + 2*10 gap = 260px) should fit on one line.

        let item_a = child(80.0, 50.0);
        let item_b = child(80.0, 50.0);
        let item_c = child(80.0, 50.0);

        let mut wrapping_row = row(None, None, vec![item_a, item_b, item_c]);
        wrapping_row.style.flex_wrap = FlexWrap::Wrap;
        wrapping_row.style.gap = 10.0;

        let mut inner_column = column(None, None, vec![Rc::new(wrapping_row)]);
        inner_column.style.flex_grow = [1.0, 0.0];

        let fixed = child(100.0, 100.0);

        let outer_row = row(Some(400.0), None, vec![fixed, Rc::new(inner_column)]);

        let mut tree = LayoutTree::new();
        let result = outer_row.layout(
            &mut tree,
            &BoxConstraints::from_max(Vector2::new(500.0, 500.0)),
        );

        let inner_col_box = tree.get(result.children[1]).unwrap();
        let wrapping_row_box = tree.get(inner_col_box.children[0]).unwrap();

        // All 3 items should be on the same line (same y-position)
        let child_a = tree.get(wrapping_row_box.children[0]).unwrap();
        let child_b = tree.get(wrapping_row_box.children[1]).unwrap();
        let child_c = tree.get(wrapping_row_box.children[2]).unwrap();

        assert_eq!(
            child_a.bounds.y, child_b.bounds.y,
            "item A and B should be on same line (no wrap)"
        );
        assert_eq!(
            child_a.bounds.y, child_c.bounds.y,
            "item A and C should be on same line (no wrap)"
        );

        // Items should be positioned sequentially on the x axis
        assert_eq!(child_a.bounds.x, 0.0);
        assert_eq!(child_b.bounds.x, 80.0 + 10.0); // 80 width + 10 gap
        assert_eq!(child_c.bounds.x, 80.0 * 2.0 + 10.0 * 2.0);
    }
}
