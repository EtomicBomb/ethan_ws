use std::collections::HashMap;
use std::net::TcpStream;
use std::time::{UNIX_EPOCH, SystemTime};
use rand::{thread_rng, Rng, random};

use crate::log;
use crate::websocket_apps::{WebSocketClientState, write_string_to};
use crate::server_state::{StreamState, WebsocketMessage, ClientId};
use json::Json;
use std::str::FromStr;
use crate::god_set::GodSet;
use rand::seq::SliceRandom;

const MAP_WIDTH: f64 = 500.0;
const MAP_HEIGHT: f64 = 500.0;
const TAU: f64 = 2.0 * std::f64::consts::PI;
const NUM_STARS: usize = 30;
const PLAYER_VELOCITY: f64 = 0.04;
const PLAYER_RADIUS: f64 = 10.0;
const LASER_DURATION_MILLIS: f64 = 300.0;

pub struct TanksClientState {
    id: ClientId,
}

impl TanksClientState {
    pub fn new(id: ClientId, _database: &mut HashMap<String, String>, tank_state: &mut GlobalTanksGameState, writers: &mut HashMap<ClientId, TcpStream>) -> TanksClientState {
        // create a new entry in our database

        tank_state.new_player(id);

        for other in writers.keys().cloned().filter(|&k| tank_state.has_player(k)).collect::<Vec<ClientId>>() {
            write_string_to(other, tank_state.game_state_message_to(other).to_string(), writers);
        }

        TanksClientState { id }
    }
}

impl WebSocketClientState for TanksClientState {
    fn on_receive_message(&mut self, _database: &mut HashMap<String, String>, tank_state: &mut GlobalTanksGameState, writers: &mut HashMap<ClientId, TcpStream>, message: WebsocketMessage) -> StreamState {
        match do_stuff(self.id, writers, tank_state, message) {
            Some(_) => StreamState::Keep,
            None => StreamState::Drop,
        }
    }
    fn on_socket_close(&mut self, _database: &mut HashMap<String, String>, tank_state: &mut GlobalTanksGameState, writers: &mut HashMap<ClientId, TcpStream>) {
        tank_state.remove_player(self.id);

        for other in writers.keys().cloned().filter(|&k| tank_state.has_player(k)).collect::<Vec<ClientId>>() {
            write_string_to(other, tank_state.game_state_message_to(other).to_string(), writers);
        }
    }
}

fn do_stuff(id: ClientId, writers: &mut HashMap<ClientId, TcpStream>, tank_state: &mut GlobalTanksGameState, message: WebsocketMessage) -> Option<()> {
    let a = Json::from_str(message.get_text()?).ok()?;
    let map = a.get_object()?;

    match map.get("kind")?.get_string()? {
        "updateFacing" => {
            tank_state.update_facing(id, map.get("newFacing")?.get_number()?);

            for other in writers.keys().cloned().filter(|&k| tank_state.has_player(k)).collect::<Vec<ClientId>>() {
                write_string_to(other, tank_state.game_state_message_to(other).to_string(), writers);
            }

            Some(())
        },
        "guess" => {
            if tank_state.guessed_correctly(id, map.get("guessIsLeft")?.get_bool()?) {
                for other in writers.keys().cloned().filter(|&k| tank_state.has_player(k)).collect::<Vec<ClientId>>() {
                    write_string_to(other, tank_state.game_state_message_to(other).to_string(), writers);
                }

                Some(())
            } else {
                None
            }
        },
        "fire" => {
            // boom bam bop
            tank_state.shoot_laser(id);

            for other in writers.keys().cloned().filter(|&k| tank_state.has_player(k)).collect::<Vec<ClientId>>() {
                write_string_to(other, tank_state.game_state_message_to(other).to_string(), writers);
            }

            Some(())
        },
        _ => None,
    }
}

