#![warn(clippy::all, clippy::pedantic)]

mod color_space;
mod decoder;
mod error;
mod image_buffer;
mod image_format;

pub use color_space::ColorSpace;
pub use decoder::{DecoderRegistry, ImageDecoder, ImageDimensions};
pub use error::{ImageError, ImageResult};
pub use image_buffer::ImageBuffer;
pub use image_format::ImageFormat;
