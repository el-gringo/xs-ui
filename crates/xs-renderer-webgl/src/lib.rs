mod bindings;
mod renderer_webgl;
pub mod webgl;
mod glsl;

pub use webgl as WebGl;
pub use glsl::StandardShader as Shader;
pub use renderer_webgl::RendererWebGL as Renderer;
pub use renderer_webgl::RendererWebGL;
