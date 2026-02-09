use std::{cell::RefCell, collections::VecDeque, rc::Rc};

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

    pub fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn next(&mut self) {
        if self.selected_index < self.children.len() - 1 {
            self.selected_index += 1;
        }
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
        self.nodes.push(RefCell::new(lnode));
        self.nodes.len() - 1
    }

    pub fn iter(&self) -> NavigationTreeIterator<'_> {
        NavigationTreeIterator {
            tree: self,
            parents: match self.root {
                Some(root) => VecDeque::from([root]),
                None => VecDeque::new(),
            },
            remaining: match self.root {
                Some(root) => VecDeque::from([(0, root)]),
                None => VecDeque::new(),
            },
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
        for (pos, child) in child.borrow().children.iter().rev().enumerate() {
            self.parents.push_front(child_id);
            self.remaining.push_front((pos, *child));
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
        let mut root = NavigationNode::new("root").vertical();
        root.has_focus = true;
        let id = tree.insert(root);
        tree.set_root(Some(id));
        FocusManager {
            tree: Rc::new(RefCell::new(tree)),
            current_container_id: id,
            active_element_id: id,
        }
    }

    pub fn update(&mut self) {}

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
        if parent.children.len() == 0 {
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
        let tree = self.tree.borrow();

        let active_element_id = self.active_element_id;
        while active_element_id > 0 {
            let parent_id = tree.get(active_element_id).unwrap().borrow_mut().parent;
            let mut parent = tree.get(parent_id).unwrap().borrow_mut();
            let layout = parent.layout;
            match layout {
                Horizontal => {
                    match key_code {
                        KeyCode::Left => {
                            parent.previous();
                        }
                        KeyCode::Right => {
                            parent.next();
                        }
                        _ => {}
                    };
                }
                Verical => {
                    match key_code {
                        KeyCode::Up => {
                            parent.previous();
                        }
                        KeyCode::Down => {
                            parent.next();
                        }
                        _ => {}
                    };
                }
                _ => {}
            }
            let selected_index = parent.selected_index;
            for (index, child_id) in parent.children.iter().enumerate() {
                tree.get(*child_id).unwrap().borrow_mut().has_focus = index == selected_index;
            }
            break;
        }
        self.debug();
    }

    pub fn debug(&self) {
        let tree = self.tree.borrow();
        for (_parent, _index, child) in tree.iter() {
            let child = child.borrow();
            println!(
                "node: {} {:?} {} {}",
                child.key,
                child.layout,
                child.has_focus,
                child.children.len()
            );
        }
    }
}
