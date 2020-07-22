use crate::play::{Play, PlayKind};
use crate::cards::Card;
use crate::Cards;

// TODO: since this entire module is based off of this one file, it should be refactored heavily
pub fn all_plays(cards: Cards) -> Vec<Play> {
    let mut plays = Vec::new();

    let mut rank_blocks = RankBlocks::new(cards);

    plays.push(Play::pass());

    // five card hands
    plays.append(&mut rank_blocks.strait_flushes());
    plays.append(&mut rank_blocks.four_of_a_kinds());
    plays.append(&mut rank_blocks.full_houses());
    plays.append(&mut flushes(cards));
    plays.append(&mut rank_blocks.straits);

    // pairs
    plays.append(&mut rank_blocks.pairs());

    // singles
    plays.append(&mut singles(cards));

    plays
}

struct RankBlocks {
    cards: Cards,
    straits: Vec<Play>,
    blocks: [Cards; 13],
}

impl RankBlocks {
    fn new(cards: Cards) -> RankBlocks {
        let mut blocks: [Cards; 13] = [Cards::EMPTY; 13];

        for card in cards.iter() {
            blocks[card.rank() as usize].insert(card);
        }

        RankBlocks {
            cards,
            blocks,
            straits: straits(blocks),
        }
    }

    fn n_of_a_kinds(&self, n: usize) -> Vec<Cards> {
        let mut chunks = Vec::new();

        for &block in self.blocks.iter() {
            if block.len() < n { continue } // this block is useless to us

            chunks.append(&mut permute(block, n));
        }

        chunks
    }

    fn pairs(&self) -> Vec<Play> {
        self.n_of_a_kinds(2).into_iter()
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

    fn four_of_a_kinds(&self) -> Vec<Play> {
        let mut four_of_a_kinds = Vec::new();

        for four_of_a_kind in self.n_of_a_kinds(4) {
            for trash_card in self.cards.iter() {
                if !four_of_a_kind.contains(trash_card) {
                    let mut collection = four_of_a_kind;
                    collection.insert(trash_card);
                    let play = Play::new(PlayKind::FourOfAKind, four_of_a_kind.max_card().unwrap(), collection);
                    four_of_a_kinds.push(play);
                }
            }
        }

        four_of_a_kinds
    }


    fn strait_flushes(&self) -> Vec<Play> {
        self.straits.iter()
            .filter(|strait| strait.cards().all_same_suit())
            .map(|strait| strait.with_kind(PlayKind::StraitFlush))
            .collect()
    }
}

fn singles(cards: Cards) -> Vec<Play> {
    cards.iter()
        .map(|card| Play::new(PlayKind::Single, card, Cards::single(card)))
        .collect()
}


fn straits(rank_blocks: [Cards; 13]) -> Vec<Play> {
    let mut straits = Vec::new();

    let mut blocks = Vec::with_capacity(5);

    for i in 0..13 {
        blocks.clear();
        blocks.extend((i .. i+5).map(|i| rank_blocks[i % 13].cards_vec()));

        strait_from_block(&blocks, &mut straits);
    }

    straits
}


fn flushes(cards: Cards) -> Vec<Play> {
    // collect all of the cards
    let mut suit_blocks: [Cards; 4] = [Cards::EMPTY; 4];

    for card in cards.iter() {
        suit_blocks[card.suit() as usize].insert(card);
    }

    suit_blocks.iter()
        .copied()
        .filter(|b| b.len() >= 5)
        .map(|block| {
            permute(block, 5).into_iter()
                .map(|cs| Play::new(PlayKind::Flush, cs.max_card().unwrap(), cs))
        })
        .flatten()
        .collect()
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
fn counter(base: &[usize], mut f: impl FnMut(&[usize])) {
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