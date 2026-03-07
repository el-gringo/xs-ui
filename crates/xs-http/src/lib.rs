pub mod digest;
pub mod headers;
pub mod response;
pub mod utils;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::HttpClient;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::HttpClient;
#[cfg(not(target_arch = "wasm32"))]
pub use native::HttpServer;
