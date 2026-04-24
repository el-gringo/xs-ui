use std::{any::Any, cell::RefCell, marker::PhantomData};

// Wrapper with unsafe Sync (WASM is single-threaded)
struct ClosureTable {
    table: RefCell<Vec<Option<AnyClosure>>>,
}

unsafe impl Sync for ClosureTable {}

static CLOSURE_TABLE: ClosureTable = ClosureTable {
    table: RefCell::new(Vec::new()),
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
    let mut table = CLOSURE_TABLE.table.borrow_mut();
    let handle = table
        .iter()
        .position(|slot| slot.is_none())
        .unwrap_or_else(|| {
            let idx = table.len();
            table.push(None);
            idx
        });
    table[handle] = Some(closure);
    handle
}

// Get AnyClosure from table and call with args
pub fn invoke_closure<Args>(handle: usize, args: Args)
where
    Args: 'static,
{
    let mut table = CLOSURE_TABLE.table.borrow_mut();
    if let Some(closure) = table[handle].as_mut() {
        let inner = closure.downcast_mut::<Closure<Args>>().unwrap();
        inner.call(args);
    }
}

// Remove closure from table
pub fn drop_closure(handle: usize) {
    let mut table = CLOSURE_TABLE.table.borrow_mut();
    if handle < table.len() {
        table[handle] = None;
    }
}
