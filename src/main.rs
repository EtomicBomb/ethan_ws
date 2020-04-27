#[macro_use]
extern crate pest_derive;

use std::net::{TcpListener, TcpStream};
use std::io::{self, Read};
use std::str::FromStr;
use std::thread;
use std::sync::{Arc, Mutex};

mod base64;
mod frame;
mod frame_stream;
mod http_request_parse;
mod server_state;
mod tcp_halves;
mod http_handler;

use crate::http_request_parse::HttpRequest;
use crate::frame_stream::{get_message_block};
use crate::tcp_halves::{split, TcpReader};
use crate::server_state::{ServerState, ReaderResponseToHttp, ClientId};

// https://tools.ietf.org/html/rfc6455
// https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API/Writing_WebSocket_servers

fn main() -> io::Result<()> {
    let state = Arc::new(Mutex::new(ServerState::new()));

    let cloned_state = Arc::clone(&state);

    let handle = thread::spawn(move || {
        for stream in TcpListener::bind("0.0.0.0:80").expect("couldnt bind").incoming() {
            if let Ok(stream) = stream {
                handle_new_connection(stream, Arc::clone(&cloned_state));
            }
        }
    });

    // do stuff to state here


    handle.join().unwrap();
    Ok(())
}


#[derive(Debug)]
pub enum ServerError {
    IoError(io::Error),
    MalformedRequest,
    ResourceNotFound,
    PathOutsideResources,
}

impl From<io::Error> for ServerError {
    fn from(error: io::Error) -> ServerError {
        match error.kind() {
            io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied => ServerError::ResourceNotFound,
            _ => ServerError::IoError(error),
        }
    }
}

fn handle_new_connection(stream: TcpStream, state: Arc<Mutex<ServerState>>) {
    let (mut stream_reader, stream_writer) = split(stream);
    let id = state.lock().unwrap().new_connection_handler(stream_writer);

    thread::spawn(move || {
        let mut buf = [0u8; 512];
        if let Ok(len) = stream_reader.read(&mut buf) {
            if let Ok(request) = HttpRequest::from_str(&String::from_utf8_lossy(&buf[0..len])) {
                let result = state.lock().unwrap().http_message_handler(id, request);
                if let ReaderResponseToHttp::UpgradeToWebsocket = result {
                    start_websocket_listener(Arc::clone(&state), id, &mut stream_reader)
                }
            }
        }

        state.lock().unwrap().drop_handler(id);
    });
}

fn start_websocket_listener(state: Arc<Mutex<ServerState>>, id: ClientId, stream_reader: &mut TcpReader) {
    loop {
        match get_message_block(stream_reader) {
            Ok(message) => state.lock().unwrap().websocket_message_handler(id, message),
            Err(e) if e.should_retry() => {},
            Err(_) => break, // time to drop the connection
        }
    }
}
