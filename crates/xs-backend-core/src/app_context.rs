use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;

use xs_layout::{DummyContainer, View};
use xs_platform_event::KeyEvent;

use crate::focus::{FocusHandle, FocusManager, NavigationNodeId};
use crate::node::NodeId;
use crate::{Element, NavigationNode, Render};

pub struct RootElement;
impl Element for RootElement {
    fn paint(&self, _ctx: &mut AppContext, _views: Vec<View>) -> View {
        Box::new(DummyContainer {})
    }
}

impl Render for RootElement {
    fn render(&self, _ctx: &mut AppContext) -> Box<dyn Element> {
        Box::new(RootElement {})
    }
}

pub struct AppContext {
    pub running: bool,
    pub key_down: Option<KeyEvent>,
    pub nav: FocusManager,
    pub nodes: HashMap<NodeId, Box<dyn Any>>,
    pub current_node_id: NodeId,
    pub current_nav_node_id: NavigationNodeId,
}

impl AppContext {
    pub fn new() -> Self {
        AppContext {
            running: true,
            key_down: None,
            nav: FocusManager::new(),
            current_node_id: (0, 0, TypeId::of::<RootElement>()),
            current_nav_node_id: 0,
            nodes: HashMap::new(),
        }
    }

    pub fn dispatch(&mut self) {
        if let Some(KeyEvent { code, .. }) = self.key_down {
            self.nav.process_key(code);
            self.key_down = None;
        }
    }

    pub fn render(&mut self, widget: Rc<Box<dyn Render>>, depth: usize, index: usize) -> View {
        let element = widget.render(self);
        let type_id = element.type_id().clone();
        self.current_node_id = (depth, index, type_id);
        let container_id = self.nav.current_container_id;
        let mut views = Vec::new();
        for (index, child) in element.children().iter().enumerate() {
            self.nav.current_container_id = container_id;
            let view = self.render(child.clone(), depth + 1, index);
            views.push(view);
        }
        let frame = element.paint(self, views);
        frame
    }

    pub fn use_state<S>(&mut self, init: impl FnOnce(&mut AppContext) -> S) -> &mut S
    where
        S: Any,
    {
        let node_id = self.current_node_id.clone();
        if self.nodes.get(&node_id).is_none() {
            let state = Box::new(init(self));
            self.nodes.insert(node_id, state);
        }
        self.nodes
            .get_mut(&node_id)
            .unwrap()
            .downcast_mut()
            .unwrap()
    }

    pub fn focus_handle(&mut self, node: NavigationNode) -> FocusHandle {
        let handle = self.nav.create_handle(node);
        self.current_nav_node_id = handle.id;
        handle
    }
}
