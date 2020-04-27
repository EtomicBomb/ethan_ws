use std::collections::HashMap;
use pest::iterators::{Pairs, Pair};
use pest::Parser;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "http_request.pest"]
struct HttpRequestParser;

#[derive(Debug)]
pub struct HttpRequest {
    request_type: RequestType,
    resource_location: String,
    pub headers: HashMap<String, String>,
}

impl FromStr for HttpRequest {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        let mut pairs: Pairs<Rule> = HttpRequestParser::parse(Rule::TOP, s)?;
        let mut pairs_iter: Pairs<Rule> = pairs.next().unwrap().into_inner();

        let request_type = pairs_iter.next().unwrap().as_str().parse().unwrap();

        let resource_location = pairs_iter.next().unwrap().as_str().to_string();

        let headers = pairs_iter
            .filter(|p| p.as_rule() != Rule::EOI)
            .map(|pair: Pair<Rule>| {
                let mut iter = pair.into_inner();
                let name = iter.next().unwrap().as_str().to_string();
                let value = iter.next().unwrap().as_str().to_string();
                (name, value)
            })
            .collect();

        Ok(HttpRequest { request_type, resource_location, headers })
    }
}

impl From<pest::error::Error<Rule>> for ParseError {
    fn from(e: pest::error::Error<Rule>) -> ParseError {
        ParseError::PestError(e)
    }
}

#[derive(Debug)]
pub enum ParseError {
    PestError(pest::error::Error<Rule>),
}

#[derive(Debug)]
enum RequestType {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Options,
    Connect,
    Patch,
}
impl FromStr for RequestType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(match s {
            "GET" => RequestType::Get,
            "HEAD" => RequestType::Head,
            "POST" => RequestType::Post,
            "PUT" => RequestType::Put,
            "DELETE" => RequestType::Delete,
            "TRACE" => RequestType::Trace,
            "OPTIONS" => RequestType::Options,
            "CONNECT" => RequestType::Connect,
            "PATCH" => RequestType::Patch,
            _ => return Err(()),
        })
    }
}
