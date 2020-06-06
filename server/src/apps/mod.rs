use std::collections::HashMap;
use std::net::TcpStream;
use web_socket::{Frame, FrameKind, WebSocketMessage, WebSocketListener};
use std::io::Write;
use std::sync::atomic::{self, AtomicU64};

mod filler;
mod god_set;
mod tanks;
mod history;
mod arena;

use std::sync::{Arc, Mutex};
use std::{thread};
use crate::apps::filler::FillerGlobalState;
use crate::apps::god_set::GodSetGlobalState;
use crate::apps::tanks::TanksGlobalState;
use crate::apps::history::HistoryGlobalState;
use crate::apps::arena::ArenaGlobalState;

use json::Json;

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

        Some(CoolStuff { map, peer_id_generator: PeerIdGenerator::new() })
    }

    pub fn on_new_web_socket_connection(&self, location: &str, tcp_stream: TcpStream) {
        match self.map.get(location) {
            Some(deal) => {
                let id = self.peer_id_generator.next();

                deal.lock().unwrap().new_peer(id, TcpStreamWriter::new(tcp_stream.try_clone().unwrap()));

                let state = Arc::clone(&self.map[location]);
                thread::Builder::new().name(format!("server{}/{}", location, id.0)).spawn(move || {
                    for message in WebSocketListener::new(tcp_stream) {
                        match state.lock().unwrap().on_message_receive(id, message) {
                            StreamState::Keep => {},
                            StreamState::Drop => break,
                        }
                    }

                    state.lock().unwrap().on_drop(id);
                }).unwrap();
            },
            None => {},
        }
    }

    pub fn periodic(&self) {
        for state in self.map.values() {
            state.lock().unwrap().periodic();
        }
    }
}

trait GlobalState: Send {
    fn new_peer(&mut self, id: PeerId, tcp_stream: TcpStreamWriter);
    fn on_message_receive(&mut self, id: PeerId, message: WebSocketMessage) -> StreamState;
    fn on_drop(&mut self, id: PeerId);
    fn periodic(&mut self);
}

#[derive(Debug)]
struct TcpStreamWriter {
    inner: TcpStream,
}

impl TcpStreamWriter {
    fn new(tcp_stream: TcpStream) -> TcpStreamWriter {
        TcpStreamWriter { inner: tcp_stream }
    }


    fn write_text_or_drop(&mut self, string: String) -> StreamState {
        let frame = Frame::from_payload(FrameKind::Text, string.into_bytes());
        match self.inner.write_all(&frame.encode()) {
            Ok(_) => StreamState::Keep,
            Err(_) => StreamState::Drop,
        }
    }

    fn write_text_or_none(&mut self, string: String) -> Option<()> {
        let frame = Frame::from_payload(FrameKind::Text, string.into_bytes());
        self.inner.write_all(&frame.encode()).ok()
    }

    fn write_json_or_drop(&mut self, json: Json) -> StreamState {
        self.write_text_or_drop(json.to_string())
    }
    //
    // fn write_json_or_none(&mut self, json: Json) -> Option<()> {
    //     let frame = Frame::from_payload(FrameKind::Text, json.to_string().into_bytes());
    //     self.inner.write_all(&frame.encode()).ok()
    // }



    // fn write_text_frame(&mut self, string: String) -> io::Result<()> {
    //     let frame = Frame::from_payload(FrameKind::Text, string.into_bytes());
    //     self.inner.write_all(&frame.encode())
    // }

    // fn write_json(&mut self, json: Json) -> io::Result<()> {
    //     self.write_text_frame(json.to_string())
    // }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum StreamState {
    Keep,
    Drop,
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