use std::collections::{HashSet, HashMap};
use std::fs::{read_to_string, File, OpenOptions};
use crate::{GOD_SET_PATH, VOCABULARY_LOG_PATH};
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;
use std::io::{self, Write, BufReader, BufRead};
use super::terms::{Terms, TermId};
use std::str::{ParseBoolError};
use std::option::NoneError;
use std::num::ParseIntError;
use json::{jsont, Json};

pub struct VocabularyModel {
    pub terms: Terms,
    confusion: ConfusionModel,
}


impl VocabularyModel {
    pub fn new() -> VocabularyModel {
        let terms =  read_to_string(GOD_SET_PATH).unwrap().parse().unwrap();

        VocabularyModel {
            confusion: ConfusionModel::new(VOCABULARY_LOG_PATH, &terms).unwrap(),
            terms,
        }
    }

    pub fn log_multiple_choice_answer(&mut self, question: &MultipleChoiceQuestion, answer: usize) {
        self.confusion.log(question.correct_term_id(), question.options[answer]);
    }

    fn terms_in_range(&self, start: (u8, u8), end: (u8, u8)) -> Vec<TermId> {
        self.terms.iter()
            .filter(|&(_, term)| start <= term.location() && term.location() <= end)
            .map(|(id, _)| id)
            .collect()
    }
}

#[allow(unused)]
pub struct FillInTheBlank {
    prompt: TermId,
}

impl FillInTheBlank {
    #[allow(unused)]
    pub fn is_correct(&self, answer: &str, vocab: &VocabularyModel) -> bool {
        fn clean(s: &str) -> Vec<u8> {
            s.bytes()
                .filter(u8::is_ascii_alphanumeric)
                .map(|b| b.to_ascii_lowercase())
                .collect()
        }

        fn is_number(bytes: &[u8]) -> bool {
            bytes.iter().all(u8::is_ascii_digit)
        }

        let a = clean(answer);
        let b = clean(vocab.terms[self.prompt].get_term());
        // we could just do `a == b`, but we'd like to accept more typos

        if is_number(&a) && is_number(&b) {
            a == b
        } else {
            distance(&a, &b) < 2
        }
    }
}

#[derive(Debug)]
pub struct MultipleChoiceQuestion {
    pub options: Vec<TermId>,
    pub correct_index: usize,
}

impl MultipleChoiceQuestion {
    pub fn jsonify(&self, vocabulary: &VocabularyModel) -> Json {
        let definition = vocabulary.terms[self.options[self.correct_index]].get_definition().to_string();
        let terms = Json::Array(self.options.iter()
            .map(|&id| Json::String(vocabulary.terms[id].get_term().to_string()))
            .collect::<Vec<Json>>());

        jsont!({definition: definition, terms: terms})
    }

    fn correct_term_id(&self) -> TermId {
        self.options[self.correct_index]
    }

    pub fn is_correct(&self, answer: usize) -> bool {
        self.correct_index == answer
    }
}

#[derive(Debug)]
pub struct Query {
    start: (u8, u8),
    end: (u8, u8),
    in_range: Vec<TermId>,
    exhausted: HashSet<TermId>,
    question_index: usize, // how many questions have we done so far
    chosen_when: HashMap<TermId, usize>, // when was the last time we choose that question?
}

impl Query {
    pub fn new(start: (u8, u8), end: (u8, u8), vocabulary:  &mut VocabularyModel) -> Option<Query> {
        let in_range = vocabulary.terms_in_range(start, end);

        if in_range.len() < 4 {
            return None;
        }

        let exhausted = HashSet::with_capacity(in_range.len());

        Some(Query { start, end, in_range, exhausted, question_index: 1, chosen_when: HashMap::new() })
    }

    fn select_prompt(&mut self, vocab: &VocabularyModel) -> TermId {
        let prompt_fitness = |prompt_id: &TermId| {
            let term = &vocab.terms[*prompt_id];

            let _last_seen = self.chosen_when.get(prompt_id).copied().unwrap_or(0);
            let _obvious = term.obvious();

            1
        };

        *self.in_range.choose_weighted(&mut thread_rng(), prompt_fitness).unwrap()
    }

    #[allow(unused)] // todo: remove these
    pub fn get_fill_in_the_blank(&mut self, vocab: &VocabularyModel) -> FillInTheBlank {
        FillInTheBlank { prompt: self.select_prompt(vocab) }
    }

    pub fn get_multiple_choice(&mut self, vocab: &VocabularyModel) -> MultipleChoiceQuestion {
        // we should choose our prompt based on the following factors:
        //   * when was the last time we chose that question?
        //   * how obvious is it?
        //   * a dash of randomness

        // we should choose our answers based on
        //   * do they have similar tags
        //   * how frequently is it confused
        //   * a dash of randomness

        // we just need an equation to weigh all of those factors

        let prompt = self.select_prompt(vocab);

        let answer_fitness = |answer: &TermId| {
            let term = &vocab.terms[*answer];

            let tags_equal = term.get_tag() == vocab.terms[prompt].get_tag();
            let confusion = vocab.confusion.confusions_for(prompt, *answer);

            confusion + if tags_equal { 10 } else { 1 }
        };

        let mut options = Vec::with_capacity(4);
        while options.len() < 3 {
            let option = *self.in_range.choose_weighted(&mut thread_rng(), answer_fitness).unwrap();
            if option != prompt && !options.contains(&option) {
                options.push(option);
            }
        }

        let correct_index = thread_rng().gen_range(0, 4);
        options.insert(correct_index, prompt);

        MultipleChoiceQuestion { correct_index, options }
    }


