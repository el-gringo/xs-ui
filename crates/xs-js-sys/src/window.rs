use std::rc::Rc;

use crate::Closure;

mod js {
    unsafe extern "C" {
        pub fn console_log(ptr: u32, len: u32);
        pub fn request_animation_frame(closure: *mut (), vcall: usize) -> u32;
    }
}

pub fn log(message: &str) {
    let bytes = message.as_bytes();
    unsafe {
        js::console_log(bytes.as_ptr() as u32, bytes.len() as u32);
    }
}

pub fn request_animation_frame(closure: &mut Closure<impl FnMut(u32) + 'static, u32>) -> u32
{
    unsafe {
        let (ptr, len) = closure.to_js();
        js::request_animation_frame(ptr, len)
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn request_animation_frame_callback(now: u64, closure: *mut (), vcall: usize) {
    unsafe {
        let vcall: fn(u64, *mut ()) = std::mem::transmute(vcall);
        vcall(now, closure);
    }
}
