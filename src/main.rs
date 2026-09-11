use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Rust test server running on port 8080...");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 512];
                let _ = stream.read(&mut buffer);
                let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello from Rust Core on Klade! 🦀";
                let _ = stream.write_all(response.as_bytes());
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}