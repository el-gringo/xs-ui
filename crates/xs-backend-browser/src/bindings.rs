#[cfg(target_arch = "wasm32")]
use xs_js_sys::Closure;

mod js {
    unsafe extern "C" {
        pub fn init(width: i32, height: i32);
        pub fn load_image(ptr_src: u32, ptr_len: u32, ptr_resolve: u32, len_resolve: u32);
        pub fn new_image(ptr: u32, len: u32, width: u32, height: u32) -> u32;
    }
}

pub fn init(width: i32, height: i32) {
    unsafe {
        js::init(width, height);
    }
}

pub fn load_image(src: String, resolve: impl FnMut(u32) + 'static) {
    let mut closure = Closure::new(resolve);
    let bytes = src.as_bytes();
    let ptr_src = bytes.as_ptr();
    let len_src = bytes.len();
    unsafe {
        let (ptr_resolve, len_resolve) = closure.to_js();
        js::load_image(
            ptr_src as _,
            len_src as _,
            ptr_resolve as _,
            len_resolve as _,
        );
    }
}

pub fn new_image(mut bytes: Vec<u8>, width: u32, height: u32) -> u32 {
    let ptr = bytes.as_mut_ptr() as u32;
    let len = bytes.len().try_into().unwrap();
    unsafe { js::new_image(ptr, len, width, height) }
}
