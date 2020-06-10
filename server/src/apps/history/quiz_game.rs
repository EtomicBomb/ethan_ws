use crate::apps::history::{GameSpecific, Users};
use crate::apps::PeerId;
use json::{Json, jsons, jsont};
use std::collections::HashMap;
use crate::apps::Drop;
use crate::apps::history::vocabulary_model::{TermId, VocabularyModel, Query, MultipleChoiceQuestion};

#[derive(Debug)]
pub struct QuizGame {
    host: PeerId,
    peers: Vec<PeerId>,
    query: Query,
    current_question: MultipleChoiceQuestion,
    submitted_answers: HashMap<PeerId, usize>,
    scores: HashMap<PeerId, f64>,
}

impl QuizGame {
    pub fn new(host: PeerId, peers: Vec<PeerId>, query: Query, vocabulary: &mut VocabularyModel, users: &mut Users) -> QuizGame {
        let current_question = vocabulary.get_multiple_choice_question(query);
        let question_json = current_question.jsonify(vocabulary);

        for &peer in peers.iter() {
            let _ = users.get_writer(peer).write_string(&jsons!({
                kind: "initialStuff",
                question: (question_json.clone()),
            }));
        }

        let _ = users.get_writer(host).write_string(&jsons!({
            kind: "initialStuff",
        }));

        QuizGame { host, peers, query, submitted_answers: HashMap::new(), scores: HashMap::new(), current_question }
    }
}

impl GameSpecific for QuizGame {
    fn receive_message(&mut self, id: PeerId, message: &HashMap<String, Json>, users: &mut Users, vocabulary: &mut VocabularyModel) -> Result<(), Drop> {
        match message.get("kind")?.get_string()? {
            "nextQuestion" if id == self.host => {
                // update all of our scores first
                for (&responder, &answer) in self.submitted_answers.iter() {
                    if self.current_question.is_correct(answer) {
                        *self.scores.entry(responder).or_insert(0.0) += 1.0;
                    }
                }

                let new_question = vocabulary.get_multiple_choice_question(self.query);
                let new_question_json = new_question.jsonify(vocabulary);

                for &peer in self.peers.iter() {
                    // generate our response
                    let was_correct = self.submitted_answers.get(&peer)
                        .map(|&response| self.current_question.is_correct(response))
                        .unwrap_or(false);

                    let score = *self.scores.get(&peer).unwrap_or(&0.0);

                    users.get_writer(peer).write_string(&jsons!({
                        kind: "updateStuff",
                        newQuestion: (new_question_json.clone()),
                        wasCorrect: was_correct,
                        score: score,
                    }))?;
                }

                self.current_question = new_question;
                self.submitted_answers.clear();
            },
            "submitAnswer" => if id != self.host {
                let response = message.get("answer")?.get_number()?;
                self.submitted_answers.insert(id, response as usize);
            },
            _ => return Err(Drop),
        }

        Ok(())
    }

    fn periodic(&mut self, users: &mut Users, vocabulary: &mut VocabularyModel) {

    }

    fn leave(&mut self, id: PeerId, users: &mut Users, vocabulary: &mut VocabularyModel) -> bool {
        todo!()
    }
}