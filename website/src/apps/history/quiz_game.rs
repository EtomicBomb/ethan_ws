use crate::apps::history::{GameSpecific, Users};
use crate::apps::history::vocabulary_model::{VocabularyModel, Query, MultipleChoiceQuestion};

use server::{PeerId, Disconnect};
use json::{Json, jsons, jsont};
use std::collections::HashMap;

#[derive(Debug)]
pub struct QuizGame {
    host: PeerId,
    players: Vec<PeerId>,
    query: Query,
    current_question: MultipleChoiceQuestion,
    submitted_answers: HashMap<PeerId, usize>,
    scores: HashMap<PeerId, f64>,
}

impl QuizGame {
    pub fn new(host: PeerId, players: Vec<PeerId>, mut query: Query, vocabulary: &mut VocabularyModel, users: &mut Users) -> QuizGame {
        let current_question = query.get_multiple_choice(vocabulary);
        let question_json = current_question.jsonify(vocabulary);

        for &peer in players.iter().chain(Some(&host)) {
            let _ = users.get_writer(peer).write_string(&jsons!({
                kind: "initialStuff",
                question: (question_json.clone()),
            }));
        }

        QuizGame { host, players, query, submitted_answers: HashMap::new(), scores: HashMap::new(), current_question }
    }

    fn jsonify_scores(&self, users: &Users) -> Json {
        Json::Array(self.players.iter()
            .map(|id| {
                let username = users.get_username(*id).to_string();
                let score = *self.scores.get(id).unwrap_or(&0.0);
                jsont!({username: username, score: score})
            })
            .collect())
    }
}

impl GameSpecific for QuizGame {
    fn receive_message(&mut self, id: PeerId, message: &HashMap<String, Json>, users: &mut Users, vocabulary: &mut VocabularyModel) -> Result<(), Disconnect> {
        match message.get("kind")?.get_string()? {
            "nextQuestion" if id == self.host => {
                // update all of our scores first
                for (&responder, &answer) in self.submitted_answers.iter() {
                    if self.current_question.is_correct(answer) {
                        *self.scores.entry(responder).or_insert(0.0) += 1.0;
                    }
                }

                let new_question = self.query.get_multiple_choice(vocabulary);
                let new_question_json = new_question.jsonify(vocabulary);

                for &player in self.players.iter() {
                    // generate our response
                    let was_correct = self.submitted_answers.get(&player)
                        .map(|&response| self.current_question.is_correct(response))
                        .unwrap_or(false);

                    users.get_writer(player).write_string(&jsons!({
                        kind: "updateStuff",
                        newQuestion: (new_question_json.clone()),
                        wasCorrect: was_correct,
                        score: (self.scores.get(&player).copied().unwrap_or(0.0)),
                    }))?;
                }

                let scores = self.jsonify_scores(users);
                // what message are we gonna send the host
                users.get_writer(self.host).write_string(&jsons!({
                    kind: "updateStuff",
                    newQuestion: (new_question_json.clone()),
                    scores: scores,
                }))?;

                self.current_question = new_question;
                self.submitted_answers.clear();
            },
            "submitAnswer" => if id != self.host {
                let response = message.get("answer")?.get_number()? as usize;
                vocabulary.log_multiple_choice_answer(&self.current_question, response);
                self.submitted_answers.insert(id, response);
            },
            _ => return Err(Disconnect),
        }

        Ok(())
    }

    fn periodic(&mut self, _users: &mut Users, _vocabulary: &mut VocabularyModel) {}

    fn leave(&mut self, id: PeerId, users: &mut Users, _vocabulary: &mut VocabularyModel) -> bool {
        let was_host = id == self.host;

        if was_host {
            let message = jsons!({kind:"hostAbandoned"});
            for &player in self.players.iter() {
                let _ = users.get_writer(player).write_string(&message);
            }

        } else {
            self.players.remove_item(&id);
            self.scores.remove(&id);
            self.submitted_answers.remove(&id);

        }

        was_host
    }
}