fn intersect_circle(circle_x: f64, circle_y: f64, radius: f64, ray_x: f64, ray_y: f64, ray_angle: f64) -> bool {
    let b = circle_y * ray_angle.sin() - circle_x * ray_angle.cos() + ray_x * ray_angle.cos() - ray_y * ray_angle.sin();
    let discriminant = b*b - ray_x * ray_x - ray_y * ray_y + 2.0* circle_x * ray_x + 2.0* circle_y * ray_y - circle_y * circle_y - circle_x * circle_x + radius * radius;

    discriminant >= 0.0 && -b+discriminant.sqrt() >= 0.0
}


pub struct GlobalTanksGameState {
    last_updated: u64, // our last updated time
    stars_json: Json,
    players: HashMap<ClientId, PlayerInfo>,
    questions: Vec<(String, String)>,
    lasers: Vec<Laser>, // x, y, facing
}

impl GlobalTanksGameState {
    pub fn new() -> GlobalTanksGameState {
        let stars: Vec<_> =  (0..NUM_STARS)
            .map(|_| (thread_rng().gen_range(0.0, MAP_WIDTH as f64), thread_rng().gen_range(0.0, MAP_HEIGHT as f64)))
            .collect();

        let stars_json = Json::Array(stars.iter()
            .map(|&(x, y)| {
                let mut map = HashMap::new();
                map.insert("x".into(), Json::Number(x));
                map.insert("y".into(), Json::Number(y));
                Json::Object(map)
            })
            .collect());

        let questions = match GodSet::cool_vector() {
            Some(questions) if !questions.is_empty() => questions,
            _ => {
                log!("couldn't read questions file");
                panic!("couldn't read questions file");
            },
        };

        GlobalTanksGameState {
            last_updated: unix_time_millis(),
            stars_json,
            questions,
            players: HashMap::new(),
            lasers: Vec::new(),
        }
    }

    fn has_player(&self, id: ClientId) -> bool {
        self.players.contains_key(&id)
    }

    fn new_player(&mut self, id: ClientId) {
        self.players.insert(id, PlayerInfo::from_random(&self.questions));
    }

    fn remove_player(&mut self, id: ClientId) {
        // bye bye!
        self.players.remove(&id);
    }

    fn guessed_correctly(&mut self, id: ClientId, guess_is_left: bool) -> bool {
        let player = self.players.get_mut(&id).unwrap();

        let was_correct = player.question.guess(guess_is_left);
        player.question = Question::new(&self.questions);

        if was_correct { player.shield += 1 }
        was_correct
    }

    fn shoot_laser(&mut self, id: ClientId) {
        self.update();

        let a = self.players.get_mut(&id).unwrap();
        if a.shield == 0 { return }
        a.shield -= 1;

        let ray_x = a.x;
        let ray_y = a.y;
        let facing = a.facing;

        for (_, other) in self.players.iter_mut().filter(|&(&other_id, _)| other_id != id) {
            if intersect_circle(other.x, other.y, PLAYER_RADIUS, ray_x, ray_y, facing) {
                 if other.shield > 0 { other.shield -= 1 }
            }
        }

        self.lasers.push(Laser { x: ray_x, y: ray_y, facing, expire: self.last_updated as f64 + LASER_DURATION_MILLIS })
    }

    fn update_facing(&mut self, id: ClientId, new_facing: f64) {
        // where are they now?
        self.update();
        self.players.get_mut(&id).unwrap().facing = new_facing;
    }

    fn game_state_message_to(&self, receiver: ClientId) -> Json {
        // contains:
        //      a number called `time` that stores the time that x and y were last valid
        //      an array called `stars` each element with an x and y
        //      an array called 'players' each with properties x, y, vx, vy
        //      an array called `lasers` each with an x, y, facing, and expire
        //      an object called 'us' with the same x, y, vx, vy

        let lasers = Json::Array(self.lasers.iter()
            .map(|laser| {
                let mut map = HashMap::new();
                map.insert("x".into(), Json::Number(laser.x));
                map.insert("y".into(), Json::Number(laser.y));
                map.insert("facing".into(), Json::Number(laser.facing));
                map.insert("expire".into(), Json::Number(laser.expire));
                Json::Object(map)
            })
            .collect());



        let question = &self.players[&receiver].question;
        let mut question_map = HashMap::new();
        question_map.insert("definition".into(), Json::String(question.definition.clone()));
        question_map.insert("left".into(), Json::String(question.left.clone()));
        question_map.insert("right".into(), Json::String(question.right.clone()));

        let mut game_state_map = HashMap::new();
        game_state_map.insert("time".into(), Json::Number(self.last_updated as f64));
        game_state_map.insert("stars".into(), self.stars_json.clone());
        game_state_map.insert("players".into(), Json::Array(self.players.values().map(PlayerInfo::jsonify).collect::<Vec<Json>>()));
        game_state_map.insert("us".into(), self.players[&receiver].jsonify());
        game_state_map.insert("lasers".into(), lasers);

        Json::Object([
            ("kind".into(), Json::String("updateGameState".into())),
            ("gameState".into(), Json::Object(game_state_map)),
            ("question".into(), Json::Object(question_map)),
        ].iter().cloned().collect())
    }

