use std::error::Error;
use std::io::{BufRead, BufReader, Read};
use std::collections::BTreeMap;
use std::sync::Arc;

use xs_url::url::Url;

use crate::response::{HttpHeaders, HttpResponse};
use crate::socket::Socket;


#[derive(Debug)]
enum HttpResponseErrorKind {
    InvalidStatusLine,
    InvalidStatusCode
}

#[derive(Debug)]
struct HttpResponseError {
    kind: HttpResponseErrorKind
}

impl std::fmt::Display for HttpResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_string().fmt(f)
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

fn parse_header_line(header_line: String) -> (String, String) {
    let (key, value) = header_line.split_once(":").unwrap();
    (key.trim().to_string(), value.trim().to_string())
}

fn prepare_headers(method: String, url: Url, headers: HttpHeaders) -> Vec<String> {
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

fn parse_response_status(status_line: String) -> Result<(u16, String), HttpResponseError> {
    let mut split: Vec<&str> = status_line
        .splitn(3," ")
        .map(|s| s.trim())
        .collect();
    if split.len() == 2 {
        split.push("");
    }
    if split.len() != 3 {
        return Err(
            HttpResponseError::new(HttpResponseErrorKind::InvalidStatusLine)
        );
    }
    match split[1].parse() {
        Ok(code) => Ok((code, split[2].to_string())),
        Err(_) => Err(
            HttpResponseError::new(HttpResponseErrorKind::InvalidStatusCode)
        )
    }
}

pub struct HttpClient {
    url: Url,
    headers: HttpHeaders,
    socket: Option<Arc<Socket>>
}

impl HttpClient {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            headers: HttpHeaders::new(),
            socket: None
        }
    }

    pub fn socket(mut self, socket: Arc<Socket>) -> Self {
        self.socket = Some(socket);
        self
    }

    pub fn header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn headers(mut self, headers: HttpHeaders) -> Self {
        self.headers = headers;
        self
    }

    pub fn get(self)  -> HttpResponse { self.send("GET".into(), None) }
    pub fn post(self, body: Option<&[u8]>)  -> HttpResponse { self.send("POST".into(), body) }
    pub fn put(self, body: Option<&[u8]>)  -> HttpResponse { self.send("PUT".into(), body) }
    pub fn patch(self, body: Option<&[u8]>)  -> HttpResponse { self.send("PATCH".into(), body) }
    pub fn delete(self, body: Option<&[u8]>)  -> HttpResponse { self.send("DELETE".into(), body) }

    fn create_socket(&mut self) -> Socket {
        let port = self.url.port.unwrap();
        let host = self.url.host.as_str();
        Socket::bind(host, port, self.url.secure).unwrap()
    }

    fn send(mut self, method: &str, body: Option<&[u8]>) -> HttpResponse {
        let url = self.url.to_owned();
        let raw_headers= self.headers.to_owned();
        let headers = prepare_headers(method.into(), url, raw_headers.clone());
        let socket = match self.socket.clone() {
            Some(s) => s.clone(),
            None => Arc::new(self.create_socket())
        };
        let binding = socket.clone();
        let mut output_stream = binding.output_stream.lock().unwrap();
        output_stream.write_all(headers.join("\r\n").as_bytes()).unwrap();
        if let Some(body) = body {
            output_stream.write_all(body).unwrap();
        }
        self.read_response(socket)
    }

    fn read_response(&mut self, socket: Arc<Socket>) -> HttpResponse {
        let mut input_stream = socket.input_stream.lock().unwrap();
        let mut reader = BufReader::new(&mut *input_stream);
        let mut done = false;
        let mut status_line = String::new();
        let mut headers = HttpHeaders::new();
        let mut len = reader.read_line(&mut status_line).unwrap();
        let (status_code, _) = parse_response_status(status_line.clone()).unwrap();
        while len > 0 && !done {
            let mut line = String::new();
            len = reader.read_line(&mut line).unwrap();
            if line.trim() == "" {
                done = true;
            } else {
                let (key, value) = parse_header_line(line);
                headers.insert(key.to_lowercase(), value);
            }
        }
        let body = match headers.get("content-length") {
            Some(length_str) => {
                let size: usize = length_str.parse().unwrap();
                let mut body: Vec<u8> = vec![0; size];
                reader.read_exact(&mut body).unwrap();
                body
            },
            _ => Vec::new()
        };
        HttpResponse::new(status_code, headers, body)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, io::Write};
    use xs_url::url::Url;
    use crate::client::{HttpClient, prepare_headers};

    const TEST_HOST: &str = "127.0.0.1";
    const TEST_PORT: u16 = 63215;

    fn listen() {
        let address = std::net::SocketAddr::new(TEST_HOST.parse().unwrap(), TEST_PORT);
        let listener = std::net::TcpListener::bind(address).expect(
            &format!("unable to bind port {:?} {}", TEST_HOST, TEST_PORT)
        );
        std::thread::spawn(move || {
            let (mut tcp_stream, _) = listener.accept()
                .expect("unable to accept connections");
            tcp_stream.write(
                "HTTP/1.1 200\r\n
                content-length: 10\r\n
                \r\n
                \r\n
                it works !".as_bytes()
            ).expect("unable to write tcp_stream");
        });
    }

    #[test]
    fn generate_request_with_headers() {
        let url = Url::parse("http://localhost:8080/").unwrap();
        let mut headers = BTreeMap::new();
        headers.insert("Authorization".to_string(), vec!["Bearer token".to_string()]);
        headers.insert("Content-Type".to_string(), vec!["application/json".to_string()]);
        let payload = prepare_headers("GET".to_string(), url, headers);
        assert_eq!(payload.len(), 6);
        assert_eq!(payload[0], "GET / HTTP/1.1");
        assert_eq!(payload[1], "Host: localhost:8080");
        assert_eq!(payload[2], "Authorization: Bearer token");
        assert_eq!(payload[3], "Content-Type: application/json");
        assert_eq!(payload[4], "");
        assert_eq!(payload[5], "");
    }

    #[test]
    fn it_works() {
        listen();
        let http = HttpClient::new(format!("http://{}:{}/", TEST_HOST, TEST_PORT).into());
        let response = http.get();
        assert_eq!(response.status_code, 200);
    }
}
