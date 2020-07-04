use server::{PeerId, Disconnect};
use crate::apps::pusoy::Member;
use crate::apps::pusoy::state::GameState;
use json::Json;
use std::collections::HashMap;

pub struct PusoyGame {
    players: Vec<Member>, // includes host now

    state: GameState,
}

impl PusoyGame {
    ////////////////////////// HANDLERS //////////////////////////////////
    pub fn new(players: Vec<Member>) -> PusoyGame {
        let state = GameState::new(players.len());
        PusoyGame { players, state }
    }

    pub fn receive_message(&mut self, _id: PeerId, _message: &HashMap<String, Json>) -> Result<(), Disconnect> {
        todo!()
    }

    pub fn periodic(&mut self) {
        todo!()
    }

    pub fn leave(&mut self, _id: PeerId) -> bool {
        todo!()
    }

    //////////////////////////// OTHER FUNCTIONS /////////////////////

    fn tell_plays(&mut self) {

        for (player, &hand) in self.players.iter_mut().zip(self.state.hands()) {

        }

    }


}
