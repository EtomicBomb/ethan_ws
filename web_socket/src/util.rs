use std::io;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FrameKind {
    Continue = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl FrameKind {
    pub fn from_opcode(opcode: u8) -> io::Result<FrameKind> {
        Ok(match opcode {
            0x0 => FrameKind::Continue,
            0x1 => FrameKind::Text,
            0x2 => FrameKind::Binary,
            0x8 => FrameKind::Close,
            0x9 => FrameKind::Ping,
            0xA => FrameKind::Pong,
            _ => return Err(io::Error::from(io::ErrorKind::InvalidData)),
        })
    }
}
