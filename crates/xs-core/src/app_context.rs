use std::{
    any::{Any, TypeId},

    collections::HashMap,
    rc::Rc,
};

use xs_layout::{Image, View};
use xs_platform_event::KeyEvent;

use crate::{Backend, BackendDummy, FocusHandle, FocusManager, NavigationNode, Renderer, RendererDummy, TextSystem, core::NodeId, focus::NavigationNodeId, text_system::TextSystemDummy};


pub struct AppContext {
    pub running: bool,
    pub backend: Box<dyn Backend>,
    pub renderer: Box<dyn Renderer>,
    pub text_system: Box<dyn TextSystem>,
    pub key_down: Option<KeyEvent>,
    pub nav: FocusManager,
    pub nodes: HashMap<NodeId, Box<dyn Any>>,
    pub current_node_id: NodeId,
    pub current_nav_node_id: NavigationNodeId,
    pub ressources: HashMap<u32, Option<Image>>,
}

impl AppContext {
    pub fn new() -> Self {
        AppContext {
            running: true,
            backend: Box::new(BackendDummy),
            renderer: Box::new(RendererDummy),
            text_system: Box::new(TextSystemDummy),
            key_down: None,
            nav: FocusManager::new(),
            current_node_id: (TypeId::of::<NodeId>(), 0, 0),
            current_nav_node_id: 0,
            nodes: HashMap::new(),
            ressources: HashMap::new(),
        }
    }

    pub fn dispatch(&mut self) {
        if let Some(KeyEvent { code, .. }) = self.key_down {
            self.nav.process_key(code);
            self.key_down = None;
        }
    }

    pub fn use_state<S>(&mut self, init: impl FnOnce(&mut AppContext) -> S) -> &mut S
    where
        S: Any,
    {
        let node_id = self.current_node_id.clone();
        if self.nodes.get(&node_id).is_none() {
            println!("use_state: {:?}", node_id);
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
