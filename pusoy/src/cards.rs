use serde::{Deserialize, Serialize};

use std::fmt;
use std::str::FromStr;

use self::Rank::*;
use self::Suit::*;
use std::iter::{FromIterator, FusedIterator};

/// Represents a collection of cards.
/// Implemented as a bitset, with 3♣ as the least significant bit, in suit-major order
#[derive(Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Cards {
    bits: u64,
}

impl Cards {
    pub const EMPTY: Cards = Cards::empty();

    pub const fn entire_deck() -> Cards {
        Cards { bits: (1 << 52) - 1 } // lower 52 bits all set to 1
    }

    pub const fn empty() -> Cards {
        Cards { bits: 0 }
    }

    pub const fn single(card: Card) -> Cards {
        Cards { bits: 1 << card.get_index() }
    }

    pub fn insert(&mut self, card: Card) {
        self.bits |= 1 << card.get_index()
    }

    pub fn insert_all(&mut self, other: Cards) {
        self.bits |= other.bits;
    }

    pub fn remove(&mut self, card: Card) {
        self.bits &= !(1 << card.get_index());
    }

    pub fn remove_all(&mut self, other: Cards) {
        self.bits &= !other.bits;
    }

    pub fn is_disjoint(self, other: Cards) -> bool {
        self.bits & other.bits == 0
    }

    pub fn is_superset_of(self, other: Cards) -> bool {
        other.bits & !self.bits == 0
    }

    pub fn contains(self, card: Card) -> bool {
        (self.bits >> card.get_index()) & 1 == 1
    }

    pub fn len(self) -> usize {
        self.bits.count_ones() as usize
    }

    pub fn is_empty(self) -> bool {
        self.bits == 0 // equivalently, self.len() == 0
    }

    pub fn all_same_rank(self) -> bool {
        let after = (self.bits.trailing_zeros() | 3) - 3; // round down to multiple of 4
        let rank_cluster = self.bits >> after; // move our rank cluster to the lower 4 bits
        rank_cluster < 16 // if they're all the same rank, then this should be the only rank cluster
    }

    #[inline]
    pub fn all_same_suit(self) -> bool {
        const CLUBS_MASK:    u64 = 0b_1110_1110_1110_1110_1110_1110_1110_1110_1110_1110_1110_1110_1110;
        const SPADES_MASK:   u64 = 0b_1101_1101_1101_1101_1101_1101_1101_1101_1101_1101_1101_1101_1101;
        const HEARTS_MASK:   u64 = 0b_1011_1011_1011_1011_1011_1011_1011_1011_1011_1011_1011_1011_1011;
        const DIAMONDS_MASK: u64 = 0b_0111_0111_0111_0111_0111_0111_0111_0111_0111_0111_0111_0111_0111;

        self.bits & CLUBS_MASK == 0 // is it a club flush?
            || self.bits & SPADES_MASK == 0 // is it a spade flush?
            || self.bits & HEARTS_MASK == 0 // etc.
            || self.bits & DIAMONDS_MASK == 0
    }

    pub fn max_card(self) -> Option<Card> {
        match self.bits.leading_zeros() {
            64 => None,
            n => Some(Card { inner: 63 - n as u8 }),
        }
    }

    fn min_card(self) -> Option<Card> {
        match self.bits.trailing_zeros() {
            64 => None,
            n => Some(Card { inner: n as u8 })
        }
    }

    pub fn cards_vec(self) -> Vec<Card> {
        self.iter().collect()
    }

    pub fn iter(self) -> CardsIter {
        CardsIter::new(self)
    }
}

impl fmt::Debug for Cards {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.cards_vec())
    }
}

impl IntoIterator for Cards {
    type Item = Card;
    type IntoIter = CardsIter;

    fn into_iter(self) -> CardsIter {
        self.iter()
    }
}

impl FromIterator<Card> for Cards {
    fn from_iter<I: IntoIterator<Item=Card>>(iter: I) -> Cards {
        let mut cards = Cards::empty();
        cards.extend(iter);
        cards
    }
}

impl Extend<Card> for Cards {
    fn extend<I: IntoIterator<Item=Card>>(&mut self, iter: I) {
        for card in iter.into_iter() {
            self.insert(card);
        }
    }
}

