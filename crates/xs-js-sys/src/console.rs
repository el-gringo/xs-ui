mod window {
    unsafe extern "C" {
        pub fn console_log(ptr: u32, len: u32);
    }

}

pub fn log(message: &str) {
    let bytes = message.as_bytes();
    unsafe {
        window::console_log(bytes.as_ptr() as u32, bytes.len() as u32);
    }
}
