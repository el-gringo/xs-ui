use crate::{Closure, EventEmitter, core::JsValue};

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn console_log(ptr: u32, len: u32);
        pub fn console_debug2(ptr_arg1: u32, len_arg1: u32, ptr_arg2: u32, len_arg2: u32);
        pub fn console_info2(ptr_arg1: u32, len_arg1: u32, ptr_arg2: u32, len_arg2: u32);
        pub fn console_error2(ptr_arg1: u32, len_arg1: u32, ptr_arg2: u32, len_arg2: u32);
        pub fn console_trace2(ptr_arg1: u32, len_arg1: u32, ptr_arg2: u32, len_arg2: u32);
        pub fn console_warn2(ptr_arg1: u32, len_arg1: u32, ptr_arg2: u32, len_arg2: u32);
        pub fn request_animation_frame(closure: *mut (), vcall: usize);
        pub fn window() -> u32;
        pub fn performance_now() -> u32;
    }
}

pub trait Window: Sized + EventEmitter {
    fn request_animation_frame(&self, closure: &mut Closure<u32>);
}

impl Window for JsValue {
    fn request_animation_frame(&self, closure: &mut Closure<u32>) {
        unsafe {
            let (ptr, len) = closure.to_js();
            js::request_animation_frame(ptr as _, len)
        }
    }
}

pub fn window() -> impl Window {
    unsafe {
        let ptr = js::window();
        JsValue::new(ptr)
    }
}

pub fn performance_now() -> u32 {
    unsafe { js::performance_now() }
}

pub fn log(message: &str) {
    let bytes = message.as_bytes();
    unsafe {
        js::console_log(bytes.as_ptr() as u32, bytes.len() as u32);
    }
}

pub fn log_debug(message: &str, extra: &str) {
    let arg1 = message.as_bytes();
    let arg2 = extra.as_bytes();
    unsafe {
        js::console_debug2(
            arg1.as_ptr() as u32,
            arg1.len() as u32,
            arg2.as_ptr() as u32,
            arg2.len() as u32,
        );
    }
}

pub fn log_info(message: &str, extra: &str) {
    let arg1 = message.as_bytes();
    let arg2 = extra.as_bytes();
    unsafe {
        js::console_info2(
            arg1.as_ptr() as u32,
            arg1.len() as u32,
            arg2.as_ptr() as u32,
            arg2.len() as u32,
        );
    }
}

pub fn log_error(message: &str, extra: &str) {
    let arg1 = message.as_bytes();
    let arg2 = extra.as_bytes();
    unsafe {
        js::console_error2(
            arg1.as_ptr() as u32,
            arg1.len() as u32,
            arg2.as_ptr() as u32,
            arg2.len() as u32,
        );
    }
}

pub fn log_trace(message: &str, extra: &str) {
    let arg1 = message.as_bytes();
    let arg2 = extra.as_bytes();
    unsafe {
        js::console_trace2(
            arg1.as_ptr() as u32,
            arg1.len() as u32,
            arg2.as_ptr() as u32,
            arg2.len() as u32,
        );
    }
}

pub fn log_warn(message: &str, extra: &str) {
    let arg1 = message.as_bytes();
    let arg2 = extra.as_bytes();
    unsafe {
        js::console_warn2(
            arg1.as_ptr() as u32,
            arg1.len() as u32,
            arg2.as_ptr() as u32,
            arg2.len() as u32,
        );
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
