use std::net::{TcpListener, TcpStream};
use crate::HttpRequest;
use std::str::FromStr;
use std::io::Read;
use std::io;

pub struct HttpIterator {
    listener: TcpListener,
    buf: Vec<u8>,
}

impl HttpIterator {
    pub fn new(port: u16, max_request_size: usize) -> io::Result<HttpIterator> {
        Ok(HttpIterator {
            listener: TcpListener::bind(("0.0.0.0", port))?,
            buf: vec![0u8; max_request_size],
        })
    }
}

impl Iterator for HttpIterator {
    type Item = (HttpRequest, TcpStream);

    fn next(&mut self) -> Option<(HttpRequest, TcpStream)> {
        loop {
            match try_read(&mut self.listener, &mut self.buf) {
                Some(result) => break Some(result),
                None => {},
            }
        }
    }
}

fn try_read(listener: &mut TcpListener, buf: &mut [u8]) -> Option<(HttpRequest, TcpStream)> {
    let (mut tcp_stream, _) = listener.accept().ok()?;

    let len = tcp_stream.read(buf).ok()?;

    let request = HttpRequest::from_str(&String::from_utf8_lossy(&buf[0..len])).ok()?;

    Some((request, tcp_stream))
}