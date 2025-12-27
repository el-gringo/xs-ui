pub use xs_backend_core::*;

#[cfg(target_arch = "wasm32")]
pub use xs_backend_browser::Backend;

#[cfg(target_os = "linux")]
pub use xs_backend_wayland::Backend;
