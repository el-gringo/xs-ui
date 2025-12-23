use std::borrow::BorrowMut;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::digest::{Sha1, base64_encode};
use std::time::{SystemTime, UNIX_EPOCH};

const MASKING_KEY_SIZE: usize = 4;
type MaskingKey = [u8; MASKING_KEY_SIZE];

pub trait ReadWrite: Read + Write {}
impl<T> ReadWrite for T where T: Read + Write {}

type WebSocketStream = Box<dyn ReadWrite + Send>;


fn mask(masking_key: &MaskingKey, payload: &[u8]) -> Vec<u8> {
    payload
        .iter()
        .zip(masking_key.iter().cycle())
        .map(|(byte, key)| byte ^ key)
        .collect::<Vec<u8>>()
}

pub const SERVER_WEB_SOCKET_PING_MESSAGE:[u8;2] = [128 | 9,0];
pub const SERVER_WEB_SOCKET_PONG_MESSAGE:[u8;2] = [128 | 10,0];

#[derive(Debug, PartialEq)]
enum State {
    Opcode,
    Len1,
    Len2,
    Len8,
    Data,
    Mask
}

impl State {
    fn head_expected(&self) -> usize {
        match self {
            State::Opcode => 1,
            State::Len1 => 1,
            State::Len2 => 2,
            State::Len8 => 8,
            State::Data => 0,
            State::Mask => 4
        }
    }
}

/*
impl Clone for WebSocketStream {
    fn clone(&self) -> WebSocketStream {
        Box::new((*self).clone())
    }
}
*/

