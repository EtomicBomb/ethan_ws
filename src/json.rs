use std::collections::HashMap;
use std::fmt;

#[derive(Clone)]
pub enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Json::Null => write!(f, "null"),
            Json::Boolean(b) => write!(f, "{}", b),
            Json::Number(n) => write!(f, "{}", n),
            Json::String(ref s) => write!(f, "\"{}\"", s),
            Json::Array(ref a) => {
                let maybe_comma = |i| if i < a.len()-1 { "," } else { "" };
                write!(f, "[")?;
                for (i, elem) in a.iter().enumerate() {
                    write!(f, "{}{} ", elem, maybe_comma(i))?;
                }
                write!(f, "]")
            },
            Json::Object(ref m) => {
                let maybe_comma = |i| if i < m.len()-1 { "," } else { "" };
                write!(f, "{{")?;

                for (i, (k, v)) in m.iter().enumerate() {
                    write!(f, "\"{}\": {}{} ", k, v, maybe_comma(i))?;
                }

                write!(f, "}}")
            },
        }
    }
}