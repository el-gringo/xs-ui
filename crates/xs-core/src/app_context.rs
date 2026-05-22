use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use xs_layout::{Image, TextProperties};
use xs_platform_event::KeyEvent;

use crate::{
    BackendRequest, FocusManager, TextSystem, TextSystemDummy, TextVertices,
    core::NodeId,
    navigation::NavigationNodeId,
    state::{AnyState, StateId},
};

pub struct Screen {
    pub width: u32,
    pub height: u32,
}

pub struct AppContext {
    pub running: bool,
    pub now: u128,
    pub screen: Screen,
    pub key_down: Option<KeyEvent>,
    pub nav: FocusManager,
    pub nodes: HashMap<NodeId, Vec<Box<dyn Any>>>,
    pub states: HashMap<NodeId, HashMap<StateId, AnyState>>,
    pub current_node_id: NodeId,
    pub current_nav_node_id: NavigationNodeId,
    pub text_system: Box<dyn TextSystem>,
    pub texts: HashMap<(String, TextProperties), Option<(TextVertices, u32)>>,
    pub textures: HashMap<String, Option<Image>>,
    pub requests: HashMap<(String, String), Option<String>>,
    pub messages: Vec<BackendRequest>,
    pub dirty: bool,
}

impl Default for AppContext {
    fn default() -> Self {
        AppContext {
            //app,
            running: true,
            now: 0,
            key_down: None,
            screen: Screen {
                width: 0,
                height: 0,
            },
            nav: FocusManager::new(),
            current_node_id: (TypeId::of::<Self>(), 0, 0),
            current_nav_node_id: 0,
            nodes: Default::default(),
            states: Default::default(),
            text_system: Box::new(TextSystemDummy::default()),
            textures: Default::default(),
            texts: Default::default(),
            requests: Default::default(),
            messages: Default::default(),
            dirty: true,
        }
    }
}

impl AppContext {
    pub fn new(text_system: Box<dyn TextSystem>) -> Self {
        AppContext {
            text_system,
            ..Default::default()
        }
    }

    pub fn dispatch(&mut self) {
        if let Some(KeyEvent { code, .. }) = self.key_down {
            self.nav.process_key(code);
            self.key_down = None;
            self.dirty = true;
        }
    }

    pub fn load_image(&mut self, src: Option<String>) -> Option<Image> {
        if let Some(src) = src {
            self.textures
                .entry(src.clone())
                .or_insert_with(|| {
                    self.messages.push(BackendRequest::LoadImage(src));
                    None
                })
                .clone()
        } else {
            None
        }
    }

    pub fn add_texture(&mut self, src: String, image: Image) {
        self.textures.insert(src.clone(), Some(image));
        self.dirty = true;
    }
}
