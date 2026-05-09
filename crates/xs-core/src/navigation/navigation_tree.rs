use std::{cell::RefCell, collections::VecDeque};

use crate::{NavigationNode, navigation::navigation_node::NavigationNodeId};

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

    pub fn iter_from_node(&self, root: Option<NavigationNodeId>) -> NavigationTreeIterator<'_> {
        let mut parents = VecDeque::new();
        let mut remaining = VecDeque::new();
        if let Some(root) = root {
            if let Some(node) = self.get(root) {
                for (index, child) in node.borrow().children.iter().enumerate() {
                    parents.push_back(root);
                    remaining.push_back((index, *child));
                }
            }
        }
        NavigationTreeIterator {
            tree: self,
            parents,
            remaining,
        }
    }

    pub fn debug_node(&self, prefix: &str, node_id: NavigationNodeId) {
        if let Some(node) = self.get(node_id) {
            let node = node.borrow();
            eprintln!(
                "DEBUG NavigationTree: {} node {}: {} {} {:?} {} {}",
                prefix,
                if node.has_focus { "*" } else { " " },
                node_id,
                node.id.unwrap_or("none"),
                node.layout,
                node.selected_index,
                node.children.len()
            );
        } else {
            println!("DEBUG NavigationTree: node {} not found", node_id);
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
