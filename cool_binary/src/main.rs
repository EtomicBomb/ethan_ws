use lisp::{Handlers, Node, Expression, Atom, Span};
use std::fs::read_to_string;

const COOL_FILE_PATH: &'static str = "/home/etomicbomb/RustProjects/ethan_ws/cool_binary/cool.lisp";

fn main() {
    let handlers = Handlers::parse(read_to_string(COOL_FILE_PATH).unwrap()).unwrap();

    let request_string = Node {
        expression: Expression::Atom(Atom::ByteVector("cool.txt".as_bytes().to_vec())),
        span: Span::single_byte(0),
    };


    match handlers.eval("http-handler", vec![request_string]) {
        Ok(result) => println!("ok: {}", result),
        Err(e) => println!("{}", handlers.stringify_error(e)),
    }
}
