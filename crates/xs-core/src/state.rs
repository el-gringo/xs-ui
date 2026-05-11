use std::{
    any::Any,
    cell::{Ref, RefCell},
    marker::PhantomData,
    rc::Rc,
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

    pub fn downcast<S: 'static>(&mut self) -> Option<&mut State<S>> {
        self.0.as_any().downcast_mut::<State<S>>()
    }
}

pub struct State<S> {
    inner: Rc<RefCell<S>>,
}

impl<S: 'static> Clone for State<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<S: 'static> State<S> {
    pub fn new(value: S) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
        }
    }

    pub fn read(&self) -> Ref<'_, S> {
        self.inner.borrow()
    }

    pub fn update(&self, mutator: impl FnOnce() -> S) {
        *self.inner.borrow_mut() = mutator();
    }
}

pub struct Updater<S> {
    node_id: crate::core::NodeId,
    state_id: StateId,
    _marker: PhantomData<S>,
}

impl<S: 'static> Clone for Updater<S> {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id.clone(),
            state_id: self.state_id.clone(),
            _marker: PhantomData,
        }
    }
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
        self
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
