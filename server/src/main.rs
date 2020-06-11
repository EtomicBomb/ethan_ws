#![feature(try_trait)]

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
const VOCABULARY_LOG_PATH: &'static str = "/home/pi/Desktop/server/vocabularyLog.txt";
const PASSWORD_LOG_PATH: &'static str = "/home/pi/Desktop/server/passwordLog.txt";

const WEBSOCKET_SECURE_KEY_MAGIC_NUMBER: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
const PERIOD_LENGTH: Duration = Duration::from_millis(100);

fn main() -> io::Result<()> {
    let global = Arc::new(CoolStuff::new().unwrap());

    let global_listener = Arc::clone(&global);
    let listener = thread::Builder::new().name("server_listener".into()).spawn(move || {
        for (request, tcp_stream) in HttpIterator::new(8080, MAX_HTTP_REQUEST_SIZE).unwrap() {
            handle_connection(request, tcp_stream, &global_listener);
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

fn handle_connection(request: HttpRequest, mut tcp_stream: TcpStream, global: &Arc<CoolStuff>) {
    if let Some(sec_key) = request.get_header_value("Sec-WebSocket-Key") {
        let to_hash = format!("{}{}", sec_key, WEBSOCKET_SECURE_KEY_MAGIC_NUMBER);
        let digest = to_base64(&Sha1::from(to_hash.as_bytes()).digest().bytes());
        let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n", digest);

        if tcp_stream.write_all(response.as_bytes()).is_ok() {
            global.on_new_web_socket_connection(request.resource_location(), tcp_stream);
        }

    } else {
        // just a regular old http request!
        let _ = send_resource(&request, &mut tcp_stream);
    }
}