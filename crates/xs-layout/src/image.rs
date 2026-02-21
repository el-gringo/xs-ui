#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Image {
    ptr: i32,
    pub width: u32,
    pub height: u32,
    pub texture_index: u32
}

impl Image {
    pub fn new(ptr: i32, width: u32, height: u32, texture_index: u32) -> Self {
        Self {
            ptr,
            width,
            height,
            texture_index
        }
    }

    pub fn as_ptr(&self) -> i32 {
        self.ptr
    }
}
