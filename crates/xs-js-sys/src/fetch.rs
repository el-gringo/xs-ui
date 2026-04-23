use crate::{
    AnyClosure, Closure,
    closure::{drop_closure, invoke_closure, store_closure},
};

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn fetch(handle: usize, ptr: u32, len: u32);
    }
}

#[allow(unused)]
pub struct FetchInit {
    method: String,
}

pub fn fetch(url: &str, _init: Option<&FetchInit>, resolve: Closure<Vec<u8>>) {
    let bytes = url.as_bytes();
    let ptr = bytes.as_ptr();
    let len = bytes.len();
    unsafe {
        let handle = store_closure(AnyClosure::new(resolve));
        js::fetch(handle as _, ptr as _, len as _);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn fetch_callback(handle: usize, ptr: *mut u8, len: usize) {
    unsafe {
        let bytes = Vec::from_raw_parts(ptr, len, len);
        invoke_closure(handle, bytes);
        drop_closure(handle);
    }
}
