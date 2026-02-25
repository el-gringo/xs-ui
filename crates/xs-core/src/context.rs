use crate::{
    Application, FocusHandle, NavigationNode, core::NodeId,
    navigation::NavigationNodeId,
};
use std::any::Any;

pub struct Context<'a> {
    pub app: &'a mut Application,
    pub node_id: NodeId,
    pub nav_id: NavigationNodeId,
}

impl<'a> Context<'a> {
    pub fn new(app: &'a mut Application, node_id: NodeId, nav_id: NavigationNodeId) -> Self {
        Self {
            app,
            node_id,
            nav_id,
        }
    }

    pub fn use_state<S>(&mut self, init: impl FnOnce(&mut Context) -> S) -> &mut S
    where
        S: Any,
    {
        if self.app.ctx.nodes.get(&self.node_id).is_none() {
            let state = Box::new(init(self));
            self.app.ctx.nodes.insert(self.node_id, state);
        }
        self.app
            .ctx
            .nodes
            .get_mut(&self.node_id)
            .unwrap()
            .downcast_mut()
            .unwrap()
    }

    pub fn focus_handle(&mut self, node: NavigationNode) -> FocusHandle {
        let handle = self.app.ctx.nav.create_handle(self.nav_id, node);
        self.nav_id = handle.id;
        handle
    }
}
