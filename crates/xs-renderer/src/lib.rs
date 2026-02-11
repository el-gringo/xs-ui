#[cfg(target_arch = "wasm32")]
pub use xs_renderer_webgl::RendererWebGl as Renderer;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
pub use xs_renderer_opengl::RendererOpenGl as Renderer;
