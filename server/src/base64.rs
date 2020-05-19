#![allow(dead_code)]

pub fn to_base64(bytes: &[u8]) -> String {
    // sets of three bytes are converted into sets of 3 base 64 characters
    let mut ret = String::with_capacity(4*ceiling_divide(bytes.len(), 3));

    for array in bytes.chunks(3) {
        let sextets = match *array {
            [a, b, c] => [Some(a>>2),  Some((a&0b11) << 4 | b >> 4), Some((b&0b1111) << 2 | c>>6), Some(c&0b111111)],
            [a, b] =>         [Some(a>>2), Some((a&0b11) << 4 | b >> 4), Some((b&0b1111) << 2),        None],
            [a] =>                [Some(a>>2), Some((a&0b11) << 4),          None,                         None],
            _ => unreachable!(),
        };

        for &maybe_byte in sextets.iter() {
            ret.push(match maybe_byte {
                Some(byte) => to_base64_char(byte),
                None => '=',
            });
        }
    }

    ret
}

pub fn from_base64(s: &str) -> Vec<u8> {
    assert_eq!(s.len() % 4, 0);
    assert!(s.as_bytes().is_ascii());

    let mut ret = Vec::with_capacity(3*ceiling_divide(s.len(), 4));

    for sextets in s.as_bytes().chunks(4) {
        let (b1, b2, b3, b4) = match *sextets {
            [b1, b2, b3, b4] => (b1, b2, b3, b4),
            _ => unreachable!(),
        };

        let v = |n| numeric_value(n as char);

        dbg!(b1, b2, b3, b4);

        if b3 == b'=' {
            ret.push(v(b1) << 2 | v(b2) >> 4);
        } else if b4 == b'=' {
            ret.push(v(b1) << 2 | v(b2) >> 4);
            ret.push((v(b2)&0b1111) << 4 | v(b3) >> 2);
        } else {
            ret.push(v(b1) << 2 | v(b2) >> 4);
            ret.push((v(b2)&0b1111) << 4 | v(b3) >> 2);
            ret.push((v(b3)&0b11) << 6 | v(b4));
        }
    }

    ret
}

pub fn to_base64_char(n: u8) -> char {
    match n {
        0..=25 =>  (n + b'A') as char,
        26..=51 => (n - 26 + b'a') as char,
        52..=61 => (n - 52 + b'0') as char,
        62 => '+',
        63 => '/',
        _ => panic!("{} is not in the range [0, 63]", n),
    }
}

pub fn numeric_value(c: char) -> u8 {
    match c {
        'A'..='Z' =>  c as u8 - b'A',
        'a'..='z' => (c as u8 - b'a') + 26,
        '0'..='9' => (c as u8 - b'0') + 52,
        '+' => 62,
        '/' => 63,
        _ => panic!("{} is not a character in base64", c),
    }
}


pub fn ceiling_divide(a: usize, b: usize) -> usize {
    if a % b == 0 {
        a / b
    } else {
        1 + a / b
    }
}