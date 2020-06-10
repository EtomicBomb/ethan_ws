use std::net::TcpStream;
use std::io;
use crate::util::{FrameKind, PayloadLength};
use std::io::{Write, BufWriter};

#[derive(Debug)]
pub struct WebSocketWriter {
    writer: BufWriter<TcpStream>,
}

impl WebSocketWriter {
    pub fn new(tcp_stream: TcpStream) -> WebSocketWriter {
        WebSocketWriter { writer: BufWriter::new(tcp_stream) }
    }

    pub fn write_string(&mut self, string: &str) -> io::Result<()> {
        write_frame(&mut self.writer, string.as_bytes(), FrameKind::Text)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        write_frame(&mut self.writer, bytes, FrameKind::Binary)
    }
}

pub fn write_frame(writer: &mut impl Write, payload: &[u8], frame_kind: FrameKind) -> io::Result<()> {
    let len = payload.len();
    let len_descriptor = PayloadLength::from_len(len);

    writer.write_all(&[0b_1000_0000 | frame_kind as u8, len_descriptor.to_byte()])?;

    match len_descriptor {
        PayloadLength::Small(_) => {},
        PayloadLength::Extended =>
            writer.write_all(&(len as u64).to_be_bytes()[6..])?,
        PayloadLength::ExtraExtended =>
            writer.write_all(&(len as u64).to_be_bytes())?,
    }

    writer.write_all(payload)?;

    writer.flush()
}
