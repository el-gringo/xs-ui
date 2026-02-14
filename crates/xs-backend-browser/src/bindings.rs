unsafe extern "C" {
    pub fn run();
    pub fn init_canvas();
    pub fn now() -> f64;
    pub fn load_image(ptr: u32, len: u32);
}
