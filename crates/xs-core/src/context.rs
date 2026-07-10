use std::{any::Any, cell::RefCell, rc::Rc};

use xs_layout::{BoxConstraints, Color, TextProperties, TextVertices};

use crate::{
    AppContext, FocusHandle, NavigationNode,
    navigation::NavigationNodeId,
    state::{AnyState, State},
};

pub struct Context<'a> {
    pub app_context: &'a mut AppContext,
    pub node_id: crate::core::NodeId,
    pub nav_id: NavigationNodeId,
    pub text_properties: TextProperties,
    pub color: Color,
    pub constraints: BoxConstraints,
}

impl<'a> Context<'a> {
    pub fn new(
        app_context: &'a mut AppContext,
        node_id: crate::core::NodeId,
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
        }
    }

    #[track_caller]
    pub fn use_state<S>(
        &mut self,
        init: impl FnOnce(&mut Self, State<S>) -> S,
    ) -> State<S>
    where
        S: Any + 'static,
    {
        let caller = core::panic::Location::caller();
        let key = &format!("{}:{}:{}", caller.file(), caller.line(), caller.column());
        self.use_keyed_state(key, init)
    }

    #[track_caller]
    pub fn use_keyed_state<S>(
        &mut self,
        key: &str,
        init: impl FnOnce(&mut Self, State<S>) -> S,
    ) -> State<S>
    where
        S: Any + 'static,
    {
        let node_id = self.node_id;
        let caller = core::panic::Location::caller();
        let prefixed = format!("{}:{}:{}:{}", caller.file(), caller.line(), caller.column(), key);
        let key = &prefixed;
        let needs_init = {
            let entry = self.app_context.states.entry(node_id).or_default();
            !entry.contains_key(key)
        };
        if needs_init {
            let state = State {
                inner: Rc::new(RefCell::new(None)),
            };
            let value = init(self, state.clone());
            *state.inner.borrow_mut() = Some(value);
            self.app_context
                .states
                .entry(node_id)
                .or_default()
                .insert(key.into(), AnyState::new(state));
        }
        self.app_context
            .states
            .get_mut(&node_id)
            .expect("use_state: no state map for node_id")
            .get_mut(key)
            .unwrap_or_else(|| panic!("use_state: no state for state_id={key}"))
            .downcast::<S>()
            .unwrap_or_else(|| panic!("use_state: downcast failed for state_id={key}"))
            .clone()
    }

    fn new_focus_handle(&mut self, node: NavigationNode) -> FocusHandle {
        let handle = self.app_context.nav.create_handle(self.nav_id, node);
        self.nav_id = handle.id;
        handle
    }

    pub fn focus_handle(&mut self, node: NavigationNode) -> FocusHandle {
        let handle_state =
            self.use_state::<FocusHandle>(|ctx, _| ctx.new_focus_handle(node));
        let result = handle_state.read().clone();
        self.nav_id = result.id;
        result
    }

    pub fn focus_handle_with_key(&mut self, key: &str, node: NavigationNode) -> FocusHandle {
        let state =
            self.use_keyed_state::<FocusHandle>(key, |ctx, _| ctx.new_focus_handle(node));
        let result = state.read().clone();
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
