use std::{cell::RefCell, rc::Rc};

use xs_platform_event::KeyCode;

use crate::{
    NavigationNode,
    navigation::{navigation_node::NavigationNodeId, navigation_tree::NavigationTree},
};

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
        let tree = self.tree.borrow();
        let mut node = tree.get(self.id).unwrap().borrow_mut();
        node.has_focus = has_focus;
        node.focus_handler
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

    pub fn on_focus<H>(&mut self, focus_handler: H)
    where
        H: Fn() + 'static,
    {
        let handler = Rc::new(focus_handler);
        self.tree
            .borrow()
            .get(self.id)
            .unwrap()
            .borrow_mut()
            .focus_handler = Some(handler.clone());
    }

    pub fn on_key_down<H>(&mut self, keydown_handler: H)
    where
        H: Fn(KeyCode) -> bool + 'static,
    {
        let handler = Rc::new(keydown_handler);
        self.tree
            .borrow()
            .get(self.id)
            .unwrap()
            .borrow_mut()
            .keydown_handler = Some(handler.clone());
    }
}
