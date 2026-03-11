mod common;
pub use common::*;

#[cfg(not(target_arch = "wasm32"))]
mod opengl;
#[cfg(not(target_arch = "wasm32"))]
pub use opengl::*;

#[cfg(target_arch = "wasm32")]
mod webgl;
#[cfg(target_arch = "wasm32")]
pub use webgl::*;
