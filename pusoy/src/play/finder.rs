use crate::play::{Play, PlayKind};
use crate::cards::Card;
use crate::Cards;

fn rank_blocks(cards: Cards) -> [Cards; 13] {
    let mut blocks: [Cards; 13] = [Cards::EMPTY; 13];

    for card in cards.iter() {
        blocks[card.rank() as usize].insert(card);
    }

    blocks
}

fn flushes(cards: Cards) -> Vec<Play> {
    // collect all of the cards
    let mut suit_blocks: [Cards; 4] = [Cards::EMPTY; 4];

    for card in cards.iter() {
        suit_blocks[card.suit() as usize].insert(card);
    }

    // now, we just need to compute all of the flushes
    let mut chunks = Vec::new();

    for &block in suit_blocks.iter() {
        if block.len() < 5 {
            continue;
        }

        chunks.extend(
            permute(block, 5)
                .into_iter()
                .map(|cs| Play::new(PlayKind::Flush, cs.max_card().unwrap(), cs)),
        );
    }

    chunks
}

// i will probably refactor this to be optimized for the types of queries we are giving finder
// currently, we want to support the operations
//      all_plays
//      infer_from_cards - we only want to return the type of hand corresponding to the number of cards we give

#[derive(Clone, Debug)]
pub struct Finder {
    cards: Cards,
    rank_blocks: [Cards; 13],
    flushes: Vec<Play>, // we store the flushes, because a `suit_blocks` data structure would be useless for anything else
}

impl Finder {
    pub fn new(cards: Cards) -> Finder {
        let rank_blocks = rank_blocks(cards);
        let flushes = flushes(cards);

        Finder {
            cards,
            rank_blocks,
            flushes,
        }
    }

    pub fn all_plays(&self) -> Vec<Play> {
        let mut plays = Vec::new();

        // five card hands
        plays.append(&mut self.strait_flushes());
        plays.append(&mut self.four_of_a_kinds());
        plays.append(&mut self.full_houses());
        plays.append(&mut self.flushes());
        plays.append(&mut self.straits());

        // pairs
        plays.append(&mut self.pairs());

        // singles
        plays.append(&mut self.singles());

        plays
    }

    pub fn infer(&self) -> Option<Play> {
        Some(match self.cards.len() {
            0 => Play::pass(),
            1 => Play::new(PlayKind::Single, self.cards.max_card().unwrap(), self.cards),
            2 => Play::new(PlayKind::Pair, self.cards.max_card().unwrap(), if self.cards.all_same_rank() { self.cards } else { return None }, ),
            5 => self.max_five_of_a_kind()?,
            _ => return None,
        })
    }

    fn max_five_of_a_kind(&self) -> Option<Play> {
        let strait_flushes = self.strait_flushes();
        if !strait_flushes.is_empty() {
            return strait_flushes.iter().max().cloned();
        }

        let four_of_a_kinds = self.four_of_a_kinds();
        if !four_of_a_kinds.is_empty() {
            return four_of_a_kinds.iter().max().cloned();
        }
        
        let full_houses = self.full_houses();
        if !full_houses.is_empty() {
            return full_houses.iter().max().cloned();
        }

        let flushes = self.flushes();
        if !flushes.is_empty() {
            return flushes.iter().max().cloned();
        }

        let straits = self.straits();
        if !straits.is_empty() {
            return straits.iter().max().cloned();
        }

        None
    }

    fn singles(&self) -> Vec<Play> {
        self.cards.iter()
            .map(|card| {
                let mut cards = Cards::empty();
                cards.insert(card);
                Play::new(PlayKind::Single, card, cards)
            })
            .collect()
    }


    fn n_of_a_kinds(&self, n: usize) -> Vec<Cards> {
        let mut chunks = Vec::new();

        for &block in self.rank_blocks.iter() {
            if block.len() < n { continue } // this block is useless to us

            chunks.append(&mut permute(block, n));
        }

        chunks
    }

