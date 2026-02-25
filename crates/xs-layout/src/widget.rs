use xs_math::Vector2;

use super::{BoxConstraints, Layout, LayoutBox, LayoutTree, SizedLayoutBox};
use crate::View;

// --------------------------------------------------
// Positioned
// --------------------------------------------------

pub struct Positioned {
    pub position: Vector2,
    pub child: View,
}

impl Layout for Positioned {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let child_constraints = BoxConstraints {
            min: Vector2::new(0.0, 0.0),
            max: constraints.max - self.position,
        };
        let sbox = self.child.layout(tree, &child_constraints);
        let lbox = LayoutBox::from_child(sbox.clone(), self.position);
        let child_id = tree.insert(lbox);
        SizedLayoutBox {
            size: constraints.max,
            children: vec![child_id],
            material: None,
            ..Default::default()
        }
    }
}
