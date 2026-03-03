use crate::{Closure, log};

mod js {
    unsafe extern "C" {
        pub fn fetch(ptr: u32, len: u32, resolve: *mut (), vcall: usize);
    }
}

#[allow(unused)]
pub struct FetchInit {
    method: String,
}

pub fn fetch(url: String, _init: Option<&FetchInit>, resolve: impl FnMut(String) + 'static) {
    let mut closure = Closure::new(resolve);
    let bytes = url.as_bytes();
    let ptr = bytes.as_ptr();
    let len = bytes.len();
    unsafe {
        let (ptr_closure, len_closure) = closure.to_js();
        js::fetch(ptr as _, len as _, ptr_closure, len_closure);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn fetch_callback(ptr: *mut u8, len: usize, closure: *mut (), vcall: usize) {
    unsafe {
        let body = String::from_raw_parts(ptr, len, len);
        let vcall: fn(String, *mut ()) = std::mem::transmute(vcall);
        vcall(body, closure);
    }
}
