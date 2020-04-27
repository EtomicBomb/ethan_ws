use std::io::{Read};
use crate::frame::{Frame, FrameError};
use crate::tcp_halves::TcpReader;

const BYTES_AT_A_TIME: usize = 1024;


pub fn get_message_block(reader: &mut TcpReader) -> Result<Vec<u8>, FrameError> {
    // blocks the current thread until we recieve a full message from the client

    let mut bytes_so_far = Vec::new();

    loop {
        match read_next_frame(reader) {
            Ok(frame) => {
                bytes_so_far.extend_from_slice(frame.payload());
                if frame.is_final_frame() {
                    break Ok(bytes_so_far);
                } // else: retry
            },
            Err(e) => break Err(e),
        }
    }
}

fn read_next_frame(tcp_reader: &mut TcpReader) -> Result<Frame, FrameError> {
    let mut buf = Vec::new();

    loop {
        let mut to_add = [0; BYTES_AT_A_TIME];
        let len = tcp_reader.read(&mut to_add)?;
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
