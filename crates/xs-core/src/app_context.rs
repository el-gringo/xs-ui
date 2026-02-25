use std::{
    any::{Any, TypeId},
    collections::HashMap
};

use xs_layout::Image;
use xs_platform_event::KeyEvent;

use crate::{
    Backend, BackendDummy, FocusManager, Renderer, RendererDummy,
    TextSystem, core::NodeId, navigation::NavigationNodeId, text_system::TextSystemDummy,
};

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
}
