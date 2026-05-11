use std::{
    any::Any,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub type StateId = String;

pub trait StateElement: Any {
    fn inner_element(&mut self) -> &mut dyn Any;
    fn as_any(&mut self) -> &mut dyn Any;
}

pub struct AnyState(Box<dyn StateElement>);
impl AnyState {
    pub fn new<S>(state: S) -> Self
    where
        S: 'static + StateElement,
    {
        Self(Box::new(state))
    }

    pub fn downcast<S: Clone + 'static>(&mut self) -> &mut State<S> {
        //let value = self.0.inner_element().downcast_mut().unwrap();
        //State::new(value)
        self.0.as_any().downcast_mut::<State<S>>()
    }
}

pub struct State<S> {
    value: S,
    _marker: PhantomData<S>,
}

impl<S: 'static> State<S> {
    pub fn new(value: S) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    pub fn read(&self) -> &S {
        &self.value
    }

    pub fn update(&mut self, mutator: impl FnOnce() -> S) {
        self.value = mutator();
    }
}

impl<S: 'static> Deref for State<S> {
    type Target = S;
    fn deref(&self) -> &S {
        &self.value
    }
}

impl<S: 'static> DerefMut for State<S> {
    fn deref_mut(&mut self) -> &mut S {
        &mut self.value
    }
}

pub struct Updater<S> {
    node_id: crate::core::NodeId,
    state_id: StateId,
    _marker: PhantomData<S>,
}

impl<S: 'static> Updater<S> {
    pub(crate) fn new(node_id: crate::core::NodeId, state_id: StateId) -> Self {
        Self {
            node_id,
            state_id,
            _marker: PhantomData,
        }
    }

    pub fn update(&self, value: impl FnOnce() -> S) {
        let app = crate::get_app_context();
        if let Some(state) = app
            .states
            .get_mut(&self.node_id)
            .and_then(|s| s.get_mut(&self.state_id))
            .and_then(|s| s.downcast::<S>())
        {
            state.update(value);
        }
    }
}

impl<S: 'static> StateElement for State<S> {
    fn inner_element(&mut self) -> &mut dyn Any {
        &mut self.value
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
