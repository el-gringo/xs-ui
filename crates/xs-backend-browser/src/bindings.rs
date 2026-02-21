unsafe extern "C" {
    pub fn init(width: i32, height: i32);
    pub fn now() -> f64;
    pub fn load_image(ptr: u32, len: u32, texture_index: u32);
    pub fn new_image(ptr: u32, len: u32, width: u32) -> u32;
}
