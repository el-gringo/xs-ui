use xs_math::{Rectangle, Vector2};

use super::{BoxConstraints, Layout, LayoutBox, LayoutTree, SizedLayoutBox};
use crate::{EdgeInsets, Material, StylePosition, View};

// --------------------------------------------------
// Positioned
// --------------------------------------------------

pub struct Positioned {
    pub position: Vector2,
    pub child: View,
}

impl Layout for Positioned {
    fn position(&self) -> StylePosition {
        StylePosition::Absolute
    }

    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let child_constraints = BoxConstraints {
            min: Vector2::new(0.0, 0.0),
            max: constraints.max - self.position,
        };
        let sbox = self.child.layout(tree, &child_constraints);
        let lbox = LayoutBox::from_child(sbox.clone(), self.position);
        let child_id = tree.insert(lbox);
        SizedLayoutBox {
            size: sbox.size,
            children: vec![child_id],
            material: None,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug)]
pub struct FixedSize {
    pub width: f32,
    pub height: f32,
    pub material: Option<Material>,
}

impl Layout for FixedSize {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let lbox = LayoutBox {
            bounds: Rectangle::new(
                constraints.min.x,
                constraints.min.y,
                self.width,
                self.height,
            ),
            ..Default::default()
        };
        let child_id = tree.insert(lbox);
        SizedLayoutBox {
            size: Vector2::new(self.width, self.height),
            margin: EdgeInsets::zero(),
            children: vec![child_id],
            material: self.material.clone(),
        }
    }
}
