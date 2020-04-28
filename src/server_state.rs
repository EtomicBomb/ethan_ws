use std::io::{Write, BufReader, BufRead};
use crate::frame::{Frame, FrameType};
use crate::tcp_halves::TcpWriter;
use crate::http_request_parse::HttpRequest;
use crate::base64::to_base64;
use sha1::Sha1;
use crate::http_handler::get_response_to_http;
use std::ops::{RangeInclusive};

const WEBSOCKET_SECURE_KEY_MAGIC_NUMBER: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

// TODO: add SPICE
struct GodSet {
    inner: Vec<(String, String, RangeInclusive<u16>, bool, bool, bool)>,
}

impl GodSet {
    fn new() -> GodSet {
        // todo: make this tiype return option
        match GodSet::get_vec() {
            Some(inner) => GodSet { inner },
            None => GodSet { inner: Vec::new() }
        }
    }

    fn get_vec() -> Option<Vec<(String, String, RangeInclusive<u16>, bool, bool, bool)>> {
        let file = BufReader::new(std::fs::File::open("/home/pi/Desktop/server/resources/apush/godset.txt").ok()?);

        file.lines()
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
                Some((term, definition, year_start..=year_end, social, political, economic))
            })
            .collect()
    }

    fn search(&self, keyword: Option<&str>, search_range: Option<RangeInclusive<u16>>, search_s: bool, search_p: bool, search_e: bool) -> Vec<(String, String)> {
        self.inner.iter()
            .filter(|&&(ref term, ref def, ref range, s, p, e)| {
                let text_contains = match keyword {
                    Some(keyword) => term.contains(keyword) || def.contains(keyword),
                    None => true,
                };
                let range_contains = match search_range {
                    Some(ref search_range) => search_range.contains(range.start()) || search_range.contains(range.end()),
                    None => true,
                };

                let themes_match = (search_s || !s) && (search_p || !p) && (search_e || !e);

                text_contains && range_contains && themes_match
            })
            .map(|(term, def, _, _, _, _)| (term.to_string(), def.to_string()))
            .collect()
    }

    pub fn parse_request(&self, s: &str) -> Option<Vec<(String, String)>> {
        let split: Vec<String> = s.split("|").map(|s| s.to_string()).collect();
        let keyword = split.get(0)?;
        let start_range = split.get(1)?;
        let end_range = split.get(2)?;
        let society_and_culture: bool = split.get(3)?.parse().ok()?;
        let politics: bool = split.get(4)?.parse().ok()?;
        let economy: bool = split.get(5)?.parse().ok()?;

        Some(self.search(
            if keyword.is_empty() { None } else { Some(keyword) },
            if start_range.is_empty() || end_range.is_empty()
            { None } else {
                Some(start_range.parse().ok()?..=end_range.parse().ok()?)
            },
            society_and_culture, politics, economy
        ))
    }
}


pub struct ServerState {
    clients: Vec<Client>,
    id_generator: ClientIdGenerator,
    god_set: GodSet,
}

impl ServerState {
    pub fn new() -> ServerState {
        ServerState { clients: Vec::new(), id_generator: ClientIdGenerator::new(), god_set: GodSet::new() }
    }

    pub fn new_connection_handler(&mut self, stream: TcpWriter) -> ClientId {
        let id = self.id_generator.next();

        println!("new connection: {:?}", id);

        self.clients.push(Client::new(id, stream));

        id
    }

    pub fn drop_handler(&mut self, id: ClientId) {
        let (i, _) = self.clients.iter().enumerate().find(|(_, c)| c.id == id).unwrap();
        self.clients.remove(i);
    }


    pub fn websocket_message_handler(&mut self, client: ClientId, message: Vec<u8>) {
        let message = String::from_utf8_lossy(&message);
        println!("client #{:?} sent `{}`", client, message);

        let result = match self.god_set.parse_request(&message) {
            Some(ok) => ok,
            None => Vec::new(),
        };

        let response = if result.is_empty() {
            b"Couldn't find match".to_vec()
        } else {
            result.iter().map(|(key, term)| format!("{}: {}\n\n", key, term)).collect::<String>().as_bytes().to_vec()
        };

        match self.get_writer(client) {
            Some(writer) => {
                let _ = writer.write_all(&Frame::from_payload(FrameType::Text, response).encode());
            },
            None => {},
        }
    }

    pub fn http_message_handler(&mut self, client: ClientId, message: HttpRequest) -> ReaderResponseToHttp {
        // returns true if should upgrade to websocket connection
        match self.get_writer(client) {
            Some(writer) => match handle_deelio(&message) {
                Some(websocket_upgrade_response) => match writer.write_all(websocket_upgrade_response.as_bytes()) {
                    Ok(_) => ReaderResponseToHttp::UpgradeToWebsocket,
                    Err(_) => ReaderResponseToHttp::Drop,
                },
                None => {
                    let _ = get_response_to_http(&message, writer);
                    ReaderResponseToHttp::Drop
                },
            },
            None => panic!("could not find {:?}", client),
        }
    }

    fn get_client_mut(&mut self, id: ClientId) -> Option<&mut Client> {
        self.clients.iter_mut()
            .find(|c| c.id == id)
    }

    fn get_writer(&mut self, id: ClientId) -> Option<&mut TcpWriter> {
        self.get_client_mut(id).map(|c| &mut c.writer)
    }

}


pub struct Client {
    pub id: ClientId,
    pub writer: TcpWriter,
}

impl Client {
    fn new(id: ClientId, writer: TcpWriter) -> Client {
        Client { id, writer }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ClientId(u64);


struct ClientIdGenerator(u64);
impl ClientIdGenerator {
    fn new() -> ClientIdGenerator {
        ClientIdGenerator(0)
    }

    fn next(&mut self) -> ClientId {
        self.0 += 1;
        ClientId(self.0)
    }
}


pub enum ReaderResponseToHttp {
    UpgradeToWebsocket,
    Drop,
}

fn handle_deelio(request: &HttpRequest) -> Option<String> {
    if request.get_header_value("Upgrade") == Some("websocket") {
        match request.get_header_value("Sec-WebSocket-Key") {
            Some(key) => {
                if !key.is_ascii() { return None };
                let to_hash = format!("{}{}", key, WEBSOCKET_SECURE_KEY_MAGIC_NUMBER);
                let response = to_base64(&Sha1::from(to_hash.as_bytes()).digest().bytes());
                // NOTE: excludes header: nSec-WebSocket-Protocol: chat
                Some(format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n", response))
            },
            None => None,
        }
    } else {
        None
    }
}
