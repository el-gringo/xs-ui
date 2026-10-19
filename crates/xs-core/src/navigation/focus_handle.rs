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

    /// The element currently holds the focus
    pub fn has_focus(&self) -> bool {
        self.tree.borrow().get(self.id).unwrap().borrow().has_focus
    }

    /// Whether this is the child its parent currently points at, whether or not it
    /// holds focus.
    pub fn is_selected(&self) -> bool {
        let tree = self.tree.borrow();
        let parent_id = tree.get(self.id).unwrap().borrow().parent;
        let parent = tree.get(parent_id).unwrap().borrow();
        parent.children.get(parent.selected_index) == Some(&self.id)
    }

    pub fn set_focus(&mut self, has_focus: bool) {
        let tree = self.tree.borrow();
        let mut node = tree.get(self.id).unwrap().borrow_mut();
        node.has_focus = has_focus;
        if let Some(focus_handler) = node.focus_handler.clone() {
            focus_handler();
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::navigation::focus_manager::FocusManager;
    use xs_platform_event::KeyCode;

    /// The parent's selection has to outlive its focus: a menu still says which
    /// item you were on after the user has left the menu entirely, and nothing on
    /// the child itself records that.
    #[test]
    fn the_selected_child_is_remembered_after_focus_leaves() {
        let mut manager = FocusManager::new();
        manager.update();

        let root = manager.create_handle(
            manager.current_container_id,
            NavigationNode::new().horizontal(),
        );
        let menu = manager.create_handle(root.id, NavigationNode::new().vertical());
        let first = manager.create_handle(menu.id, NavigationNode::new());
        let second = manager.create_handle(menu.id, NavigationNode::new());
        manager.create_handle(root.id, NavigationNode::new());
        manager.update();

        assert!(first.has_focus(), "the first item starts focused");
        assert!(first.is_selected(), "and is the one selected");
        assert!(!second.is_selected());

        manager.process_key(KeyCode::Down);
        manager.update();
        assert!(second.has_focus());
        assert!(second.is_selected());
        assert!(!first.is_selected(), "the selection moved with focus");

        // Out to the grid. The menu holds no focus at all now, but it still points
        // at the item the user left from.
        manager.process_key(KeyCode::Right);
        manager.update();
        assert!(!second.has_focus(), "focus left the menu");
        assert!(!menu.has_focus(), "the menu itself has none either");
        assert!(second.is_selected(), "but it still names where they were");

        // Coming back lands on the same item, which is the point of keeping it.
        manager.process_key(KeyCode::Left);
        manager.update();
        assert!(second.has_focus(), "focus returns where it was left");
    }
}
