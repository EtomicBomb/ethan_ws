use std::net::TcpStream;
use std::io;
use crate::util::{FrameKind, write_frame};

#[derive(Debug)]
pub struct WebSocketWriter {
    tcp_stream: TcpStream,
}

impl WebSocketWriter {
    pub fn new(tcp_stream: TcpStream) -> WebSocketWriter {
        WebSocketWriter { tcp_stream }
    }

    pub fn write_string(&mut self, string: &str) -> io::Result<()> {
        self.write(string.as_bytes(), FrameKind::Text)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.write(bytes, FrameKind::Binary)
    }

    fn write(&mut self, payload: &[u8], frame_kind: FrameKind) -> io::Result<()> {
        write_frame(&mut self.tcp_stream, payload, frame_kind)
    }
}

