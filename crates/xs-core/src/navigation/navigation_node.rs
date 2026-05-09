pub type NavigationNodeId = usize;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum NavigationLayout {
    Horizontal,
    Verical,
    #[default]
    Leaf,
}

pub use NavigationLayout::*;

#[derive(Default)]
pub struct NavigationNode {
    pub id: Option<&'static str>,
    pub layout: NavigationLayout,
    pub selected_index: usize,
    pub has_focus: bool,
    pub children: Vec<NavigationNodeId>,
    pub parent: NavigationNodeId,
}

impl NavigationNode {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn id(mut self, id: &'static str) -> Self {
        self.id = Some(id);
        self
    }

    pub fn vertical(mut self) -> Self {
        self.layout = Verical;
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.layout = Horizontal;
        self
    }

    pub fn leaf(mut self) -> Self {
        self.layout = Leaf;
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
