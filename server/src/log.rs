use std::fs::{File, OpenOptions};
use chrono::{Local};
use std::fmt;
use std::io::Write;
use std::sync::Mutex;

use lazy_static::lazy_static;
use crate::LOG_FILE_PATH;

lazy_static! {
    static ref LOG_FILE: Mutex<File> = Mutex::new(OpenOptions::new().append(true).open(LOG_FILE_PATH).unwrap());
}

#[macro_export]
macro_rules! log {
    ($($args:tt)*) => {{
        use crate::log::log_fn;
        log_fn(format_args!($($args)*));
    }};
}

pub fn log_fn(args: fmt::Arguments) {
    let time = Local::now().format("[%A, %B %d, %Y %I:%M:%S%P] ").to_string();
    let mut file = LOG_FILE.lock().unwrap();
    file.write(time.as_bytes()).unwrap();
    file.write_fmt(args).unwrap();
    file.write(b"\n").unwrap();
}