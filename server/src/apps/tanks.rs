use std::collections::HashMap;
use std::time::{UNIX_EPOCH, SystemTime};
use rand::{thread_rng, Rng, random};

use crate::{GOD_SET_PATH};
use crate::apps::{GlobalState, PeerId, Drop};
use json::Json;
use std::str::FromStr;
use rand::seq::SliceRandom;
use web_socket::{WebSocketMessage, WebSocketWriter};
use std::fs::File;
use std::io::{BufReader, BufRead};

const MAP_WIDTH: f64 = 500.0;
const MAP_HEIGHT: f64 = 500.0;
const TAU: f64 = 2.0 * std::f64::consts::PI;
const NUM_STARS: usize = 30;
const PLAYER_VELOCITY: f64 = 0.04;
const PLAYER_RADIUS: f64 = 10.0;
const LASER_DURATION_MILLIS: f64 = 300.0;

pub struct TanksGlobalState {
    last_updated: u64, // our last updated time
    stars_json: Json,
    players: HashMap<PeerId, PlayerInfo>,
    questions: Vec<(String, String)>,
    lasers: Vec<Laser>, // x, y, facing
}

impl GlobalState for TanksGlobalState {
    fn new_peer(&mut self, id: PeerId, tcp_stream: WebSocketWriter) {
        self.new_player(id, tcp_stream);
        self.announce();
    }
    
    fn on_message_receive(&mut self, id: PeerId, message: WebSocketMessage) -> Result<(), Drop> {
        if !self.has_player(id) { return Err(Drop) } // that means we're dead!

        let a = Json::from_str(message.get_text()?).ok()?;
        let map = a.get_object()?;

        match map.get("kind")?.get_string()? {
            "updateFacing" => {
                self.update_facing(id, map.get("newFacing")?.get_number()?);
                self.announce();
                Ok(())
            },
            "guess" => {
                if self.guessed_correctly(id, map.get("guessIsLeft")?.get_bool()?) {
                    self.announce();
                    Ok(())
                } else {
                    Err(Drop)
                }
            },
            "fire" => {
                // boom bam bop
                self.shoot_laser(id);
                self.announce();
                Ok(())
            },
            _ => Err(Drop),
        }
    }

    fn on_drop(&mut self, id: PeerId) {
        self.remove_player(id);
    }

    fn periodic(&mut self) { }
}

impl TanksGlobalState {
    pub fn new() -> TanksGlobalState {
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

        let questions = match cool_vector() {
            Some(questions) if !questions.is_empty() => questions,
            _ => panic!("couldn't read questions file"),
        };

        TanksGlobalState {
            last_updated: unix_time_millis(),
            stars_json,
            questions,
            players: HashMap::new(),
            lasers: Vec::new(),
        }
    }

    fn announce(&mut self) {
        for id in self.players.keys().cloned().collect::<Vec<PeerId>>() {
            let message = self.game_state_message_to(id).to_string();
            let writer = &mut self.players.get_mut(&id).unwrap().tcp_stream;
            let _ = writer.write_string(&message);
        }
    }

    fn kill(&mut self, id: PeerId) {
        let tcp_stream = &mut self.players.get_mut(&id).unwrap().tcp_stream;
        let mut map = HashMap::new();
        map.insert("kind".into(), Json::String("kill".into()));
        let _ = tcp_stream.write_string(&Json::Object(map).to_string());
        self.remove_player(id);
    }

    fn new_player(&mut self, id: PeerId, tcp_stream: WebSocketWriter) {
        self.players.insert(id, PlayerInfo::from_random(&self.questions, tcp_stream));
    }

    fn remove_player(&mut self, id: PeerId) {
        self.players.remove(&id); // bye bye!
    }

    fn has_player(&self, id: PeerId) -> bool {
        self.players.contains_key(&id)
    }

    fn guessed_correctly(&mut self, id: PeerId, guess_is_left: bool) -> bool {
        let player = self.players.get_mut(&id).unwrap();

        let was_correct = player.question.guess(guess_is_left);
        player.question = Question::new(&self.questions);

        if was_correct { player.shield += 1 }
        was_correct
    }

    fn shoot_laser(&mut self, id: PeerId) {
        self.update();

        let a = self.players.get_mut(&id).unwrap();
        if a.shield == 0 { return }
        a.shield -= 1;

        let ray_x = a.x;
        let ray_y = a.y;
        let facing = a.facing;

        let mut to_kill = Vec::new();

        for (&other_id, other) in self.players.iter_mut().filter(|&(&other_id, _)| other_id != id) {
            if intersect_circle(other.x, other.y, PLAYER_RADIUS, ray_x, ray_y, facing) {
                if other.shield > 0 {
                    other.shield -= 1;
                } else {
                    // kill them !
                    to_kill.push(other_id);
                }
            }
        }

        for id in to_kill {
            self.kill(id);
        }

        self.lasers.push(Laser { x: ray_x, y: ray_y, facing, expire: self.last_updated as f64 + LASER_DURATION_MILLIS })
    }

    fn update_facing(&mut self, id: PeerId, new_facing: f64) {
        // where are they now?
        self.update();
        self.players.get_mut(&id).unwrap().facing = new_facing;
    }

    fn game_state_message_to(&self, receiver: PeerId) -> Json {
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

#[derive(Debug)]
struct PlayerInfo {
    x: f64,
    y: f64,
    facing: f64,
    color: String,
    shield: usize,
    question: Question,
    tcp_stream: WebSocketWriter,
}

impl PlayerInfo {
    fn from_random(questions: &[(String, String)], tcp_stream: WebSocketWriter) -> PlayerInfo {
        PlayerInfo {
            x: thread_rng().gen_range(0.0, MAP_WIDTH as f64),
            y: thread_rng().gen_range(0.0, MAP_HEIGHT as f64),
            facing: thread_rng().gen_range(0.0, TAU),
            color: format!("rgb({},{},{})", random::<u8>(), random::<u8>(), random::<u8>()),
            shield: 3,
            question: Question::new(questions),
            tcp_stream,
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

#[derive(Clone, Debug)]
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

fn intersect_circle(circle_x: f64, circle_y: f64, radius: f64, ray_x: f64, ray_y: f64, ray_angle: f64) -> bool {
    let b = circle_y * ray_angle.sin() - circle_x * ray_angle.cos() + ray_x * ray_angle.cos() - ray_y * ray_angle.sin();
    let discriminant = b*b - ray_x * ray_x - ray_y * ray_y + 2.0* circle_x * ray_x + 2.0* circle_y * ray_y - circle_y * circle_y - circle_x * circle_x + radius * radius;

    discriminant >= 0.0 && -b+discriminant.sqrt() >= 0.0
}

fn wrap(mut n: f64, range: f64) -> f64 {
    n %= range;
    if n < 0.0 { n += range }
    n
}

fn cool_vector() -> Option<Vec<(String, String)>> {
    let file = BufReader::new(File::open(GOD_SET_PATH).ok()?);

    file.lines()
        .map(|line| {
            let line = line.ok()?;
            let split: Vec<_> = line.trim_end().split("\t").collect();
            if split.len() != 10 { return None }
            Some((split[8].to_string(), split[9].to_string()))
        }).collect::<Option<_>>()
}

fn unix_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock read failed")
        .as_millis() as u64
}