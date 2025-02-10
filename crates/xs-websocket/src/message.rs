use crate::{helpers::mask, opcode::OpCode::{self, *}};

/*
#[derive(Debug)]
pub enum WebSocketMessage {
    Unimplemented,
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Text(String),
    Binary(Vec<u8>),
    Close
}
*/

fn to_hex(bytes: &Vec<u8>) -> Vec<String> {
    bytes.iter().map(|b| format!("{:#02x}", b).to_string()).collect()
}

pub struct WebSocketMessage {
    pub op_code: OpCode,
    pub masking_key: Option<[u8;4]>,
    pub payload: Vec<u8>,
}

impl std::fmt::Debug for WebSocketMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.op_code {
            Text => {
                let text = std::str::from_utf8(&self.payload).unwrap();
                write!(f, "[WebSocketMessage {:?} {:?}]", self.op_code, text)
            },
            _ => {
                write!(f, "{:?} {:?}", self.op_code, to_hex(&self.payload))
            }
        }

    }
}

impl WebSocketMessage {
    pub fn new(op_code: OpCode, masking_key: Option<[u8;4]>, payload: Vec<u8>) -> Self {
        Self { op_code, masking_key, payload }
    }
}

impl From<Vec<u8>> for WebSocketMessage {
    fn from(bytes: Vec<u8>) -> Self {
        let op_code = (bytes[0] & 0x0F).into();
        let masked = bytes[0] & 0x80 != 0;
        let len1 = bytes[1] & 0x7F;
        let (payload_size, mut payload_offset) = match len1 {
            126 => {
                let len2 = u16::from_be_bytes(bytes[2..2 + 2].try_into().unwrap());
                let len = (len1 as u16 + len2) as usize;
                (len, 5)
            }
            127 => {
                let len8 = u64::from_be_bytes(bytes[2..2 + 8].try_into().unwrap());
                let len = (len1 as u64 + len8) as usize;
                (len, 11)
            }
            _ => (len1 as usize, 2),
        };
        let masking_key: Option<[u8; 4]> = match masked {
            true => {
                let header_size = payload_offset;
                payload_offset += 4;
                Some(bytes[header_size..payload_offset].try_into().unwrap())
            },
            false => None
        };
        println!("payload_offset: {:?}", payload_offset);
        println!("payload_size: {:?}", payload_size);
        println!("bytes: {:?}", to_hex(&bytes));
        println!("masking_key: {:?}", masking_key);
        let payload = &bytes[payload_offset..payload_offset+payload_size];
        let payload = if let Some(masking_key) = masking_key {
            mask(&masking_key, payload)
        } else {
            payload.to_vec()
        };
        println!("payload: {:?}", to_hex(&payload));
        Self {
            op_code,
            masking_key,
            payload,
        }
    }
}

impl Into<Vec<u8>> for WebSocketMessage {
    fn into(self) -> Vec<u8> {
        let mut header = [0u8; 14];
        header[0] = 0x80 | self.op_code as u8;
        header[1] = if self.masking_key.is_some() { 0x80 } else { 0 };
        let len = self.payload.len();
        let mut header_size = match self.payload.len() {
            usize::MIN..126 => {
                header[1] |= len as u8;
                2
            },
            usize::MIN..65536 => {
                header[1] |= 126;
                let bytes = &(len as u16).to_be_bytes();
                for (i, &byte) in bytes.iter().enumerate() {
                    header[i + 2] = byte;
                }
                4
            },
            _ => {
                header[1] |= 127;
                let bytes = &(len as u64).to_be_bytes();
                for (i, &byte) in bytes.iter().enumerate() {
                    header[i + 2] = byte;
                }
                10
            }
        };
        if self.masking_key.is_some() {
            let masking_key = self.masking_key.unwrap();
            for (i, &byte) in masking_key.iter().enumerate() {
                header[header_size + i] = byte;
            }
            header_size += 4;
        }
        let mut payload = match self.masking_key {
            Some(masking_key) => mask(&masking_key, &self.payload).into(),
            None => self.payload
        };
        let mut frame = header[0..header_size].to_vec();
        frame.append(&mut payload);
        frame
    }
}