#[derive(Debug)]
pub enum ServerWebSocketMessage<'a> {
    Unimplemented,
    Ping(&'a [u8]),
    Pong(&'a [u8]),
    Text(&'a str),
    Binary(&'a [u8]),
    Close
}

pub enum WebSocketMessage {
    Unimplemented,
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Text(String),
    Binary(Vec<u8>),
    Close
}

#[derive(Debug)]
pub enum ServerWebSocketError<'a> {
    OpcodeNotSupported(u8),
    TextNotUTF8(&'a [u8]),
    Unexpected
}

pub enum ServerWebSocketMessageFormat {
    Binary,
    Text
}

pub struct ServerWebSocketMessageHeader {
    pub format: ServerWebSocketMessageFormat,
    len: usize,
    masked: bool,
    data: [u8;14]
}

impl ServerWebSocketMessageHeader {
    pub fn from_len(len: usize, format: ServerWebSocketMessageFormat, masked: bool)->Self{
        let mut data = [0u8;14];

        match format {
            ServerWebSocketMessageFormat::Binary => data[0] = 128 | 2,
            ServerWebSocketMessageFormat::Text => data[0] = 128 | 1,
        }

        if masked {
            data[1] = 128;
        } else {
            data[1] = 0;
        }

        let header_len;
        if len < 126{
            data[1] |= len as u8;
            header_len = 2;
        }
        else if len < 65536{
            data[1] |= 126;
            let bytes = &(len as u16).to_be_bytes();
            for (i, &byte) in bytes.iter().enumerate() {
                data[i + 2] = byte;
            }
            header_len = 4;
        }
        else{
            data[1] |= 127;
            let bytes = &(len as u64).to_be_bytes();
            for (i, &byte) in bytes.iter().enumerate() {
                data[i + 2] = byte;
            }
            header_len = 10;
        }

        if masked {
            for i in header_len..header_len + 4 {
                data[i] = Self::random_byte();
            }
            return ServerWebSocketMessageHeader{len: header_len + 4, data, format, masked}
        } else {
            return ServerWebSocketMessageHeader{len: header_len, data, format, masked}
        }
    }

    pub fn as_slice(&self)->&[u8]{
        &self.data[0..self.len]
    }

    pub fn mask(&mut self)->Option<&[u8]> {
        if self.masked {
            match self.len {
                6 => Some(&self.data[2..6]),
                10 => Some(&self.data[6..10]),
                14 => Some(&self.data[10..14]),
                _ => None
            }
        } else {
            None
        }
    }

    // TODO Improve this using a proper random number generator
    fn random_byte() -> u8 {
        let num = SystemTime::now().duration_since(UNIX_EPOCH).expect("duration_since failed").subsec_nanos();
        num as u8
    }
}


pub struct ServerWebSocket {
    head: [u8; 8],
    head_expected: usize,
    head_written: usize,
    data: Vec<u8>,
    data_len: usize,
    input_read: usize,
    mask: MaskingKey,
    mask_counter: usize,
    is_ping: bool,
    is_pong: bool,
    is_partial: bool,
    is_text: bool,
    is_masked: bool,
    state: State,
    stream: WebSocketStream
}

fn message_to_frame(msg:ServerWebSocketMessage) ->Vec<u8>
{
    match &msg{
        ServerWebSocketMessage::Text(data)=>{
            let header = ServerWebSocketMessageHeader::from_len(data.len(), ServerWebSocketMessageFormat::Text, true);
            ServerWebSocket::build_message(header, &data.to_string().into_bytes())
        }
        ServerWebSocketMessage::Binary(data)=>{
            let header = ServerWebSocketMessageHeader::from_len(data.len(), ServerWebSocketMessageFormat::Text, true);
            ServerWebSocket::build_message(header, &data)
        }
        _=>panic!()
    }
}

impl Into<WebSocketMessage> for Vec<u8> {
    fn into(self) -> WebSocketMessage {
        let bytes = &self;
        let opcode = bytes[0] & 0x0F;
        //let masked = bytes[0] & 0x80;
        let len1 = bytes[1] & 0x7F;
        let (payload_size, payload_offset) = match len1 {
            126 => {
                let len2 = u16::from_be_bytes(bytes[2..2+2].try_into().unwrap());
                let len = (len1 as u16 + len2) as usize;
                (len, 5)
            }
            127 => {
                let len8 = u64::from_be_bytes(bytes[2..2+8].try_into().unwrap());
                let len = (len1 as u64 + len8) as usize;
                (len, 11)
            }
            _ => {
                (len1 as usize, 2)
            }
        };
        let payload  = bytes[payload_offset..payload_size].to_vec();
        let payload = payload.to_owned().clone();
        match opcode {
            1 => WebSocketMessage::Text(
                    std::str::from_utf8(&payload).unwrap().into()
                ),
            2 => WebSocketMessage::Binary(payload),
            8 => WebSocketMessage::Close,
            9 => WebSocketMessage::Ping(payload),
            10 => WebSocketMessage::Pong(payload),
            _ => WebSocketMessage::Unimplemented
        }
    }
}

impl<'a> Into<ServerWebSocketMessage<'a>> for &'a [u8] {
    fn into(self) -> ServerWebSocketMessage<'a> {
        let bytes = self;
        let opcode = bytes[0] & 0x0F;
        //let masked = bytes[0] & 0x80;
        let len1 = bytes[1] & 0x7F;
        let (payload_size, payload_offset) = match len1 {
            126 => {
                let len2 = u16::from_be_bytes(bytes[2..2+2].try_into().unwrap());
                let len = (len1 as u16 + len2) as usize;
                (len, 5)
            }
            127 => {
                let len8 = u64::from_be_bytes(bytes[2..2+8].try_into().unwrap());
                let len = (len1 as u64 + len8) as usize;
                (len, 11)
            }
            _ => {
                (len1 as usize, 2)
            }
        };
        //let masking_key = &bytes[payload_offset..payload_offset+4];
        let payload  = &bytes[payload_offset+4..payload_size];
        match opcode {
            1 => ServerWebSocketMessage::Text(
                    std::str::from_utf8(payload).unwrap()
                ),
            2 => ServerWebSocketMessage::Binary(payload),
            8 => ServerWebSocketMessage::Close,
            9 => ServerWebSocketMessage::Ping(payload),
            10 => ServerWebSocketMessage::Pong(payload),
            _ => ServerWebSocketMessage::Unimplemented
        }
    }
}

impl ServerWebSocket {
    pub fn new(stream: WebSocketStream) -> Self {
        let mut mask: MaskingKey = [0; MASKING_KEY_SIZE];
        for i in 0..MASKING_KEY_SIZE {
            mask[i] = 0;//ServerWebSocketMessageHeader::random_byte();
        }
        Self {
            head: [0u8; 8],
            head_expected: 1,
            head_written: 0,
            data: Vec::new(),
            data_len: 0,
            input_read: 0,
            mask,
            mask_counter: 0,
            is_ping: false,
            is_pong: false,
            is_masked: false,
            is_partial: false,
            is_text: false,
            state: State::Opcode,
            stream
        }
    }

    pub fn create_upgrade_response(key: &str) -> String {
        let to_hash = format!("{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11", key);
        let mut sha1 = Sha1::new();
        sha1.update(to_hash.as_bytes());
        let out_bytes = sha1.finalise();
        let base64 = base64_encode(&out_bytes);
        let response_ack = format!(
            "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
            base64
        );
        response_ack
    }

    pub fn build_message(mut header: ServerWebSocketMessageHeader, data: &[u8])->Vec<u8>{
        let mut frame = header.as_slice().to_vec();
        if let Some(mask) = header.mask(){
            for (i, &byte) in data.iter().enumerate() {
                frame.push(byte ^ mask[i % 4]);
            }
        } else {
            frame.extend_from_slice(data);
        }
        frame
    }

    fn parse_head(&mut self, input: &[u8]) -> bool {
        while self.head_expected > 0
            && self.input_read < input.len()
            && self.head_written < self.head.len()
        {
            self.head[self.head_written] = input[self.input_read];
            self.input_read += 1;
            self.head_written += 1;
            self.head_expected -= 1;
        }
        self.head_expected != 0
    }

    fn to_state(&mut self, state: State) {
        match state {
            State::Data => {
                self.mask_counter = 0;
                self.data.clear();
            }
            State::Opcode => {
                self.is_ping = false;
                self.is_pong = false;
                self.is_partial = false;
                self.is_text = false;
                self.is_masked = false;
            },
            _ => ()
        }
        self.head_written = 0;
        self.head_expected = state.head_expected();
        self.state = state;
    }

    pub fn parse(&mut self, input: &[u8]) -> Result<ServerWebSocketMessage, ServerWebSocketError> {
        self.input_read = 0;
        // parse a header
        loop {
            match self.state {
                State::Opcode => {
                    if self.parse_head(input) {
                        break;
                    }
                    let opcode = self.head[0] & 15;
                    if opcode <= 2 {
                        self.is_partial = (self.head[0] & 128) != 0;
                        self.is_text = opcode == 1;
                        self.to_state(State::Len1);
                    }
                    else if opcode == 8 {
                        return Ok(ServerWebSocketMessage::Close);
                        break;
                    }
                    else if opcode == 9 {
                        self.is_ping = true;
                        self.to_state(State::Len1);
                    }
                    else if opcode == 10 {
                        self.is_pong = true;
                        self.to_state(State::Len1);
                    }
                    else {
                        return Err(ServerWebSocketError::OpcodeNotSupported(opcode));
                        break;
                    }
                },
                State::Len1 => {
                    if self.parse_head(input) {
                        break;
                    }
                    self.is_masked = (self.head[0] & 128) > 0;
                    let len_type = self.head[0] & 127;
                    if len_type < 126 {
                        self.data_len = len_type as usize;
                        if !self.is_masked {
                            self.to_state(State::Data);
                        }
                        else {
                            self.to_state(State::Mask);
                        }
                    }
                    else if len_type == 126 {
                        self.to_state(State::Len2);
                    }
                    else if len_type == 127 {
                        self.to_state(State::Len8);
                    }
                },
                State::Len2 => {
                    if self.parse_head(input) {
                        break;
                    }
                    self.data_len = u16::from_be_bytes(
                        self.head[0..2].try_into().unwrap()
                    ) as usize;
                    if self.is_masked {
                        self.to_state(State::Mask);
                    }
                    else {
                        self.to_state(State::Data);
                    }
                },
                State::Len8 => {
                    if self.parse_head(input) {
                        break;
                    }
                    self.data_len = u64::from_be_bytes(
                        self.head[0..8].try_into().unwrap()
                    ) as usize;
                    if self.is_masked {
                        self.to_state(State::Mask);
                    }
                    else {
                        self.to_state(State::Data);
                    }
                },
                State::Mask => {
                    if self.parse_head(input) {
                        break;
                    }
                    self.to_state(State::Data);
                },
                State::Data => {
                    if self.is_masked {
                        while self.data.len() < self.data_len && self.input_read < input.len() {
                            self.data.push(input[self.input_read] ^ self.head[self.mask_counter]);
                            self.mask_counter = (self.mask_counter + 1) & 3;
                            self.input_read += 1;
                        }
                    }
                    else {
                        while self.data.len() < self.data_len && self.input_read < input.len() {
                            self.data.push(input[self.input_read]);
                            self.input_read += 1;
                        }
                    }
                    if self.data.len() < self.data_len { // not enough data yet
                        break;
                    }
                    self.to_state(State::Opcode);
                    if (self.state == State::Data) {
                        if self.is_ping {
                            return Ok(ServerWebSocketMessage::Ping(&self.data));
                        }
                        else if self.is_pong {
                            return Ok(ServerWebSocketMessage::Pong(&self.data));
                        }
                        else if self.is_text {
                            if let Ok(text) = std::str::from_utf8(&self.data){
                                return Ok(ServerWebSocketMessage::Text(text));
                            }
                            else{
                                return Err(ServerWebSocketError::TextNotUTF8(&self.data));
                            }
                        }
                        else{
                            return Ok(ServerWebSocketMessage::Binary(&self.data));
                        }
                    }
                },
            }
        }
        Err(ServerWebSocketError::Unexpected)
    }

    pub fn send_message(&mut self, message: ServerWebSocketMessage) {
        let frame = message_to_frame(message);
        self.stream.write(&frame).unwrap();
    }

    pub fn incoming_messages<F>(&mut self, mut on_message: F)
    where
        F: FnMut(ServerWebSocketMessage) + Send + Sync + 'static
    {
        //let stream = Arc::new(Mutex::new((&self.stream)));
        //let mut stream = Box::clone(&self.stream);

        //let mut stream = Arc::new(self.stream);
        let pin = Box::pin(self.stream);
        let stream = &self.stream.as_mut();
        let mut reader = std::io::BufReader::new(pin);
            let mut writer = std::io::BufWriter::new(std::mem::transmute(&*pin));

        let masking_key = self.mask;
        thread::scope(move |scope| {
            scope.spawn(move || {
                loop {
                    //let stream = Arc::clone(&stream);
                    let mut buffer: Vec<u8> = Vec::new();
                    let mut buf1 = vec![0; 2];

                    reader
                        .read_exact(&mut buf1).unwrap();
                    buffer.append(&mut buf1);
                    let len1 = buffer[1] & 0x7F;
                    let payload_size = match len1 {
                        126 => {
                            let mut buf2 = [0u8; 2];
                            reader
                                .read_exact(&mut buf2).unwrap();
                            buffer.append(&mut buf2.to_vec());
                            let len2 = u16::from_be_bytes(buffer[2..2+2].try_into().unwrap());
                            (len1 as u16 + len2) as usize
                        }
                        127 => {
                            let mut buf8 = [0u8; 8];
                            reader
                                .read_exact(&mut buf8).unwrap();
                            buffer.append(&mut buf8.to_vec());
                            let len8 = u64::from_be_bytes(buffer[2..2+8].try_into().unwrap());
                            (len1 as u64 + len8) as usize
                        }
                        _ => {
                            len1 as usize
                        }
                    };
                    let mut buffer_payload = vec![0; payload_size + 5];
                    reader.read_exact(&mut buffer_payload).unwrap();
                    buffer.append(&mut buffer_payload);
                    match buffer.into() {
                        WebSocketMessage::Ping(ping) => {
                            let mut pong_frame: Vec<u8> = Vec::new();
                            pong_frame.append(&mut [0x8a, 0x80 | ping.len() as u8].to_vec());
                            pong_frame.append(&mut masking_key.to_vec());
                            pong_frame.append(&mut mask(&masking_key, &ping));
                            //stream.write_all(pong_frame.as_slice()).unwrap();
                            //stream.borrow_mut().write_all(pong_frame.as_slice()).unwrap();
                            writer.write_all(pong_frame.as_slice()).unwrap();
                        }
                        message => {
                            //on_message(message);
                        }
                    };

                }
            });
        });
    }
}
