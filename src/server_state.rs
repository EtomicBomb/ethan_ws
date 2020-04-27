use std::io::Write;
use crate::frame::{Frame, FrameType};
use crate::tcp_halves::TcpWriter;
use crate::http_request_parse::HttpRequest;
use crate::base64::to_base64;

const ERROR_404_RESPONSE: &'static [u8] = b"HTTP/1.1 404 Page Not Found\r\n\r\n<!DOCTYPE html><html lang='en-US'><head><meta charset='UTF-8'><title>ethan.ws</title></head><body><h1>Error 404 - Page Not Found</h1></body></html>";
const WEBSOCKET_SECURE_KEY_MAGIC_NUMBER: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";


pub struct ServerState {
    clients: Vec<Client>,
    id_generator: ClientIdGenerator,
}

impl ServerState {
    pub fn new() -> ServerState {
        ServerState { clients: Vec::new(), id_generator: ClientIdGenerator::new() }
    }

    pub fn new_connection_handler(&mut self, stream: TcpWriter) -> ClientId {
        // spin up a thread locked on listening

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
        println!("client #{:?} sent `{}`", client, String::from_utf8_lossy(&message));

        match self.get_writer(client) {
            Some(writer) => {
                let mut response = Vec::new();
                response.extend_from_slice(b"I dont have a response to: `");
                response.extend_from_slice(&message);
                response.extend_from_slice(b"`. sorry!");
                let _ = writer.write_all(&Frame::from_payload(FrameType::Text, response).encode());
            },
            None => {},
        }
    }

    pub fn http_message_handler(&mut self, client: ClientId, message: HttpRequest) -> ReaderResponseToHttp {
        // returns true if should upgrade to websocket connection
        match self.get_writer(client) {
            Some(writer) => match handle_deelio(message) {
                Some(websocket_upgrade_response) => match writer.write_all(websocket_upgrade_response.as_bytes()) {
                    Ok(_) => ReaderResponseToHttp::UpgradeToWebsocket,
                    Err(_) => ReaderResponseToHttp::Drop,
                },
                None => {
                    let _ = writer.write_all(ERROR_404_RESPONSE); // we dont really care
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

fn handle_deelio(request: HttpRequest) -> Option<String> {
    if request.headers.get("Upgrade").map(String::as_str) == Some("websocket") {
        match request.headers.get("Sec-WebSocket-Key") {
            Some(key) => {
                if !key.is_ascii() { return None };
                let to_hash = format!("{}{}", key, WEBSOCKET_SECURE_KEY_MAGIC_NUMBER);
                let result = sha1::Sha1::from(to_hash.as_bytes()).digest().bytes();
                let response = to_base64(&result);
                let r = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\nSec-WebSocket-Protocol: chat\r\n\r\n", response);
                Some(r)
            },
            None => None,
        }
    } else {
        None
    }
}
