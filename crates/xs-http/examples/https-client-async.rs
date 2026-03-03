use xs_http::HttpClient;

pub fn main() {
    let mut client = HttpClient::new("https://httpbin.io/status/418");
    let _ = client
        .send_async("GET", None, |response| {
            println!("response status code: {}", response.status_code);
            if let Ok(body) = String::from_utf8(response.body) {
                println!("response body:");
                println!("{}", body);
            }
        })
        .join();
}
