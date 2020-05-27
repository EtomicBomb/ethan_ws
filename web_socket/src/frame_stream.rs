use std::io::{Read, Write};
use std::net::TcpStream;

use crate::frame::{Frame, FrameError, FrameKind};

const BYTES_AT_A_TIME: usize = 1024;

// TODO:
// PROBLEMS:
//      are we correctly handling io error Interrupted?
//      are we should we return an iterator over Result<WebSockteMessage, Something>
//      we shouldn't call iterator next recursively, we could overflow the stack
//      we don't correctly deal with FrameKind::Continue websocket messages


// any internal errors are errors are handled by returning None on the iterator
pub struct WebSocketListener {
    socket: TcpStream,
}

impl WebSocketListener {
    pub fn new(socket: TcpStream) -> WebSocketListener {
        WebSocketListener { socket }
    }
}

impl Iterator for WebSocketListener {
    type Item = WebSocketMessage;

    fn next(&mut self) -> Option<WebSocketMessage> {
        loop {
            let (content, kind) = read_next_message(&mut self.socket).ok()?;

            match kind {
                FrameKind::Binary => break Some(WebSocketMessage::Binary(content)),
                FrameKind::Text => break Some(WebSocketMessage::Text(String::from_utf8_lossy(&content).into())),
                FrameKind::Continue => break None,
                FrameKind::Close => break None,
                FrameKind::Ping => {
                    let response_frame = Frame::from_payload(FrameKind::Pong, content);
                    self.socket.write_all(&response_frame.encode()).ok()?;
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


pub fn read_next_message(reader: &mut TcpStream) -> Result<(Vec<u8>, FrameKind), FrameError> {
    // blocks the current thread until we receive a full message from the client
    let mut bytes_so_far = Vec::new();

    loop {
        match read_next_frame(reader) {
            Ok(frame) => {
                bytes_so_far.extend_from_slice(frame.payload());
                if frame.is_final_frame() {
                    break Ok((bytes_so_far, frame.kind()));
                } // else: retry
            },
            Err(e) => break Err(e),
        }
    }
}

fn read_next_frame(tcp_reader: &mut TcpStream) -> Result<Frame, FrameError> {
    let mut buf = Vec::new();

    loop {
        let mut to_add = [0; BYTES_AT_A_TIME];
        let len = tcp_reader.read(&mut to_add)?;
        if len == 0 {
            return Err(FrameError::ConnectionClosed);
        }

        buf.extend_from_slice(&to_add[..len]);

        match Frame::decode(&buf) {
            Ok(frame) => {
                buf.clear();
                break Ok(frame)
            },
            Err(e) if e.should_retry() => {},
            Err(e) => break Err(e),
        }
    }
}
