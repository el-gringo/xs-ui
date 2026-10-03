#![warn(clippy::all, clippy::pedantic)]

mod jpeg_xl;
mod libjxl_sys;

pub use jpeg_xl::JpegXlDecoder;
pub use libjxl_sys::LibJxl;
