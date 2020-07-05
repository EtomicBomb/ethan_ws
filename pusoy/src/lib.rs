// use rand::{Rng, thread_rng};
// use std::collections::HashSet;
// use std::fmt::Debug;
// use std::collections::HashMap;
// use std::io::{BufReader};
// use play::{Play, PlayKind};
// use std::fs::{File};
// use std::iter::once;
// use lazy_static::lazy_static;
// use std::io::BufRead;

pub use cards::{Card, Cards};
pub use state::*;
pub use bot::*;
pub use play::{PlayKind, Play};

mod bot;
mod cards;
mod state;
mod play;

const PUSOY_PASSING_MODEL_PATH: &str = "/home/etomicbomb/Desktop/passingModel.dat";




// fn website() {
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
