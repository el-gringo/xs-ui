use crate::native::request::HttpRequest;

/// How long a connected peer has to finish sending its request.
const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

pub struct HttpServer {
    pub listener: std::net::TcpListener,
    /// How long a connected peer has to finish sending its request.
    pub read_timeout: std::time::Duration,
}

pub struct IncomingRequests<'a> {
    pub listener: &'a std::net::TcpListener,
    pub read_timeout: std::time::Duration,
}

impl Iterator for IncomingRequests<'_> {
    type Item = HttpRequest;

    fn next(&mut self) -> Option<HttpRequest> {
        let mut incoming = self.listener.incoming();
        match incoming.next() {
            Some(next) => match next {
                // Requests are read one at a time on the caller's thread, so a
                // peer that connects and then says nothing holds up every other
                // client for as long as it likes. A browser opens speculative
                // sockets and may not write to them for a while, so this is
                // ordinary rather than hostile, and the answer is a deadline on
                // the read rather than a thread per connection.
                Ok(stream) => {
                    let _ = stream.set_read_timeout(Some(self.read_timeout));
                    HttpRequest::read(stream).ok()
                }
                Err(_) => None,
            },
            None => None,
        }
    }
}

impl HttpServer {
    pub fn listen(host: &str, port: u16) -> Self {
        let address = std::net::SocketAddr::new(host.parse().unwrap(), port);
        let listener = std::net::TcpListener::bind(address).unwrap_or_else(|err| {
            panic!("unable to bind port {:?} {}\nERROR: {:?}", host, port, err)
        });
        Self {
            listener,
            read_timeout: READ_TIMEOUT,
        }
    }

    /// Shortens how long a connected peer has to finish sending its request.
    pub fn with_read_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.read_timeout = timeout;
        self
    }

    pub fn incoming_requests(&mut self) -> IncomingRequests<'_> {
        IncomingRequests {
            listener: &self.listener,
            read_timeout: self.read_timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpStream;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    /// A peer that connects and stays silent must not hold up the ones behind it.
    /// Requests are read serially, so without a deadline a single half-open
    /// connection stops the server answering anything at all - which is what a
    /// browser's speculative connection looks like from here.
    #[test]
    fn a_silent_peer_does_not_stall_the_server() {
        let mut server =
            HttpServer::listen("127.0.0.1", 0).with_read_timeout(Duration::from_millis(300));
        let address = server.listener.local_addr().unwrap();

        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut requests = server.incoming_requests();
            let first = requests.next();
            let second = requests.next();
            let _ = tx.send((first.is_some(), second.is_some()));
        });

        // Connected, and deliberately never writes anything.
        let _silent = TcpStream::connect(address).unwrap();
        std::thread::sleep(Duration::from_millis(50));

        let mut well_behaved = TcpStream::connect(address).unwrap();
        well_behaved
            .write_all(b"GET / HTTP/1.1\r\nHost: x\r\n\r\n")
            .unwrap();

        let started = Instant::now();
        let (first_is_some, second_is_some) = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(
            !first_is_some,
            "the silent connection should not have produced a request"
        );
        assert!(
            second_is_some,
            "the server never served the well-behaved request"
        );
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "serving waited for the silent connection instead of timing out"
        );
    }
}
