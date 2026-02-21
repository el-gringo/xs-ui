use std::cell::{RefMut};

use crate::{AppContext, Application, core::NodeId};



pub struct Context<'a> {
    pub app: &'a mut Application,
    pub node_id: NodeId
}

impl<'a> Context<'a> {
    pub fn new(app: &'a mut Application, node_id: NodeId) -> Self  {
        Self { app, node_id }
    }

    pub fn with_app_context<T>(&mut self, callback: impl FnOnce(&mut AppContext) -> T) -> T {
        self.app.with_context(callback)
    }
}