    fn update(&mut self) {
        let now = unix_time_millis();
        let elapsed = (now - self.last_updated) as f64;
        self.last_updated = now;

        self.lasers.retain(|l| l.expire >= now as f64);

        for player in self.players.values_mut() {
            let (vx, vy) = player.velocity();

            player.x = wrap(player.x + vx*elapsed, MAP_WIDTH);
            player.y = wrap(player.y - vy*elapsed, MAP_HEIGHT);
        }
    }
}


struct Laser {
    x: f64,
    y: f64,
    facing: f64,
    expire: f64,
}

#[derive(Clone)]
struct PlayerInfo {
    x: f64,
    y: f64,
    facing: f64,
    color: String,
    shield: usize,
    question: Question,
}



impl PlayerInfo {
    fn from_random(questions: &[(String, String)]) -> PlayerInfo {
        PlayerInfo {
            x: thread_rng().gen_range(0.0, MAP_WIDTH as f64),
            y: thread_rng().gen_range(0.0, MAP_HEIGHT as f64),
            facing: thread_rng().gen_range(0.0, TAU),
            color: format!("rgb({},{},{})", random::<u8>(), random::<u8>(), random::<u8>()),
            shield: 0,
            question: Question::new(questions),
        }
    }

    fn velocity(&self) -> (f64, f64) {
        let vx = PLAYER_VELOCITY * self.facing.cos();
        let vy = PLAYER_VELOCITY * self.facing.sin();
        (vx, vy)
    }

    fn jsonify(&self) -> Json {
        let (vx, vy) = self.velocity();

        let mut map = HashMap::new();
        map.insert("x".into(), Json::Number(self.x));
        map.insert("y".into(), Json::Number(self.y));
        map.insert("vx".into(), Json::Number(vx));
        map.insert("vy".into(), Json::Number(vy));
        map.insert("color".into(), Json::String(self.color.clone()));
        map.insert("shield".into(), Json::Number(self.shield as f64));
        Json::Object(map)
    }
}

#[derive(Clone)]
struct Question {
    definition: String,
    left: String,
    right: String,
    left_correct: bool,
}

impl Question {
    fn new(questions: &[(String, String)]) -> Question {
        let (correct_term, definition) = questions.choose(&mut thread_rng()).unwrap().clone();
        let (wrong_term, _) = questions.choose(&mut thread_rng()).unwrap().clone();

        if random::<bool>() {
            Question {
                definition,
                left: correct_term,
                right: wrong_term,
                left_correct: true,
            }
        } else {
            Question {
                definition,
                left: wrong_term,
                right: correct_term,
                left_correct: false,
            }
        }
    }

    fn guess(&self, guess_is_left: bool) -> bool {
        (guess_is_left && self.left_correct) || (!guess_is_left && !self.left_correct)
    }
}


fn wrap(mut n: f64, range: f64) -> f64 {
    n %= range;
    if n < 0.0 { n += range }
    n
}

fn unix_time_millis() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let millis = duration.as_millis();
            if millis < u64::MAX as u128 {
                millis as u64
            } else {
                log!("clock reading too big");
                panic!("clock reading too big");
            }
        },
        Err(_) => {
            log!("clock read failed");
            panic!("clock read failed");
        }
    }
}