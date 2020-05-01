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
mod god_set;
mod json;

#[macro_use]
mod log;

use crate::http_request_parse::HttpRequest;
use crate::frame_stream::{get_message_block};
use crate::tcp_halves::{split, TcpReader};
use crate::server_state::{ServerState, StreamState, ClientId};

const MAX_HTTP_REQUEST_SIZE: usize = 2048;

// https://tools.ietf.org/html/rfc6455
// https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API/Writing_WebSocket_servers

fn main() -> io::Result<()> {
    let state = Arc::new(Mutex::new(ServerState::new()));

    let cloned_state = Arc::clone(&state);

    log!("server started");

    let handle = thread::Builder::new().name(String::from("ethan_ws_listener")).spawn(move || {
        for stream in TcpListener::bind("0.0.0.0:80").expect("couldnt bind").incoming() {
            if let Ok(stream) = stream {
                handle_new_connection(stream, Arc::clone(&cloned_state));
            }
        }
    }).unwrap();

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

    thread::Builder::new().name(format!("ethan_ws{}", id)).spawn(move ||{
        let mut buf = [0u8; MAX_HTTP_REQUEST_SIZE];
        if let Ok(len) = stream_reader.read(&mut buf) {
            if let Ok(request) = HttpRequest::from_str(&String::from_utf8_lossy(&buf[0..len])) {
                let result = state.lock().unwrap().http_message_handler(id, request);
                if let StreamState::Keep = result {
                    start_websocket_listener(Arc::clone(&state), id, &mut stream_reader)
                }
            }
        }

        state.lock().unwrap().drop_handler(id);
    }).expect("couldnt spawn thread");
}

fn start_websocket_listener(state: Arc<Mutex<ServerState>>, id: ClientId, stream_reader: &mut TcpReader) {
    loop {
        match get_message_block(stream_reader) {
            Ok((payload, kind)) => match state.lock().unwrap().websocket_message_handler(id, payload, kind) {
                StreamState::Keep => {},
                StreamState::Drop => break,
            }
            Err(e) if e.should_retry() => {},
            Err(_) => break, // time to drop the connection
        }
    }
}
