use std::collections::HashMap;
use std::net::TcpStream;
use web_socket::{WebSocketMessage, WebSocketListener, WebSocketWriter};
use std::io::{self, Write, Read};
use std::sync::atomic::{self, AtomicU64};

mod filler;
mod god_set;
mod tanks;
mod history;
mod arena;
mod secure;

use std::sync::{Arc, Mutex};
use std::{thread};
use crate::apps::filler::{FillerGlobalState};
use crate::apps::god_set::GodSetGlobalState;
use crate::apps::tanks::TanksGlobalState;
use crate::apps::history::HistoryGlobalState;
use crate::apps::arena::ArenaGlobalState;
use crate::apps::secure::SecureGlobalState;

use std::option::NoneError;
use http::HttpRequest;
use crate::{WEBSOCKET_SECURE_KEY_MAGIC_NUMBER, MAX_HTTP_REQUEST_SIZE};
use crate::util::to_base64;
use sha1::Sha1;
use crate::http_handler::send_resource;

pub struct CoolStuff {
    map: HashMap<String, Arc<Mutex<dyn GlobalState>>>,
    peer_id_generator: PeerIdGenerator,
}

impl CoolStuff {
    pub fn new() -> Option<CoolStuff> {
        let mut map: HashMap<String, Arc<Mutex<dyn GlobalState>>> = HashMap::new();
        map.insert("/filler".into(), Arc::new(Mutex::new(FillerGlobalState::new())));
        map.insert("/godset".into(), Arc::new(Mutex::new(GodSetGlobalState::new())));
        map.insert("/tanks".into(), Arc::new(Mutex::new(TanksGlobalState::new())));
        map.insert("/history".into(), Arc::new(Mutex::new(HistoryGlobalState::new())));
        map.insert("/arena".into(), Arc::new(Mutex::new(ArenaGlobalState::new())));
        map.insert("/secure".into(), Arc::new(Mutex::new(SecureGlobalState::new())));

        Some(CoolStuff { map, peer_id_generator: PeerIdGenerator::new() })
    }

    pub fn handle_new_connection(self: &Arc<CoolStuff>, mut tcp_stream: TcpStream) {
        // let's get our http request

        fn get_request(tcp_stream: &mut TcpStream) -> Option<HttpRequest> {
            let mut buf = [0u8; MAX_HTTP_REQUEST_SIZE];

            let len = tcp_stream.read(&mut buf).ok()?;

            let request = String::from_utf8_lossy(&buf[0..len]).parse().ok()?;

            Some(request)
        }

        if let Some(request) = get_request(&mut tcp_stream) {
            self.handle_request(request, tcp_stream);
        }
    }

    fn handle_request(self: &Arc<CoolStuff>, request: HttpRequest, mut tcp_stream: TcpStream) {
        let id = self.peer_id_generator.next();
        let self_clone = Arc::clone(self);

        thread::Builder::new().name(format!("server/{}", id.get_label())).spawn(move || {

            // check if we have a regular old http get or a websocket request
            if let Some(sec_key) = request.get_header_value("Sec-WebSocket-Key") {
                let to_hash = format!("{}{}", sec_key, WEBSOCKET_SECURE_KEY_MAGIC_NUMBER);
                let digest = to_base64(&Sha1::from(to_hash.as_bytes()).digest().bytes());
                let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n", digest);

                if tcp_stream.write_all(response.as_bytes()).is_ok() {
                    self_clone.on_new_web_socket_connection(request.resource_location(), tcp_stream, id);
                }

            } else {
                // just a regular old http request!
                let _ = send_resource(&request, &mut tcp_stream);
            }

        }).unwrap();
    }

    fn on_new_web_socket_connection(&self, location: &str, tcp_stream: TcpStream, id: PeerId) {
        if let Some(state) = self.map.get(location) {
            state.lock().unwrap().new_peer(id, WebSocketWriter::new(tcp_stream.try_clone().unwrap()));

            for message in WebSocketListener::new(tcp_stream) {
                match state.lock().unwrap().on_message_receive(id, message) {
                    Ok(()) => {},
                    Err(Drop) => break,
                }
            }

            state.lock().unwrap().on_drop(id);
        }
    }

    pub fn periodic(&self) {
        for state in self.map.values() {
            state.lock().unwrap().periodic();
        }
    }
}

trait GlobalState: Send {
    fn new_peer(&mut self, id: PeerId, tcp_stream: WebSocketWriter);
    fn on_message_receive(&mut self, id: PeerId, message: WebSocketMessage) -> Result<(), Drop>;
    fn on_drop(&mut self, id: PeerId);
    fn periodic(&mut self);
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Drop;

impl From<io::Error> for Drop {
    fn from(_: io::Error) -> Drop { Drop }
}

impl From<NoneError> for Drop {
    fn from(_: NoneError) -> Drop { Drop }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PeerId(u64);

impl PeerId {
    fn get_label(&self) -> String {
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