use std::rc::Rc;

pub type NavigationNodeId = usize;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum NavigationLayout {
    Horizontal,
    Verical,
    Grid {
        columns: usize,
    },
    #[default]
    Leaf,
}

pub use NavigationLayout::*;
use xs_platform_event::KeyCode;

#[derive(Default)]
pub struct NavigationNode {
    pub id: Option<&'static str>,
    pub layout: NavigationLayout,
    pub selected_index: usize,
    pub has_focus: bool,
    pub children: Vec<NavigationNodeId>,
    pub parent: NavigationNodeId,
    pub focus_handler: Option<Rc<dyn Fn() + 'static>>,
    pub keydown_handler: Option<Rc<dyn Fn(KeyCode) -> bool + 'static>>,
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

    pub fn grid(mut self, columns: usize) -> Self {
        self.layout = Grid { columns };
        self
    }

    pub fn leaf(mut self) -> Self {
        self.layout = Leaf;
        self
    }

    pub fn previous_sibling(&mut self) -> bool {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            return false;
        }
        true
    }

    pub fn next_sibling(&mut self) -> bool {
        if self.selected_index < self.children.len() - 1 {
            self.selected_index += 1;
            return false;
        }
        true
    }

    pub fn previous_sibling_in_row(&mut self, columns: usize) -> bool {
        if self.selected_index > 0 && !self.selected_index.is_multiple_of(columns) {
            self.selected_index -= 1;
            return false;
        }
        true
    }

    pub fn next_sibling_in_row(&mut self, columns: usize) -> bool {
        if self.selected_index < self.children.len() - 1
            && self.selected_index % columns < columns - 1
        {
            self.selected_index += 1;
            return false;
        }
        true
    }

    pub fn previous_row(&mut self, columns: usize) -> bool {
        if self.selected_index >= columns {
            self.selected_index -= columns;
            false
        } else {
            true
        }
    }

    pub fn next_row(&mut self, columns: usize) -> bool {
        let next_index = self.selected_index + columns;
        if next_index < self.children.len() {
            self.selected_index = next_index;
            false
        } else {
            true
        }
    }
}
