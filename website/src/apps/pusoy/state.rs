use crate::apps::pusoy::play::Play;
use crate::apps::pusoy::cards::{Card};
use rand::{thread_rng, Rng};
use crate::apps::pusoy::Cards;

// MOTIVATION: if HumanPlayer or MachinePlayer had access to the regular GameState object,
// they could call .hands and other info that would just be cheating. This struct only gives
// access to data that isn't cheating
#[derive(Clone, Copy)]
pub struct SafeGameInterface<'a> {
    pub inner: &'a GameState,
}

impl<'a> SafeGameInterface<'a> {
    pub fn from_game(game: &'a GameState) -> SafeGameInterface<'a> {
        SafeGameInterface { inner: game }
    }

    pub fn can_play(&self, cards: Cards) -> Result<Play, GameError> {
        self.inner.can_play(cards)
    }

    #[inline]
    pub fn can_play_bool(&self, play: &Play) -> bool {
        self.inner.can_play_bool(play)
    }

    pub fn my_hand(&self) -> Cards {
        self.inner.my_hand()
    }

    #[inline]
    pub fn get_play_on_table(&self) -> Option<&Play> {
        self.inner.get_play_on_table()
    }
}

#[derive(Debug)]
pub struct GameState {
    hands: Vec<Cards>,
    pub current_player: usize,
    cards_down: Option<Play>,
    turn_index: usize, // need to store because on first turn, must play a hand with three of clubs
    last_player_to_not_pass: usize,
    players_count: usize,
    winning_player: Option<usize>,
}

impl GameState {
    pub fn new(players_count: usize) -> GameState {
        let hands = deal(players_count);

        // figure out who has the three of clubs
        let mut current_player = 0;
        for (i, hand) in hands.iter().enumerate() {
            if hand.contains(Card::THREE_OF_CLUBS) {
                current_player = i;
                break;
            }
        }

        GameState {
            hands,
            current_player,
            cards_down: None,
            turn_index: 0,
            last_player_to_not_pass: current_player,
            players_count,
            winning_player: None,
        }
    }

    #[inline]
    pub fn can_play_bool(&self, play: &Play) -> bool {
        // make sure we have all the cards in that play
        if !self.my_hand().is_superset_of(play.cards()) {
            return false;
        }

        if self.is_first_turn() {
            if !play.cards().contains(Card::THREE_OF_CLUBS) {
                // the only requirement on the first move is that they play the three of clubs somehow
                return false;
            }
        } else if self.has_control() {
            // if we have control, we can pretty much do anything except passing
            if play.is_pass() {
                return false;
            }
        } else if !play.is_pass() {
            // here, we have our standard conditions, where we are not passing, and we don't have control

            // since we don't have control, we have to make sure they are making a valid play in the context
            // of the cards that they are trying to play on.
            let cards_down = self.cards_down.as_ref().unwrap();

            // this is the problem
            if !play.len_eq(cards_down) {
                return false;
            }

            if !play.can_play_on(cards_down) {
                return false;
            }
        } // we don't have to list out the condition where we don't have control are passing, because this is always legal

        true
    }

    pub fn can_play(&self, cards: Cards) -> Result<Play, GameError> {
        let play = Play::infer_from_cards(cards).ok_or(GameError::PlayDoesntExist)?;

        if !self.my_hand().is_superset_of(play.cards()) {
            return Err(GameError::DontHaveCard);
        }

        // make sure the move we are trying to do is legal
        if self.is_first_turn() {
            if !play.cards().contains(Card::THREE_OF_CLUBS) {
                // the only requirement on the first move is that they play the three of clubs somehow
                return Err(GameError::IsntPlayingThreeOfClubs);
            }
        } else if self.has_control() {
            // if we have control, we can pretty much do anything except passing
            if play.is_pass() {
                return Err(GameError::CannotPass);
            }
        } else if !play.is_pass() {
            // here, we have our standard conditions, where we are not passing, and we don't have control

            // since we don't have control, we have to make sure they are making a valid play in the context
            // of the cards that they are trying to play on.
            let cards_down = self.cards_down.as_ref().unwrap();

            // this is the problem
            if !play.len_eq(cards_down) {
                return Err(GameError::WrongLength);
            }

            if !play.can_play_on(cards_down) {
                return Err(GameError::TooLow);
            }
        } // we don't have to list out the condition where we don't have control are passing, because this is always legal


        Ok(play)
    }

    pub fn play(&mut self, play: Play) {
        // assumes that play is_legal

        self.hands[self.current_player].remove_all(play.cards());

        if self.hands[self.current_player].is_empty() {
            self.winning_player = Some(self.current_player);
        }

        if !play.is_pass() {
            self.last_player_to_not_pass = self.current_player;
            self.cards_down = Some(play); // if we are passing, the card that the next person has to play on doesn't change
        }

        self.turn_index += 1;
        self.current_player = (self.current_player + 1) % self.players_count;
    }

    pub fn has_control(&self) -> bool {
        self.last_player_to_not_pass == self.current_player
    }

    pub fn winning_player(&self) -> Option<usize> {
        self.winning_player
    }

    pub fn get_play_on_table(&self) -> Option<&Play> {
        self.cards_down.as_ref()
    }

    #[inline]
    pub fn my_hand(&self) -> Cards {
        self.hands[self.current_player]
    }

    pub fn hands(&self) -> &[Cards] {
        &self.hands
    }

    pub fn is_first_turn(&self) -> bool {
        self.turn_index == 0
    }
}

#[derive(Debug)]
pub enum GameError {
    DontHaveCard,
    IsntPlayingThreeOfClubs,
    TooLow,
    WrongLength,
    CannotPass,
    PlayDoesntExist,
}

fn deal(players_count: usize) -> Vec<Cards> {
    assert_eq!(52 % players_count, 0, "Cannot deal out the cards evenly");

    let mut ret = vec![Cards::empty(); players_count];

    let mut deck_remaining: Vec<Card> = Cards::entire_deck().iter().collect();

    for i in (0..players_count).cycle() {
        let index = thread_rng().gen_range(0, deck_remaining.len());
        ret[i].insert(deck_remaining.remove(index));
    }

    ret
}