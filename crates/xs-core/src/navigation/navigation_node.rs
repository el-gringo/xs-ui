
pub type NavigationNodeId = usize;


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NavigationLayout {
    Horizontal,
    Verical,
    Leaf,
}

pub use NavigationLayout::*;

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
