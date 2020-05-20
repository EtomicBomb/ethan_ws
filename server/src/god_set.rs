use crate::json::Json;
use std::io::{BufReader, BufRead};
use crate::GOD_SET_PATH;

pub struct GodSet {
    json: Vec<u8>,
}

impl GodSet {
    pub fn new() -> Option<GodSet> {
        let file = BufReader::new(std::fs::File::open(GOD_SET_PATH).ok()?);

        let json = Json::Array(file.lines()
            .map(|line| {
                let line = line.ok()?;
                let mut split = line.trim_end().split("\t");
                let year_start: u16 = split.next()?.parse().ok()?;
                let year_end: u16 = split.next()?.parse().ok()?;
                let social: bool = split.next()?.parse().ok()?;
                let political: bool = split.next()?.parse().ok()?;
                let economic: bool = split.next()?.parse().ok()?;
                let term = split.next()?.to_string();
                let definition = split.next()?.to_string();

                Some(Json::Object([
                    ("yearStart", Json::Number(year_start as f64)),
                    ("yearEnd", Json::Number(year_end as f64)),
                    ("social", Json::Boolean(social)),
                    ("political", Json::Boolean(political)),
                    ("economic", Json::Boolean(economic)),
                    ("term", Json::String(term)),
                    ("definition", Json::String(definition)),
                ].iter().map(|(a, b)| (a.to_string(), b.clone())).collect()))
            })
            .collect::<Option<Vec<Json>>>()?);

        Some(GodSet { json: json.to_string().into_bytes() })
    }

    pub fn raw_bytes(&self) -> Vec<u8> {
        self.json.clone()
    }

}
