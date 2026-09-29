use std::{any::Any, cell::RefCell, marker::PhantomData};

// Wrapper with unsafe Sync (WASM is single-threaded)
struct ClosureTable {
    table: RefCell<Vec<Option<AnyClosure>>>,
    /// Handles whose slot is empty and can be handed out again.
    free: RefCell<Vec<usize>>,
}

unsafe impl Sync for ClosureTable {}

static CLOSURE_TABLE: ClosureTable = ClosureTable {
    table: RefCell::new(Vec::new()),
    free: RefCell::new(Vec::new()),
};

pub trait ClosureElement {
    fn inner_element(&mut self) -> &mut dyn Any;
}

pub struct Closure<Args: 'static> {
    inner: Box<dyn FnMut(Args)>,
    _marker: PhantomData<Args>,
}

impl<Args: 'static> Closure<Args> {
    pub fn new<F>(closure: F) -> Self
    where
        F: FnMut(Args) + 'static,
    {
        Self {
            inner: Box::new(closure),
            _marker: PhantomData,
        }
    }

    pub fn to_js(&mut self) -> (usize, usize) {
        let ptr = &mut self.inner as *mut Box<dyn FnMut(Args)> as *mut ();
        let vcall: fn(*mut (), Args) = |ptr, args| {
            let closure = unsafe { &mut *(ptr as *mut Box<dyn FnMut(Args)>) };
            closure(args);
        };
        (ptr as usize, vcall as usize)
    }

    pub fn call(&mut self, args: Args) {
        self.inner.as_mut()(args);
    }
}

impl<Args> ClosureElement for Closure<Args> {
    fn inner_element(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct AnyClosure(Box<dyn ClosureElement>);
impl AnyClosure {
    pub fn new<C>(closure: C) -> Self
    where
        C: 'static + ClosureElement,
    {
        Self(Box::new(closure))
    }

    pub fn downcast<T: 'static>(&mut self) -> Option<&mut T> {
        self.0.inner_element().downcast_mut::<T>()
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.0.inner_element().downcast_mut::<T>()
    }
}

// Store AnyClosure in table, return handle
pub fn store_closure(closure: AnyClosure) -> usize {
    // The free list, not the slots themselves: a slot is only on it when it is
    // really free, so a handler that runs right now keeps its handle.
    let handle = match CLOSURE_TABLE.free.borrow_mut().pop() {
        Some(handle) => handle,
        None => {
            let mut table = CLOSURE_TABLE.table.borrow_mut();
            let handle = table.len();
            table.push(None);
            handle
        }
    };
    CLOSURE_TABLE.table.borrow_mut()[handle] = Some(closure);
    handle
}

// Get AnyClosure from table and call with args
pub fn invoke_closure<Args>(handle: usize, args: Args)
where
    Args: 'static,
{
    // Take the closure out of the table before calling it, and give it back
    // after. A handler may well store or invoke another closure, which would
    // borrow the table again.
    let Some(mut closure) = CLOSURE_TABLE.table.borrow_mut()[handle].take() else {
        return;
    };
    if let Some(inner) = closure.downcast_mut::<Closure<Args>>() {
        inner.call(args);
    }
    CLOSURE_TABLE.table.borrow_mut()[handle] = Some(closure);
}

// Remove closure from table
pub fn drop_closure(handle: usize) {
    let mut table = CLOSURE_TABLE.table.borrow_mut();
    if handle < table.len() && table[handle].take().is_some() {
        drop(table);
        CLOSURE_TABLE.free.borrow_mut().push(handle);
    }
}
