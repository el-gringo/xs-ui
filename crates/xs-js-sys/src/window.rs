use crate::Closure;

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn console_log(ptr: u32, len: u32);
        pub fn request_animation_frame(closure: *mut (), vcall: usize);
    }
}

pub fn log(message: &str) {
    let bytes = message.as_bytes();
    unsafe {
        js::console_log(bytes.as_ptr() as u32, bytes.len() as u32);
    }
}

pub fn request_animation_frame(closure: &mut Closure<u32>) {
    unsafe {
        let (ptr, len) = closure.to_js();
        js::request_animation_frame(ptr as _, len)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn callback_request_animation_frame(closure: *mut (), vcall: usize, now: u32) {
    unsafe {
        let vcall: fn(*mut (), u32) = std::mem::transmute(vcall);
        vcall(closure, now);
    }
}
