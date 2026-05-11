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
        let mut root = NavigationNode::new().horizontal();
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
            container.borrow_mut().children.insert(0, handle.id);
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
                let parent_id = node.borrow().parent;
                current_element_id = parent_id;
                let parent = tree.get(parent_id).unwrap();
                let layout = parent.borrow().layout;
                match (layout, key_code) {
                    (Horizontal, KeyCode::Left) | (Verical, KeyCode::Up) => {
                        propagate = parent.borrow_mut().previous();
                    }
                    (Horizontal, KeyCode::Right) | (Verical, KeyCode::Down) => {
                        propagate = parent.borrow_mut().next();
                    }
                    (Grid { columns }, KeyCode::Left) => {
                        let sel = parent.borrow().selected_index;
                        if sel > 0 && sel % columns != 0 {
                            parent.borrow_mut().selected_index -= 1;
                            propagate = false;
                        }
                    }
                    (Grid { columns }, KeyCode::Right) => {
                        let sel = parent.borrow().selected_index;
                        let len = parent.borrow().children.len();
                        if sel < len - 1 && sel % columns != columns - 1 {
                            parent.borrow_mut().selected_index += 1;
                            propagate = false;
                        }
                    }
                    (Grid { columns }, KeyCode::Up) => {
                        let sel = parent.borrow().selected_index;
                        if sel >= columns {
                            parent.borrow_mut().selected_index -= columns;
                            propagate = false;
                        }
                    }
                    (Grid { columns }, KeyCode::Down) => {
                        let sel = parent.borrow().selected_index;
                        let len = parent.borrow().children.len();
                        if sel + columns < len {
                            parent.borrow_mut().selected_index += columns;
                            propagate = false;
                        }
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
            let parent_has_focus = parent.borrow().has_focus;
            let selected_index = parent.borrow().selected_index;
            if parent_has_focus {
                if index == selected_index && parent.borrow().children.len() > index {
                    child.borrow_mut().has_focus = true;
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
