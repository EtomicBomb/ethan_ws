use rand::{Rng, thread_rng};
use std::collections::HashSet;
use crate::apps::{GlobalState, PeerId, Drop};
use web_socket::{WebSocketWriter, WebSocketMessage};
use std::fmt::Debug;
use std::collections::HashMap;
use json::{Json, jsons, jsont};
use std::io::{BufReader};
use crate::apps::pusoy::play::{Play, PlayKind};
use crate::apps::pusoy::card::Card;
use std::fs::{File};
use crate::apps::pusoy::pusoy_game::PusoyGame;
use crate::PUSOY_PASSING_MODEL_PATH;
use std::iter::once;
use lazy_static::lazy_static;
use crate::WORD_LIST_PATH;
use std::io::BufRead;

mod bot;
mod card;
mod game;
mod util;
mod play;
mod pusoy_game;

lazy_static! { 
    static ref WORD_LIST: Vec<String> = {
        let mut reader = BufReader::new(File::open(WORD_LIST_PATH).unwrap());

        let words: Vec<String> = reader.lines()
            .map(|line| line.unwrap().trim().to_string())
            .collect();


        assert!(words.is_sorted());
        assert!(words.iter().all(|w| w.chars().all(|c| matches!(c, 'a'..='z' | '-'))));

        words
    };
}

pub struct PusoyGlobalState {
    unregistered_users: HashMap<PeerId, WebSocketWriter>,

    game_id_generator: GameIdGenerator,

    in_game: HashMap<PeerId, GameId>,
    lobbies: HashMap<GameId, Lobby>,
    active_games: HashMap<GameId, PusoyGame>,
}

impl PusoyGlobalState {
    pub fn new() -> PusoyGlobalState {
        PusoyGlobalState {
            unregistered_users: HashMap::new(),
            game_id_generator: GameIdGenerator::new(),
            in_game: HashMap::new(),
            lobbies: HashMap::new(),
            active_games: HashMap::new(),
        }
    }

    pub fn lobby_from_id(&mut self, json: &Json) -> Option<GameId> {
        let game_id = GameId::from_json(json)?;

        if self.lobbies.contains_key(&game_id) {
            Some(game_id)
        } else {
            None
        }
    }
}

impl GlobalState for PusoyGlobalState {
    fn new_peer(&mut self, id: PeerId, writer: WebSocketWriter) {
        self.unregistered_users.insert(id, writer);
    }

    fn on_message_receive(&mut self, id: PeerId, message: WebSocketMessage) -> Result<(), Drop> {
        println!("{:?}: {}", id, message.get_text()?);
        let json_text: Json = message.get_text()?.parse().ok()?;
        let json = json_text.get_object()?;

        match json.get("kind")?.get_string()? {
            "create" => {
                let username = json.get("username")?.get_string()?.to_string();

                let writer = self.unregistered_users.remove(&id)?;

                let game_id = self.game_id_generator.next();

                let lobby = Lobby::new(id, writer, username, game_id);

                self.lobbies.insert(game_id, lobby);
                self.in_game.insert(id, game_id);
            },
            "join" => {
                let username = json.get("username")?.get_string()?.to_string();

                match self.lobby_from_id(json.get("gameId")?) {
                    Some(game_id) => {
                        self.in_game.insert(id, game_id);
                        let lobby = self.lobbies.get_mut(&game_id).unwrap();
                        lobby.join(id, self.unregistered_users.remove(&id)?, username, game_id)
                    },
                    None => { let _ = self.unregistered_users.get_mut(&id)?.write_string(&jsons!({kind:"invalidGameId"})); }
                }
            },
            "begin" => {
                let game_id = *self.in_game.get(&id)?;

                let mut lobby = self.lobbies.remove(&game_id)?;
                lobby.announce_beginning();
                self.active_games.insert(game_id, PusoyGame::new(lobby.host, lobby.players));
            }
            _ => {
                let game_id = self.in_game.get(&id)?;

                if let Some(active_game) = self.active_games.get_mut(game_id) {
                    active_game.receive_message(id, json)?;
                }
            }
        }

        Ok(())

    }

    fn on_drop(&mut self, id: PeerId) {
        if let Some(game_id) = self.in_game.get(&id) {
            if let Some(lobby) = self.lobbies.get_mut(&game_id) {
                let host_left = lobby.leave(id);
                if host_left {
                    self.lobbies.remove(&game_id);
                }

            } else if let Some(game) = self.active_games.get_mut(&game_id) {
                let host_left = game.leave(id);
                if host_left {
                    self.active_games.remove(&game_id);
                }
            }
        }

        self.in_game.remove(&id);
        self.unregistered_users.remove(&id);
    }

    fn periodic(&mut self) {
        for game in self.active_games.values_mut() {
            game.periodic();
        }
    }
}




#[derive(Debug)]
struct Lobby {
    host: Member,
    players: Vec<Member>,
}

