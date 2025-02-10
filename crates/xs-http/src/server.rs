use crate::request::HttpRequest;


pub struct HttpServer {
    pub listener: std::net::TcpListener,
}

pub struct IncomingRequests<'a> {
    pub listener: &'a std::net::TcpListener
}

impl Iterator for IncomingRequests<'_> {
    type Item = HttpRequest;

    fn next(&mut self) -> Option<HttpRequest> {
        let mut incoming = self.listener.incoming();
        match incoming.next() {
            Some(next) => match next {
                Ok(tcp_listener) => match HttpRequest::read(tcp_listener) {
                    Ok(request) => Some(request),
                    _ => None
                },
                Err(_) => None
            },
            None => None
        }
    }
}

impl HttpServer {
    pub fn listen(host: &str, port: u16) -> Self {
        let address = std::net::SocketAddr::new(host.parse().unwrap(), port);
        let listener = std::net::TcpListener::bind(address).expect(
            &format!("unable to bind port {:?} {}", host, port)
        );
        Self { listener }
    }

    pub fn incoming_requests(&mut self) -> IncomingRequests<'_> {
        IncomingRequests { listener: &self.listener }
    }
}
