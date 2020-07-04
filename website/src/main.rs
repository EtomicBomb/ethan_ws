#![feature(try_trait, is_sorted)]

use std::sync::{Arc, Mutex};
use server::Server;
use std::time::Duration;
use std::path::PathBuf;

mod apps;
use apps::*;

const RESOURCES_PATH: &'static str = "/home/pi/Desktop/server/resources";
const GOD_SET_PATH: &'static str = "/home/pi/Desktop/server/resources/apush/godset.txt";
const VOCABULARY_LOG_PATH: &'static str = "/home/pi/Desktop/server/vocabularyLog.txt";
const PASSWORD_LOG_PATH: &'static str = "/home/pi/Desktop/server/passwordLog.txt";
const PUSOY_PASSING_MODEL_PATH: &'static str = "/home/pi/Desktop/server/passingModel.dat";
const WORD_LIST_PATH: &'static str = "/home/pi/Desktop/server/wordList.txt";

const MAX_HTTP_REQUEST_SIZE: usize = 2048;
const PERIOD_LENGTH: Duration = Duration::from_millis(100);

fn main() {
    let mut server = Server::new("website".into(), PathBuf::from(RESOURCES_PATH), MAX_HTTP_REQUEST_SIZE, PERIOD_LENGTH);

    server.web_socket_add("/filler".into(), Arc::new(Mutex::new(FillerGlobalState::new())));
    server.web_socket_add("/godset".into(), Arc::new(Mutex::new(GodSetGlobalState::new())));
    server.web_socket_add("/tanks".into(), Arc::new(Mutex::new(TanksGlobalState::new())));
    server.web_socket_add("/history".into(), Arc::new(Mutex::new(HistoryGlobalState::new())));
    server.web_socket_add("/arena".into(), Arc::new(Mutex::new(ArenaGlobalState::new())));
    server.web_socket_add("/secure".into(), Arc::new(Mutex::new(SecureGlobalState::new())));
    server.web_socket_add("/pusoy".into(), Arc::new(Mutex::new(PusoyGlobalState::new())));

    server.start();
}