impl Lobby {
    fn new(host_id: PeerId, writer: WebSocketWriter, username: String, game_id: GameId) -> Lobby {
        let host = Member::new(host_id, writer, username.clone());
        let mut ret = Lobby { host, players: Vec::new() };

        ret.host.write_ignore(&jsons!({
            kind: "createSuccess",
            host: username,
            gameId: (game_id.stringify()),
        }));

        ret
    }

    fn contains_player(&self, id: PeerId) -> bool {
        self.players.iter().any(|p| p.id == id)
    }

    fn join(&mut self, user: PeerId, writer: WebSocketWriter, username: String, game_id: GameId) {
        if !self.contains_player(user) && self.host.id != user {
            let mut player = Member::new(user, writer, username);

            let host_username = self.host.username.clone();

            player.write_ignore(&jsons!({
                kind: "joinSuccess",
                host: host_username,
                gameId: (game_id.stringify()),
            }));

            self.players.push(player);

            self.announce_players();
        }
    }

    fn leave(&mut self, id: PeerId) -> bool {
        let host_left = id == self.host.id;

        if host_left {
            self.send_to_all(&jsons!({kind:"hostAbandoned"})); // what the fuck????

        } else if let Some(i) = self.players.iter().position(|u| u.id == id) {
            self.players.remove(i);
            self.announce_players();
        }

        host_left
    }

    fn announce_players(&mut self) {
        let string = jsons!({
            kind: "refreshLobby",
            players: (Json::Array(self.players.iter().map(|u| Json::String(u.username.clone())).collect())),
        });

        self.send_to_all(&string);
    }

    fn announce_beginning(&mut self) {
        let string = jsons!({
            kind: "beginGame",
            host: (self.host.username.clone()),
            users: (Json::Array(self.players.iter().map(|p| Json::String(p.username.clone())).collect())),
        });

        self.send_to_all(&string);
    }

    fn send_to_all(&mut self, string: &str) {
        for player in self.players.iter_mut().chain(once(&mut self.host)) {
            player.write_ignore(string);
        }
    }
}

#[derive(Debug)]
struct Member {
    id: PeerId,
    writer: WebSocketWriter,
    username: String,
}

impl Member {
    fn new(id: PeerId, writer: WebSocketWriter, username: String) -> Member {
        Member { id, writer, username }
    }

    fn write_ignore(&mut self, string: &str) {
        let _ = self.writer.write_string(string);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct GameId {
    word_index: usize,
}

impl GameId {
    fn from_json(json: &Json) -> Option<GameId> {
        let word: &str = json.get_string()?;

        WORD_LIST.binary_search_by(|probe| probe.as_str().cmp(word)).ok()
            .map(|word_index| GameId { word_index })
    }

    fn stringify(&self) -> String {
        WORD_LIST[self.word_index].clone()
    }
}

struct GameIdGenerator {
    unavailable: HashSet<usize>,
    current_len: usize,
}

impl GameIdGenerator {
    fn new() -> GameIdGenerator {
        GameIdGenerator {
            unavailable: HashSet::new(),
            current_len: 5,
        }
    }

    fn next(&mut self) -> GameId {
        let len = WORD_LIST.len(); 
        assert!(self.unavailable.len() < len, "ran out of game id's");

        loop {
            let word_index = thread_rng().gen_range(0, len);
            let is_new = self.unavailable.insert(word_index);
            if is_new { break GameId { word_index } }
        }
    }
}

pub fn get_expected_pass_count(play1: &Play, play2: &Play) -> f64 {
    lazy_static! {
        static ref PASSING_MODEL: HashMap<((PlayKind, Card), (PlayKind, Card)), f64> = {
            let reader = BufReader::new(File::open(PUSOY_PASSING_MODEL_PATH).unwrap());
            bincode::deserialize_from(reader).unwrap()
        };
    }

    match PASSING_MODEL.get(&(classify(play1), classify(play2))) {
        Some(&count) => count,
        None => 3.0, // guess
    }
}

fn classify(play: &Play) -> (PlayKind, Card) {
    (play.kind(), play.ranking_card().unwrap())
}


// fn main() {
//     let players: Vec<Box<dyn Player>> = vec![
//         Box::new(HumanPlayer),
//         Box::new(MachinePlayer),
//         Box::new(MachinePlayer),
//         Box::new(MachinePlayer),
//     ];
//
//     let mut deck = entire_deck();
//     deck.shuffle(&mut thread_rng());
//
//     let mut game = GameState::new(4, deck);
//
//     while game.winning_player().is_none() {
//         let interface = SafeGameInterface::from_game(&game);
//
//         let play = players[game.current_player].choose_play(interface);
//
//         // lets report
//         if play.is_pass() {
//             println!("player {} passed", game.current_player);
//         } else {
//             println!("player {} played {:?}", game.current_player, play.cards());
//         }
//
//         game.play(play);
//     }
// }
