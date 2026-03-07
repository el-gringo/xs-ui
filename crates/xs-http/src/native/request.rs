use std::io::{BufRead, Read, Write};

use crate::response::HttpResponse;


#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub pathname: String,
    pub protocol: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Vec<u8>,
    pub stream: std::net::TcpStream
}

impl HttpRequest {
    fn read_header_lines(reader: &mut std::io::BufReader<&mut std::net::TcpStream>) -> Vec<String> {
        let mut header_lines: Vec<String> = Vec::new();
        let mut line = String::new();
        while reader.read_line(&mut line).is_ok() {
            if line == "\r\n" { break; }
            header_lines.push(line.clone());
            line.clear();
        }
        header_lines
    }

    pub fn read(mut stream: std::net::TcpStream) -> Result<Self, ()> {
        let mut reader = std::io::BufReader::new(&mut stream);
        let mut headers = std::collections::HashMap::<String, String>::new();
        let header_lines = Self::read_header_lines(&mut reader);
        if header_lines.len() == 0 {
            return Err(());
        }
        let request_line = header_lines[0].to_owned();
        let splitted_request_line: Vec<&str> = request_line
            .split(" ")
            .collect();

        let (method, pathname, protocol) = match splitted_request_line.len() {
            3 => (
                splitted_request_line[0].to_string(),
                splitted_request_line[1].to_string(),
                splitted_request_line[2].to_string()
            ),
            _ => return Err(())
        };
        let method = method.to_uppercase();
        let protocol = protocol.trim().to_uppercase();

        for header_line in header_lines[1..header_lines.len()].iter() {
            let (name, value) = header_line.split_once(":").expect("unable to parse HTTP header");
            headers.insert(name.to_lowercase().into(), value.trim().into());
        }

        let content_length = match headers.get("content-length") {
            Some(content_length) => match content_length.parse::<usize>() {
                Ok(content_length) => content_length,
                _ => 0
            },
            _ => 0
        };
        let mut body = vec![0u8; content_length];

        if reader.read_exact(&mut body).is_ok() {
            return Ok(
                Self {
                    method,
                    pathname,
                    protocol,
                    headers,
                    body,
                    stream
                }
            );
        }
        Err(())
    }

    pub fn respond(mut self, response: HttpResponse) {
        let buffer: Vec<u8> = response.into();
        if let Err(_error) = self.stream.write_all(&buffer) {

        };
    }
}
