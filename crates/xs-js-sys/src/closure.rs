use std::{any::Any, marker::PhantomData};

pub struct Closure<T, E>(T, PhantomData<E>);
impl<T, E> Closure<T, E>
where
    T: FnMut(E) + 'static,
    E: Any
{
    pub fn new(closure: T) -> Self {
        Self(closure, PhantomData)
    }

    pub fn to_js(&mut self) -> (*mut (), usize) {
        unsafe {
            let vcall: fn(E, *mut ()) = |e, ptr| (*(ptr as *mut T))(e);
            ((&mut self.0) as *mut T as *mut (), vcall as usize)
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn callback(closure: *mut (), vcall: usize) {
    let vcall: fn(*mut ()) = unsafe { std::mem::transmute(vcall) };
    vcall(closure);
}
