#[cfg(target_arch = "wasm32")]
pub mod wasm;
pub mod client;


pub mod request;
pub mod digest;
pub mod utils;
pub mod server;
pub mod response;
pub mod socket;


#[cfg(target_arch = "wasm32")]
pub use wasm::HttpClient;

#[cfg(not(target_arch = "wasm32"))]
pub use client::HttpClient;
