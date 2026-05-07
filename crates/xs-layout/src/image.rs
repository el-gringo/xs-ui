#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum ImageFormat {
    Rgb,
    #[default]
    Rgba,
    Alpha,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub texture: u32,
    pub bytes: Vec<u8>,
    pub format: ImageFormat,
}

impl Image {
    #[must_use]
    pub fn new(width: u32, height: u32, texture: u32, bytes: Vec<u8>, format: ImageFormat) -> Self {
        Self {
            width,
            height,
            texture,
            bytes,
            format,
        }
    }
}
