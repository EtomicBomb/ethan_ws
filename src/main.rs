extern crate chrono;
use chrono::{Local};

use lazy_static::lazy_static;

use regex::bytes;

use std::net::{TcpListener, TcpStream};
use std::io::{self, Read, Write};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

const RESOURCES_ROOT: &'static str = "/home/pi/Desktop/ethan_ws/resources";
const LOG_FILE_PATH: &'static str = "/home/pi/Desktop/server_log.txt";

// hard-coded error messages
const ERROR_404_RESPONSE: &'static [u8] = b"HTTP/1.1 404 Page Not Found\r\n\r\n<!DOCTYPE html><html lang='en-US'><head><meta charset='UTF-8'><title>ethan.ws</title></head><body><h1>Error 404 - Page Not Found</h1></body></html>";
const ERROR_500_RESPONSE: &'static [u8] = b"HTTP/1.1 500 Internal Server Error\r\n\r\n<!DOCTYPE html><html lang='en-US'><head><meta charset='UTF-8'><title>ethan.ws</title></head><body><h1>Error 500 - Internal Server Error</h1></body></html>";

lazy_static! {
    static ref RESOURCE_IDENTIFIER: bytes::Regex = bytes::Regex::new(r#"^GET (\S*)"#).unwrap();
}

fn main() -> io::Result<()> {
    let mut log_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(LOG_FILE_PATH).unwrap();

    let result = start_server(&mut log_file);

    writeln!(log_file, "{}\tsession ended with error: {:?}", time_string(), result).unwrap();

    result
}

fn start_server(log_file: &mut File) -> io::Result<()> {
    let mut response_index = 0;

    writeln!(log_file, "{}\tsession started", time_string())?;

    for stream in TcpListener::bind("0.0.0.0:80")?.incoming() {
        response_index += 1;

        if let Ok(stream) = stream {
            let _ = handle_client(stream, response_index, log_file);
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, response_index: usize, log_file: &mut File) -> Result<(), ServerError> {
    writeln!(log_file, "{}\t {}", time_string(), stream.peer_addr()?)?;

    let mut buf = [0u8; 512];
    stream.read(&mut buf)?;

    // parse our request for what resource they were requesting
    let resource_title = get_resource_title(&buf)?;

    match get_data(&resource_title) {
        Ok(data) => {
            write!(stream, "HTTP/1.1 200 OK\r\n\r\n")?;
            stream.write_all(&data)?
        },
        Err(ServerError::ResourceNotFound) => stream.write_all(ERROR_404_RESPONSE)?,
        Err(_) => stream.write_all(ERROR_500_RESPONSE)?,
    };

    stream.flush()?;

    Ok(())
}

fn get_resource_title(request: &[u8]) -> Result<String, ServerError> {
    match bytes::Regex::captures(&RESOURCE_IDENTIFIER, &request) {
        Some(captures) => {
            match String::from_utf8(captures[1].to_vec()) {
                Ok(resource_title) => Ok(resource_title),
                Err(_) => Err(ServerError::MalformedRequest),
            }
        },
        None => Err(ServerError::MalformedRequest),
    }
}

fn get_data(request: &str) -> Result<Vec<u8>, ServerError> {
    let request =
        if request.starts_with("/") {
            &request[1..]
        } else {
            return Err(ServerError::MalformedRequest);
        };

    let mut path=  PathBuf::from(RESOURCES_ROOT);
    path.push(request);

    path = path.canonicalize()?;

    if path.is_dir() {
        path.push("index.html");
    }

    if !is_to_resources_folder(&path) {
        return Err(ServerError::PathOutsideResources);
    }

    Ok(read_to_vec(path)?)
    
}

fn is_to_resources_folder(path: &PathBuf) -> bool {
    // make sure request doesn't look like /../../../Desktop/secrets.txt or something
    path.ancestors().any(|a| a == Path::new(RESOURCES_ROOT))
}

#[derive(Debug)]
enum ServerError {
    IoError(io::Error),
    MalformedRequest,
    ResourceNotFound,
    PathOutsideResources,
}

impl From<io::Error> for ServerError {
    fn from(error: io::Error) -> ServerError {
        match error.kind() {
            io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied => ServerError::ResourceNotFound,
            _ => ServerError::IoError(error),
        }
    }
}

fn read_to_vec(name: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut file = File::open(name)?;

    let buf_size = file.metadata().map(|m| m.len() + 1).unwrap_or(0) as usize;

    let mut buf = Vec::with_capacity(buf_size);

    file.read_to_end(&mut buf)?;

    Ok(buf)
}

fn time_string() -> String {
    Local::now().format("%A, %B %d, %Y %I:%M:%S%P").to_string()
}

