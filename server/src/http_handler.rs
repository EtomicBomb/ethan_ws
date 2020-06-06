use http::HttpRequest;

use std::path::{Path, PathBuf};
use std::io::{self, Write, Read, ErrorKind};
use std::fs::{File, self};
use std::net::TcpStream;

use crate::RESOURCES_ROOT;

// hardcoded error messages
const ERROR_404_RESPONSE: &'static [u8] = b"HTTP/1.1 404 Page Not Found\r\n\r\n<!DOCTYPE html><html lang='en-US'><head><meta charset='UTF-8'><title>ethan.ws</title></head><body><h1>Error 404 - Page Not Found</h1></body></html>";
const ERROR_500_RESPONSE: &'static [u8] = b"HTTP/1.1 500 Internal Server Error\r\n\r\n<!DOCTYPE html><html lang='en-US'><head><meta charset='UTF-8'><title>ethan.ws</title></head><body><h1>Error 500 - Internal Server Error</h1></body></html>";

pub fn send_resource(request: &HttpRequest, writer: &mut TcpStream) -> io::Result<()> {
    match get_data(request.resource_location()) {
        Ok(data) => {
            writer.write_all(b"HTTP/1.1 200 OK\r\n\r\n")?;
            writer.write_all(&data)?;
        },
        Err(e) if e.kind() == ErrorKind::PermissionDenied || e.kind() == ErrorKind::NotFound =>
            writer.write_all(ERROR_404_RESPONSE)?,
        Err(_) => writer.write_all(ERROR_500_RESPONSE)?,
    }

    writer.flush()?;

    Ok(())
}


fn get_data(request: &str) -> io::Result<Vec<u8>> {
    let request =
        if request.starts_with("/") {
            &request[1..]
        } else {
            return Err(io::ErrorKind::InvalidInput.into());
        };

    let mut path = PathBuf::from(RESOURCES_ROOT);
    path.push(request);

    path = path.canonicalize()?;

    if path.is_dir() {
        path.push("index.html");
    }

    if !is_to_resources_folder(&path) {
        return Err(io::ErrorKind::PermissionDenied.into());
    }

    fs::read(path)
}

fn is_to_resources_folder(path: &PathBuf) -> bool {
    // make sure request doesn't look like /../../../Desktop/secrets.txt or something
    // we already know path is in cannonical form
    path.ancestors().any(|a| a == Path::new(RESOURCES_ROOT))
}
