use rand::{Rng, random};
use rand::distributions::{Distribution, Standard};

use std::collections::{HashSet, HashMap};

use web_socket::{WebSocketMessage, WebSocketWriter};
use json::{Json, jsont, jsons};

use server::{PeerId, GlobalState, Disconnect};


const WIDTH: usize = 8;
const HEIGHT: usize = 7;
const N_COLORS: u8 = 6;
const DEPTH: usize = 4;


struct Player {
    writer: WebSocketWriter,
    game_state: GameState,
}

impl Player {
    fn new(writer: WebSocketWriter) -> Player {
        Player { writer, game_state: GameState::new() }
    }
}

pub struct FillerGlobalState {
    active_players: HashMap<PeerId, Player>,
}


impl FillerGlobalState {
    pub fn new() -> FillerGlobalState {
        FillerGlobalState { active_players: HashMap::new() }
    }
}

impl GlobalState for FillerGlobalState {
    fn new_peer(&mut self, id: PeerId, writer: WebSocketWriter) {
        self.active_players.insert(id, Player::new(writer));
        let player = self.active_players.get_mut(&id).unwrap();
        let _ = player.writer.write_string(&player.game_state.jsonify().to_string());
    }

    fn on_message_receive(&mut self, from: PeerId, message: WebSocketMessage) -> Result<(), Disconnect> {
        let player = self.active_players.get_mut(&from).unwrap();

        let color_chosen = Color::from_string(message.get_text()?)?;

        player.game_state.do_move(color_chosen).ok()?;

        player.game_state.do_move(
            player.game_state.get_colors().into_iter()
                .map(|color| {
                    let mut next = player.game_state.clone();
                    next.do_move(color).unwrap();
                    let evaluation = max_advantage(next, false, false, DEPTH);
                    (color, evaluation)
                })
                .max_by_key(|&(_, e)| e)
                .unwrap()
                .0
        ).ok()?;

        player.writer.write_string(&player.game_state.jsonify())?;

        Ok(())
    }

    fn on_disconnect(&mut self, id: PeerId) {
        self.active_players.remove(&id);
    }

    fn periodic(&mut self) { }
}




fn max_advantage(game_state: GameState, is_left: bool, is_our_turn: bool, depth_left: usize) -> isize {
    if depth_left == 0 {
        game_state.left_advantage() * if is_left { 1 } else { -1 }
    } else {
        let a = game_state.get_colors().into_iter()
            .map(|c| {
                let mut new_game_state = game_state.clone();
                new_game_state.do_move(c).unwrap();
                max_advantage(new_game_state, is_left, !is_our_turn, depth_left-1)
            });

        if is_our_turn {
            a.max().unwrap()
        } else {
            a.min().unwrap()
        }
    }
}


#[derive(Clone)]
pub struct GameState {
    field: Field,
    left_territory: HashSet<(usize, usize)>,
    right_territory: HashSet<(usize, usize)>,
    is_left_turn: bool,
}

impl GameState {
    fn new() -> GameState {
        GameState {
            field: Field::from_random(),
            left_territory: vec![(0, HEIGHT-1)].into_iter().collect(),
            right_territory: vec![(WIDTH-1, 0)].into_iter().collect(),
            is_left_turn: true,
        }
    }

    fn get_colors(&self) -> Vec<Color> {
        let reasonable = self.reasonable_colors();

        if reasonable.is_empty() {
            self.valid_colors().to_vec()
        } else {
            reasonable.into_iter().collect()
        }
    }

    pub fn jsonify(&self) -> String {
        let left_territory = self.left_territory.iter()
            .map(|&(x, y)| jsont!({x: (x as f64), y: (y as f64)}))
            .collect();
        let right_territory = self.right_territory.iter()
            .map(|&(x, y)| jsont!({x: (x as f64), y: (y as f64)}))
            .collect();

        let available_colors = Json::Array(self.valid_colors().iter().map(|c| c.jsonify()).collect());

        jsons!({
            field: (self.field.jsonify()),
            leftTerritory: (Json::Array(left_territory)),
            rightTerritory: (Json::Array(right_territory)),
            isLeftTurn: (self.is_left_turn),
            availableColors: available_colors,
        })
    }

    fn reasonable_colors(&self) -> HashSet<Color> {
        let territory =
            if self.is_left_turn {
                &self.left_territory
            } else {
                &self.right_territory
            };

        let mut surrounding_colors = HashSet::new();

        for &(x, y) in territory.iter() {
            // we use wrapping sub mostly because i'm lazy and it works because if x == 0 and we do a wrapping sub,
            // we're gonna to get a None value from our field.get
            for &(around_x, around_y) in [(x, y.wrapping_sub(1)), (x, y + 1), (x.wrapping_sub(1), y), (x + 1, y)].iter() {
                if let Some(color) = self.field.get(around_x, around_y) {
                    if color != self.left_color() && color != self.right_color() {
                        surrounding_colors.insert(color);
                    }
                }
            }
        }
        surrounding_colors
    }

