use std::io::{Read};
use std::net::TcpStream;

use crate::util::{FrameKind};
use crate::util::write_frame;

// TODO: correctly handle io error Interrupted?

pub struct WebSocketListener {
    tcp_stream: TcpStream,
}

impl WebSocketListener {
    pub fn new(tcp_stream: TcpStream) -> WebSocketListener {
        WebSocketListener { tcp_stream }
    }
}

impl Iterator for WebSocketListener {
    type Item = WebSocketMessage;

    fn next(&mut self) -> Option<WebSocketMessage> {
        loop {
            let (message, kind) = read_next_message(&mut self.tcp_stream).ok()?;

            match kind {
                FrameKind::Binary => break Some(WebSocketMessage::Binary(message)),
                FrameKind::Text => break Some(WebSocketMessage::Text(String::from_utf8_lossy(&message).into())),
                FrameKind::Continue => break None,
                FrameKind::Close => break None,
                FrameKind::Ping => {
                    write_frame(&mut self.tcp_stream, &message, FrameKind::Pong).ok()?;
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

fn read_next_message(tcp_stream: &mut TcpStream) -> Result<(Vec<u8>, FrameKind), CoolError> {
    // blocks the current thread until we receive a full message from the client

    let Frame { mut is_last_frame, payload: mut message, frame_kind } = read_next_frame(tcp_stream)?;

    while !is_last_frame {
        let Frame { is_last_frame: last, mut payload, .. } = read_next_frame(tcp_stream)?;

        message.append(&mut payload);

        is_last_frame = last;
    }

    Ok((message, frame_kind))
}

macro_rules! read_bytes {
    ($tcp_stream:ident $len:literal) => {{
        let mut buf = [0u8; $len];
        match $tcp_stream.read(&mut buf) {
            Ok($len) => buf,
            Ok(_) | Err(_) => return Err(CoolError::Unrecoverable),
        }
    }};
}

macro_rules! read_bytes_to_vec {
    ($tcp_stream:ident $len:expr) => {{
        let mut buf = vec![0u8; $len];
        match $tcp_stream.read(&mut buf) {
            Ok(i) if i == $len => buf,
            Ok(_) | Err(_) => return Err(CoolError::Unrecoverable),
        }
    }};
}

// this function is blocking (epic style)
fn read_next_frame(tcp_stream: &mut TcpStream) -> Result<Frame, CoolError> {
    // read two bytes so we can get the payload length
    let tcp_stream = tcp_stream;
    &tcp_stream;

    let [first_byte, second_byte] = read_bytes!(tcp_stream 2);

    let is_last_frame = (first_byte >> 7) == 1;
    let frame_kind = FrameKind::from_number(first_byte & 0b1111).ok_or(CoolError::Unrecoverable)?;

    let payload_length =
        match second_byte & 0b01111111 {
            n @ 0..=125 => n as usize,
            126 => u16::from_be_bytes(read_bytes!(tcp_stream 2)) as usize,
            127 => u64::from_be_bytes(read_bytes!(tcp_stream 8)) as usize,
            _ => unreachable!("bit mask doesn't allow for greater"),
        };

    let masking_key = read_bytes!(tcp_stream 4);
    let mut payload = read_bytes_to_vec!(tcp_stream payload_length);

    for (i, byte) in payload.iter_mut().enumerate() {
        let mask = masking_key[i % 4];
        *byte = *byte ^ mask;
    }

    Ok(Frame { is_last_frame, frame_kind, payload })
}

struct Frame {
    is_last_frame: bool,
    frame_kind: FrameKind,
    payload: Vec<u8>,
}

enum CoolError {
    Unrecoverable,
}
