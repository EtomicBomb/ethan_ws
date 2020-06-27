use server::{PeerId, Disconnect, GlobalState};
use std::collections::HashMap;
use web_socket::{WebSocketMessage, WebSocketWriter};
use json::{jsont, Json};
use rand::{random, thread_rng, Rng};
use std::str::FromStr;

const MAP_WIDTH: f64 = 9.0;
const MAP_HEIGHT: f64 = 9.0;

pub struct ArenaGlobalState {
    players: HashMap<PeerId, Player>,
}

impl GlobalState for ArenaGlobalState {
    fn new_peer(&mut self, id: PeerId, writer: WebSocketWriter) {
        self.players.insert(id, Player::new(writer));
        // self.announce();
    }

    fn on_message_receive(&mut self, id: PeerId, message: WebSocketMessage) -> Result<(), Disconnect> {
        match self.message_receive_handler(id, message) {
            Some(()) => Ok(()),
            None => Err(Disconnect),
        }
    }

    fn on_disconnect(&mut self, id: PeerId) {
        self.players.remove(&id);
        // self.announce();
    }

    fn periodic(&mut self) {
        self.announce();
    }
}

impl ArenaGlobalState {
    pub fn new() -> ArenaGlobalState {
        ArenaGlobalState { players: HashMap::new() }
    }

    fn message_receive_handler(&mut self, id: PeerId, message: WebSocketMessage) -> Option<()> {
        let json = Json::from_str(message.get_text()?).ok()?;

        let map = json.get_object()?;

        let mut player = self.players.get_mut(&id)?;
        player.x = map.get("x")?.get_number()?;
        player.y = map.get("y")?.get_number()?;

        // self.announce();
        Some(())
    }

    fn announce(&mut self) {
        for id in self.players.keys().cloned().collect::<Vec<PeerId>>() {
            let array = self.players.iter()
                .filter(|&(&i, _)| i != id)
                .map(|(_, player)| player.as_json())
                .collect();

            let json_string = Json::Array(array).to_string();

            let _ = self.players.get_mut(&id).unwrap().writer.write_string(&json_string);
        }


        // for info in self.players.values_mut() {
        //     info.writer.write_text_or_drop(json_string.clone());
        // }
    }
}

struct Player {
    writer: WebSocketWriter,
    color: Json,
    x: f64,
    y: f64,
}

impl Player {
    fn new(writer: WebSocketWriter) -> Player {
        Player {
            writer,
            color: jsont!({r: (random::<u8>()), g:(random::<u8>()), b:(random::<u8>())}),
            x: thread_rng().gen_range(0.0, MAP_WIDTH),
            y: thread_rng().gen_range(0.0, MAP_HEIGHT),
        }
    }

    fn as_json(&self) -> Json {
        jsont!({
            x: (self.x),
            y: (self.y),
            color: (self.color.clone()),
        })
    }
}
