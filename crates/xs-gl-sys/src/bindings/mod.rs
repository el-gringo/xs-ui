pub mod common_sys;

#[cfg(not(target_arch = "wasm32"))]
mod opengl_sys;
#[cfg(not(target_arch = "wasm32"))]
pub use opengl_sys::*;

#[cfg(target_arch = "wasm32")]
mod webgl_sys;

#[cfg(target_arch = "wasm32")]
pub use webgl_sys::*;
