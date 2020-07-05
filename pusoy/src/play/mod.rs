use serde::{Deserialize, Serialize};

pub mod finder;
use finder::Finder;

use crate::cards::Card;
use std::cmp::Ordering;
use crate::Cards;


#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Deserialize, Serialize)]
pub enum PlayKind {
    Pass = 0,
    Single = 1,
    Pair = 2,

    Strait = 3,
    Flush = 4,
    FullHouse = 5,
    FourOfAKind = 6,
    StraitFlush = 7,
}

impl PlayKind {
    fn len(self) -> usize {
        match self {
            PlayKind::Pass => 0,
            PlayKind::Single => 1,
            PlayKind::Pair => 2,
            PlayKind::Strait => 5,
            PlayKind::Flush => 5,
            PlayKind::FullHouse => 5,
            PlayKind::FourOfAKind => 5,
            PlayKind::StraitFlush => 5,
        }
    }

}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize, Serialize)]
pub struct Play {
    cards: Cards,
    kind: PlayKind,
    ranking_card: Option<Card>,
}

impl Ord for Play {
    fn cmp(&self, other: &Play) -> Ordering {
        if self.is_pass() { unimplemented!() }

        self.kind.cmp(&other.kind)
            .then_with(|| self.ranking_card.unwrap().cmp(&other.ranking_card.unwrap()))
    }
}

impl PartialOrd for Play {
    fn partial_cmp(&self, other: &Play) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


impl Play {
    pub fn pass() -> Play {
        Play {
            cards: Cards::empty(),
            kind: PlayKind::Pass,
            ranking_card: None,
        }
    }

    pub fn new(kind: PlayKind, ranking_card: Card, cards: Cards) -> Play {
        Play {
            cards,
            kind,
            ranking_card: Some(ranking_card),
        }
    }

    pub fn infer_from_cards(cards: Cards) -> Option<Play> {
        Finder::new(cards).infer()
    }

    #[inline]
    pub fn is_pass(&self) -> bool {
        self.kind == PlayKind::Pass
    }

    #[inline]
    pub fn len_eq(&self, other: &Play) -> bool {
        self.kind.len() == other.kind.len()
    }

    #[inline]
    pub fn can_play_on(&self, other: &Play) -> bool {
        if self.is_pass() { return true }

        if self.kind.len() != other.kind.len() {
            false
        } else if self.kind != other.kind {
            self.kind > other.kind
        } else {
            self.ranking_card.unwrap() > other.ranking_card.unwrap()
        }
    }

    #[inline]
    pub fn doesnt_contain(&self, card: Card) -> bool {
        !self.cards.contains(card)
    }


    pub fn kind(&self) -> PlayKind {
        self.kind
    }

    #[inline]
    pub fn ranking_card(&self) -> Option<Card> {
        self.ranking_card
    }


    #[inline]
    pub fn cards(&self) -> Cards {
        self.cards
    }

    pub fn replace_kind(&mut self, kind: PlayKind) {
        self.kind = kind;
    }
}

