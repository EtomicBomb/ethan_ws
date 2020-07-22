mod players;
mod cards;
mod state;
mod play;

pub use cards::{Card, Cards, Rank, Suit};
pub use state::{SafeGameInterface, GameState, GameError};
pub use players::{Player, MachinePlayer, RandomPlayer, HumanPlayer};
pub use play::{PlayKind, Play, all_plays};


// const PUSOY_PASSING_MODEL_PATH: &str = "/home/etomicbomb/Desktop/passingModel.dat";
const PUSOY_PASSING_MODEL_PATH: &str = "/home/pi/Desktop/server/passingModel.dat";
