#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub texture: u32,
    pub rgba: Vec<u8>
}

impl Image {
    pub fn new(width: u32, height: u32, texture_index: u32, rgba: Vec<u8>) -> Self {
        Self {
            width,
            height,
            texture: texture_index,
            rgba
        }
    }
}
