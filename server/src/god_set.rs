use crate::json::Json;
use std::io::{BufReader, BufRead};
use std::fs::File;
use crate::GOD_SET_PATH;

pub struct GodSet {
    json: String,
}

impl GodSet {
    pub fn new() -> Option<GodSet> {
        let file = BufReader::new(File::open(GOD_SET_PATH).ok()?);

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

        Some(GodSet { json: json.to_string() })
    }

    pub fn cool_vector() -> Option<Vec<(String, String)>> {
        let file = BufReader::new(File::open(GOD_SET_PATH).ok()?);

        file.lines()
            .map(|line| {
                let line = line.ok()?;
                let split: Vec<_> = line.trim_end().split("\t").collect();
                if split.len() != 7 { return None }
                Some((split[5].to_string(), split[6].to_string()))
            }).collect::<Option<_>>()
    }

    pub fn stringify(&self) -> String {
        self.json.clone()
    }

}
