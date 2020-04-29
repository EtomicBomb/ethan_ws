use std::fs::read_to_string;

const GOD_SET_PATH: &'static str = "/home/pi/Desktop/server/resources/apush/godset.txt";

pub struct GodSet {
    raw_text: String,
}

impl GodSet {
    pub fn new() -> Option<GodSet> {
        let raw_text = read_to_string(GOD_SET_PATH).ok()?;
        Some(GodSet { raw_text })
    }

    pub fn raw_bytes(&self) -> Vec<u8> {
        // TODO: deliver json
        self.raw_text.as_bytes().to_vec()
    }
}
