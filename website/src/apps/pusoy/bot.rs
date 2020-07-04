use vec_map::{VecMap};

use std::f64::INFINITY;
use std::io;

use crate::apps::pusoy::cards::{Card};
use crate::apps::pusoy::state::SafeGameInterface;
use crate::apps::pusoy::play::finder::Finder;
use crate::apps::pusoy::play::{Play};
use crate::apps::pusoy::{get_expected_pass_count, Cards};

pub trait Player {
    fn choose_play(&self, game: SafeGameInterface) -> Play;
}

pub struct HumanPlayer;

impl Player for HumanPlayer {
    fn choose_play(&self, game: SafeGameInterface) -> Play {
        loop {
            let your_hand = game.my_hand();
            println!("your turn - {:?}", your_hand);
            let mut cards_string = String::new();
            io::stdin().read_line(&mut cards_string).unwrap();

            let cards: Cards = cards_string
                .split_whitespace()
                .map(|c| c.parse().unwrap())
                .collect();

            // try to play these cards
            match game.can_play(cards) {
                Ok(play) => {
                    // it worked
                    break play;
                }
                Err(e) => {
                    eprintln!("invalid turn: {:?}", e);
                    // we're gonna have to prompt the user again
                }
            }
        }
    }
}

pub struct MachinePlayer;

impl Player for MachinePlayer {
    fn choose_play(&self, game: SafeGameInterface) -> Play {

        let desired_play = best_play(game);

        match game.can_play(desired_play.cards()) {
            Ok(play) => play,
            Err(_) => {
                // we're gonna have to pass here
                match game.can_play(Cards::empty()) {
                    Ok(pass) => pass,
                    Err(e) => unreachable!("{:?}", e),
                }
            }
        }
    }
}

pub fn best_play(game: SafeGameInterface) -> Play {
    let my_hand = game.my_hand();
    let plays_available = Finder::new(my_hand).all_plays();


    let potential_inserts = PotentialInserts::new(my_hand);
    let depth_left = my_hand.len();

    let mut memo = VecMap::with_capacity(MEMO_TABLE_CAPACITIES[depth_left-1]);
    let cards_used_so_far = CardsUsedSoFar::new();

    let result = cost_of_tail(plays_available, depth_left, &potential_inserts, game, cards_used_so_far, &mut memo);

    result.first_play()
} 

fn cost_of_tail(
    plays_available: Vec<Play>,
    depth_left: usize,
    potential_inserts: &PotentialInserts,
    game_interface: SafeGameInterface,
    cards_used_so_far: CardsUsedSoFar,
    memo: &mut VecMap<SearchState>,
) -> SearchState {

    match memo.get(cards_used_so_far.get_digest()) {
        Some(state) => return state.clone(), // we already have the best tail computed for this
        None => {}, // we're gonna have to do this the old fasioned way
    }

    if depth_left == 0 {
        // then, we have the tail and its cost; zero
        SearchState::new(game_interface)

    } else {
        let mut best_tail: Option<SearchState> = None;

        for play in plays_available.iter() {
            let n_cards = play.cards().len();

            if depth_left < n_cards {
                continue; 
            }

            let mut plays_available_to_child = Vec::with_capacity(plays_available.len());
            for p in plays_available.iter() {
                if p.cards().is_disjoint(play.cards()) {
                    plays_available_to_child.push(p.clone());
                }
            }

            let mut child_state_keeper = cards_used_so_far.clone();
            child_state_keeper.add_cards(play.cards(), potential_inserts);

            let mut result = cost_of_tail(plays_available_to_child, depth_left-n_cards, potential_inserts, game_interface, child_state_keeper, memo);

            result.add_play(play, game_interface);
            if best_tail.is_none() || result.total_cost < best_tail.as_ref().unwrap().total_cost {
                best_tail = Some(result);
            }
        }

        let ret = best_tail.unwrap();
        memo.insert(cards_used_so_far.get_digest(), ret.clone());
        ret
    }
}


// describes the state of the game after a move has been played
#[derive(Clone)]
pub struct SearchState {
    // this is the play that on our turn, we are looking to play on top of.
    // None if it is the first turn
    status: Status,
    total_cost: f64,
    first_play: Option<Play>,
}

#[derive(Clone)]
enum Status {
    FirstTurnOfGame,
    FirstAnalysis(Play), // previous term
    Rest(Play),          // four terms before
}

impl Status {
    fn is_first_turn(&self) -> bool {
        match *self {
            Status::Rest(_) => false,
            _ => true, 
        }
    }
}

impl<'a> SearchState {
    pub fn new(game_interface: SafeGameInterface) -> SearchState {
        let status = match game_interface.get_play_on_table() {
            Some(play) => Status::FirstAnalysis(play.clone()),
            None => Status::FirstTurnOfGame,
        };

        SearchState {
            status,
            total_cost: 0.0,
            first_play: None,
        }
    }

    #[inline]
    fn add_play(&mut self, play: &Play, game_interface: SafeGameInterface) {
        self.total_cost += match self.status {
            Status::FirstTurnOfGame => {
                if play.cards().contains(Card::THREE_OF_CLUBS) {
                    1.0 // we literally won't be able to pass
                } else {
                    INFINITY // this is always unplayable
                }
            }
            Status::FirstAnalysis(ref _before) => {
                // we are trying to play directly on these cards
            
                if game_interface.can_play_bool(play) {
                    1.0
                } else {
                    // how many turns do we think it will take
                    // TODO: include numbers from research!
                    get_expected_pass_count(_before, play)
                }
            }
            Status::Rest(ref _four_turns_before) => {
                // TODO: include numbers from the research!
                get_expected_pass_count(_four_turns_before, play)

            }
        };
        // change the status going forward
        if self.status.is_first_turn() {
            self.first_play = Some(play.clone());
        }
        self.status = Status::Rest(play.clone()); 
    }

    #[inline]
    fn first_play(self) -> Play {
        self.first_play.unwrap()
    }
}



struct PotentialInserts {
    map: [usize; 52],
}

impl PotentialInserts {
    fn new(cards: Cards) -> PotentialInserts {
        let mut map = [0; 52];
        for (i, card) in cards.iter().enumerate() {
            map[card.get_index() as usize] = i;
        }

        PotentialInserts { map }
    }

    #[inline]
    fn get_offset(&self, card: Card) -> usize {
        self.map[card.get_index() as usize]
    }
}

// we cannot just keep a u64 and store all of the cards because then get_digest wont fit into the VecMap
#[derive(Clone)]
struct CardsUsedSoFar {
    seen_so_far: u16, // only  use lower 13 bits
}

impl CardsUsedSoFar {
    fn new() -> CardsUsedSoFar {
        CardsUsedSoFar {
            seen_so_far: 0, // empty
        }
    }

    #[inline]
    fn add_card(&mut self, card: Card, potential_inserts: &PotentialInserts) {
        let offset = potential_inserts.get_offset(card);
        self.seen_so_far |= 1 << offset;
    }


    #[inline]
    fn add_cards(&mut self, cards: Cards, potential_inserts: &PotentialInserts) {
        for card in cards.iter() {
            self.add_card(card, potential_inserts);
        }
    }

    #[inline]
    fn get_digest(&self) -> usize {
        self.seen_so_far as usize // result always in 0..2^13
    }
}

// determined experimentally
const MEMO_TABLE_CAPACITIES: [usize; 13] = [
    1,
    14,
    92,
    378,
    1093,
    2380,
    4096,
    5812,
    7099,
    7814,
    8100,
    8178,
    8191,
];