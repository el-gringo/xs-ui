use self::OpCode::*;

#[derive(Debug)]
pub enum OpCode {
    /// Indicates a continuation frame of a fragmented message.
    Continue,
    /// Indicates a text data frame.
    Text,
    /// Indicates a binary data frame.
    Binary,
    /// Indicates a close control frame.
    Close,
    /// Indicates a ping control frame.
    Ping,
    /// Indicates a pong control frame.
    Pong,
    /// Indicates an invalid opcode was received.
    Bad,
}

impl From<OpCode> for u8 {
    fn from(opcode: OpCode) -> u8 {
        match opcode {
            OpCode::Continue => 0,
            OpCode::Text => 1,
            OpCode::Binary => 2,
            OpCode::Close => 8,
            OpCode::Ping => 9,
            OpCode::Pong => 10,
            OpCode::Bad => {
                debug_assert!(
                    false,
                    "Attempted to convert invalid opcode to u8. This is a bug."
                );
                8 // if this somehow happens, a close frame will help us tear down quickly
            }
        }
    }
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> OpCode {
        match byte {
            0 => Continue,
            1 => Text,
            2 => Binary,
            8 => Close,
            9 => Ping,
            10 => Pong,
            _ => Bad,
        }
    }
}
