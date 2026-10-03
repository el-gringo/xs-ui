#![warn(clippy::all, clippy::pedantic)]

mod libpng_sys;
mod png;

pub use libpng_sys::LibPng;
pub use png::{PngDecoder, PngReader};
