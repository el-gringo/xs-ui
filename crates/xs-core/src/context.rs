use xs_layout::{BoxConstraints, Color, TextProperties, TextVertices};

use crate::{
    AppContext, FocusHandle, NavigationNode,
    core::NodeId,
    get_app_context,
    navigation::NavigationNodeId,
    state::{AnyState, State, StateId, Updater},
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
    pub color: Color,
    pub state_index: usize,
    pub constraints: BoxConstraints,
}

impl<'a> Context<'a> {
    pub fn new(
        app_context: &'a mut AppContext,
        node_id: NodeId,
        nav_id: NavigationNodeId,
        text_properties: TextProperties,
        color: Color,
        constraints: BoxConstraints,
    ) -> Self {
        Self {
            app_context,
            node_id,
            nav_id,
            text_properties,
            color,
            constraints,
            state_index: Default::default(),
        }
    }

    pub fn use_state<S>(&mut self, init: impl FnOnce(&mut Self) -> S) -> &mut State<S>
    where
        S: Any + 'static,
    {
        let state_id = StateId::from(std::any::type_name::<S>());
        let node_id = self.node_id;
        let needs_init = {
            let entry = self.app_context.states.entry(node_id).or_default();
            !entry.contains_key(&state_id)
        };
        if needs_init {
            let value = init(self);
            self.app_context
                .states
                .entry(node_id)
                .or_default()
                .insert(state_id.clone(), AnyState::new(State::new(value)));
        }
        self.app_context
            .states
            .get_mut(&node_id)
            .unwrap()
            .get_mut(&state_id)
            .unwrap()
            .downcast_state::<S>()
            .unwrap()
    }

    pub fn updater<S: 'static>(&self) -> Updater<S> {
        Updater::new(self.node_id, StateId::from(std::any::type_name::<S>()))
    }

    fn new_focus_handle(&mut self, node: NavigationNode) -> FocusHandle {
        let handle = self.app_context.nav.create_handle(self.nav_id, node);
        self.nav_id = handle.id;
        handle
    }

    pub fn focus_handle(&mut self, node: NavigationNode) -> FocusHandle {
        let state = self.use_state::<FocusHandle>(|ctx| ctx.new_focus_handle(node));
        let result = (*state).clone();
        self.nav_id = result.id;
        result
    }

    pub fn render_text(
        &self,
        text: &str,
        text_properties: &TextProperties,
        max_width: f32,
        max_height: f32,
    ) -> TextVertices {
        self.app_context
            .text_system
            .render_text(text, text_properties, max_width, max_height)
    }
}
