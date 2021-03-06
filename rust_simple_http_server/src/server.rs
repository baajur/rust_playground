use crate::http::{ParseError, Request, Response, StatusCode};
use std::convert::TryFrom;
use std::io::Read;
use std::net::TcpListener;

pub trait Handler {
    fn handle_request(&mut self, request: &Request) -> Response;

    fn handle_bad_request(&mut self, err: &ParseError) -> Response {
        // This is the default implementation. Of course,
        // any implementer of this trait can override it.
        println!("Error: Failed to parse request. Details: {}", err);
        Response::new(StatusCode::BadRequest, None)
    }

    fn public_path(&self) -> &String;
}

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
    pub fn run(self, mut handler: impl Handler) {
        println!("Serving files from {} ...", handler.public_path());
        println!("Listening on {} ...", self.addr);
        let listener = match TcpListener::bind(&self.addr) {
            Ok(listener) => listener,
            Err(error) => {
                println!("Error: Cannot listen on '{}'. Details: {}", self.addr, error);
                return;
            }
        };

        loop {
            match listener.accept() {
                Ok((mut stream, client_addr)) => {
                    // get all the data sent by the client
                    let mut buffer = [0; 1024];
                    match stream.read(&mut buffer) {
                        Ok(_) => {
                            println!("Received request from {}: '{}'.", client_addr, String::from_utf8_lossy(&buffer));
                            let response = match Request::try_from(&buffer[..]) {
                                Ok(request) => handler.handle_request(&request),
                                Err(err) => handler.handle_bad_request(&err),
                            };

                            if let Err(e) = response.send(&mut stream) {
                                println!("Error: Failed to send response. Details: {}", e);
                            }
                        }
                        Err(e) => println!("Error: Failed to read from conn. Details: {}", e),
                    }
                }
                Err(err) => println!("Error: Cannot establish connection. Details: {}", err),
            }
        }
    }
}
