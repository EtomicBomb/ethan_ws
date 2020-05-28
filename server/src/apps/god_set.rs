use crate::apps::{GlobalState, PeerId, write_text, StreamState};
use std::net::TcpStream;
use std::io::{BufReader, BufRead};
use std::fs::File;
use crate::GOD_SET_PATH;
use json::Json;
use web_socket::WebSocketMessage;

pub struct GodSetGlobalState {
    json: String,
}

impl GodSetGlobalState {
    pub fn new() -> GodSetGlobalState {
        let file = BufReader::new(File::open(GOD_SET_PATH).unwrap());

        let json = Json::Array(file.lines()
            .map(|line| {
                let line = line.ok()?;
                let mut split = line.trim_end().split("\t");
                let year_start: u16 = split.next()?.parse().ok()?;
                let year_end: u16 = split.next()?.parse().ok()?;
                let social: bool = split.next()?.parse().ok()?;
                let political: bool = split.next()?.parse().ok()?;
                let economic: bool = split.next()?.parse().ok()?;
                let term = split.next()?.to_string();
                let definition = split.next()?.to_string();

                Some(Json::Object([
                    ("yearStart", Json::Number(year_start as f64)),
                    ("yearEnd", Json::Number(year_end as f64)),
                    ("social", Json::Boolean(social)),
                    ("political", Json::Boolean(political)),
                    ("economic", Json::Boolean(economic)),
                    ("term", Json::String(term)),
                    ("definition", Json::String(definition)),
                ].iter().map(|(a, b)| (a.to_string(), b.clone())).collect()))
            })
            .collect::<Option<Vec<Json>>>().unwrap());

        GodSetGlobalState { json: json.to_string() }
    }
}

impl GlobalState for GodSetGlobalState {
    fn new_peer(&mut self, _id: PeerId, mut tcp_stream: TcpStream) {
        write_text(&mut tcp_stream, self.json.clone());
    }

    fn on_message_receive(&mut self, _id: PeerId, _message: WebSocketMessage) -> StreamState { StreamState::Drop }

    fn periodic(&mut self) { }
}

//
// pub struct GodSetClientState;
//
// impl GodSetClientState {
//     pub fn new(id: ClientId, database: &mut HashMap<String, String>, writers: &mut HashMap<ClientId, TcpStream>) -> GodSetClientState {
//         let string = database["godset"].to_string();
//         write_string_to(id, string, writers);
//         GodSetClientState
//     }
// }
//
// impl WebSocketClientState for GodSetClientState {
//     fn on_receive_message(&mut self, _database: &mut HashMap<String, String>, _tank_state: &mut GlobalTanksGameState, _writers: &mut HashMap<ClientId, TcpStream>, _message: WebSocketMessage) -> StreamState {
//         StreamState::Drop
//     }
//     fn on_socket_close(&mut self, _database: &mut HashMap<String, String>, _tank_state: &mut GlobalTanksGameState, _writers: &mut HashMap<ClientId, TcpStream>) { }
// }