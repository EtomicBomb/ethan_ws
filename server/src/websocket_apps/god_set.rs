use crate::websocket_apps::{WebSocketClientState, write_string_to};
use crate::server_state::{WebsocketMessage, StreamState, ClientId};
use std::collections::HashMap;
use std::net::TcpStream;
use crate::websocket_apps::tanks::GlobalTanksGameState;

pub struct GodSetClientState;

impl GodSetClientState {
    pub fn new(id: ClientId, database: &mut HashMap<String, String>, writers: &mut HashMap<ClientId, TcpStream>) -> GodSetClientState {
        let string = database["godset"].to_string();
        write_string_to(id, string, writers);
        GodSetClientState
    }
}

impl WebSocketClientState for GodSetClientState {
    fn on_receive_message(&mut self, _database: &mut HashMap<String, String>, _tank_state: &mut GlobalTanksGameState, _writers: &mut HashMap<ClientId, TcpStream>, _message: WebsocketMessage) -> StreamState {
        StreamState::Drop
    }
    fn on_socket_close(&mut self, _database: &mut HashMap<String, String>, _tank_state: &mut GlobalTanksGameState, _writers: &mut HashMap<ClientId, TcpStream>) { }
}