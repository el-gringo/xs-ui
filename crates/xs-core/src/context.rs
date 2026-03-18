use xs_layout::TextProperties;

use crate::{
    AppContext, FocusHandle, NavigationNode, core::NodeId, get_app_context,
    navigation::NavigationNodeId,
};
use std::{any::Any, fmt::Debug, marker::PhantomData};

pub struct StateMutator<S> {
    node_id: NodeId,
    state_index: usize,
    _marker: PhantomData<S>,
}

impl<S: Any + Debug> StateMutator<S> {
    pub fn mutate(&self, mutator: impl FnOnce(&mut S)) {
        let ctx = get_app_context();
        if let Some(node_states) = ctx.nodes.get_mut(&self.node_id)
            && let Some(state) = node_states.get_mut(self.state_index)
            && let Some(value) = state.downcast_mut::<S>()
        {
            mutator(value);
        }
    }
}

pub struct Context<'a> {
    pub app_context: &'a mut AppContext,
    pub node_id: NodeId,
    pub nav_id: NavigationNodeId,
    pub text_properties: TextProperties,
    pub state_index: usize,
}

impl<'a> Context<'a> {
    pub fn new(
        app_context: &'a mut AppContext,
        node_id: NodeId,
        nav_id: NavigationNodeId,
        text_properties: TextProperties,
    ) -> Self {
        Self {
            app_context,
            node_id,
            nav_id,
            text_properties,
            state_index: 0,
        }
    }

    pub fn use_state<S>(&mut self, init: impl FnOnce(&mut Context<'a>, StateMutator<S>) -> S) -> &S
    where
        S: Any,
    {
        let state_index = self.state_index;
        self.app_context.nodes.entry(self.node_id).or_default();
        let state_count = self.app_context.nodes.get(&self.node_id).unwrap().len();
        if state_index >= state_count {
            let mutator: StateMutator<S> = StateMutator {
                node_id: self.node_id,
                state_index: self.state_index,
                _marker: PhantomData,
            };
            // push new state
            let new_state = Box::new(init(self, mutator));
            self.app_context
                .nodes
                .get_mut(&self.node_id)
                .unwrap()
                .push(new_state);
        }
        let states = self.app_context.nodes.get_mut(&self.node_id).unwrap();
        self.state_index = state_index + 1;
        states.get_mut(state_index).unwrap().downcast_mut().unwrap()
    }

    pub fn focus_handle(&mut self, node: NavigationNode) -> FocusHandle {
        let handle = self.app_context.nav.create_handle(self.nav_id, node);
        self.nav_id = handle.id;
        handle
    }
}
