use crate::apps::{GlobalState, PeerId, Drop};
use web_socket::{WebSocketMessage, WebSocketWriter};
use std::collections::{HashMap};

use json::{Json, jsons, jsont};
use std::str::FromStr;
use std::option::NoneError;
use std::fmt;
use std::fmt::Debug;
use crate::apps::history::quiz_game::QuizGame;
use crate::apps::history::vocabulary_model::{TermId, VocabularyModel, Query};

mod quiz_game;
mod vocabulary_model;

pub struct HistoryGlobalState {
    users: Users,
    lobbies: HashMap<GameId, Lobby>,
    active_games: HashMap<GameId, Box<dyn GameSpecific>>,
    game_id_generator: GameIdGenerator,
    vocabulary_model: VocabularyModel,
}

impl GlobalState for HistoryGlobalState {
    fn new_peer(&mut self, id: PeerId, writer: WebSocketWriter) {
        self.users.insert(id, writer);
    }

    fn on_message_receive(&mut self, id: PeerId, message: WebSocketMessage) -> Result<(), Drop> {
        println!("{:?}: {}", id, message.get_text()?);
        let json_text = Json::from_str(message.get_text()?).ok()?;
        let json = json_text.get_object()?;

        match json.get("kind")?.get_string()? {
            "create" => {
                let username = json.get("username")?.get_string()?.to_string();
                self.users.add_username(id, username.clone());

                match Lobby::new(id, json.get("settings")?, &mut self.vocabulary_model) {
                    Ok(lobby) => {
                        let game_id = self.game_id_generator.next();
                        self.lobbies.insert(game_id, lobby);
                        self.users.add_game_id(id, game_id);

                        let _ = self.users.get_writer(id).write_string(&jsons!({
                            kind: "createSuccess",
                            hostName: username,
                            gameId: (game_id.stringify()),
                        }));
                    },
                    Err(e) => {
                        let _ = self.users.get_writer(id).write_string(&jsons!({
                            kind: "createFailed",
                            message: (e.to_string()),
                        }));
                    },
                }
            },
            "join" => {
                let username = json.get("username")?.get_string()?.to_string();
                self.users.add_username(id, username);

                if let Some(game_id) = GameId::from_json(json.get("id")?) {
                    match self.lobbies.get_mut(&game_id) {
                        Some(lobby) => {
                            lobby.join(id, game_id, &mut self.users);
                        },
                        None => {
                            let writer = self.users.get_writer(id);
                            let _ = writer.write_string(&jsons!({kind:"invalidGameId"}));
                        },
                    }
                } else {
                    let writer = self.users.get_writer(id);
                    let _ = writer.write_string(&jsons!({kind:"invalidGameId"}));
                }
            },
            "start" => {
                let game_id = self.users.get_game_id(id)?;

                let lobby = self.lobbies.remove(&game_id)?;
                lobby.announce_starting(&mut self.users);
                self.active_games.insert(game_id, lobby.into_game(&mut self.vocabulary_model, &mut self.users));
            }
            _ => {
                if let Some(game_id) = self.users.get_game_id(id) {
                    if let Some(active_game) = self.active_games.get_mut(&game_id) {
                        active_game.receive_message(id, json, &mut self.users, &mut self.vocabulary_model)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn on_drop(&mut self, id: PeerId) {
        // what game_id were they in?

        if let Some(game_id) = self.users.get_game_id(id) {
            if let Some(lobby) = self.lobbies.get_mut(&game_id) {
                let host_left = lobby.leave(id, &mut self.users);
                if host_left {
                    self.lobbies.remove(&game_id);
                }

            } else if let Some(game) = self.active_games.get_mut(&game_id) {
                let host_left = game.leave(id, &mut self.users, &mut self.vocabulary_model);
                if host_left {
                    self.active_games.remove(&game_id);
                }

            }
        }

        self.users.remove(id);
    }

    fn periodic(&mut self) {
        for game in self.active_games.values_mut() {
            game.periodic(&mut self.users, &mut self.vocabulary_model);
        }
    }
}

impl HistoryGlobalState {
    pub fn new() -> HistoryGlobalState {
        HistoryGlobalState {
            users: Users::new(),
            lobbies: HashMap::new(),
            active_games: HashMap::new(),
            game_id_generator: GameIdGenerator::new(),
            vocabulary_model: VocabularyModel::new().unwrap()
        }
    }
}


#[derive(Debug)]
struct Lobby {
    host: PeerId,
    peers: Vec<PeerId>,
    query: Query,
    game_kind: GameKind,
}

impl Lobby {
    fn new(host: PeerId, json: &Json, vocabulary: &mut VocabularyModel) -> Result<Lobby, LobbyCreateError> {
        let json_map = json.get_object()?;

        let start = get_chapter_thing(json_map.get("startSection")?.get_string()?)?;
        let end = get_chapter_thing(json_map.get("endSection")?.get_string()?)?;

        let query = Query::new(start, end, vocabulary).ok_or(LobbyCreateError::BlankRange)?;

        let game_kind = GameKind::from_str(json_map.get("gameKind")?.get_string()?)?;

        Ok(Lobby { host, peers: Vec::new(), query, game_kind })
    }

    fn into_game(self, vocabulary: &mut VocabularyModel, users: &mut Users) -> Box<dyn GameSpecific> {
        self.game_kind.into_game(self.host, self.peers, self.query, vocabulary, users)
    }

    fn join(&mut self, user: PeerId, game_id: GameId, users: &mut Users) {
        if !self.peers.contains(&user) {
            self.peers.push(user);
            users.add_game_id(user, game_id);
            let host_username = users.get_username(self.host).to_string();
            let _ = users.get_writer(user).write_string(&jsons!({
                kind: "joinSuccess",
                hostName: host_username,
            }));
            self.announce_members(users);
        }
    }

    fn leave(&mut self, id: PeerId, users: &mut Users) -> bool {
        let host_left = id == self.host;

        if host_left {
            self.send_to_all(users,jsons!({kind:"hostAbandoned"})); // what the fuck????
        } else if self.peers.contains(&id) {
            self.peers.remove(self.peers.iter().position(|&u| u == id).unwrap());
            self.announce_members(users);
        }

        host_left
    }

    fn announce_members(&self, users: &mut Users) {
        let string = jsons!({
            kind: "refreshLobby",
            users: (Json::Array(self.peers.iter().map(|&u| Json::String(users.get_username(u).to_string())).collect())),
        });

        self.send_to_all(users, string);
    }

    fn announce_starting(&self, users: &mut Users) {
        let string = jsons!({
            kind: "startingGame",
            host: (users.get_username(self.host)),
            users: (Json::Array(self.peers.iter().map(|&u| Json::String(users.get_username(u).to_string())).collect())),
        });

        self.send_to_all(users, string);
    }

    fn send_to_all(&self, users: &mut Users, string: String) {
        let _ = users.get_writer(self.host).write_string(&string.clone());

        for &user in self.peers.iter() {
            let _ = users.get_writer(user).write_string(&string);
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum GameKind {
    Quiz,
    Rocket,
    Clicker,
}

impl GameKind {
    fn from_str(string: &str) -> Result<GameKind, LobbyCreateError> {
        match string {
            "gameKindQuiz" => Ok(GameKind::Quiz),
            "gameKindRocket" => Ok(GameKind::Rocket),
            "gameKindClicker" => Ok(GameKind::Clicker),
            _ => Err(LobbyCreateError::Other),
        }
    }

    fn into_game(self, host: PeerId, peers: Vec<PeerId>, query: Query, vocabulary: &mut VocabularyModel, users: &mut Users) -> Box<dyn GameSpecific> {
        match self {
            GameKind::Quiz => Box::new(QuizGame::new(host, peers, query, vocabulary, users)),
            GameKind::Rocket => Box::new(QuizGame::new(host, peers, query, vocabulary, users)),
            GameKind::Clicker => Box::new(QuizGame::new(host, peers, query, vocabulary, users)),
        }
    }
}


#[derive(Copy, Clone )]
enum LobbyCreateError {
    UnableToParseChapters,
    BlankRange,
    Other,
}

impl From<NoneError> for LobbyCreateError {
    fn from(_: NoneError) -> LobbyCreateError {
        LobbyCreateError::Other
    }
}

impl From<std::io::Error> for LobbyCreateError {
    fn from(_: std::io::Error) -> LobbyCreateError {
        LobbyCreateError::Other
    }
}

impl fmt::Display for LobbyCreateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            LobbyCreateError::UnableToParseChapters => "Unable to interpret your chapter range",
            LobbyCreateError::BlankRange => "No terms were found in that range",
            LobbyCreateError::Other => "Error in creating your game",
        })
    }
}

fn get_chapter_thing(string: &str) -> Result<(u8, u8), LobbyCreateError> {
    let blang: Vec<&str> = string.split(".").collect();
    if blang.len() != 2 {
        return Err(LobbyCreateError::UnableToParseChapters);
    }

    let chapter = blang[0].parse().map_err(|_| LobbyCreateError::UnableToParseChapters)?;
    let section = blang[1].parse().map_err(|_| LobbyCreateError::UnableToParseChapters)?;

    Ok((chapter, section))
}


// struct Game {
//     host: PeerId,
//     users: Vec<PeerId>,
//     terms: Vec<TermId>,
//     game_specific: Box<dyn GameSpecific>,
// }
//
// impl Game {
//     fn from_lobby(lobby: Lobby) -> Game {
//         Game {
//             host: lobby.host,
//             users: lobby.users,
//             terms: lobby.terms,
//             game_specific: lobby.game_specific,
//         }
//     }
//
//     /// returns true if host left
//     fn leave(&mut self, id: PeerId, users: &mut Users) -> bool {
//         let host_left = id == self.host;
//
//         if host_left {
//             self.send_to_all(users,jsons!({kind:"hostAbandoned"})); // what the fuck????
//         } else if self.users.contains(&id) {
//             self.game_specific.leave(id, users);
//             self.users.remove(self.users.iter().position(|&u| u == id).unwrap());
//         }
//
//         host_left
//     }
//
//     fn receive_message(&mut self, id: PeerId, message: &HashMap<String, Json>, users: &mut Users) -> Result<(), Drop> {
//         self.game_specific.receive_message(id, message, users)
//     }
//
//     fn periodic(&mut self, users: &mut Users) {
//         self.game_specific.periodic(users);
//     }
// }

trait GameSpecific: Send+Debug {
    fn receive_message(&mut self, id: PeerId, message: &HashMap<String, Json>, users: &mut Users, vocabulary: &mut VocabularyModel) -> Result<(), Drop>;
    fn periodic(&mut self, users: &mut Users, vocabulary: &mut VocabularyModel);
    fn leave(&mut self, id: PeerId, users: &mut Users, vocabulary: &mut VocabularyModel) -> bool;
}

pub struct Users {
    map: HashMap<PeerId, (WebSocketWriter, Option<GameId>, Option<String>)>,
}

impl Users {
    fn new() -> Users {
        Users { map: HashMap::new() }
    }

    fn insert(&mut self, id: PeerId, writer: WebSocketWriter) {
        self.map.insert(id, (writer, None, None));
    }

    fn remove(&mut self, id: PeerId) {
        self.map.remove(&id);
    }

    fn add_game_id(&mut self, id: PeerId, game_id: GameId) {
        self.map.get_mut(&id).unwrap().1 = Some(game_id);
    }

    fn add_username(&mut self, id: PeerId, username: String) {
        self.map.get_mut(&id).unwrap().2 = Some(username);
    }

    fn get_writer(&mut self, id: PeerId) -> &mut WebSocketWriter {
        &mut self.map.get_mut(&id).unwrap().0
    }

    fn get_game_id(&self, id: PeerId) -> Option<GameId> {
        self.map[&id].1
    }

    fn get_username(&self, id: PeerId) -> &str {
        self.map[&id].2.as_ref().unwrap()
    }
}


#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct GameId(u32);

impl GameId {
    fn from_json(json: &Json) -> Option<GameId> {
        Some(GameId(json.get_number()? as u32))
    }

    fn stringify(&self) -> String {
        self.0.to_string()
    }
}

struct GameIdGenerator(u32);
impl GameIdGenerator {
    fn new() -> GameIdGenerator {
        GameIdGenerator(0)
    }
    fn next(&mut self) -> GameId {
        self.0 = self.0.checked_add(1).expect("game id generator overflow");
        GameId(self.0)
    }
}



