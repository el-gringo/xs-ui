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
    pub writer: BufWriter<&'a TcpStream>
}

impl<'a> Iterator for IncomingMessages<'a> {
    type Item = WebSocketMessage;

    fn next(&mut self) -> Option<WebSocketMessage> {
        let reader = self.reader.borrow_mut();
        let mut buffer: Vec<u8> = Vec::new();
        let mut buf1 = vec![0;2];
        reader.read_exact(&mut buf1).expect("reading first 2 bytes");
        let len1 = buf1[1] & 0x7F;
        let masked = buf1[0] & 0x80 != 0;
        buffer.append(&mut buf1);
        let payload_size = match len1 {
            126 => {
                let mut buf2 = [0u8; 2];
                reader.read_exact(&mut buf2).unwrap();
                buffer.append(&mut buf2.to_vec());
                let len2 = u16::from_be_bytes(buffer[2..2+2].try_into().unwrap());
                (len1 as u16 + len2) as usize
            },
            127 => {
                let mut buf8 = [0u8; 8];
                reader.read_exact(&mut buf8).unwrap();
                buffer.append(&mut buf8.to_vec());
                let len8 = u64::from_be_bytes(buffer[2..2+8].try_into().unwrap());
                (len1 as u64 + len8) as usize
            },
            _ => {
                len1 as usize
            }
        };
        if masked {
            let mut masking_key_buf = vec![0; 4];
            reader.read_exact(&mut masking_key_buf).unwrap();
            buffer.append(&mut masking_key_buf);
        }
        let mut buffer_payload = vec![0; payload_size];
        reader.read_exact(&mut buffer_payload).unwrap();
        buffer.append(&mut buffer_payload);
        println!("buffer: {:?}", buffer.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>());
        let message: WebSocketMessage = buffer.into();
        if let OpCode::Ping = message.op_code {
            let pong = WebSocketMessage::new(OpCode::Pong, None, buffer_payload);
            let buf: Vec<u8> = pong.into();
            self.writer.write(&buf).unwrap();
            println!("Sent pong");
        }
        Some(message)
    }
}

impl WebSocketServer {
    pub fn with_stream(stream: std::net::TcpStream) -> Self {
        Self {
            masking_key: None,
            stream
        }
    }

    pub fn send_text(&mut self, text: String) {
        let message = WebSocketMessage::new(Text, self.masking_key, text.as_bytes().to_vec());
        //let mut writer = BufWriter::new(&mut self.stream);
        let buffer: Vec<u8> = message.into();
        self.stream.write(&mut buffer.as_slice())
            .expect("unable to send message");
        self.stream.flush().unwrap();
        println!("WebSocketServer send_text");
    }

    pub fn incoming_messages(&mut self) -> IncomingMessages<'_> {
        IncomingMessages {
            reader: BufReader::new(&self.stream),
            writer: BufWriter::new(&self.stream),
        }
    }
}
