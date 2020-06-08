use std::collections::HashMap;
use std::fmt;
use std::str::{FromStr, from_utf8};

#[derive(Clone, Debug)]
pub enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

impl Json {
    pub fn get_null(&self) -> Option<()> {
        match *self {
            Json::Null => Some(()),
            _ => None,
        }
    }
    pub fn get_bool(&self) -> Option<bool> {
        match *self {
            Json::Boolean(b) => Some(b),
            _ => None,
        }
    }
    pub fn get_number(&self) -> Option<f64> {
        match *self {
            Json::Number(n) => Some(n),
            Json::String(ref s) => s.parse().ok(),
            _ => None,
        }
    }
    pub fn get_string(&self) -> Option<&str> {
        match *self {
            Json::String(ref s) => Some(s.as_str()),
            _ => None,
        }
    }
    pub fn get_array(&self) -> Option<&[Json]> {
        match *self {
            Json::Array(ref a) => Some(a.as_slice()),
            _ => None,
        }
    }
    pub fn get_object(&self) -> Option<&HashMap<String, Json>> {
        match *self {
            Json::Object(ref o) => Some(o),
            _ => None,
        }
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Json::Null => write!(f, "null"),
            Json::Boolean(b) => write!(f, "{}", b),
            Json::Number(n) => write!(f, "{}", n),
            Json::String(ref s) => write!(f, "{}", into_json_string(s)),
            Json::Array(ref a) => {
                let maybe_comma = |i| if i < a.len()-1 { "," } else { "" };
                write!(f, "[")?;
                for (i, elem) in a.iter().enumerate() {
                    write!(f, "{}{}", elem, maybe_comma(i))?;
                }
                write!(f, "]")
            },
            Json::Object(ref m) => {
                let maybe_comma = |i| if i < m.len()-1 { "," } else { "" };
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    write!(f, "\"{}\":{}{}", k, v, maybe_comma(i))?;
                }
                write!(f, "}}")
            },
        }
    }
}

impl FromStr for Json {
    type Err = ();

    fn from_str(s: &str) -> Result<Json, ()> {
        let s = s.trim();

        if let "null" = s {
            Ok(Json::Null)
        } else if let Ok(b) = s.parse::<bool>() {
            Ok(Json::Boolean(b))
        } else if let Ok(n) = s.parse::<f64>() {
            Ok(Json::Number(n))
        } else if let Ok(ret) = parse_json_string(s) {
            Ok(Json::String(ret))
        } else if s.starts_with('[') && s.ends_with(']') {

            Ok(Json::Array(SplitTopLevel::new(&s[1..s.len()-1], b',')
                .filter(|value| !value.chars().all(char::is_whitespace))
                .map(|value| value.parse())
                .collect::<Result<Vec<Json>, ()>>()?
            ))

        } else if s.starts_with('{') && s.ends_with('}') {
            Ok(Json::Object(SplitTopLevel::new(&s[1..s.len()-1], b',')
                .map(|keypair| {
                    let mut a = SplitTopLevel::new(keypair, b':');
                    let key = a.next().ok_or(())?;
                    let value = a.next().ok_or(())?;
                    Ok((parse_json_string(key.trim())?, value.parse()?))
                })
                .collect::<Result<HashMap<String, Json>, ()>>()?
            ))
        } else {
            Err(())
        }
    }
}

struct SplitTopLevel<'a> {
    bytes: &'a [u8],
    split_on: u8,
}

impl<'a> SplitTopLevel<'a> {
    fn new(s: &'a str, split_on: u8) -> SplitTopLevel<'a> {
        SplitTopLevel {
            bytes: s.as_bytes(),
            split_on
        }
    }
}

impl<'a> Iterator for SplitTopLevel<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.bytes.is_empty() { return None }
        let mut bracket_count = 0;
        let mut mustache_count = 0;
        let mut quote_count_even = true;
        let mut char_is_escaped = false;

        for (i, &b) in self.bytes.iter().enumerate() {
            if !char_is_escaped {
                match b {
                    b'[' if quote_count_even => bracket_count += 1,
                    b']' if quote_count_even => bracket_count -= 1,
                    b'{' if quote_count_even => mustache_count += 1,
                    b'}' if quote_count_even => mustache_count -= 1,
                    b'"' => quote_count_even = !quote_count_even,
                    _ if b == self.split_on && quote_count_even && bracket_count == 0 && mustache_count == 0 => {
                        let ret = from_utf8(&self.bytes[..i]).unwrap(); // we were passed in valid utf8
                        self.bytes = &self.bytes[i+1..];
                        return Some(ret)
                    },
                    _ => {},
                }
            }

            char_is_escaped = b == b'\\' && !char_is_escaped;
        }

        if bracket_count == 0 && mustache_count == 0 && quote_count_even {
            let ret = from_utf8(self.bytes).unwrap(); // we were passed in valid utf8
            self.bytes = &[];
            Some(ret)
        } else {
            self.bytes = &[];
            None
        }
    }
}



fn into_json_string(s: &str) -> String {
    let mut ret = String::with_capacity(s.len()); // might be longer but why not

    for c in s.chars() {
        match c {
            '"' => ret.push_str("\\\""),
            '\\' => ret.push_str("\\\\"),
            '\x08' => ret.push_str("\\b"),
            '\x0c' => ret.push_str("\\f"),
            '\n' => ret.push_str("\\n"),
            '\r' => ret.push_str("\\r"),
            '\t' => ret.push_str("\\t"),
            _ if c.is_ascii() && !c.is_ascii_control() => ret.push(c),
            _ => ret.push_str({
                let mut buf = [0u16; 2];
                &match *c.encode_utf16(&mut buf) {
                    [a] => format!(r#"\u{:04X?}"#, a),
                    [a, b] => format!(r#"\u{:04X?}\u{:04X?}"#, a, b),
                    _ => unreachable!(),
                }
            }),
            // we don't encode forward slashes
        }
    }

    format!(r#""{}""#, ret)
}

fn parse_json_string(s: &str) -> Result<String, ()> {
    if !(s.len() > 1 && s.starts_with('"') && s.ends_with('"')) {
        return Err(());
    }

    let mut ret = Vec::new();
    let mut chars = s[1..s.len()-1].chars();

    loop {
        let c = match chars.next() {
            Some(c) => c,
            None => break String::from_utf16(&ret).map_err(|_| ()),
        };

        match c {
            '"' => return Err(()),
            '\\' => match chars.next().ok_or(())? {
                '"' => ret.push(b'"' as u16),
                '\\' => ret.push(b'\\' as u16),
                '/' => ret.push(b'/' as u16),
                'b' => ret.push(b'\x08' as u16),
                'f' => ret.push(b'\x0c' as u16),
                'n' => ret.push(b'\n' as u16),
                'r' => ret.push(b'\r' as u16),
                't' => ret.push(b'\t' as u16),
                'u' => {
                    let mut z = String::new();
                    for _ in 0..4 {
                        z.push(chars.next().ok_or(())?);
                    }
                    ret.push(u16::from_str_radix(&z, 16).map_err(|_| ())?);
                },
                _ => return Err(()),
            },
            _ => ret.push(c as u16),
        }
    }
}
