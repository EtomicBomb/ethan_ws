use std::net::TcpStream;
use std::io;
use crate::util::{FrameKind};
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
    writer.write_all(&[0b_1000_0000 | frame_kind as u8])?;

    write_len_header(payload.len(), writer)?;

    writer.write_all(payload)?;

    writer.flush()
}

fn write_len_header(len: usize, writer: &mut impl Write) -> io::Result<()> {
    match len {
        0..=125 => writer.write_all(&[len as u8]),
        126..=65535 => {
            writer.write_all(&[126])?;
            writer.write_all(&(len as u16).to_be_bytes())
        },
        _ => {
            writer.write_all(&[127])?;
            writer.write_all(&(len as u64).to_be_bytes())
        },
    }
}