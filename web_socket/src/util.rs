use std::net::TcpStream;
use std::io;
use std::io::{Write, Read, BufWriter};
use crate::listener::CoolError;

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
    pub fn from_number(opcode: u8) -> Option<FrameKind> {
        Some(match opcode {
            0x0 => FrameKind::Continue,
            0x1 => FrameKind::Text,
            0x2 => FrameKind::Binary,
            0x8 => FrameKind::Close,
            0x9 => FrameKind::Ping,
            0xA => FrameKind::Pong,
            _ => return None,
        })
    }
}


#[derive(Copy, Clone)]
pub enum PayloadLength {
    Small(u8),
    Extended,
    ExtraExtended,
}

impl PayloadLength {
    pub fn from_len(len: usize) -> PayloadLength {
        match len {
            0..=125 => PayloadLength::Small(len as u8),
            126 ..= 65535 => PayloadLength::Extended,
            _ => PayloadLength::ExtraExtended,
        }
    }

    pub fn to_byte(self) -> u8 {
        match self {
            PayloadLength::Small(len) => len,
            PayloadLength::Extended => 126,
            PayloadLength::ExtraExtended => 127,
        }
    }
}