    fn strait_flushes(&self) -> Vec<Play> {
        let mut strait_flushes = Vec::new();

        for mut strait in self.straits() {
            if strait.cards().all_same_suit() {
                strait.replace_kind(PlayKind::StraitFlush);
                strait_flushes.push(strait);
            }
        }

        strait_flushes
    }

    fn flushes(&self) -> Vec<Play> {
        self.flushes.clone()
    }

    fn four_of_a_kinds(&self) -> Vec<Play> {
        // in pusoy, the four of a kind is played with a trash card

        let mut four_of_a_kinds = Vec::new();

        for four_of_a_kind in self.n_of_a_kinds(4) {
            for card in self.cards.iter() {
                if !four_of_a_kind.contains(card) {
                    let mut collection = four_of_a_kind;
                    collection.insert(card);
                    let play = Play::new(PlayKind::FourOfAKind, four_of_a_kind.max_card().unwrap(), collection);
                    four_of_a_kinds.push(play);
                }
            }
        }

        four_of_a_kinds
    }

    fn pairs(&self) -> Vec<Play> {
        self.n_of_a_kinds(2)
            .into_iter()
            .map(|cards| Play::new(PlayKind::Pair, cards.max_card().unwrap(), cards))
            .collect()
    }

    fn full_houses(&self) -> Vec<Play> {
        let mut full_houses = Vec::new();
        let pairs = self.n_of_a_kinds(2);

        for three_of_a_kind in self.n_of_a_kinds(3) {
            for &pair in pairs.iter() {
                if three_of_a_kind.is_disjoint(pair) {
                    let mut collection = pair;
                    collection.insert_all(three_of_a_kind);
                    let play = Play::new(PlayKind::FullHouse, three_of_a_kind.max_card().unwrap(), collection);
                    full_houses.push(play);
                }
            }
        }

        full_houses
    }

    fn straits(&self) -> Vec<Play> {
        let mut straits = Vec::new();

        let mut blocks = Vec::with_capacity(5);

        for i in 0..13 {
            blocks.clear();
            blocks.extend((i .. i+5).map(|i| self.rank_blocks[i % 13].cards_vec()));

            strait_from_block(&blocks, &mut straits);
        }

        straits
    }
}

fn strait_from_block(
    blocks: &[Vec<Card>],
    straits: &mut Vec<Play>,
) {
    let base: Vec<usize> = blocks.iter().map(|b| b.len()).collect();

    let f = |x: &[usize]| {
        let entry: Cards = blocks.iter().zip(x.iter())
            .map(|(block, &i)| block[i])
            .collect();

        let play = Play::new(PlayKind::Strait, entry.max_card().unwrap(), entry);
        straits.push(play);
    };

    counter(&base, f);
}

fn permute(cards: Cards, len: usize) -> Vec<Cards> {
    permute_helper(&cards.cards_vec(), len).into_iter()
        .map(|cards| cards.into_iter().collect())
        .collect()
}

fn permute_helper(list: &[Card], n: usize) -> Vec<Vec<Card>> {
    assert!(list.len() >= n);
    let mut ret = Vec::new();

    if list.len() == n {
        ret.push(list.to_vec());
    } else if n == 1 {
        ret.extend(list.iter().map(|i| vec![i.clone()]));
    } else {
        for i in 0..=list.len() - n {
            let results = permute_helper(&list[i + 1..], n - 1);

            for mut r in results {
                r.insert(0, list[i].clone());
                ret.push(r);
            }
        }
    }

    ret
}

#[inline]
pub fn counter(base: &[usize], mut f: impl FnMut(&[usize])) {
    // a generalized version of counting in an arbitrary base
    // calls f on each number generated in the count
    // for example, counter(&[2, 2, 2], f) calls f on:
    //      &[0, 0, 0]
    //      &[1, 0, 0]
    //      &[0, 1, 0]
    //      &[1, 1, 0]
    //      etc.

    let len = base.len();

    let mut x = vec![0; len];

    let iter_count: usize = base.iter().product();

    for _ in 0..iter_count {
        f(&x);

        // try to "add one"
        for i in 0..len {
            if x[i] < base[i] - 1 {
                x[i] += 1;
                break;
            }

            x[i] = 0;
        }
    }
}