    fn left_color(&self) -> Color {
        let &(x, y) = self.left_territory.iter().next().unwrap();
        self.field.get(x, y).unwrap()
    }

    fn right_color(&self) -> Color {
        let &(x, y) = self.right_territory.iter().next().unwrap();
        self.field.get(x, y).unwrap()
    }

    fn valid_colors(&self) -> [Color; 4] {
        let mut ret = [Color::Black; 4];
        let mut index = 0;

        for i in 1..=6 {
            let color = Color::from_u8(i);
            if color != self.left_color() && color != self.right_color() {
                ret[index] = color;
                index += 1;
            }
        }

        ret
    }

    fn left_advantage(&self) -> isize {
        self.left_territory.len() as isize - self.right_territory.len() as isize
    }

    fn do_move(&mut self, fill_color: Color) -> Result<(), ()> {
        // check if fill_color is valid (ie. not our or opponents current color)
        if fill_color == self.left_color() || fill_color == self.right_color() {
            return Err(());
        }

        let territory =
            if self.is_left_turn {
                &mut self.left_territory
            } else {
                &mut self.right_territory
            };

        let mut to_add = HashSet::new();

        for &(x, y) in territory.iter() {
            // we use wrapping sub mostly because i'm lazy and it works because if x == 0 and we do a wrapping sub,
            // we're gonna to get a None value from our field.get
            for &(around_x, around_y) in [(x, y.wrapping_sub(1)), (x, y+1), (x.wrapping_sub(1), y), (x+1, y)].iter() {
                if self.field.get(around_x, around_y) == Some(fill_color) {
                    // yes, this value might already be in to_add, but that's fine cause this is a set
                    to_add.insert((around_x, around_y));
                }
            }
        }

        territory.extend(to_add.into_iter());

        for &(x, y) in territory.iter() {
            self.field.set(x, y, fill_color);
        }

        self.is_left_turn = !self.is_left_turn;

        Ok(())
    }


}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Color {
    Red = 1,
    Yellow = 2,
    Green = 3,
    Blue = 4,
    Purple = 5,
    Black = 6,
}

impl Color {
    fn from_u8(n: u8) -> Color {
        match n {
            1 => Color::Red,
            2 => Color::Yellow,
            3 => Color::Green,
            4 => Color::Blue,
            5 => Color::Purple,
            6 => Color::Black,
            _ => panic!("color index out of range"),
        }
    }

    fn jsonify(self) -> Json {
        match self {
            Color::Red => Json::String(String::from("red")),
            Color::Yellow => Json::String(String::from("yellow")),
            Color::Green => Json::String(String::from("green")),
            Color::Blue => Json::String(String::from("blue")),
            Color::Purple => Json::String(String::from("purple")),
            Color::Black => Json::String(String::from("black")),
        }
    }

    fn from_string(string: &str) -> Option<Color> {
        Some(match string {
            "red" => Color::Red,
            "yellow" => Color::Yellow,
            "green" => Color::Green,
            "blue" => Color::Blue,
            "purple" => Color::Purple,
            "black" => Color::Black,
            _ => return None,
        })
    }
}

impl Distribution<Color> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Color {
        let n = rng.gen_range(1, N_COLORS+1);

        Color::from_u8(n)
    }
}


#[derive(Clone)]
struct Field {
    inner: [[Color; WIDTH]; HEIGHT],
}

impl Field {
    pub fn jsonify(&self) -> Json {
        Json::Array(
            self.inner.iter()
                .map(|row|
                    Json::Array(row.iter().map(|color| color.jsonify()).collect::<Vec<Json>>()))
                .collect::<Vec<Json>>()
        )
    }

    pub fn from_random() -> Field {
        let mut inner = [[Color::Black; WIDTH]; HEIGHT]; // not gonna stay black

        for y in 0..HEIGHT {
            for x in 0..WIDTH {

                // we want a color such that the colors above and to the left are different
                // if this is true for every color on the map, then we get a checkerboard deelio
                // (no two adjacent colors are the same)
                let mut color: Color;

                loop {
                    color = random();

                    // extra check: bottom right and upper left cannot be the same color
                    if x == 0 && y == HEIGHT-1 && color == inner[0][WIDTH-1] { continue }

                    if (x!=0 && y!=0 && inner[y][x-1] != color && inner[y-1][x] != color)
                        || (x==0 && y==0)
                        || (x==0 && inner[y-1][x] != color)
                        || (y==0 && inner[y][x-1] != color)
                    { break }
                }

                inner[y][x] = color;
            }
        }

        Field { inner }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<Color> {
        self.inner.get(y)?.get(x).copied()
    }

    pub fn set(&mut self, x: usize, y: usize, color: Color) {
        self.inner[y][x] = color;
    }
}
