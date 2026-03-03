use xs_layout::TextProperties;

use crate::{AppContext, FocusHandle, NavigationNode, core::NodeId, navigation::NavigationNodeId};
use std::any::Any;

pub struct Context<'a> {
    pub app_context: &'a mut AppContext,
    pub node_id: NodeId,
    pub nav_id: NavigationNodeId,
    pub text_properties: &'a mut TextProperties,
}

impl<'a> Context<'a> {
    pub fn new(
        app_context: &'a mut AppContext,
        node_id: NodeId,
        nav_id: NavigationNodeId,
        text_properties: &'a mut TextProperties,
    ) -> Self {
        Self {
            app_context,
            node_id,
            nav_id,
            text_properties
        }
    }

    pub fn use_state<S>(&mut self, init: impl FnOnce(&mut Context) -> S) -> &mut S
    where
        S: Any,
    {
        if self.app_context.nodes.get(&self.node_id).is_none() {
            let state = Box::new(init(self));
            self.app_context.nodes.insert(self.node_id, state);
        }
        self.app_context
            .nodes
            .get_mut(&self.node_id)
            .unwrap()
            .downcast_mut()
            .unwrap()
    }

    pub fn focus_handle(&mut self, node: NavigationNode) -> FocusHandle {
        let handle = self.app_context.nav.create_handle(self.nav_id, node);
        self.nav_id = handle.id;
        handle
    }
}
