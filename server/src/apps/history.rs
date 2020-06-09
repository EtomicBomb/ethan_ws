
use crate::apps::{GlobalState, PeerId, Drop};
use web_socket::{WebSocketMessage, WebSocketWriter};
use std::collections::{HashMap};

use json::{Json, jsons, jsont};
use std::str::FromStr;
use std::io::{BufReader, BufRead};
use std::fs::File;
use crate::GOD_SET_PATH;
use std::option::NoneError;
use std::fmt;
use std::fmt::Debug;

pub struct HistoryGlobalState {
    users: Users,
    lobbies: HashMap<GameId, Lobby>,
    active_games: HashMap<GameId, Game>,
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

                match Lobby::new(id, json.get("settings")?, &self.vocabulary_model) {
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
                self.active_games.insert(game_id, Game::from_lobby(lobby));
            }
            _ => return Err(Drop),
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
                game.leave(id, &mut self.users);
            }
        }

        self.users.remove(id);
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
            vocabulary_model: VocabularyModel::new().unwrap()
        }
    }
}

#[derive(Debug)]
struct Lobby {
    host: PeerId,
    users: Vec<PeerId>,
    terms: Vec<TermId>,
    game_specific: Box<dyn GameSpecific>,
}

impl Lobby {
    fn new(host: PeerId, json: &Json, vocabulary: &VocabularyModel) -> Result<Lobby, LobbyCreateError> {
        let json_map = json.get_object()?;

        let start = get_chapter_thing(json_map.get("startSection")?.get_string()?)?;
        let end = get_chapter_thing(json_map.get("endSection")?.get_string()?)?;

        let terms = vocabulary.terms_in_range(start, end);

        if terms.is_empty() {
            return Err(LobbyCreateError::BlankRange);
        }

        let game_specific =
            match json_map.get("gameKind")?.get_string()? {
                "gameKindQuiz" => Box::new(QuizGame::new()),
                "gameKindRocket" => return Err(LobbyCreateError::Other), // TODO
                "gameKindClicker" => return Err(LobbyCreateError::Other), // TODO
                _ => return Err(LobbyCreateError::Other),
            };

        Ok(Lobby { host, users: Vec::new(), terms, game_specific })
    }

    fn join(&mut self, user: PeerId, game_id: GameId, users: &mut Users) {
        if !self.users.contains(&user) {
            self.users.push(user);
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
        } else if self.users.contains(&id) {
            self.users.remove(self.users.iter().position(|&u| u == id).unwrap());
            self.announce_members(users);
        }

        host_left
    }

    fn announce_members(&self, users: &mut Users) {
        let string = jsons!({
            kind: "refreshLobby",
            users: (Json::Array(self.users.iter().map(|&u| Json::String(users.get_username(u).to_string())).collect())),
        });

        self.send_to_all(users, string);
    }

    fn announce_starting(&self, users: &mut Users) {
        let string = jsons!({
            kind: "startingGame",
            host: (users.get_username(self.host)),
            users: (Json::Array(self.users.iter().map(|&u| Json::String(users.get_username(u).to_string())).collect())),
        });

        self.send_to_all(users, string);
    }

    fn send_to_all(&self, users: &mut Users, string: String) {
        let _ = users.get_writer(self.host).write_string(&string.clone());

        for &user in self.users.iter() {
            let _ = users.get_writer(user).write_string(&string);
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


struct Game {
    host: PeerId,
    users: Vec<PeerId>,
    terms: Vec<TermId>,
    game_specific: Box<dyn GameSpecific>,
}

impl Game {
    fn from_lobby(lobby: Lobby) -> Game {
        Game {
            host: lobby.host,
            users: lobby.users,
            terms: lobby.terms,
            game_specific: lobby.game_specific,
        }
    }

    fn leave(&mut self, id: PeerId, users: &mut Users) {
        todo!("game leave")
    }


}

trait GameSpecific: Send+Debug {

}

#[derive(Debug)]
struct QuizGame {

}

impl QuizGame {
    fn new() -> QuizGame {
        QuizGame { }
    }
}

impl GameSpecific for QuizGame {

}


struct Users {
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

    fn contains(&self, id: PeerId) -> bool {
        self.map.contains_key(&id)
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


struct VocabularyModel {
    terms: HashMap<TermId, Term>,
}


impl VocabularyModel {
    fn new() -> Option<VocabularyModel> {
        let file = BufReader::new(File::open(GOD_SET_PATH).unwrap());

        let terms = file.lines().zip(1..)
            .map(|(line, i)| Some((TermId(i), Term::from_line(line.ok()?)?)))
            .collect::<Option<HashMap<TermId, Term>>>()?;

        Some(VocabularyModel { terms })
    }

    fn terms_in_range(&self, start: (u8, u8), end: (u8, u8)) -> Vec<TermId> {
        self.terms.iter()
            .filter(|&(_, term)| {
                start <= (term.chapter, term.section) && (term.chapter, term.section) <= end
            })
            .map(|(&id, _)| id)
            .collect()
    }
}

#[derive(Clone)]
struct Term {
    chapter: u8,
    section: u8,
    year_start: u16,
    year_end: u16,
    social: bool,
    political: bool,
    economic: bool,
    term: String,
    definition: String,
}

impl Term {
    fn from_line(line: String) -> Option<Term> {
        let mut split = line.trim_end().split("\t");
        Some(Term {
            chapter: split.next()?.parse().ok()?,
            section: split.next()?.parse().ok()?,
            year_start: split.next()?.parse().ok()?,
            year_end: split.next()?.parse().ok()?,
            social: split.next()?.parse().ok()?,
            political: split.next()?.parse().ok()?,
            economic: split.next()?.parse().ok()?,
            term: split.next()?.to_string(),
            definition: split.next()?.to_string(),
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct TermId(u32);


