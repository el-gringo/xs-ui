use xs_http::HttpClient;

#[cfg(target_arch = "wasm32")]
use xs_js_sys::log;
#[cfg(not(target_arch = "wasm32"))]
fn log(message: &str) {
    println!("{message}")
}

pub fn main() {
    let client = HttpClient::new();
    let _ = client
        .get("https://httpbin.io/status/418", |response| {
            if let Ok(body) = String::from_utf8(response.body) {
                log(&format!("response body:"));
                log(&format!("{}", body));
            }
        })
        .join();
}
