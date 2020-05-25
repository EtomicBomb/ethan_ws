use crate::websocket_apps::WebSocketClientState;
use crate::server_state::{StreamState, WebsocketMessage, ClientId};
use std::collections::HashMap;
use std::net::TcpStream;
use crate::websocket_apps::tanks::GlobalTanksGameState;

pub struct DummyClientState;

impl WebSocketClientState for DummyClientState {
    fn on_receive_message(&mut self, _database: &mut HashMap<String, String>, _tank_state: &mut GlobalTanksGameState, _writers: &mut HashMap<ClientId, TcpStream>, _message: WebsocketMessage) -> StreamState { StreamState::Drop }
    fn on_socket_close(&mut self, _database: &mut HashMap<String, String>, _tank_state: &mut GlobalTanksGameState, _writers: &mut HashMap<ClientId, TcpStream>) { }
}