#![feature(try_trait, vec_remove_item, is_sorted)]

extern crate web_socket;
extern crate json;
extern crate lisp;
extern crate http;

use std::net::{TcpListener};
use std::thread;
use std::sync::{Arc};

mod apps;
mod util;
mod http_handler;

use crate::apps::{CoolStuff};
use std::time::Duration;

const MAX_HTTP_REQUEST_SIZE: usize = 2048;
const RESOURCES_ROOT: &'static str = "/home/pi/Desktop/server/resources";
const GOD_SET_PATH: &'static str = "/home/pi/Desktop/server/resources/apush/godset.txt";
const VOCABULARY_LOG_PATH: &'static str = "/home/pi/Desktop/server/vocabularyLog.txt";
const PASSWORD_LOG_PATH: &'static str = "/home/pi/Desktop/server/passwordLog.txt";
const PUSOY_PASSING_MODEL_PATH: &'static str = "/home/pi/Desktop/server/passingModel.dat";
const WORD_LIST_PATH: &'static str = "/home/pi/Desktop/server/wordList.txt";

const WEBSOCKET_SECURE_KEY_MAGIC_NUMBER: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
const PERIOD_LENGTH: Duration = Duration::from_millis(100);

fn main() {
    let global = Arc::new(CoolStuff::new().unwrap());
    let global_clone = Arc::clone(&global);

    thread::Builder::new().name("server_listener".into()).spawn(move || {

        for tcp_stream in TcpListener::bind("0.0.0.0:8080").unwrap().incoming() {
            if let Ok(tcp_stream) = tcp_stream {
                global_clone.handle_new_connection(tcp_stream);
            }
        }

    }).unwrap();

    // our periodic loop
    loop {
        global.periodic();
        thread::sleep(PERIOD_LENGTH);
    }
}