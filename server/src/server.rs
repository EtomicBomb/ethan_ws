use std::collections::HashMap;
use std::net::{TcpStream, TcpListener};
use web_socket::{WebSocketMessage, WebSocketListener, WebSocketWriter};
use std::io::{self, Write, Read};
use std::sync::atomic::{self, AtomicU64};

use std::sync::{Arc, Mutex};
use std::{thread};

use std::option::NoneError;
use http::HttpRequest;
use crate::util::to_base64;
use sha1::Sha1;
use crate::http_handler::send_resource;
use std::path::{PathBuf};
use std::time::Duration;
use std::hash::Hash;

pub struct Server {
    name: String,
    map: HashMap<String, Arc<Mutex<dyn GlobalState>>>,
    peer_id_generator: PeerIdGenerator,

    resources_root: PathBuf,
    max_http_request_size: usize,
    period_length: Duration,
}

impl Server {
    pub fn new(name: String, resources_root: PathBuf, max_http_request_size: usize, period_length: Duration) -> Server {
        Server {
            name,
            map: HashMap::new(),
            peer_id_generator: PeerIdGenerator::new(),
            resources_root,
            max_http_request_size,
            period_length
        }
    }

    pub fn start(self) {
        let name = self.name.clone();
        let period_length = self.period_length;

        let arc = Arc::new(self);

        let arc_clone = Arc::clone(&arc);
        thread::Builder::new().name(format!("{}_listen", name)).spawn(move || {

            for tcp_stream in TcpListener::bind("0.0.0.0:8080").unwrap().incoming() {
                if let Ok(tcp_stream) = tcp_stream {
                    arc_clone.handle_new_connection(tcp_stream);
                }
            }

        }).unwrap();

        // our periodic loop
        loop {
            arc.periodic();
            thread::sleep(period_length);
        }
    }

    pub fn web_socket_add(&mut self, location: String, global_state: Arc<Mutex<dyn GlobalState>>) {
        self.map.insert(location, global_state);
    }

    pub fn handle_new_connection(self: &Arc<Server>, mut tcp_stream: TcpStream) {
        let id = self.peer_id_generator.next();
        let self_clone = Arc::clone(self);

        thread::Builder::new().name(format!("{}/{}", self.name, id.stringify())).spawn(move || {

            if let Some(request) = get_request(&mut tcp_stream, self_clone.max_http_request_size) {
                self_clone.handle_request(request, tcp_stream, id);
            }

        }).unwrap();
    }

    fn handle_request(&self, request: HttpRequest, mut tcp_stream: TcpStream, id: PeerId) {
        // check if we have a regular old http get or a websocket request
        if let Some(sec_key) = request.get_header_value("Sec-WebSocket-Key") {
            let mut hasher = Sha1::new();
            hasher.update(sec_key.as_bytes());
            hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"); // magic number
            let digest = to_base64(&hasher.digest().bytes());
            let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n", digest);

            if tcp_stream.write_all(response.as_bytes()).is_ok() {
                if let Some(state) = self.map.get(request.resource_location()) {
                    Server::on_new_web_socket_connection(tcp_stream, id, state);
                }
            }

        } else {
            // just a regular old http request!
            let _ = send_resource(&request, &mut tcp_stream, &self.resources_root);
        }
    }

    fn on_new_web_socket_connection(tcp_stream: TcpStream, id: PeerId, state: &Arc<Mutex<dyn GlobalState>>) {
        state.lock().unwrap().new_peer(id, WebSocketWriter::new(tcp_stream.try_clone().unwrap()));

        for message in WebSocketListener::new(tcp_stream) {
            match state.lock().unwrap().on_message_receive(id, message) {
                Ok(()) => {},
                Err(Disconnect) => break,
            }
        }

        state.lock().unwrap().on_disconnect(id);
    }

    pub fn periodic(&self) {
        for state in self.map.values() {
            state.lock().unwrap().periodic();
        }
    }
}

pub trait GlobalState: Send {
    fn new_peer(&mut self, id: PeerId, tcp_stream: WebSocketWriter);
    fn on_message_receive(&mut self, id: PeerId, message: WebSocketMessage) -> Result<(), Disconnect>;
    fn on_disconnect(&mut self, id: PeerId);
    fn periodic(&mut self);
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Disconnect;

impl From<io::Error> for Disconnect {
    fn from(_: io::Error) -> Disconnect { Disconnect }
}

impl From<NoneError> for Disconnect {
    fn from(_: NoneError) -> Disconnect { Disconnect }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PeerId(u64);

impl PeerId {
    fn stringify(&self) -> String {
        self.0.to_string()
    }
}

struct PeerIdGenerator(AtomicU64);
impl PeerIdGenerator {
    fn new() -> PeerIdGenerator {
        PeerIdGenerator(AtomicU64::new(0))
    }
}

impl PeerIdGenerator {
    fn next(&self) -> PeerId {
        PeerId(self.0.fetch_add(1, atomic::Ordering::Relaxed))
    }
}

fn get_request(tcp_stream: &mut TcpStream, request_size: usize) -> Option<HttpRequest> {
    let mut buf = vec![0u8; request_size];

    let len = tcp_stream.read(&mut buf).ok()?;

    let request = String::from_utf8_lossy(&buf[0..len]).parse().ok()?;

    Some(request)
}
