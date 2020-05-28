
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

fn to_base64_char(n: u8) -> char {
    match n {
        0..=25 =>  (n + b'A') as char,
        26..=51 => (n - 26 + b'a') as char,
        52..=61 => (n - 52 + b'0') as char,
        62 => '+',
        63 => '/',
        _ => panic!("{} is not in the range [0, 63]", n),
    }
}

fn ceiling_divide(a: usize, b: usize) -> usize {
    if a % b == 0 {
        a / b
    } else {
        1 + a / b
    }
}