    // pub fn get_multiple_choice_question(&mut self, vocabulary: &mut VocabularyModel) -> MultipleChoiceQuestion {
    //     if self.exhausted.len() == self.in_range.len() { // we exhausted our entire question pool, start repeating
    //         self.exhausted.clear();
    //     }
    //
    //     let definition = loop {
    //         let term = *self.in_range.choose(&mut thread_rng()).unwrap();
    //         let is_new = self.exhausted.insert(term);
    //
    //         if is_new { break term }
    //     };
    //
    //     let mut options = Vec::with_capacity(4);
    //
    //     // get our incorrect option choices
    //     while options.len() < 3 {
    //         let wrong_option = *self.in_range.choose(&mut thread_rng()).unwrap();
    //         if wrong_option != definition && !options.contains(&wrong_option) {
    //             options.push(wrong_option);
    //         }
    //     }
    //
    //     // insert our correct (term, definition) pair
    //     let correct_index = thread_rng().gen_range(0, 4);
    //     options.insert(correct_index, definition);
    //
    //     MultipleChoiceQuestion { correct_index, options }
    // }

}
//
// struct LocalConfusionModel {
//     map: HashMap<(TermId, TermId), (usize, usize)>,
// }
//
// impl FromStr for LocalConfusionModel {
//     type Err = ();
//
//     fn from_str(s: &str) -> Result<LocalConfusionModel, ()> {
//         let mut map = HashMap::new();
//
//         for line in s.lines() {
//             let mut split = line.split("\t");
//
//             let mut get_number = || -> Option<usize> { split.next()?.parse().ok() };
//             let mut get_term_id = || -> Option<TermId> { TermId::from_inner(get_number()?, terms) };
//
//             let wrong = get_term_id().ok_or(())?;
//             let right = get_term_id().ok_or(())?;
//             let was_correct = split.next()?.parse().ok().ok_or(())?;
//
//             map.entry((wrong, right)).or_insert_with(Deal::new).update(was_correct);
//         }
//
//         Ok(LocalConfusionModel { map })
//     }
// }


#[derive(Debug)]
enum ConfusionError {
    IoError(io::Error),
    ParseIntError(ParseIntError),
    ParseBoolError,
    Other,
}

impl From<ParseIntError> for ConfusionError {
    fn from(e: ParseIntError) -> ConfusionError {
        ConfusionError::ParseIntError(e)
    }
}

impl From<io::Error> for ConfusionError {
    fn from(e: io::Error) -> ConfusionError {
        ConfusionError::IoError(e)
    }
}

impl From<NoneError> for ConfusionError {
    fn from(_: NoneError) -> ConfusionError {
        ConfusionError::Other
    }
}

impl From<ParseBoolError> for ConfusionError {
    fn from(_: ParseBoolError) -> ConfusionError {
        ConfusionError::ParseBoolError
    }
}

struct ConfusionModel {
    map: HashMap<(TermId, TermId), usize>, // (correct, incorrect)
    file: File,
}

impl ConfusionModel {
    fn new(path: &str, terms: &Terms) -> Result<ConfusionModel, ConfusionError> {
        let mut map = HashMap::new();

        if let Ok(file) = OpenOptions::new().read(true).create(true).open(path) {
            for line in BufReader::new(file).lines() {
                let line = line?;

                let mut split = line.split("\t");

                let wrong = TermId::from_inner(split.next()?.parse()?, terms)?;
                let right = TermId::from_inner(split.next()?.parse()?, terms)?;

                *map.entry((wrong, right)).or_insert(0) += 1;
            }
        }

        Ok(ConfusionModel { map, file: OpenOptions::new().append(true).open(&path)? })
    }

    fn log(&mut self, right: TermId, wrong: TermId) {
        let _ = writeln!(self.file, "{}\t{}", right.inner(), wrong.inner());
        *self.map.entry((wrong, right)).or_insert(0) += 1;
    }

    fn confusions_for(&self, right: TermId, wrong: TermId) -> usize {
        self.map.get(&(right, wrong)).copied().unwrap_or(0)
    }
}

fn distance(a: &[u8], b: &[u8]) -> usize {
    // easiest implementation from https://en.wikipedia.org/wiki/Levenshtein_distance
    distance_helper(a, b, &mut HashMap::new())
}

fn distance_helper<'a, 'b>(
    a: &'a [u8],
    b: &'b [u8],
    memo: &mut HashMap<(&'a [u8], &'b [u8]), usize>
) -> usize {

    match memo.get(&(a, b)) {
        Some(&result) => result,
        None => {
            let result =
                if a.is_empty() {
                    b.len() // all insertions of b
                } else if b.is_empty() {
                    a.len() // insertions of a
                } else if a[0] == b[0] {
                    distance_helper(&a[1..], &b[1..], memo)
                } else {
                    1+ [
                        (a, &b[1..]), // b[0] was inserted
                        (&a[1..], b), // a[0] was inserted
                        (&a[1..], &b[1..]) // replacement
                    ].iter()
                        .map(|&(a, b)| distance_helper(a, b, memo))
                        .min()
                        .unwrap()
                };

            memo.insert((a, b), result);

            result
        }
    }
}
