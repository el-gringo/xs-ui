mod shader_glsl;
mod texture;

pub use shader_glsl::{ShaderGlSl, compile_shader, link_program};
pub use texture::load_texture;

pub use xs_gl_sys::*;
