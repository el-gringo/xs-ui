#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_os = "linux")]
pub mod client;

pub mod digest;
pub mod request;
pub mod response;
#[cfg(target_os = "linux")]
pub mod server;
#[cfg(target_os = "linux")]
pub mod socket;
pub mod utils;

#[cfg(target_arch = "wasm32")]
pub use wasm::HttpClient;

#[cfg(not(target_arch = "wasm32"))]
pub use client::HttpClient;
