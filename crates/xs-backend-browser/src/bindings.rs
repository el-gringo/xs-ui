#[cfg(target_arch = "wasm32")]
use xs_js_sys::Closure;
use xs_js_sys::{AnyClosure, drop_closure, invoke_closure, store_closure};

mod js {
    unsafe extern "C" {
        pub fn init(width: i32, height: i32);
        pub fn load_image(handle: usize, ptr_src: u32, ptr_len: u32);
        pub fn new_image(ptr: u32, len: u32, width: u32, height: u32) -> u32;
    }
}

pub fn init(width: i32, height: i32) {
    unsafe {
        js::init(width, height);
    }
}

pub fn load_image(src: String, resolve: impl FnMut((usize, u32, u32)) + 'static) {
    let closure = Closure::new(resolve);
    let bytes = src.as_bytes();
    let ptr_src = bytes.as_ptr();
    let len_src = bytes.len();
    let handle = store_closure(AnyClosure::new(closure));
    unsafe {
        js::load_image(handle, ptr_src as _, len_src as _);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn load_image_callback(handle: usize, ptr_img: usize, width: u32, height: u32) {
    invoke_closure(handle, (ptr_img, width, height));
    drop_closure(handle);
}

pub fn new_image(mut bytes: Vec<u8>, width: u32, height: u32) -> u32 {
    let ptr = bytes.as_mut_ptr() as u32;
    let len = bytes.len().try_into().unwrap();
    unsafe { js::new_image(ptr, len, width, height) }
}
