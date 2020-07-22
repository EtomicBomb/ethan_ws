use crate::Play;
use crate::all_plays;
use crate::cards::{Card, Cards};
use rand::{thread_rng, Rng};

// MOTIVATION: if HumanPlayer or MachinePlayer had access to the regular GameState object,
// they could call .hands and other info that would just be cheating. This struct only gives
// access to data that isn't cheating
#[derive(Clone, Copy)]
pub struct SafeGameInterface<'a> {
    inner: &'a GameState,
}

impl<'a> SafeGameInterface<'a> {
    pub fn valid_plays(&self) -> Vec<Play> {
        all_plays(self.my_hand()).into_iter()
            .filter(|&p| self.can_play(p).is_ok())
            .collect()
    }

    #[inline]
    pub fn can_play(&self, play: Play) -> Result<(), GameError> {
        self.inner.can_play(play)
    }

    pub fn my_hand(&self) -> Cards {
        self.inner.my_hand()
    }

    #[inline]
    pub fn cards_on_table(&self) -> Option<Play> {
        self.inner.cards_on_table()
    }
}

#[derive(Debug)]
pub struct GameState {
    hands: Vec<Cards>,
    current_player: usize,
    cards_on_table: Option<Play>,
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
            cards_on_table: None,
            turn_index: 0,
            last_player_to_not_pass: current_player,
            players_count,
            winning_player: None,
        }
    }

    pub fn get_interface(&self) -> SafeGameInterface {
        SafeGameInterface { inner: self }
    }

    #[inline]
    pub fn can_play(&self, play: Play) -> Result<(), GameError> {
        if !self.my_hand().is_superset_of(play.cards()) {
            return Err(GameError::DontHaveCard);
        }

        if self.is_first_turn() {
            if !play.cards().contains(Card::THREE_OF_CLUBS) {
                return Err(GameError::IsntPlayingThreeOfClubs);
            }
        } else if self.has_control() {
            // if we have control, we can pretty much do anything except passing
            if play.is_pass() {
                return Err(GameError::MustPlayOnPass);
            }
        } else if !play.is_pass() {
            // here, we have our standard conditions, where we are not passing, and we don't have control

            // since we don't have control, we have to make sure they are making a valid play in the context
            // of the cards that they are trying to play on.
            let cards_down = self.cards_on_table.unwrap();

            // this is the problem
            if !play.len_eq(cards_down) {
                return Err(GameError::WrongLength);
            }

            if !play.can_play_on(cards_down) {
                return Err(GameError::TooLow);
            }
        } // we don't have to list out the condition where we don't have control are passing, because this is always legal

        Ok(())
    }

    pub fn play(&mut self, play: Play) {
        // assumes that play is_legal
        self.can_play(play).unwrap(); // TODO: handle properly

        self.hands[self.current_player].remove_all(play.cards());

        if self.hands[self.current_player].is_empty() {
            self.winning_player = Some(self.current_player);
        }

        if !play.is_pass() {
            self.last_player_to_not_pass = self.current_player;
            self.cards_on_table = Some(play); // if we are passing, the card that the next person has to play on doesn't change
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

    pub fn cards_on_table(&self) -> Option<Play> {
        self.cards_on_table
    }

    #[inline]
    pub fn my_hand(&self) -> Cards {
        self.hands[self.current_player]
    }

    pub fn hands(&self) -> &[Cards] {
        &self.hands
    }

    pub fn current_player(&self) -> usize { self.current_player }

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
    MustPlayOnPass,
    PlayDoesntExist,
}

fn deal(players_count: usize) -> Vec<Cards> {
    assert_eq!(52 % players_count, 0, "Cannot deal out the cards evenly");

    let mut ret = vec![Cards::empty(); players_count];

    let mut deck_remaining: Vec<Card> = Cards::entire_deck().iter().collect();

    for i in (0..players_count).cycle().take(52) {
        let index = thread_rng().gen_range(0, deck_remaining.len());
        ret[i].insert(deck_remaining.remove(index));
    }

    ret
}