use std::collections::HashMap;
use std::net::TcpStream;
use websocket::{Frame, FrameKind};
use std::io::Write;

use crate::server_state::{StreamState, WebsocketMessage, ClientId};

mod filler;
mod dummy;
mod god_set;
mod tanks;

pub use crate::websocket_apps::filler::FillerClientState;
pub use crate::websocket_apps::god_set::GodSetClientState;
pub use crate::websocket_apps::dummy::DummyClientState;
pub use crate::websocket_apps::tanks::GlobalTanksGameState;
pub use crate::websocket_apps::tanks::TanksClientState;

pub trait WebSocketClientState {
    fn on_receive_message(&mut self, database: &mut HashMap<String, String>, tank_state: &mut GlobalTanksGameState, writers: &mut HashMap<ClientId, TcpStream>, message: WebsocketMessage) -> StreamState;
    fn on_socket_close(&mut self, database: &mut HashMap<String, String>, tank_state: &mut GlobalTanksGameState, writers: &mut HashMap<ClientId, TcpStream>);
}

pub fn write_string_to(receiver: ClientId, string: String, writers: &mut HashMap<ClientId, TcpStream>) -> StreamState {
    let frame = Frame::from_payload(FrameKind::Text, string.into_bytes());
    match writers.get_mut(&receiver) {
        Some(writer) => match writer.write_all(&frame.encode()) {
            Ok(_) => StreamState::Keep,
            Err(_) => StreamState::Drop,
        },
        None => StreamState::Drop,
    }
}