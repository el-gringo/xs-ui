use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;

use xs_core::{Render, RootElement};
use xs_layout::{DummyContainer, Image, View};
use xs_platform_event::KeyEvent;
use xs_text_system::TextSystem;

use crate::focus::{FocusHandle, FocusManager, NavigationNodeId};
use crate::node::NodeId;
use crate::NavigationNode;

pub struct AppContext {
    pub running: bool,
    pub key_down: Option<KeyEvent>,
    pub nav: FocusManager,
    pub nodes: HashMap<NodeId, Box<dyn Any>>,
    pub current_node_id: NodeId,
    pub current_nav_node_id: NavigationNodeId,
    pub ressources: HashMap<u32, Option<Image>>,
    pub text_system: TextSystem,
}

impl AppContext {
    pub fn new() -> Self {
        let mut text_system = TextSystem::new();
        let font_family = "IBMPlexSans Regular";
        let data = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");
        text_system.load_font(font_family, data);

        AppContext {
            running: true,
            key_down: None,
            nav: FocusManager::new(),
            current_node_id: (0, 0, TypeId::of::<RootElement>()),
            current_nav_node_id: 0,
            nodes: HashMap::new(),
            ressources: HashMap::new(),
            text_system,
        }
    }

    pub fn dispatch(&mut self) {
        if let Some(KeyEvent { code, .. }) = self.key_down {
            self.nav.process_key(code);
            self.key_down = None;
        }
    }

    pub fn render(&mut self, widget: Rc<Box<dyn Render>>, depth: usize, index: usize) -> View {
        // let type_id = widget.tag().clone();
        // self.current_node_id = (depth, index, type_id);
        // let element = widget.render(self);
        // let container_id = self.nav.current_container_id;
        // let mut views = Vec::new();
        // for (index, child) in element.children().iter().enumerate() {
        //     self.nav.current_container_id = container_id;
        //     let view = self.render(child.clone(), depth + 1, index);
        //     views.push(view);
        // }
        // let frame = element.paint(self, views);
        // frame
        Rc::new(DummyContainer)
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

    // FIXME
    // Should find a way to access backend from AppContext
    #[cfg(target_arch = "wasm32")]
    pub fn load_image(&mut self, src: &'static str, texture_index: u32) {
        let bytes = src.as_bytes();
        unsafe {
            load_image(bytes.as_ptr() as u32, bytes.len() as u32, texture_index);
        }
    }

    pub fn new_image(&mut self, array: Vec<u8>, width: u32) -> u32 {
        unsafe {
            let ptr = array.as_slice().as_ptr() as u32;
            new_image(ptr, array.len() as u32, width)
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_image(&mut self, _src: &'static str) {}
}

unsafe extern "C" {
    #[cfg(target_arch = "wasm32")]
    pub fn load_image(ptr: u32, len: u32, texture_index: u32);

    #[cfg(target_arch = "wasm32")]
    pub fn new_image(ptr: u32, len: u32, width: u32) -> u32;
}
