#![feature(test)]

const _COOL_FILE_PATH: &'static str = "/home/etomicbomb/RustProjects/ethan_ws/cool_binary/cool.lisp";

extern crate test;

use pusoy::{MachinePlayer, Player, GameState, Cards, Card};
use std::str::FromStr;


fn main() {
    println!("{:?}", Card::from_str("A♦"));

    let mut cards = Cards::entire_deck();
    cards.remove(Card::THREE_OF_CLUBS);
    cards.remove(Card::TWO_OF_DIAMONDS);
    cards.remove(Card::JACK_OF_SPADES);

    dbg!(cards.all_same_rank());
    dbg!(cards);

    loop {
        let players: Vec<Box<dyn Player>> = vec![
            Box::new(MachinePlayer),
            Box::new(MachinePlayer),
            Box::new(MachinePlayer),
            Box::new(MachinePlayer),
        ];

        let mut game = GameState::new(4);

        while game.winning_player().is_none() {
            let interface = game.get_interface();

            let valid_plays = interface.valid_plays();

            let play_index = players[game.current_player()].choose_play(&valid_plays, interface);

            let play = valid_plays[play_index];
            // lets report
            if play.is_pass() {
                println!("player {} passed", game.current_player());
            } else {
                println!("player {} played {:?}", game.current_player(), play.cards());
            }

            game.play(play);
        }

        println!("**********************");
    }
}

