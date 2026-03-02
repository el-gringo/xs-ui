#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub texture: u32,
    pub bytes: Vec<u8>,
    pub alpha: bool
}

impl Image {
    pub fn new(width: u32, height: u32, texture: u32, bytes: Vec<u8>, alpha: bool) -> Self {
        Self {
            width,
            height,
            texture,
            bytes,
            alpha
        }
    }
}
