use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NavigationLayout {
    Horizontal,
    Verical,
    Leaf,
}

use NavigationLayout::*;
use xs_platform_event::KeyCode;

pub type NavigationNodeId = usize;

pub struct NavigationNode {
    pub key: &'static str,
    pub layout: NavigationLayout,
    pub selected_index: usize,
    pub has_focus: bool,
    pub children: Vec<NavigationNodeId>,
    pub parent: NavigationNodeId,
}

impl NavigationNode {
    pub fn new(key: &'static str) -> Self {
        NavigationNode {
            key,
            parent: 0,
            layout: Leaf,
            selected_index: 0,
            has_focus: false,
            children: Vec::new(),
        }
    }

    pub fn vertical(mut self) -> Self {
        self.layout = Verical;
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.layout = Horizontal;
        self
    }

    pub fn previous(&mut self) -> bool {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            return false;
        }
        true
    }

    pub fn next(&mut self) -> bool {
        if self.selected_index < self.children.len() - 1 {
            self.selected_index += 1;
            return false;
        }
        true
    }
}

pub struct NavigationTree {
    pub root: Option<NavigationNodeId>,
    pub nodes: Vec<RefCell<NavigationNode>>,
}

impl NavigationTree {
    pub fn new() -> Self {
        Self {
            root: None,
            nodes: Vec::new(),
        }
    }

    pub fn set_root(&mut self, root: Option<NavigationNodeId>) {
        self.root = root;
    }

    #[must_use]
    pub fn get(&self, id: NavigationNodeId) -> Option<&RefCell<NavigationNode>> {
        self.nodes.get(id)
    }

    pub fn insert(&mut self, lnode: NavigationNode) -> NavigationNodeId {
        let id = self.nodes.len();
        self.nodes.push(RefCell::new(lnode));
        id
    }

    pub fn iter(&self) -> NavigationTreeIterator<'_> {
        self.iter_from_node(self.root)
    }

    pub fn iter_from_node(
        &self,
        root: Option<NavigationNodeId>,
    ) -> NavigationTreeIterator<'_> {
        NavigationTreeIterator {
            tree: self,
            parents: match root {
                Some(root) => VecDeque::from([root]),
                None => VecDeque::new(),
            },
            remaining: match root {
                Some(root) => VecDeque::from([(0, root)]),
                None => VecDeque::new(),
            },
        }
    }

    pub fn debug_node(&self, prefix: &str, node_id: NavigationNodeId) {
        if let Some(node) = self.get(node_id) {
            let node = node.borrow();
            println!(
                "{} node {}: {} {} {:?} {} {}",
                prefix,
                if node.has_focus { "*" } else { " " },
                node_id,
                node.key,
                node.layout,
                node.selected_index,
                node.children.len()
            );
        } else {
            println!("node {} not found", node_id);
        }
    }

    pub fn debug(&self, node_id: NavigationNodeId, depth: u32) {
        self.debug_node(&"  ".repeat(depth as usize), node_id);
        if let Some(node) = self.get(node_id) {
            let ids = node.borrow().children.clone();
            for id in ids {
                self.debug(id, depth + 1);
            }
        }
    }
}

pub struct NavigationTreeIterator<'a> {
    tree: &'a NavigationTree,
    parents: VecDeque<NavigationNodeId>,
    remaining: VecDeque<(usize, NavigationNodeId)>,
}

impl<'a> Iterator for NavigationTreeIterator<'a> {
    type Item = (
        &'a RefCell<NavigationNode>,
        usize,
        &'a RefCell<NavigationNode>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        let parent_id = self.parents.pop_front()?;
        let (index, child_id) = self.remaining.pop_front()?;
        let parent = self.tree.get(parent_id)?;
        let child = self.tree.get(child_id)?;
        let count = child.borrow().children.len();
        for (pos, child) in child.borrow().children.iter().rev().enumerate() {
            self.parents.push_front(child_id);
            self.remaining.push_front((count - pos - 1, *child));
        }
        Some((parent, index, child))
    }
}

pub struct FocusHandle {
    pub id: NavigationNodeId,
    tree: Rc<RefCell<NavigationTree>>,
}

impl FocusHandle {
    pub fn new(node: NavigationNode, tree: Rc<RefCell<NavigationTree>>) -> Self {
        let id = tree.borrow_mut().insert(node);
        tree.borrow().debug_node(&format!("new {}", id), id);
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

pub struct FocusManager {
    pub tree: Rc<RefCell<NavigationTree>>,
    pub current_container_id: NavigationNodeId,
    pub active_element_id: NavigationNodeId,
}

impl FocusManager {
    pub fn new() -> Self {
        let mut tree = NavigationTree::new();
        let mut root = NavigationNode::new("root").horizontal();
        root.has_focus = true;
        let id = tree.insert(root);
        tree.set_root(Some(id));
        FocusManager {
            tree: Rc::new(RefCell::new(tree)),
            current_container_id: id,
            active_element_id: id,
        }
    }

    fn initialize_node(&mut self, container_id: NavigationNodeId, node: &mut NavigationNode) {
        node.parent = container_id;
        let tree = self.tree.borrow_mut();
        let mut parent = tree.get(self.current_container_id).unwrap().borrow_mut();
        if node.has_focus {
            for child_id in parent.children.iter() {
                tree.get(*child_id).unwrap().borrow_mut().has_focus = false;
            }
            parent.selected_index = parent.children.len();
        }
        if parent.has_focus && parent.children.len() == 0 {
            node.has_focus = true
        }
    }

    fn attach(&mut self, handle: &FocusHandle) {
        self.tree
            .borrow_mut()
            .get(self.current_container_id)
            .unwrap()
            .borrow_mut()
            .children
            .push(handle.id);
        self.current_container_id = handle.id;
    }

    pub fn create_handle(&mut self, mut node: NavigationNode) -> FocusHandle {
        self.initialize_node(self.current_container_id, &mut node);
        let has_focus = node.has_focus;
        let handle = FocusHandle::new(node, self.tree.clone());
        if has_focus {
            self.active_element_id = handle.id;
        }
        self.attach(&handle);
        handle
    }

    pub fn process_key(&mut self, key_code: KeyCode) {
        self.tree.borrow().debug_node("previous focus", self.active_element_id);
        self.tree.borrow().debug(0, 0);
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
                    (Horizontal, KeyCode::Left) |
                    (Verical, KeyCode::Up) => {
                        propagate = parent.borrow_mut().previous();
                    },
                    (Horizontal, KeyCode::Right) |
                    (Verical, KeyCode::Down) => {
                        propagate = parent.borrow_mut().next();
                    }
                    _ => {}
                }
                if current_element_id == 0 || !propagate {
                    break;
                }
            }
        }
        self.update();
        self.tree.borrow().debug_node("new focus", self.active_element_id);
        self.tree.borrow().debug(0, 0);
    }

    pub fn update(&mut self) {
        let tree = self.tree.borrow();
        for (parent, index, child) in tree.iter() {
            let parent_has_focus = parent.borrow().has_focus;
            let selected_index = parent.borrow().selected_index;
            if parent_has_focus {
                if index == selected_index {
                    child.borrow_mut().has_focus = true;
                    if child.borrow().children.len() == 0 {
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
