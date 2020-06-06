use crate::apps::{GlobalState, PeerId, StreamState, TcpStreamWriter};
use web_socket::WebSocketMessage;
use std::collections::{HashMap};

use json::{Json, jsons, jsont};
use std::str::FromStr;

// API DOCUMENTATION

// CLIENT MESSAGES
// create: username, settings
// join: username,


pub struct HistoryGlobalState {
    users: Users,
    lobbies: HashMap<GameId, Lobby>,
    active_games: HashMap<GameId, Box<dyn Game>>,
    game_id_generator: GameIdGenerator,
}


impl GlobalState for HistoryGlobalState {
    fn new_peer(&mut self, id: PeerId, writer: TcpStreamWriter) {
        self.users.insert(id, writer);
    }

    fn on_message_receive(&mut self, id: PeerId, message: WebSocketMessage) -> StreamState {
        match self.message_receive_handler(id, message) {
            Some(()) => StreamState::Keep,
            None => StreamState::Drop,
        }
    }

    fn on_drop(&mut self, id: PeerId) {
        // let was_in_game = ;

        todo!("implement");
        // if let Some(game_id) = self.users.get(&id).and_then(|user| user.in_lobby) {
        //     if let Some(lobby) = self.lobbies.get_mut(&game_id)  {
        //         lobby.leave(id, &mut self.users);
        //         if lobby.host == id { self.lobbies.remove(&game_id); }
        //     }
        //
        //     if let Some(game) = self.active_games.get_mut(&game_id) {
        //         game.on_drop(id);
        //     }
        // }
    }

    fn periodic(&mut self) { }
}

impl HistoryGlobalState {
    pub fn new() -> HistoryGlobalState {
        HistoryGlobalState {
            users: Users::new(),
            lobbies: HashMap::new(),
            active_games: HashMap::new(),
            game_id_generator: GameIdGenerator::new(),
        }
    }

    pub fn message_receive_handler(&mut self, from: PeerId, message: WebSocketMessage) -> Option<()> {
        println!("{:?}: {}", from, message.get_text()?);
        let json_text = Json::from_str(message.get_text()?).ok()?;
        let json = json_text.get_object()?;

        match json.get("kind")?.get_string()? {
            "create" => {
                let username = json.get("username")?.get_string()?.to_string();
                self.users.add_username(from, username);

                let id = self.game_id_generator.next();

                self.lobbies.insert(id, Lobby::new(from, json.get("settings")?.get_string()?.to_string()));

                self.users.add_game_id(from, id);

                Some(())
            },
            "join" => {
                let username = json.get("username")?.get_string()?.to_string();
                self.users.add_username(from, username);

                let game_id = GameId::from_f64(json.get("id")?.get_number()?);
                match self.lobbies.get_mut(&game_id) {
                    Some(lobby) => Some(lobby.try_join(from, game_id, &mut self.users)),
                    None => {
                        let writer = self.users.get_writer(from);
                        writer.write_text_or_none(jsons!({kind:"invalidGameId"}))
                    },
                }
            },
            "start" => {
                todo!("fix");
                // let game_id = self.users.get(&from)?.in_lobby?;
                // let mut lobby = self.lobbies.remove(&game_id)?;
                //
                // self.active_games.insert(game_id, lobby.into_game(&mut self.users).ok()?);

                Some(())
            }
            _ => None,
        }
    }
}


struct Lobby {
    host: PeerId,
    users: Vec<PeerId>,
    game_kind_string: String,
}

impl Lobby {
    fn new(host: PeerId, game_kind_string: String) -> Lobby {
        Lobby { host, users: Vec::new(), game_kind_string }
    }

    fn into_game(self, users: &mut Users) -> Result<Box<dyn Game>, ()> {
        Ok(match self.game_kind_string.as_str() {
            "gameKindQuiz" => Box::new(QuizGame::from_lobby(self, users)),
            _ => return Err(()),
        })
    }

    fn try_join(&mut self, user: PeerId, game_id: GameId, users: &mut Users) {
        if self.users.contains(&user) {
            users.get_writer(user).write_text_or_drop(jsons!({kind:"gameJoinError"}));
        } else {
            self.users.push(user);
            users.add_game_id(user, game_id);
            self.announce_members(users);
        }
    }

    fn leave(&mut self, id: PeerId, users: &mut Users) {
        if id == self.host {
            // what the fuck??
            self.send_to_all(users,jsons!({kind:"hostAbandoned"}))

        } else if self.users.contains(&id) {
            self.users.remove(self.users.iter().position(|&u| u == id).unwrap());
            self.announce_members(users);
        }
    }

    fn announce_members(&self, users: &mut Users) {

        let string = jsons!({
            kind: "refreshLobby",
            host: (users.get_username(self.host)),
            users: (Json::Array(self.users.iter().map(|&u| Json::String(users.get_username(u).to_string())).collect())),
        });

        self.send_to_all(users, string);
    }

    fn send_to_all(&self, users: &mut Users, string: String) {
        users.get_writer(self.host).write_text_or_drop(string.clone());

        for &user in self.users.iter() {
            users.get_writer(user).write_text_or_drop(string.clone());
        }
    }
}


trait Game: Send {
    fn on_drop(&mut self, id: PeerId);
}

struct QuizGame {
    host: PeerId,
    users: Vec<PeerId>,
}

impl QuizGame {


    fn from_lobby(mut lobby: Lobby, users: &mut Users) -> QuizGame {
        lobby.send_to_all(users, jsons!({
            kind: "upgrade",
            gameKind: "gameKindQuiz",
        }));

        QuizGame { host: lobby.host, users: lobby.users }
    }
}

impl Game for QuizGame {
    fn on_drop(&mut self, id: PeerId) {

    }
}

struct Users {
    map: HashMap<PeerId, (TcpStreamWriter, Option<GameId>, Option<String>)>,
}

impl Users {
    fn new() -> Users {
        Users { map: HashMap::new() }
    }

    fn insert(&mut self, id: PeerId, writer: TcpStreamWriter) {
        self.map.insert(id, (writer, None, None));
    }

    fn remove(&mut self, id: PeerId) {
        self.map.remove(&id);
    }

    fn contains(&self, id: PeerId) -> bool {
        self.map.contains_key(&id)
    }

    fn add_game_id(&mut self, id: PeerId, game_id: GameId) {
        self.map.get_mut(&id).unwrap().1 = Some(game_id);
    }

    fn add_username(&mut self, id: PeerId, username: String) {
        self.map.get_mut(&id).unwrap().2 = Some(username);
    }

    fn get_writer(&mut self, id: PeerId) -> &mut TcpStreamWriter {
        &mut self.map.get_mut(&id).unwrap().0
    }

    fn get_username(&self, id: PeerId) -> &str {
        self.map[&id].2.as_ref().unwrap()
    }
}


// struct User {
//     writer: TcpStreamWriter,
//     in_lobby: Option<GameId>,
//     username: Option<String>,
// }








#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct GameId(u32);

impl GameId {
    fn from_f64(n: f64) -> GameId {
        GameId(n as u32)
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

