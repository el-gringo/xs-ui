unsafe extern "C" {
    pub fn run();
    pub fn init_canvas();
    pub fn console_log(ptr: u32, len: u32);
    pub fn now() -> f64;
    pub fn request_animation_frame();
}
