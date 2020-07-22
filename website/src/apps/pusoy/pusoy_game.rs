use server::{PeerId, Disconnect};
use crate::apps::pusoy::Member;
use pusoy::{GameState, all_plays, Card, Cards, Play, RandomPlayer, Player};
use json::{Json, jsons, json};
use std::collections::HashMap;
use rand::thread_rng;
use rand::seq::SliceRandom;
use std::time::{Duration, Instant};
use std::str::FromStr;

const MACHINE_PLAYER_TURN_DELAY: Duration = Duration::from_millis(2_000);
const HUMAN_PLAYER_MAX_TURN_LENGTH: Duration = Duration::from_millis(60_000);

pub struct PusoyGame {
    humans: Vec<Member>,
    virtual_players: Vec<Option<usize>>, // points to one of the humans
    turn_begin: Instant,
    available_plays: Vec<Play>,
    state: GameState,
}

impl PusoyGame {
    ////////////////////////// HANDLERS //////////////////////////////////
    pub fn new(humans: Vec<Member>) -> PusoyGame {
        let virtual_players = build_virtual_players(humans.len());
        let state = GameState::new(virtual_players.len());
        let turn_begin = Instant::now();
        let available_plays = state.get_interface().valid_plays();
        let mut ret = PusoyGame { humans, virtual_players, available_plays, turn_begin, state };
        ret.turn_transition();
        ret.give_turn_brief();

        ret
    }

    pub fn receive_message(&mut self, id: PeerId, message: &HashMap<String, Json>) -> Result<(), Disconnect> {
        let coming_from_current_player = self.humans[self.virtual_players[self.state.current_player()].unwrap()].get_id() == id;

        match message.get("kind")?.get_string()? {
            "play" if coming_from_current_player => {
                let play_index = message.get("index")?.get_number()? as usize;
                let play = all_plays(self.state.my_hand())[play_index];
                self.do_play(play);
            },
            "playCardsArray" if coming_from_current_player => {
                let cards = message.get("cards")?.get_array()?.iter()
                    .map(|c| Card::from_str(c.get_string()?).ok())
                    .collect::<Option<Cards>>()?;

                match self.available_plays.iter().find(|p| p.cards() == cards) {
                    Some(&play) => self.do_play(play),
                    None => { // invalid play
                        self.humans[self.virtual_players[self.state.current_player()].unwrap()]
                            .write_ignore(&jsons!({
                                kind: "invalidPlay",
                            }));
                    },
                }
            },
            _ => return Err(Disconnect),
        }

        Ok(())
    }

    pub fn periodic(&mut self) {
        let is_human = self.virtual_players[self.state.current_player()].is_some();
        let elapsed = self.turn_begin.elapsed();

        if is_human && elapsed >= HUMAN_PLAYER_MAX_TURN_LENGTH {
            // force a move
            self.do_play(self.available_plays[0]);

        } else if !is_human && elapsed >= MACHINE_PLAYER_TURN_DELAY {
            let play = RandomPlayer.choose_play(&self.available_plays, self.state.get_interface());
            self.do_play(self.available_plays[play]);
        }
    }

    pub fn leave(&mut self, _id: PeerId) -> bool {
        true
    }

    //////////////////////////// OTHER FUNCTIONS /////////////////////

    fn do_play(&mut self, play: Play) {
        if self.state.winning_player().is_some() { return } // TODO: just a shim

        if self.state.can_play(play).is_err() {
            println!("invalid play {:?} {:?}", play, self.state.cards_on_table());
        }
        self.state.play(play);

        self.turn_transition();

        match self.state.winning_player() {
            Some(winner) => {
                // game over
                for human in self.humans.iter_mut() {
                    human.write_ignore(&jsons!({
                        kind: "over",
                        winner: winner,
                    }))
                }
            },
            None => {
                self.available_plays = self.state.get_interface().valid_plays();
                self.give_turn_brief();

                self.turn_begin = Instant::now();
            },
        }
    }

    fn turn_transition(&mut self) {
        let card_counts = Json::Array(self.state.hands().iter().map(|c| Json::Number(c.len() as f64)).collect());

        let on_table = match self.state.cards_on_table() {
            Some(play) => jsonify_cards(play.cards()),
            None => Json::Array(vec![]),
        };

        for (i, human_id) in self.virtual_players.iter().copied().enumerate() {
            if let Some(human_id) = human_id {
                let human = &mut self.humans[human_id];
                let hand = jsonify_cards(self.state.hands()[i]);

                human.write_ignore(&jsons!({
                    kind: "transition",
                    yourId: i,
                    turnIndex: (self.state.current_player()),
                    hand: hand,
                    onTable: (on_table.clone()),
                    cardCounts: (card_counts.clone()),
                }));

            }
        }
    }

    fn give_turn_brief(&mut self) {
        if let Some(human_index) = self.virtual_players[self.state.current_player()] {
            let hand = self.state.my_hand();
            let possible_plays = Json::Array(all_plays(hand).into_iter()
                .filter(|&p| self.state.can_play(p).is_ok())
                .map(jsonify_play).collect()
            );

            let human = &mut self.humans[human_index];

            human.write_ignore(&jsons!({
                kind: "turnBrief",
                canPass: (self.state.can_play(Play::pass()).is_ok()),
                possiblePlays: possible_plays,
            }));
        }
    }

}


fn build_virtual_players(humans_count: usize) -> Vec<Option<usize>> {
    assert!(humans_count <= 4, "need to handle this case");

    let mut virtual_players = Vec::with_capacity(4);

    virtual_players.extend((0..humans_count).map(|n| Some(n)));

    virtual_players.resize(4, None);

    virtual_players.shuffle(&mut thread_rng());

    virtual_players

}

fn jsonify_play(play: Play) -> Json {
    json!({
        kind: (play.kind().to_string()),
        cards: (jsonify_cards(play.cards())),
    })
}

fn jsonify_cards(cards: Cards) -> Json {
    Json::Array(cards.iter().map(|c| jsonify_card(c)).collect())
}


fn jsonify_card(card: Card) -> Json {
    Json::String(card.to_string())
}