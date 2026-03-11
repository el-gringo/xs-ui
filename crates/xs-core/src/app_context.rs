use std::{
    any::{Any, TypeId}, collections::HashMap, sync::atomic::{AtomicPtr, Ordering}
};

use xs_layout::Image;
use xs_platform_event::KeyEvent;

use crate::{BackendRequest, FocusManager, core::NodeId, navigation::NavigationNodeId};

// Global AppContext storage
static APP_CONTEXT: AtomicPtr<AppContext> = AtomicPtr::new(std::ptr::null_mut());

pub fn set_app_context(ctx: &mut AppContext) {
    APP_CONTEXT.store(ctx as *mut AppContext, Ordering::SeqCst);
}

pub fn get_app_context() -> &'static mut AppContext {
    unsafe { &mut *APP_CONTEXT.load(Ordering::SeqCst) }
}

pub struct AppContext {
    pub running: bool,
    //pub app: &'a mut Application,
    pub key_down: Option<KeyEvent>,
    pub nav: FocusManager,
    pub nodes: HashMap<NodeId, Vec<Box<dyn Any>>>,
    pub current_node_id: NodeId,
    pub current_nav_node_id: NavigationNodeId,
    pub textures: HashMap<String, Option<Image>>,
    pub texts: HashMap<String, Option<Image>>,
    pub requests: HashMap<(String, String), Option<String>>,
    pub messages: Vec<BackendRequest>,
}

impl Default for AppContext {
    fn default() -> Self {
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
            requests: Default::default(),
            messages: Default::default(),
        }
    }
}

impl AppContext {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn dispatch(&mut self) {
        if let Some(KeyEvent { code, .. }) = self.key_down {
            self.nav.process_key(code);
            self.key_down = None;
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
}