#[derive(Copy, Clone)]
pub struct CardsIter {
    inner: Cards,
}

impl CardsIter {
    fn new(cards: Cards) -> CardsIter {
        CardsIter { inner: cards }
    }
}

impl Iterator for CardsIter {
    type Item = Card;

    fn next(&mut self) -> Option<Card> {
        self.inner.min_card()
            .map(|c| {
                self.inner.remove(c);
                c
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }
}

impl ExactSizeIterator for CardsIter {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl FusedIterator for CardsIter {}


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct Card {
    inner: u8,
}

impl Card {
    pub const THREE_OF_CLUBS: Card = Card::new(Three, Clubs);
    pub const JACK_OF_SPADES: Card = Card::new(Jack, Spades);
    pub const THREE_OF_SPADES: Card = Card::new(Three, Spades);
    pub const TWO_OF_DIAMONDS: Card = Card::new(Two, Diamonds);

    pub const fn new(rank: Rank, suit: Suit) -> Card {
        Card { inner: rank as u8 * 4 + suit as u8 }
    }

    pub fn rank(self) -> Rank {
        Rank::from_u8(self.inner / 4)
    }

    pub fn suit(self) -> Suit {
        Suit::from_u8(self.inner % 4)
    }

    pub const fn get_index(self) -> u64 {
        self.inner as u64
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.rank(), self.suit())
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self) // lets just use the Display implementation
    }
}

impl FromStr for Card {
    type Err = ();

    fn from_str(s: &str) -> Result<Card, ()> {
        let rank = s.get(0..1).ok_or(())?.parse()?;
        let suit = s.get(1..).ok_or(())?.parse()?;

        Ok(Card::new(rank, suit))
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum Rank {
    Three = 0,
    Four = 1,
    Five = 2,
    Six = 3,
    Seven = 4,
    Eight = 5,
    Nine = 6,
    Ten = 7,
    Jack = 8,
    Queen = 9,
    King = 10,
    Ace = 11,
    Two = 12,
}

impl Rank {
    fn from_u8(n: u8) -> Rank {
        match n {
            0 => Three,
            1 => Four,
            2 => Five,
            3 => Six,
            4 => Seven,
            5 => Eight,
            6 => Nine,
            7 => Ten,
            8 => Jack,
            9 => Queen,
            10 => King,
            11 => Ace,
            12 => Two,
            _ => panic!("rank index out of bounds {}", n),
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Three => "3",
            Four => "4",
            Five => "5",
            Six => "6",
            Seven => "7",
            Eight => "8",
            Nine => "9",
            Ten => "T",
            Jack => "J",
            Queen => "Q",
            King => "K",
            Ace => "A",
            Two => "2",
        })
    }
}

impl fmt::Debug for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self) // lets just use the Display implementation
    }
}

impl FromStr for Rank {
    type Err = ();

    fn from_str(s: &str) -> Result<Rank, ()> {
        Ok(match s.trim() {
            "3" => Three,
            "4" => Four,
            "5" => Five,
            "6" => Six,
            "7" => Seven,
            "8" => Eight,
            "9" => Nine,
            "T" => Ten,
            "J" => Jack,
            "Q" => Queen,
            "K" => King,
            "A" => Ace,
            "2" => Two,
            _ => return Err(()),
        })
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum Suit {
    Clubs = 0,
    Spades = 1,
    Hearts = 2,
    Diamonds = 3,
}

impl Suit {
    fn from_u8(n: u8) -> Suit {
        match n {
            0 => Clubs,
            1 => Spades,
            2 => Hearts,
            3 => Diamonds,
            _ => panic!("suits index out of bounds {}", n),
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Clubs => "♣",
            Spades => "♠",
            Hearts => "♥",
            Diamonds => "♦",
        })
    }
}

impl fmt::Debug for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self) // lets just use the Display implementation
    }
}

impl FromStr for Suit {
    type Err = ();

    fn from_str(s: &str) -> Result<Suit, ()> {
        Ok(match s.trim() {
            "♣" | "C" => Clubs,
            "♠" | "S" => Spades,
            "♥" | "H" => Hearts,
            "♦" | "D" => Diamonds,
            _ => return Err(()),
        })
    }
}
