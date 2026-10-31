use std::{cell::RefCell, rc::Rc};

use xs_platform_event::KeyCode;

use crate::{
    FocusHandle, NavigationNode,
    navigation::{
        navigation_node::{Grid, Horizontal, NavigationNodeId, Verical},
        navigation_tree::NavigationTree,
    },
};

pub struct FocusManager {
    pub tree: Rc<RefCell<NavigationTree>>,
    pub current_container_id: NavigationNodeId,
    pub active_element_id: NavigationNodeId,
}

impl Default for FocusManager {
    fn default() -> Self {
        let mut tree = NavigationTree::new();
        let mut root = NavigationNode::horizontal();
        root.has_focus = true;
        let id = tree.insert(root);
        tree.set_root(Some(id));
        Self {
            tree: Rc::new(RefCell::new(tree)),
            current_container_id: id,
            active_element_id: id,
        }
    }
}

impl FocusManager {
    pub fn new() -> Self {
        Default::default()
    }

    fn initialize_node(&mut self, container_id: NavigationNodeId, node: &mut NavigationNode) {
        node.parent = container_id;
        let tree = self.tree.borrow_mut();
        let mut parent = tree.get(container_id).unwrap().borrow_mut();
        if node.has_focus {
            for child_id in parent.children.iter() {
                tree.get(*child_id).unwrap().borrow_mut().has_focus = false;
            }
            parent.selected_index = parent.children.len();
        }
        if parent.has_focus && parent.children.is_empty() {
            node.has_focus = true
        }
    }

    fn attach(&mut self, container_id: NavigationNodeId, handle: &FocusHandle) {
        if let Some(container) = self.tree.borrow_mut().get(container_id) {
            container.borrow_mut().children.push(handle.id);
        }
    }

    pub fn create_handle(
        &mut self,
        container_id: NavigationNodeId,
        mut node: NavigationNode,
    ) -> FocusHandle {
        self.initialize_node(container_id, &mut node);
        let has_focus = node.has_focus;
        let handle = FocusHandle::new(node, self.tree.clone());
        if has_focus {
            self.active_element_id = handle.id;
        }
        self.attach(container_id, &handle);
        handle
    }

    pub fn process_key(&mut self, key_code: KeyCode) {
        {
            let tree = self.tree.borrow();
            let mut propagate = true;
            let mut current_element_id = self.active_element_id;
            loop {
                let node = tree.get(current_element_id).unwrap();
                if let Some(keydown_handler) = node.borrow_mut().keydown_handler.clone() {
                    propagate = keydown_handler(key_code);
                }
                if !propagate {
                    break;
                }
                let parent_id = node.borrow().parent;
                current_element_id = parent_id;
                let parent = tree.get(parent_id).unwrap();
                let layout = parent.borrow().layout;
                match (layout, key_code) {
                    (Horizontal, KeyCode::Left) | (Verical, KeyCode::Up) => {
                        propagate = parent.borrow_mut().previous_sibling();
                    }
                    (Horizontal, KeyCode::Right) | (Verical, KeyCode::Down) => {
                        propagate = parent.borrow_mut().next_sibling();
                    }
                    (Grid { columns }, KeyCode::Left) => {
                        propagate = parent.borrow_mut().previous_sibling_in_row(columns);
                    }
                    (Grid { columns }, KeyCode::Right) => {
                        propagate = parent.borrow_mut().next_sibling_in_row(columns);
                    }
                    (Grid { columns }, KeyCode::Up) => {
                        propagate = parent.borrow_mut().previous_row(columns);
                    }
                    (Grid { columns }, KeyCode::Down) => {
                        propagate = parent.borrow_mut().next_row(columns);
                    }
                    _ => {}
                }
                if current_element_id == 0 || !propagate {
                    break;
                }
            }
        }
        self.update();
        self.tree.borrow().debug(0, 0);
    }

    pub fn update(&mut self) {
        let tree = self.tree.borrow();
        for (parent, index, child) in tree.iter() {
            // clamp selected_index to valid range in case children were removed
            let children_len;
            let selected_index;
            let parent_has_focus;
            {
                let mut parent_node = parent.borrow_mut();
                children_len = parent_node.children.len();
                if parent_node.selected_index >= children_len && children_len > 0 {
                    parent_node.selected_index = children_len - 1;
                }
                selected_index = parent_node.selected_index;
                parent_has_focus = parent_node.has_focus;
            }
            if parent_has_focus {
                if index == selected_index && index < children_len {
                    let should_have_focus = true;
                    let mut child_node = child.borrow_mut();
                    let had_focus = child_node.has_focus;
                    child_node.has_focus = should_have_focus;
                    if should_have_focus
                        && !had_focus
                        && let Some(cb) = child_node.focus_handler.clone()
                    {
                        cb()
                    }
                    drop(child_node);
                    if child.borrow().children.is_empty() {
                        self.active_element_id = parent.borrow().children[selected_index];
                    }
                } else {
                    child.borrow_mut().has_focus = false;
                }
            } else {
                child.borrow_mut().has_focus = false;
            }
        }
    }
}
