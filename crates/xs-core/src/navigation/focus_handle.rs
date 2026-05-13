use std::{cell::RefCell, rc::Rc};

use crate::{
    NavigationNode,
    navigation::{navigation_node::NavigationNodeId, navigation_tree::NavigationTree},
};

#[derive(Clone)]
pub struct FocusHandle {
    pub id: NavigationNodeId,
    tree: Rc<RefCell<NavigationTree>>,
    focus_handler: Option<Rc<dyn Fn() + 'static>>,
}

impl FocusHandle {
    pub fn new(node: NavigationNode, tree: Rc<RefCell<NavigationTree>>) -> Self {
        let id = tree.borrow_mut().insert(node);
        Self {
            id,
            tree: tree.clone(),
            focus_handler: None,
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
        let _ = self
            .focus_handler
            .clone()
            .map(|focus_handler| focus_handler());
    }

    pub fn selected_index(&self) -> usize {
        self.tree
            .borrow()
            .get(self.id)
            .unwrap()
            .borrow()
            .selected_index
    }

    pub fn set_selected_index(&self, index: usize) {
        self.tree
            .borrow()
            .get(self.id)
            .unwrap()
            .borrow_mut()
            .selected_index = index;
    }

    pub fn on_focus(&mut self, focus_handler: impl Fn() + 'static) {
        let handler = Rc::new(focus_handler);
        self.tree
            .borrow()
            .get(self.id)
            .unwrap()
            .borrow_mut()
            .focus_callback = Some(handler.clone());
        self.focus_handler = Some(handler);
    }
}
