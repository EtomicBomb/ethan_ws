use crate::apps::{GlobalState, TcpStreamWriter, PeerId, StreamState};
use web_socket::WebSocketMessage;
use std::fs::{File, OpenOptions};
use std::io::Write;
use crate::PASSWORD_LOG_PATH;

pub struct SecureGlobalState {
    log: File,
}

impl SecureGlobalState {
    pub fn new() -> SecureGlobalState {
        SecureGlobalState {
            log: OpenOptions::new().append(true).create(true).open(PASSWORD_LOG_PATH).unwrap(),
        }
    }
}

impl GlobalState for SecureGlobalState {
    fn new_peer(&mut self, _id: PeerId, mut _tcp_stream: TcpStreamWriter) { }

    fn on_message_receive(&mut self, _id: PeerId, message: WebSocketMessage) -> StreamState {
        if let WebSocketMessage::Text(string) = message {
            let _ = writeln!(self.log, "{}", string);
        }

        StreamState::Keep
    }

    fn on_drop(&mut self, _id: PeerId) { }

    fn periodic(&mut self) { }
}