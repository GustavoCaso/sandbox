use std::fmt;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use serde::{Deserialize, Serialize};

// Define a struct to represent the JSON data
#[derive(Serialize, Deserialize, Debug)]
struct Body {
    #[serde(rename(deserialize = "start_cursor", serialize = "startCursor"))]
    start_cursor: String,
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, "Error serializing Body"),
        }
    }
}

fn main() {
    // Create a TCP listener bound to 127.0.0.1:3000
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    println!("Server listening on port 3000");

    // Listen for incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn a new thread to handle each connection
                thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e);
            }
        }
    }
}

// Function to handle a client connection
fn handle_connection(mut stream: TcpStream) {
    // Buffer to store the HTTP request
    let mut buffer = [0; 1024];

    // Read the request into the buffer
    match stream.read(&mut buffer) {
        Ok(_) => {
            // Convert the buffer to a string for parsing
            let request = String::from_utf8_lossy(&buffer[..]);

            // Parse the request to determine what the client is asking for
            let (status_line, content) = if request.starts_with("GET / HTTP/1.1") {
                // Root path - return a welcome message
                (
                    "HTTP/1.1 200 OK",
                    "Hello! Welcome to our Rust HTTP server".to_string(),
                )
            } else if request.starts_with("GET /hello HTTP/1.1") {
                // /hello path - return a different message
                ("HTTP/1.1 200 OK", "Hello, World!".to_string())
            } else if request.starts_with("GET /info HTTP/1.1") {
                // /info path - return some server info
                ("HTTP/1.1 200 OK", "Server running on Rust 🦀".to_string())
            } else if request.starts_with("POST /submit HTTP/1.1") {
                let body = match request.find("\r\n\r\n") {
                    Some(index) => {
                        // Extract the body (skipping the \r\n\r\n separator)
                        let body_start = index + 4;
                        if body_start < request.len() {
                            &request[body_start..]
                        } else {
                            ""
                        }
                    }
                    None => "",
                };

                if body.is_empty() {
                    // If the body is empty, return a 400 Bad Request
                    ("HTTP/1.1 400 BAD REQUEST", "400 Bad Request".to_string())
                } else {
                    // Clean the body - trim null bytes and whitespace
                    let clean_body = body.trim_matches(char::from(0)).trim();
                    match serde_json::from_str::<Body>(clean_body) {
                        Ok(json_data) => {
                            // Successfully parsed JSON data
                            let content_string = json_data.to_string();
                            ("HTTP/1.1 200 OK", content_string)
                        }
                        Err(e) => {
                            // Failed to parse JSON data
                            eprintln!("Failed to parse JSON: {}", e);
                            ("HTTP/1.1 400 BAD REQUEST", "400 Bad Request".to_string())
                        }
                    }
                }
            } else {
                // Any other path - return a 404 Not Found
                ("HTTP/1.1 404 NOT FOUND", "404 Not Found".to_string())
            };

            // Construct the HTTP response
            let response = format!(
                "{}\r\nContent-Length: {}\r\n\r\n{}",
                status_line,
                content.len(),
                content
            );

            // Send the response back to the client
            match stream.write(response.as_bytes()) {
                Ok(_) => match stream.flush() {
                    Ok(_) => println!("Response sent successfully"),
                    Err(e) => eprintln!("Failed to flush stream: {}", e),
                },
                Err(e) => eprintln!("Failed to send response: {}", e),
            }
        }
        Err(e) => eprintln!("Failed to read from connection: {}", e),
    }
}
