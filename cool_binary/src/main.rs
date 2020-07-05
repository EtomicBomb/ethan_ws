
const _COOL_FILE_PATH: &'static str = "/home/etomicbomb/RustProjects/ethan_ws/cool_binary/cool.lisp";

use pusoy::{HumanPlayer, MachinePlayer, Player, SafeGameInterface, GameState};

fn main() {
    let players: Vec<Box<dyn Player>> = vec![
        Box::new(MachinePlayer),
        Box::new(MachinePlayer),
        Box::new(MachinePlayer),
        Box::new(MachinePlayer),
    ];

    let mut game = GameState::new(4);

    while game.winning_player().is_none() {
        let interface = SafeGameInterface::from_game(&game);

        let play = players[game.current_player].choose_play(interface);

        // lets report
        if play.is_pass() {
            println!("player {} passed", game.current_player);
        } else {
            println!("player {} played {:?}", game.current_player, play.cards());
        }

        game.play(play);
    }
}

