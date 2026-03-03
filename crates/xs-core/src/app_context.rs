use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use xs_layout::{Image, TextProperties};
use xs_platform_event::KeyEvent;

use crate::{FocusManager, Message, core::NodeId, navigation::NavigationNodeId};

pub struct AppContext {
    pub running: bool,
    //pub app: &'a mut Application,
    pub key_down: Option<KeyEvent>,
    pub nav: FocusManager,
    pub nodes: HashMap<NodeId, Box<dyn Any>>,
    pub current_node_id: NodeId,
    pub current_nav_node_id: NavigationNodeId,
    pub textures: HashMap<String, Option<Image>>,
    pub texts: HashMap<String, Option<Image>>,
    pub messages: Vec<Message>
}

impl AppContext {
    pub fn new() -> Self {
        AppContext {
            //app,
            running: true,
            key_down: None,
            nav: FocusManager::new(),
            current_node_id: (TypeId::of::<NodeId>(), 0, 0),
            current_nav_node_id: 0,
            nodes: Default::default(),
            textures: Default::default(),
            texts: Default::default(),
            messages: Default::default()
        }
    }

    pub fn dispatch(&mut self) {
        if let Some(KeyEvent { code, .. }) = self.key_down {
            self.nav.process_key(code);
            self.key_down = None;
        }
    }
}
