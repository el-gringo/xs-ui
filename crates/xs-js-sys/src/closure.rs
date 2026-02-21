use std::marker::PhantomData;

use crate::log;


pub struct Closure<T>(T);
impl<T> Closure<T>
where
    T: FnMut() + Sized,
{
    pub fn new(closure: T) -> Self {
        Self(closure)
    }

    pub fn call(&self) {
        unsafe {
            let vcall: fn(*mut ()) = |ptr| (*(ptr as *mut T))();
            vcall((&self.0) as *const T as *mut ());
        }
    }

    pub fn to_js(&self) -> (*mut (), usize) {
        unsafe {
            let vcall: fn(*mut ()) = |ptr| (*(ptr as *mut T))();
            ((&self.0) as *const T as *mut (), vcall as usize)
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn callback(closure: *mut (), vcall: usize) {
    log("callback");
    let vcall: fn(*mut ()) = unsafe { std::mem::transmute(vcall) };
    vcall(closure);
}
