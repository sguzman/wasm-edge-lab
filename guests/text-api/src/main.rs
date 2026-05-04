use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    while match stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            println!("Received: {}", String::from_utf8_lossy(&buffer[..size]));
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!").unwrap();
            false
        }
        _ => false,
    } {}
}

fn main() {
    let listen_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8787".to_string());
    println!("Text API guest starting on {}...", listen_addr);

    let listener = TcpListener::bind(&listen_addr).expect("Failed to bind to address");
    println!("Listening on {}...", listen_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
