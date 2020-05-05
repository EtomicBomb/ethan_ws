
use std::net::{TcpStream, IpAddr};
use std::cell::UnsafeCell;
use std::io::{self, Read, Write, IoSliceMut, IoSlice};
use std::sync::Arc;

// pub fn split(tcp_stream: TcpStream) -> (TcpReader, TcpWriter) {
//     let tcp_stream = Arc::new(UnsafeCell::new(tcp_stream));
//
//     (TcpReader::new(Arc::clone(&tcp_stream)), TcpWriter::new(tcp_stream))
// }
//
//
//
// pub struct TcpReader {
//     tcp_stream: Arc<UnsafeCell<TcpStream>>,
// }
//
// impl TcpReader {
//     fn new(tcp_stream: Arc<UnsafeCell<TcpStream>>) -> TcpReader {
//         TcpReader { tcp_stream }
//     }
// }
//
//
// unsafe impl Send for TcpReader {}
// unsafe impl Sync for TcpReader {}
//
// impl Read for TcpReader {
//     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//         let mut_ref: &mut TcpStream = unsafe { &mut *self.tcp_stream.get() };
//         mut_ref.read(buf)
//     }
//
//     fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
//         let mut_ref: &mut TcpStream = unsafe { &mut *self.tcp_stream.get() };
//         mut_ref.read_vectored(bufs)
//     }
// }
//
// pub struct TcpWriter {
//     tcp_stream: Arc<UnsafeCell<TcpStream>>,
// }
//
// impl TcpWriter {
//     fn new(tcp_stream: Arc<UnsafeCell<TcpStream>>) -> TcpWriter {
//         TcpWriter { tcp_stream }
//     }
//
//     pub fn get_ip_addr(&self) -> Option<(IpAddr, u16)> {
//         let r: &TcpStream = unsafe { &*self.tcp_stream.get() };
//         r.peer_addr().ok().map(|a| (a.ip(), a.port()))
//     }
// }
//
// unsafe impl Send for TcpWriter {}
// unsafe impl Sync for TcpWriter {}
//
//
// impl Write for TcpWriter {
//     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//         let mut_ref: &mut TcpStream = unsafe { &mut *self.tcp_stream.get() };
//         mut_ref.write(buf)
//     }
//
//     fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
//         let mut_ref: &mut TcpStream = unsafe { &mut *self.tcp_stream.get() };
//
//         mut_ref.write_vectored(bufs)
//     }
//
//     fn flush(&mut self) -> io::Result<()> {
//         let mut_ref: &mut TcpStream = unsafe { &mut *self.tcp_stream.get() };
//         mut_ref.flush()
//     }
// }
//

pub fn split(tcp_stream: TcpStream) -> (TcpReader, TcpWriter) {
    let other = tcp_stream.try_clone().expect("error cloning streams");
    (TcpReader { tcp_stream }, TcpWriter { tcp_stream: other })
}

pub struct TcpReader {
    tcp_stream: TcpStream,
}

impl Read for TcpReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.tcp_stream.read(buf)
    }
}

pub struct TcpWriter {
    tcp_stream: TcpStream,
}

impl TcpWriter {
    pub fn get_ip_addr(&self) -> Option<(IpAddr, u16)> {
        self.tcp_stream.peer_addr().ok().map(|a| (a.ip(), a.port()))
    }
}

impl Write for TcpWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tcp_stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.tcp_stream.flush()
    }
}


