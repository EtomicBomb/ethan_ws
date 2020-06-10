use std::collections::HashMap;
use std::fs::{read_to_string};
use crate::GOD_SET_PATH;
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;
use json::{Json, jsont};

pub struct VocabularyModel {
    terms: HashMap<TermId, Term>,
}


impl VocabularyModel {
    pub fn new() -> Option<VocabularyModel> {
        let file = read_to_string(GOD_SET_PATH).ok()?;

        let terms = file.lines().zip(1..)
            .map(|(line, i)| Some((TermId(i), Term::from_line(line)?)))
            .collect::<Option<HashMap<TermId, Term>>>()?;

        Some(VocabularyModel { terms })
    }

    pub fn get_query(&self, start: (u8, u8), end: (u8, u8)) -> Query {
        Query { start, end }
    }

    pub fn query_is_valid(&self, query: Query) -> bool {
        self.terms_in_range(query).len() > 4
    }

    pub fn get_multiple_choice_question(&self, query: Query) -> MultipleChoiceQuestion {
        let in_range = self.terms_in_range(query);

        let mut options: Vec<TermId> = in_range.choose_multiple(&mut thread_rng(), 4).copied().collect();
        assert_eq!(options.len(), 4);

        options.shuffle(&mut thread_rng());

        let correct = thread_rng().gen_range(0, 4);

        MultipleChoiceQuestion { correct, options }
    }

    fn terms_in_range(&self, query: Query) -> Vec<TermId> {
        self.terms.iter()
            .filter(|&(_, term)| query.start <= term.location() && term.location() <= query.end)
            .map(|(&id, _)| id)
            .collect()
    }
}

#[derive(Debug)]
pub struct MultipleChoiceQuestion {
    options: Vec<TermId>,
    correct: usize,
}

impl MultipleChoiceQuestion {
    pub fn jsonify(&self, vocabulary: &VocabularyModel) -> Json {

        let definition = vocabulary.terms[&self.options[self.correct]].definition.clone();
        let terms = Json::Array(self.options.iter()
            .map(|id| Json::String(vocabulary.terms[id].term.clone()))
            .collect::<Vec<Json>>());

        jsont!({definition: definition, terms: terms})
    }

    pub fn is_correct(&self, answer: usize) -> bool {
        self.correct == answer
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Query {
    start: (u8, u8),
    end: (u8, u8),
}

#[derive(Clone)]
pub struct Term {
    chapter: u8,
    section: u8,
    year_start: u16,
    year_end: u16,
    social: bool,
    political: bool,
    economic: bool,
    term: String,
    definition: String,
}

impl Term {
    fn from_line(line: &str) -> Option<Term> {
        let mut split = line.trim_end().split("\t");
        Some(Term {
            chapter: split.next()?.parse().ok()?,
            section: split.next()?.parse().ok()?,
            year_start: split.next()?.parse().ok()?,
            year_end: split.next()?.parse().ok()?,
            social: split.next()?.parse().ok()?,
            political: split.next()?.parse().ok()?,
            economic: split.next()?.parse().ok()?,
            term: split.next()?.to_string(),
            definition: split.next()?.to_string(),
        })
    }

    fn location(&self) -> (u8, u8) {
        (self.chapter, self.section)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TermId(u32);
