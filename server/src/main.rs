extern crate web_socket;
extern crate json;
extern crate lisp;
extern crate http;

use http::{HttpRequest, HttpIterator};

use std::net::{TcpStream};
use std::io::{self, Write};
use std::thread;
use std::sync::{Arc};

mod apps;
mod base64;
mod server_state;
mod http_handler;
#[macro_use]
mod log;
mod state;

use crate::http_handler::send_resource;
use crate::base64::to_base64;
use sha1::Sha1;
use crate::apps::{CoolStuff};
use std::time::Duration;

const MAX_HTTP_REQUEST_SIZE: usize = 2048;
const RESOURCES_ROOT: &'static str = "/home/pi/Desktop/server/resources";
const LOG_FILE_PATH: &'static str = "/home/pi/Desktop/server/log.txt";
const GOD_SET_PATH: &'static str = "/home/pi/Desktop/server/resources/apush/godset.txt";

const WEBSOCKET_SECURE_KEY_MAGIC_NUMBER: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
const PERIOD_LENGTH: Duration = Duration::from_millis(100);

// https://tools.ietf.org/html/rfc6455
// https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API/Writing_WebSocket_servers

fn main() -> io::Result<()> {
    let global = Arc::new(CoolStuff::new().unwrap());

    log!("server started");

    let global_listener = Arc::clone(&global);
    let listener = thread::Builder::new().name("server_listener".into()).spawn(move || {
        for (request, tcp_stream) in HttpIterator::new(8080, MAX_HTTP_REQUEST_SIZE).unwrap() {
            foo(request, tcp_stream, &global_listener);
        }

        dbg!();
    }).unwrap();


    let global_period = Arc::clone(&global);
    thread::Builder::new().name("server_periodic".into()).spawn(move || {
        loop {
            global_period.periodic();
            thread::sleep(PERIOD_LENGTH);
        }
    }).unwrap();


    listener.join().unwrap();
    Ok(())
}

fn foo(request: HttpRequest, mut tcp_stream: TcpStream, global: &Arc<CoolStuff>) {
    match try_upgrade_connection(&request) {
        Some(web_socket_upgrade_response) => {
            if tcp_stream.write_all(web_socket_upgrade_response.as_bytes()).is_ok() {
                let _ = global.on_new_web_socket_connection(request.resource_location(), tcp_stream);
            }
        },
        None => { // that wasn't a WebSocket request !
            let _ = send_resource(&request, &mut tcp_stream);
        },
    }
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

fn try_upgrade_connection(request: &HttpRequest) -> Option<String> {
    if request.get_header_value("Upgrade") == Some("websocket") {
        match request.get_header_value("Sec-WebSocket-Key") {
            Some(key) => {
                if !key.is_ascii() { return None };
                let to_hash = format!("{}{}", key, WEBSOCKET_SECURE_KEY_MAGIC_NUMBER);
                let response = to_base64(&Sha1::from(to_hash.as_bytes()).digest().bytes());
                // NOTE: excludes header: nSec-WebSocket-Protocol: chat
                Some(format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n", response))
            },
            None => None,
        }
    } else {
        None
    }
}


//
// fn handle_new_connection(socket: TcpStream, state: Arc<GlobalStates>) {
//     let mut stream_reader = socket.try_clone().unwrap();
//
//     let mut buf = [0u8; MAX_HTTP_REQUEST_SIZE];
//     if let Ok(len) = stream_reader.read(&mut buf) {
//         if let Ok(request) = HttpRequest::from_str(&String::from_utf8_lossy(&buf[0..len])) {
//             http_message_handler(request, socket, state);
//         }
//     }
// }

//
// pub fn http_message_handler(request: HttpRequest, mut socket: TcpStream, global: Arc<GlobalStates>) {
//     // return StreamState::Keep if our connection should be updated to websockets
//     match try_upgrade_connection(&request) {
//         Some(web_socket_upgrade_response) => {
//             match socket.write_all(web_socket_upgrade_response.as_bytes()) {
//                 Ok(_) => global.spawn_from_new_connection(request.resource_location(), socket).unwrap(),
//                 Err(_) => {},
//             }
//         },
//         None => { // that wasn't a WebSocket request !
//             let _ = send_resource(&request, &mut socket);
//         },
//     }
// }

// fn start_websocket_listener(state: Arc<Mutex<ServerState>>, id: ClientId, stream_reader: &mut TcpStream) {
//     loop {
//         match read_next_message(stream_reader) {
//             Ok((payload, kind)) => match state.lock().unwrap().websocket_message_handler(id, payload, kind) {
//                 StreamState::Keep => {},
//                 StreamState::Drop => break,
//             }
//             Err(e) if e.should_retry() => {},
//             Err(_) => break, // time to drop the connection
//         }
//     }
// }
//