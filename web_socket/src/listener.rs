use std::io::{Read, BufReader};
use std::net::TcpStream;

use crate::util::{FrameKind};
use crate::writer::write_frame;
use std::io;

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

fn read_next_message(reader: &mut impl Read) -> io::Result<(Vec<u8>, FrameKind)> {
    // blocks the current thread until we receive a full message from the client

    let mut buf = Vec::new();

    let Frame { mut is_last_frame, frame_kind } = read_next_frame(reader, &mut buf)?;

    while !is_last_frame {
        let Frame { is_last_frame: last, .. } = read_next_frame(reader, &mut buf)?;

        is_last_frame = last;
    }

    Ok((buf, frame_kind))
}


fn read_next_frame(reader: &mut impl Read, buf: &mut Vec<u8>) -> io::Result<Frame> {
    // read the first byte from the stream, which tells us if this was the message's last frame and
    // what kind of frame it was
    let mut first_byte = [0u8; 1];
    reader.read_exact(&mut first_byte)?;
    let [first_byte] = first_byte;

    let is_last_frame = (first_byte >> 7) == 1;
    let frame_kind = FrameKind::from_opcode(first_byte & 0b1111)?;

    // extract our payload
    let payload_length = get_payload_len(reader)?;

    let mut masking_key = [0u8; 4];
    reader.read_exact(&mut masking_key)?;

    append_payload(reader, payload_length, masking_key, buf)?;

    Ok(Frame { is_last_frame, frame_kind })
}

fn get_payload_len(reader: &mut impl Read) -> io::Result<usize> {
    let mut heuristic_byte = [0u8; 1];
    reader.read_exact(&mut heuristic_byte)?;
    let [heuristic_byte] = heuristic_byte;

    match heuristic_byte & 0b_0111_1111 {
        n @ 0..=125 => Ok(n as usize),
        126 => {
            let mut len = [0u8; 2];
            reader.read_exact(&mut len)?;
            Ok(u16::from_be_bytes(len) as usize)
        },
        127 => {
            let mut len = [0u8; 8];
            reader.read_exact(&mut len)?;
            Ok(u64::from_be_bytes(len) as usize)
        },
        _ => unreachable!("larger values prevented by bit mask"),
    }
}

fn append_payload(reader: &mut impl Read, payload_len: usize, masking_key: [u8; 4], buf: &mut Vec<u8>) -> io::Result<()> {
    let old_len = buf.len();
    buf.resize(old_len+payload_len, 0);
    let read_into = &mut buf[old_len..];

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