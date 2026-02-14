use std::collections::HashMap;

pub type HttpHeaders = HashMap<String, String>;

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HttpHeaders,
    pub body: Vec<u8>
}

impl Into<Vec<u8>> for HttpResponse {
    fn into(self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let status = String::new();
        let status_line = format!("HTTP/1.1 {} {}\r\n", self.status_code, status);
        buffer.append(&mut status_line.as_bytes().to_vec());
        let headers = self.headers;
        /*
        headers.insert(
            "Content-Length".to_string(),
            self.body.len().to_string()
        );
        headers.insert(
            "Server".to_string(),
            "Rust XS Server".to_string(),
        );
        */
        let mut headers = headers
            .iter()
            .map(|h| format!("{}: {}\r\n", h.0, h.1))
            .collect::<Vec<String>>()
            .concat()
            .as_bytes()
            .to_vec();
        buffer.append(&mut headers);
        let mut body = self.body;
        buffer.append(&mut "\r\n".as_bytes().to_vec());
        buffer.append(&mut body);
        buffer
    }
}
/*
impl Into<&[u8]> for HttpResponse {
    fn into(self) -> &[u8] {
        let buffer: Vec<u8> = self.into();
        &buffer
    }
}
*/

impl HttpResponse {
    pub fn new(status_code: u16, headers: HttpHeaders, body: Vec<u8>) -> Self {
        Self {
            status_code,
            headers,
            body
        }
    }
}
