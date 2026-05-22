use std::{
    any::Any,
    cell::{Ref, RefCell},
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

pub type StateId = String;

pub static PENDING_UPDATE: AtomicBool = AtomicBool::new(false);

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
    pub(crate) inner: Rc<RefCell<S>>,
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
        PENDING_UPDATE.store(true, Ordering::Relaxed);
    }

    pub fn mutate(&self, f: impl FnOnce(&mut S)) {
        f(&mut *self.inner.borrow_mut());
        PENDING_UPDATE.store(true, Ordering::Relaxed);
    }
}

// SAFETY: State is only used on the main thread (single-threaded event loop).
// Rc/RefCell are !Send, but the framework guarantees single-threaded access,
// so it is safe to send the State across thread boundaries for async callbacks.
unsafe impl<S: 'static> Send for State<S> {}

pub struct Setter<S> {
    pub(crate) inner: Rc<RefCell<S>>,
}

impl<S: 'static> Clone for Setter<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
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
