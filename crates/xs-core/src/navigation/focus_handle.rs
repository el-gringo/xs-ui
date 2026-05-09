use std::{cell::RefCell, rc::Rc};

use crate::{NavigationNode, navigation::{navigation_node::NavigationNodeId, navigation_tree::NavigationTree}};


#[derive(Clone)]
pub struct FocusHandle {
    pub id: NavigationNodeId,
    tree: Rc<RefCell<NavigationTree>>,
}

impl FocusHandle {
    pub fn new(node: NavigationNode, tree: Rc<RefCell<NavigationTree>>) -> Self {
        let id = tree.borrow_mut().insert(node);
        Self {
            id,
            tree: tree.clone(),
        }
    }

    pub fn has_focus(&self) -> bool {
        self.tree.borrow().get(self.id).unwrap().borrow().has_focus
    }

    pub fn set_focus(&mut self, has_focus: bool) {
        self.tree
            .borrow()
            .get(self.id)
            .unwrap()
            .borrow_mut()
            .has_focus = has_focus;
    }
}
