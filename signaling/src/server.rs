use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write, Read};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use base64::Engine;

struct Peer{
    stream:TcpStream,
    pubkey:Vec<u8>,
}

type Rooms = Arc<Mutex<HashMap<String, Vec<Peer>>>>;


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
            if parts.len() < 3 {
                let _ = writeln!(writer, "ERROR");
                return;
            }
            let code = parts[1].to_string();
            let pubkey_b64 = parts[2];

            let pubkey = match base64::engine::general_purpose::STANDARD.decode(pubkey_b64) {
                Ok(pk) => pk,
                Err(_) => {
                    let _ = writeln!(writer, "ERROR");
                    return;
                }
            };

            let mut rooms = rooms.lock().unwrap();
            let room = rooms.entry(code.clone()).or_insert_with(Vec::new);
            
            room.push(Peer {
                stream: writer.try_clone().unwrap(),
                pubkey,
            });
            
            println!("Room {} created by {}", code, peer);
            
            // Don't send ROOM_CREATED yet - wait for second peer
            // Block to keep connection alive
            drop(rooms);
            std::thread::park();
        }

        "JOIN"=>{
            if parts.len()<3{
                let _=writeln!(writer, "ERROR");
                return;
            }
            let code=parts[1];
            let pubkey_b64=parts[2];

            let pubkey=match base64::engine::general_purpose::STANDARD.decode(pubkey_b64){
                Ok(pk)=>pk,
                Err(_)=>{
                    let _=writeln!(writer, "ERROR");
                    return;
                }
            };

            let mut rooms=rooms.lock().unwrap();

            let room=match rooms.get_mut(code){
                Some(r)=>r,
                None=>{
                    let _=writeln!(writer, "ROOM_NOT_FOUND");
                    return;
                }
            };

            if room.len()>=2{
                let _=writeln!(writer, "ROOM_FULL");
                return;
            }

            room.push(Peer{
                stream:writer.try_clone().unwrap(),
                pubkey,
            });

            println!("Room {} joined by {}",code,peer);
            let _=writeln!(writer, "ROOM_JOINED");

            //if second peer joined -> exchange pubkeys and start relaying
            if room.len()==2{
                let pk1=base64::engine::general_purpose::STANDARD.encode(&room[0].pubkey);
                let pk2=base64::engine::general_purpose::STANDARD.encode(&room[1].pubkey);

                let _=writeln!(room[0].stream, "PEER_PUBKEY {} HOST",pk2);
                let _=writeln!(room[1].stream, "PEER_PUBKEY {} CLIENT",pk1);

                // Take ownership of streams from room
                let peer1 = room.remove(0);
                let peer2 = room.remove(0);
                drop(rooms);

                // Clone streams for relaying - each direction needs independent clones
                let stream1_read = peer1.stream.try_clone().unwrap();
                let stream1_write = peer1.stream.try_clone().unwrap();
                let stream2_read = peer2.stream.try_clone().unwrap();
                let stream2_write = peer2.stream.try_clone().unwrap();
                
                // Start relay threads
                thread::spawn(move || relay_traffic(stream1_read, stream2_write, "1->2"));
                thread::spawn(move || relay_traffic(stream2_read, stream1_write, "2->1"));
                
                // Keep original streams alive
                thread::spawn(move || {
                    let _keep = (peer1.stream, peer2.stream);
                    std::thread::park();
                });
                
                return; // Don't drop writer
            }
        }

        _ => {
            let _ = writeln!(writer, "ERROR");
        }
    }
}

fn relay_traffic(mut from: TcpStream, mut to: TcpStream, label: &str) {
    let mut buf = [0u8; 8192];
    eprintln!("[RELAY {}] Starting relay loop", label);
    loop {
        match from.read(&mut buf) {
            Ok(0) => {
                eprintln!("[RELAY {}] Connection closed", label);
                break;
            }
            Ok(n) => {
                eprintln!("[RELAY {}] Read {} bytes, forwarding...", label, n);
                if let Err(e) = to.write_all(&buf[..n]) {
                    eprintln!("[RELAY {}] Failed to write: {}", label, e);
                    break;
                }
                if let Err(e) = to.flush() {
                    eprintln!("[RELAY {}] Failed to flush: {}", label, e);
                    break;
                }
                eprintln!("[RELAY {}] Successfully forwarded {} bytes", label, n);
            }
            Err(e) => {
                eprintln!("[RELAY {}] Read error: {}", label, e);
                break;
            }
        }
    }
    eprintln!("[RELAY {}] Exiting relay loop", label);
}

