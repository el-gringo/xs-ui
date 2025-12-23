use xs_math::Vector2;

use super::{BoxConstraints, Layout, LayoutBox, LayoutTree, SizedLayoutBox};

// --------------------------------------------------
// Center
// --------------------------------------------------

pub struct Center {
    pub child: Box<dyn Layout>,
}

impl Layout for Center {
    fn flex_factor(&self) -> Option<f32> {
        None
    }
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let sbox = self.child.layout(tree, constraints);
        let pos = (constraints.max / 2.0) - (sbox.size / 2.0);
        let lbox = LayoutBox::from_child(sbox, pos);
        let id = tree.insert(lbox);
        SizedLayoutBox {
            size: constraints.max,
            children: vec![id],
            material: None,
            ..SizedLayoutBox::default()
        }
    }
}

// --------------------------------------------------
// Stack
// --------------------------------------------------

pub struct Stack {
    pub children: Vec<Positioned>,
}

impl Layout for Stack {
    fn flex_factor(&self) -> Option<f32> {
        None
    }
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let children = self
            .children
            .iter()
            .map(|child| {
                let child = child.layout(tree, constraints);
                let lbox = LayoutBox::from_child(child, Vector2::new(0.0, 0.0));
                let id = tree.insert(lbox);
                id
            })
            .collect();
        SizedLayoutBox {
            size: constraints.max,
            children,
            material: None,
            ..Default::default()
        }
    }
}

// --------------------------------------------------
// Positioned
// --------------------------------------------------

pub struct Positioned {
    pub position: Vector2,
    pub child: Box<dyn Layout>,
}

impl Layout for Positioned {
    fn flex_factor(&self) -> Option<f32> {
        None
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
            size: constraints.max,
            children: vec![child_id],
            material: None,
            ..Default::default()
        }
    }
}
