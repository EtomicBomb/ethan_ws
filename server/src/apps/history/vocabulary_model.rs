use std::collections::{HashMap, HashSet};
use std::fs::{read_to_string, File, OpenOptions};
use crate::{GOD_SET_PATH, VOCABULARY_LOG_PATH};
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;
use json::{Json, jsont};
use std::io::{Write, BufWriter};
use std::fmt::Write as FmtWrite;
use std::{fmt, io};

pub struct VocabularyModel {
    terms: HashMap<TermId, Term>,
    statistics_file: BufWriter<File>,
}


impl VocabularyModel {
    pub fn new() -> Option<VocabularyModel> {
        let file = read_to_string(GOD_SET_PATH).ok()?;

        let terms = file.lines().zip(1..)
            .map(|(line, i)| Some((TermId(i), Term::from_line(line)?)))
            .collect::<Option<HashMap<TermId, Term>>>()?;

        let statistics_file = BufWriter::new(OpenOptions::new().create(true).append(true).open(VOCABULARY_LOG_PATH).unwrap());

        Some(VocabularyModel { terms, statistics_file })
    }

    pub fn log_multiple_choice_answer(&mut self, question: &MultipleChoiceQuestion, answer: usize) {
        question.stringify(&mut self.statistics_file).unwrap();
        writeln!(self.statistics_file, "|{}", answer).unwrap();
        self.statistics_file.flush().unwrap();
    }

    // pub fn get_query(&self, start: (u8, u8), end: (u8, u8)) -> Query {
    //     Query { start, end }
    // }

    // pub fn query_is_valid(&self, query: Query) -> bool {
    //     self.terms_in_range(query).len() > 4
    // }

    // pub fn get_multiple_choice_question(&self, query: Query) -> MultipleChoiceQuestion {
    //     let in_range = self.terms_in_range(query);
    //
    //     let mut options: Vec<TermId> = in_range.choose_multiple(&mut thread_rng(), 4).copied().collect();
    //     assert_eq!(options.len(), 4);
    //
    //     options.shuffle(&mut thread_rng());
    //
    //     let correct = thread_rng().gen_range(0, 4);
    //
    //     MultipleChoiceQuestion { correct, options }
    // }

    fn terms_in_range(&self, start: (u8, u8), end: (u8, u8)) -> Vec<TermId> {
        self.terms.iter()
            .filter(|&(_, term)| start <= term.location() && term.location() <= end)
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
    pub fn stringify(&self, buf: &mut impl Write) -> io::Result<()> {
        write!(buf, "{}#", self.correct)?;

        let maybe_colon = |i| if i+1 == self.options.len() { "" } else { ":" };
        for (i, term_id) in self.options.iter().enumerate() {
            write!(buf, "{}{}", term_id.0, maybe_colon(i))?;
        }

        Ok(())
    }

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

#[derive(Debug)]
pub struct Query {
    start: (u8, u8),
    end: (u8, u8),
    in_range: Vec<TermId>,
    so_far: HashSet<TermId>,
}

impl Query {
    pub fn new(start: (u8, u8), end: (u8, u8), vocabulary:  &mut VocabularyModel) -> Option<Query> {
        let in_range = vocabulary.terms_in_range(start, end);

        if in_range.len() < 4 {
            return None;
        }

        let so_far = HashSet::with_capacity(in_range.len());

        Some(Query { start, end, in_range, so_far })
    }

    pub fn get_multiple_choice_question(&mut self, vocabulary: &mut VocabularyModel) -> MultipleChoiceQuestion {
        if self.so_far.len() == self.in_range.len() { // we exhausted our entire question pool, start repeating
            self.so_far.clear();
        }

        let definition = loop {
            let term = *self.in_range.choose(&mut thread_rng()).unwrap();
            let is_new = self.so_far.insert(term);

            if is_new { break term }
        };

        let mut options: Vec<TermId> = self.in_range.choose_multiple(&mut thread_rng(), 3).copied().collect();
        assert_eq!(options.len(), 3);

        let correct = thread_rng().gen_range(0, 4);
        options.insert(correct, definition);

        MultipleChoiceQuestion { correct, options }
    }

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
