use std::collections::BTreeMap;

use xs_url::url::Url;

pub type HttpHeaders = BTreeMap<String, String>;

pub fn prepare_headers(method: String, url: &Url, headers: HttpHeaders) -> Vec<String> {
    let mut payload: Vec<String> = Vec::new();
    let request_line = format!("{} {} HTTP/1.1", method, url.pathname);
    payload.push(request_line);
    payload.push(format!("Host: {}", url.hostname));
    for (key, value) in headers {
        payload.push(format!("{}: {}", key, value))
    }
    payload.push(String::new());
    payload.push(String::new());
    payload
}

#[cfg(test)]
mod tests {
    use xs_url::url::Url;

    use crate::headers::{HttpHeaders, prepare_headers};

    #[test]
    fn generate_request_with_headers() {
        let url = Url::parse("http://localhost:8080/").unwrap();
        let mut headers = HttpHeaders::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        let payload = prepare_headers("GET".to_string(), &url, headers);
        assert_eq!(payload.len(), 6);
        assert_eq!(payload[0], "GET / HTTP/1.1");
        assert_eq!(payload[1], "Host: localhost:8080");
        assert_eq!(payload[2], "Authorization: Bearer token");
        assert_eq!(payload[3], "Content-Type: application/json");
        assert_eq!(payload[4], "");
        assert_eq!(payload[5], "");
    }
}
