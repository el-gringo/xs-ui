pub mod digest;
pub mod headers;
pub mod response;
pub mod utils;

pub use headers::HttpHeaders;
pub use response::HttpResponse;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::HttpClient;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::{HttpClient, HttpRequest, HttpServer};
