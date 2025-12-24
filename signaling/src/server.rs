use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

type Rooms = Arc<Mutex<HashMap<String, ()>>>;

pub fn start_server(addr: &str) {
    let listener = TcpListener::bind(addr).expect("Failed to bind signaling server");
    println!("Signaling server listening on {}", addr);

    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let rooms = Arc::clone(&rooms);
                std::thread::spawn(move || {
                    handle_client(stream, rooms);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn handle_client(stream: TcpStream, rooms: Rooms) {
    let peer = stream.peer_addr().unwrap();

    let reader_stream = stream.try_clone().unwrap();
    let mut reader = BufReader::new(reader_stream);
    let mut writer = stream;

    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }

    println!("Received from {}: {:?}", peer, line);

    let parts: Vec<&str> = line.trim().split_whitespace().collect();
    if parts.len() < 2 {
        let _ = writeln!(writer, "ERROR");
        return;
    }

    match parts[0].to_uppercase().as_str() {
        "CREATE" => {
            let code = parts[1].to_string();
            rooms.lock().unwrap().insert(code.clone(), ());
            println!("Room {} created by {}", code, peer);
            let _ = writeln!(writer, "ROOM_CREATED");
        }

        "JOIN" => {
            let code = parts[1];
            if rooms.lock().unwrap().contains_key(code) {
                println!("Room {} joined by {}", code, peer);
                let _ = writeln!(writer, "ROOM_EXISTS");
            } else {
                let _ = writeln!(writer, "ROOM_NOT_FOUND");
            }
        }

        _ => {
            let _ = writeln!(writer, "ERROR");
        }
    }
}

