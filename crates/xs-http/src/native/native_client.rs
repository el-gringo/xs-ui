use std::io::{BufRead, BufReader, Read};
use std::sync::Arc;
use std::thread::JoinHandle;

use xs_url::url::Url;

use crate::{
    headers::{HttpHeaders, prepare_headers},
    native::socket::Socket,
    response::{HttpResponse, parse_header_line, parse_response_status},
};

fn create_socket(url: &Url) -> Socket {
    let port = url.port.unwrap_or(if url.secure { 443 } else { 80 });
    let host = url.host.as_str();
    Socket::bind(host, port, url.secure).unwrap()
}

fn read_response(socket: Arc<Socket>) -> HttpResponse {
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
        }
        _ => Vec::new(),
    };
    HttpResponse::new(status_code, headers, body)
}

pub struct HttpClient {
    pub(crate) headers: HttpHeaders,
}

impl Default for HttpClient {
    fn default() -> Self {
        let mut headers: HttpHeaders = Default::default();
        // Some sites requires a User-Agent to avoid bots
        headers.insert("User-Agent".into(), "XS-Http".into());
        Self { headers }
    }
}

impl HttpClient {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
    pub fn headers(mut self, headers: HttpHeaders) -> Self {
        self.headers = headers;
        self
    }
    pub fn get(
        self,
        url: impl Into<Url>,
        on_response: impl FnMut(HttpResponse) + Send + 'static,
    ) -> JoinHandle<()> {
        self.send("GET", url, None, on_response)
    }
    pub fn post(
        self,
        url: impl Into<Url>,
        body: Option<&[u8]>,
        on_response: impl FnMut(HttpResponse) + Send + 'static,
    ) -> JoinHandle<()> {
        self.send("POST", url, body, on_response)
    }
    pub fn put(
        self,
        url: impl Into<Url>,
        body: Option<&[u8]>,
        on_response: impl FnMut(HttpResponse) + Send + 'static,
    ) -> JoinHandle<()> {
        self.send("PUT", url, body, on_response)
    }
    pub fn patch(
        self,
        url: impl Into<Url>,
        body: Option<&[u8]>,
        on_response: impl FnMut(HttpResponse) + Send + 'static,
    ) -> JoinHandle<()> {
        self.send("PATCH", url, body, on_response)
    }
    pub fn delete(
        self,
        url: impl Into<Url>,
        body: Option<&[u8]>,
        on_response: impl FnMut(HttpResponse) + Send + 'static,
    ) -> JoinHandle<()> {
        self.send("DELETE", url, body, on_response)
    }
    pub fn send(
        &self,
        method: impl Into<String>,
        url: impl Into<Url>,
        body: Option<&[u8]>,
        mut on_response: impl FnMut(HttpResponse) + Send + 'static,
    ) -> JoinHandle<()> {
        let method = method.into();
        let url = url.into();
        //let (tx, rx) = channel();
        let raw_headers = self.headers.to_owned();
        let headers = prepare_headers(method, &url, raw_headers.clone());
        let socket = Arc::new(create_socket(&url));
        let binding = socket.clone();
        let mut output_stream = binding.output_stream.lock().unwrap();
        output_stream
            .write_all(headers.join("\r\n").as_bytes())
            .unwrap();
        if let Some(body) = body {
            output_stream.write_all(body).unwrap();
        }
        std::thread::spawn(move || {
            let response = read_response(socket);
            on_response(response);
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::HttpClient;
    use std::io::Write;

    const TEST_HOST: &str = "127.0.0.1";
    const TEST_PORT: u16 = 63215;

    fn listen() {
        let address = std::net::SocketAddr::new(TEST_HOST.parse().unwrap(), TEST_PORT);
        let listener = std::net::TcpListener::bind(address).unwrap_or_else(|err| {
            panic!(
                "unable to bind port {:?} {}\nERROR: {:?}",
                TEST_HOST, TEST_PORT, err
            )
        });
        std::thread::spawn(move || {
            let (mut tcp_stream, _) = listener.accept().expect("unable to accept connections");
            tcp_stream
                .write_all(
                    "HTTP/1.1 200\r\n
                content-length: 10\r\n
                \r\n
                \r\n
                it works !"
                        .as_bytes(),
                )
                .expect("unable to write tcp_stream");
        });
    }

    #[test]
    fn it_works() {
        listen();
        let url = format!("http://{}:{}/", TEST_HOST, TEST_PORT);
        let http = HttpClient::new();
        let _ = http
            .get(url, |response| {
                assert_eq!(response.status_code, 200);
            })
            .join();
    }
}
