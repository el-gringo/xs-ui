use std::borrow::BorrowMut;
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;

use crate::message::WebSocketMessage;
use crate::opcode::OpCode::{self, *};

pub struct WebSocketServer {
    masking_key: Option<[u8; 4]>,
    pub stream: std::net::TcpStream,
}

pub struct IncomingMessages<'a> {
    pub reader: BufReader<&'a TcpStream>,
    pub writer: BufWriter<&'a TcpStream>,
}

impl<'a> Iterator for IncomingMessages<'a> {
    type Item = WebSocketMessage;

    /// A browser going away mid frame is normal, not an error: the read simply
    /// ends the iteration instead of bringing the process down (a panic here is
    /// fatal, `panic = "abort"`).
    fn next(&mut self) -> Option<WebSocketMessage> {
        let reader = self.reader.borrow_mut();
        let mut buffer: Vec<u8> = Vec::new();
        let mut buf1 = [0u8; 2];
        reader.read_exact(&mut buf1).ok()?;
        let len1 = buf1[1] & 0x7F;
        let masked = buf1[0] & 0x80 != 0;
        buffer.extend_from_slice(&buf1);
        let payload_size = match len1 {
            126 => {
                let mut buf2 = [0u8; 2];
                reader.read_exact(&mut buf2).ok()?;
                buffer.extend_from_slice(&buf2);
                let len2 = u16::from_be_bytes([buf2[0], buf2[1]]);
                len2 as usize
            }
            127 => {
                let mut buf8 = [0u8; 8];
                reader.read_exact(&mut buf8).ok()?;
                buffer.extend_from_slice(&buf8);
                u64::from_be_bytes(buf8) as usize
            }
            _ => len1 as usize,
        };
        if masked {
            let mut masking_key_buf = [0u8; 4];
            reader.read_exact(&mut masking_key_buf).ok()?;
            buffer.extend_from_slice(&masking_key_buf);
        }
        let mut buffer_payload = vec![0; payload_size];
        reader.read_exact(&mut buffer_payload).ok()?;
        buffer.extend_from_slice(&buffer_payload);
        let message: WebSocketMessage = buffer.into();
        if let OpCode::Ping = message.op_code {
            let pong = WebSocketMessage::new(OpCode::Pong, None, buffer_payload);
            let buf: Vec<u8> = pong.into();
            let _ = self.writer.write(&buf);
        }
        Some(message)
    }
}

impl WebSocketServer {
    pub fn with_stream(stream: std::net::TcpStream) -> Self {
        Self {
            masking_key: None,
            stream,
        }
    }

    pub fn send_text(&mut self, text: String) {
        let message = WebSocketMessage::new(Text, self.masking_key, text.as_bytes().to_vec());
        //let mut writer = BufWriter::new(&mut self.stream);
        let buffer: Vec<u8> = message.into();
        self.stream
            .write_all(&buffer)
            .expect("unable to send message");
        self.stream.flush().unwrap();
    }

    pub fn incoming_messages(&mut self) -> IncomingMessages<'_> {
        IncomingMessages {
            reader: BufReader::new(&self.stream),
            writer: BufWriter::new(&self.stream),
        }
    }
}
