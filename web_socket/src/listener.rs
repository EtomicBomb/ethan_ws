use std::io::{Read, BufReader};
use std::net::TcpStream;

use crate::util::{FrameKind};
use crate::writer::write_frame;
use std::io;

// TODO: correctly handle io error Interrupted?

pub struct WebSocketListener {
    reader: BufReader<TcpStream>,
}

impl WebSocketListener {
    pub fn new(tcp_stream: TcpStream) -> WebSocketListener {
        WebSocketListener { reader: BufReader::new(tcp_stream) }
    }
}

impl Iterator for WebSocketListener {
    type Item = WebSocketMessage;

    fn next(&mut self) -> Option<WebSocketMessage> {
        loop {
            let (message, kind) = read_next_message(&mut self.reader).ok()?;

            match kind {
                FrameKind::Binary => break Some(WebSocketMessage::Binary(message)),
                FrameKind::Text => break Some(WebSocketMessage::Text(String::from_utf8_lossy(&message).into())),
                FrameKind::Continue => break None,
                FrameKind::Close => break None,
                FrameKind::Ping => {
                    write_frame(self.reader.get_mut(), &message, FrameKind::Pong).ok()?;
                    continue
                },
                FrameKind::Pong => continue,
            }
        }
    }
}


pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
}

impl WebSocketMessage {
    pub fn get_text(&self) -> Option<&str> {
        match self {
            WebSocketMessage::Text(ref s) => Some(s.as_str()),
            WebSocketMessage::Binary(_) => None,
        }
    }
}

fn read_next_message(reader: &mut impl Read) -> Result<(Vec<u8>, FrameKind), CoolError> {
    // blocks the current thread until we receive a full message from the client

    let mut buf = Vec::new();

    let Frame { mut is_last_frame, frame_kind } = read_next_frame(reader, &mut buf)?;

    while !is_last_frame {
        let Frame { is_last_frame: last, .. } = read_next_frame(reader, &mut buf)?;

        is_last_frame = last;
    }

    Ok((buf, frame_kind))
}

// this function is blocking (epic style)
fn read_next_frame(reader: &mut impl Read, buf: &mut Vec<u8>) -> Result<Frame, CoolError> {
    macro_rules! read_bytes {
        ($len:expr) => {{
            let mut buf = [0u8; $len];
            match reader.read(&mut buf) {
                Ok($len) => buf,
                Ok(_) | Err(_) => return Err(CoolError::Unrecoverable),
            }
        }};
    }

    // read two bytes so we can get the payload length
    let [first_byte, second_byte] = read_bytes!(2);

    let is_last_frame = (first_byte >> 7) == 1;
    let frame_kind = FrameKind::from_number(first_byte & 0b1111).ok_or(CoolError::Unrecoverable)?;

    let payload_length =
        match second_byte & 0b01111111 {
            n @ 0..=125 => n as usize,
            126 => u16::from_be_bytes(read_bytes!(2)) as usize,
            127 => u64::from_be_bytes(read_bytes!(8)) as usize,
            _ => unreachable!("bit mask doesn't allow for greater"),
        };

    let masking_key = read_bytes!(4);

    append_payload(reader, payload_length, masking_key, buf)?;

    Ok(Frame { is_last_frame, frame_kind })
}

fn append_payload(reader: &mut impl Read, payload_len: usize, masking_key: [u8; 4], buf: &mut Vec<u8>) -> Result<(), CoolError> {
    let old_len = buf.len();
    buf.resize(old_len+ payload_len, 0);
    let mut read_into = &mut buf[old_len..];

    reader.read_exact(read_into)?;

    // unmask
    for (i, byte) in read_into.iter_mut().enumerate() {
        *byte ^= masking_key[i % 4];
    }

    Ok(())
}


#[derive(Debug)]
struct Frame {
    is_last_frame: bool,
    frame_kind: FrameKind,
}

#[derive(Debug)]
pub enum CoolError {
    Unrecoverable,
}

impl From<io::Error> for CoolError {
    fn from(_: io::Error) -> CoolError {
        CoolError::Unrecoverable
    }
}