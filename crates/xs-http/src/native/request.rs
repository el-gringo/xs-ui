use std::io::{BufRead, Read, Write};

use crate::response::HttpResponse;

/// How many header lines a request may carry before it is given up on.
const MAX_HEADER_LINES: usize = 200;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub pathname: String,
    pub protocol: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Vec<u8>,
    pub stream: std::net::TcpStream,
}

impl HttpRequest {
    /// Reads the header block, one line at a time, up to the blank line that ends it.
    ///
    /// `read_line` reports end of input as `Ok(0)`, not as an error, so a peer
    /// that connects and then closes - or never says anything at all - would
    /// otherwise loop forever: the empty line matches neither stop condition and
    /// the loop keeps pushing it, growing the `Vec` until the process is killed
    /// for using all the memory. Browsers open and drop connections routinely, so
    /// this is the common case rather than a corner one.
    fn read_header_lines(reader: &mut std::io::BufReader<&mut std::net::TcpStream>) -> Vec<String> {
        let mut header_lines: Vec<String> = Vec::new();
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) if line == "\r\n" => break,
                Ok(_) => header_lines.push(std::mem::take(&mut line)),
                Err(_) => break,
            }
            // A request line plus headers is a few hundred bytes; anything past
            // that is a client sending something that is not a request, and it
            // is not worth the memory to keep reading it.
            if header_lines.len() > MAX_HEADER_LINES {
                break;
            }
        }
        header_lines
    }

    pub fn read(mut stream: std::net::TcpStream) -> Result<Self, &'static str> {
        let mut reader = std::io::BufReader::new(&mut stream);
        let mut headers = std::collections::HashMap::<String, String>::new();
        let header_lines = Self::read_header_lines(&mut reader);
        if header_lines.is_empty() {
            return Err("Headers are empty");
        }
        let request_line = header_lines[0].to_owned();
        let splitted_request_line: Vec<&str> = request_line.split(" ").collect();

        let (method, pathname, protocol) = match splitted_request_line.len() {
            3 => (
                splitted_request_line[0].to_string(),
                splitted_request_line[1].to_string(),
                splitted_request_line[2].to_string(),
            ),
            _ => return Err("Failed to parse header line"),
        };
        let method = method.to_uppercase();
        let protocol = protocol.trim().to_uppercase();

        for header_line in header_lines[1..header_lines.len()].iter() {
            let (name, value) = header_line
                .split_once(":")
                .expect("Unable to parse HTTP header");
            headers.insert(name.to_lowercase(), value.trim().into());
        }

        let content_length = match headers.get("content-length") {
            Some(content_length) => content_length.parse::<usize>().unwrap_or_default(),
            _ => 0,
        };
        let mut body = vec![0u8; content_length];

        if reader.read_exact(&mut body).is_ok() {
            return Ok(Self {
                method,
                pathname,
                protocol,
                headers,
                body,
                stream,
            });
        }
        Err("Failed to read HTTP headers")
    }

    pub fn respond(mut self, response: HttpResponse) {
        let buffer: Vec<u8> = response.into();
        if let Err(_error) = self.stream.write_all(&buffer) {};
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::sync::mpsc;
    use std::time::Duration;

    /// A peer that connects and closes without saying anything must not be able to
    /// spin the header reader: `read_line` calls that `Ok(0)`, and treating it as
    /// progress grows a `Vec` until the process runs out of memory.
    #[test]
    fn a_peer_that_hangs_up_immediately_is_rejected() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();

        let client = TcpStream::connect(address).unwrap();
        drop(client);

        let (accepted, _) = listener.accept().unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = HttpRequest::read(accepted);
            let _ = tx.send(result.is_err());
        });

        assert!(
            rx.recv_timeout(Duration::from_secs(5)).is_ok(),
            "reading a request from a closed connection never returned"
        );
    }

    /// A request that is cut off part way through its headers is an error, not a
    /// reason to keep reading.
    #[test]
    fn a_truncated_request_is_rejected() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();

        let mut client = TcpStream::connect(address).unwrap();
        client.write_all(b"GET / HTTP/1.1\r\nHost: x\r\n").unwrap();
        drop(client);

        let (accepted, _) = listener.accept().unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = HttpRequest::read(accepted);
            let _ = tx.send(result.is_err());
        });

        assert!(
            rx.recv_timeout(Duration::from_secs(5)).is_ok(),
            "reading a truncated request never returned"
        );
    }
}
