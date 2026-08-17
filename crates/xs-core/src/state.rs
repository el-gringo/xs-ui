use std::{
    any::Any,
    cell::{Ref, RefCell},
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

pub type StateId = String;

pub static PENDING_UPDATE: AtomicBool = AtomicBool::new(false);

pub struct AnyState {
    inner: Box<dyn Any>,
}

impl AnyState {
    pub fn new<S: 'static>(value: S) -> Self {
        Self {
            inner: Box::new(value),
        }
    }

    pub fn downcast<S: 'static>(&mut self) -> Option<&mut S> {
        self.inner.downcast_mut::<S>()
    }
}

pub struct State<S> {
    inner: Rc<RefCell<S>>,
}

impl<S: 'static> Clone for State<S> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
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

    pub fn set(&self, value: S) {
        *self.inner.borrow_mut() = value;
        PENDING_UPDATE.store(true, Ordering::Relaxed);
    }

    pub fn update(&self, f: impl FnOnce(&mut S)) {
        f(&mut self.inner.borrow_mut());
        PENDING_UPDATE.store(true, Ordering::Relaxed);
    }
}

// SAFETY: State is only used on the main thread (single-threaded event loop).
// Rc/RefCell are !Send, but the framework guarantees single-threaded access,
// so it is safe to send the State across thread boundaries for async callbacks.
unsafe impl<S: 'static> Send for State<S> {}
