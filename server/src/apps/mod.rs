use std::collections::HashMap;
use std::net::TcpStream;
use web_socket::{WebSocketMessage, WebSocketListener, WebSocketWriter};
use std::io::{self};
use std::sync::atomic::{self, AtomicU64};

mod filler;
mod god_set;
mod tanks;
mod history;
mod arena;
mod secure;

use std::sync::{Arc, Mutex};
use std::{thread};
use crate::apps::filler::FillerGlobalState;
use crate::apps::god_set::GodSetGlobalState;
use crate::apps::tanks::TanksGlobalState;
use crate::apps::history::HistoryGlobalState;
use crate::apps::arena::ArenaGlobalState;
use crate::apps::secure::SecureGlobalState;

use std::option::NoneError;

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

    pub fn on_new_web_socket_connection(&self, location: &str, tcp_stream: TcpStream) {
        if let Some(deal) = self.map.get(location) {
            let id = self.peer_id_generator.next();

            deal.lock().unwrap().new_peer(id, WebSocketWriter::new(tcp_stream.try_clone().unwrap()));

            let state = Arc::clone(&self.map[location]);
            thread::Builder::new().name(format!("server/{}", id.0)).spawn(move || {
                for message in WebSocketListener::new(tcp_stream) {
                    match state.lock().unwrap().on_message_receive(id, message) {
                        Ok(()) => {},
                        Err(Drop) => break,
                    }
                }

                state.lock().unwrap().on_drop(id);
            }).unwrap();
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