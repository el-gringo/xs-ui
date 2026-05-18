use std::{borrow::Cow, error::Error};

use crate::headers::HttpHeaders;

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HttpHeaders,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status_code: u16, headers: HttpHeaders, body: Vec<u8>) -> Self {
        Self {
            status_code,
            headers,
            body,
        }
    }

    pub fn body_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }
}

impl From<HttpResponse> for Vec<u8> {
    fn from(value: HttpResponse) -> Self {
        let mut buffer: Vec<u8> = Vec::new();
        let status = String::new();
        let status_line = format!("HTTP/1.1 {} {}\r\n", value.status_code, status);
        buffer.append(&mut status_line.as_bytes().to_vec());
        let headers = value.headers;
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
        let mut body = value.body;
        buffer.append(&mut "\r\n".as_bytes().to_vec());
        buffer.append(&mut body);
        buffer
    }
}

#[derive(Debug)]
enum HttpResponseErrorKind {
    InvalidStatusLine,
    InvalidStatusCode,
}

#[derive(Debug)]
pub struct HttpResponseError {
    kind: HttpResponseErrorKind,
}

impl std::fmt::Display for HttpResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

impl HttpResponseError {
    fn new(kind: HttpResponseErrorKind) -> Self {
        Self { kind }
    }
}

impl Error for HttpResponseError {
    fn description(&self) -> &str {
        match self.kind {
            HttpResponseErrorKind::InvalidStatusLine => "invalid status line",
            HttpResponseErrorKind::InvalidStatusCode => "invalid status code",
        }
    }
}

pub fn parse_header_line(header_line: String) -> (String, String) {
    let (key, value) = header_line.split_once(":").unwrap();
    (key.trim().to_string(), value.trim().to_string())
}

pub fn parse_response_status(status_line: String) -> Result<(u16, String), HttpResponseError> {
    let mut split: Vec<&str> = status_line.splitn(3, " ").map(|s| s.trim()).collect();
    if split.len() == 2 {
        split.push("");
    }
    if split.len() != 3 {
        return Err(HttpResponseError::new(
            HttpResponseErrorKind::InvalidStatusLine,
        ));
    }
    match split[1].parse() {
        Ok(code) => Ok((code, split[2].to_string())),
        Err(_) => Err(HttpResponseError::new(
            HttpResponseErrorKind::InvalidStatusCode,
        )),
    }
}
