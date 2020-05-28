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
mod util;
mod http_handler;

use crate::http_handler::send_resource;
use crate::util::to_base64;
use sha1::Sha1;
use crate::apps::{CoolStuff};
use std::time::Duration;

const MAX_HTTP_REQUEST_SIZE: usize = 2048;
const RESOURCES_ROOT: &'static str = "/home/pi/Desktop/server/resources";
const GOD_SET_PATH: &'static str = "/home/pi/Desktop/server/resources/apush/godset.txt";

const WEBSOCKET_SECURE_KEY_MAGIC_NUMBER: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
const PERIOD_LENGTH: Duration = Duration::from_millis(100);

// https://tools.ietf.org/html/rfc6455
// https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API/Writing_WebSocket_servers

fn main() -> io::Result<()> {
    let global = Arc::new(CoolStuff::new().unwrap());

    let global_listener = Arc::clone(&global);
    let listener = thread::Builder::new().name("server_listener".into()).spawn(move || {
        for (request, tcp_stream) in HttpIterator::new(8080, MAX_HTTP_REQUEST_SIZE).unwrap() {
            foo(request, tcp_stream, &global_listener);
        }
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