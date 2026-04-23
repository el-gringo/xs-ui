mod attribute;
mod buffer;
mod shader;
mod texture;

pub use attribute::Attribute;
pub use buffer::Buffer;
pub use shader::{Shader, compile_shader, link_program};
pub use texture::load_texture;

pub use xs_gl_sys::*;
