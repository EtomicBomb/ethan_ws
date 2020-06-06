use json::{jsont, jsons};

const _COOL_FILE_PATH: &'static str = "/home/etomicbomb/RustProjects/ethan_ws/cool_binary/cool.lisp";

fn main() {
    // json!("hello");

    dbg!(jsons!({
        hello: [null, 1, "hello", false],
        whats: "up",
        my: "dog"
    }));
}

