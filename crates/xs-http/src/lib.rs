// #[cfg(target_arch = "wasm32")]
//pub mod wasm;
pub mod client;

pub mod digest;
pub mod request;
pub mod response;
pub mod server;
pub mod socket;
pub mod utils;

#[cfg(target_arch = "wasm32")]
pub use wasm::HttpClient;

#[cfg(not(target_arch = "wasm32"))]
pub use client::HttpClient;
