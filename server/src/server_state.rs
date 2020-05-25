use sha1::Sha1;
use websocket::{Frame, FrameKind};
use http::HttpRequest;

use std::io::{Write};
use std::fmt;
use std::net::TcpStream;
use std::collections::HashMap;

use crate::websocket_apps::{WebSocketClientState, FillerClientState, GodSetClientState, DummyClientState, GlobalTanksGameState, TanksClientState};
use crate::base64::to_base64;
use crate::http_handler::get_response_to_http;
use crate::log;
use crate::god_set::GodSet;

const WEBSOCKET_SECURE_KEY_MAGIC_NUMBER: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub struct ServerState {
    writers: HashMap<ClientId, TcpStream>,
    clients: HashMap<ClientId, Client>,
    database: HashMap<String, String>,
    tank_state: GlobalTanksGameState,
    id_generator: ClientIdGenerator,
}

impl ServerState {
    pub fn new() -> ServerState {
        match GodSet::new() {
            Some(god_set) =>
                ServerState {
                    clients: HashMap::new(),
                    writers: HashMap::new(),
                    database: [("godset".to_string(), god_set.stringify())].iter().cloned().collect(),
                    tank_state: GlobalTanksGameState::new(),
                    id_generator: ClientIdGenerator::new()
                },
            None => {
                log!("Couldn't parse godset file");
                panic!();
            }
        }
    }

    pub fn new_connection_handler(&mut self, stream: TcpStream) -> ClientId {
        let id = self.id_generator.next();

        match stream.peer_addr() {
            Ok(addr) => log!("new connection to {}:{}, id {}", addr.ip(), addr.port(), id),
            Err(_) => log!("new connection to unknown, id {}", id),
        };

        self.writers.insert(id, stream);
        self.clients.insert(id, Client::new(id));

        id
    }

    pub fn drop_handler(&mut self, id: ClientId) {
        log!("dropping connection to {}", id);

        match self.clients.get_mut(&id) {
            Some(Client { client_state: Some(client_state), .. }) =>
                client_state.on_socket_close(&mut self.database, &mut self.tank_state, &mut self.writers),
            _ => {},
        }

        self.clients.remove(&id);
        self.writers.remove(&id);
    }

    pub fn websocket_message_handler(&mut self, id: ClientId, message_bytes: Vec<u8>, kind: FrameKind) -> StreamState {
        match kind {
            FrameKind::Text | FrameKind::Binary => {
                let message = match kind {
                    FrameKind::Binary => WebsocketMessage::Binary(message_bytes),
                    FrameKind::Text => WebsocketMessage::Text(String::from_utf8_lossy(&message_bytes).to_string()),
                    _ => unreachable!(),
                };

                match self.clients.get_mut(&id) {
                    Some(Client { client_state: Some(client_state), .. }) =>
                        client_state.on_receive_message(&mut self.database, &mut self.tank_state, &mut self.writers, message),
                    _ => StreamState::Drop,
                }
            },
            FrameKind::Ping => {
                let pong_frame = Frame::from_payload(FrameKind::Pong, message_bytes).encode();
                self.write_bytes_to(id, &pong_frame)
            },
            FrameKind::Continue => {
                log!("panicking, did not expect continue message from {}", id);
                StreamState::Drop
            },
            FrameKind::Pong => StreamState::Keep,
            FrameKind::Close => StreamState::Drop, // that's what they want us to do anyway, right?
        }
    }

    pub fn http_message_handler(&mut self, id: ClientId, message: HttpRequest) -> StreamState {
        log!("client {} requested {}", id, message.resource_location());

        // return StreamState::Keep if our connection should be updated to websockets
        match self.writers.get_mut(&id) {
            Some(writer) => match handle_deelio(&message) {
                Some(websocket_upgrade_response) => {
                    match writer.write_all(websocket_upgrade_response.as_bytes()) {
                        Ok(_) => {
                            self.clients.get_mut(&id).unwrap().upgrade(message, &mut self.database, &mut self.tank_state, &mut self.writers);
                            StreamState::Keep
                        },
                        Err(_) => StreamState::Drop,
                    }
                },
                None => {
                    let _ = get_response_to_http(&message, writer);
                    StreamState::Drop
                },
            },
            None => {
                log!("Client {} should exist", id);
                StreamState::Drop
            },
        }
    }

    fn write_bytes_to(&mut self, id: ClientId, bytes: &[u8]) -> StreamState {
        match self.writers.get_mut(&id) {
            Some(writer) => match writer.write_all(bytes) {
                Ok(_) => StreamState::Keep,
                Err(_) => StreamState::Drop,
            },
            None => StreamState::Drop,
        }
    }
}


pub struct Client {
    pub id: ClientId,
    pub upgraded: Option<HttpRequest>, // the request that they used to upgrade their connection
    pub client_state: Option<Box<dyn WebSocketClientState + Send>>,
}

impl Client {
    fn new(id: ClientId) -> Client {
        Client { id, upgraded: None, client_state: None }
    }

    fn upgrade(&mut self, message: HttpRequest, database: &mut HashMap<String, String>, tank_state: &mut GlobalTanksGameState, writers: &mut HashMap<ClientId, TcpStream>) {
        self.client_state = Some(match message.resource_location() {
            "/filler" => Box::new(FillerClientState::new(self.id, database, writers)),
            "/godset" => Box::new(GodSetClientState::new(self.id, database, writers)),
            "/tanks" => Box::new(TanksClientState::new(self.id, database, tank_state, writers)),
            _ => Box::new(DummyClientState),
        });

        self.upgraded = Some(message);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClientId(u64);

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

struct ClientIdGenerator(u64);
impl ClientIdGenerator {
    fn new() -> ClientIdGenerator {
        ClientIdGenerator(0)
    }

    fn next(&mut self) -> ClientId {
        self.0 += 1;
        ClientId(self.0)
    }
}


pub enum StreamState {
    Keep,
    Drop,
}

fn handle_deelio(request: &HttpRequest) -> Option<String> {
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


pub enum WebsocketMessage {
    Text(String),
    Binary(Vec<u8>),
}

impl WebsocketMessage {
    pub fn get_text(&self) -> Option<&str> {
        match self {
            WebsocketMessage::Text(s) => Some(s),
            WebsocketMessage::Binary(_) => None,
        }
    }
}


// our interface to the server's state
// pub struct Writers<'a> {
//     database: &'a mut HashMap<String, String>,
//     inner: &'a mut HashMap<ClientId, TcpStream>,
// }

// impl<'a> Interface<'a> {
//     pub fn write_string_to(&mut self, receiver: ClientId, string: String) -> StreamState {
//         self.write_frame_to(receiver, FrameKind::Text, string.into_bytes())
//     }
//
//     pub fn database_get(&mut self, key: &str) -> Option<&'a str> {
//         self.database.get(key).map(|s| s.as_str())
//     }
//
//     pub fn database_get_mut(&mut self, key: &str) -> Option<&'a mut String> {
//         self.database.get_mut(key)
//     }
//
//     fn write_frame_to(self, receiver: ClientId, kind: FrameKind, contents: Vec<u8>) -> StreamState {
//         let frame = Frame::from_payload(kind, contents);
//
//         match self.inner.get_mut(&receiver) {
//             Some(writer) => match writer.write_all(&frame.encode()) {
//                 Ok(_) => StreamState::Keep,
//                 Err(_) => StreamState::Drop,
//             },
//             None => StreamState::Drop,
//         }
//     }
// }
