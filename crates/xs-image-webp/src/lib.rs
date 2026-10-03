#![warn(clippy::all, clippy::pedantic)]

mod libwebp_sys;
mod webp;

pub use libwebp_sys::LibWebP;
pub use webp::WebPDecoder;
