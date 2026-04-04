#[cfg(target_arch = "wasm32")]
pub use xs_backend_browser::{Backend, load_image};

#[cfg(target_os = "linux")]
pub use xs_backend_wayland::{Backend, load_image};
