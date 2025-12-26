use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
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
            let code = parts[1].to_string();
            rooms.lock().unwrap().insert(code.clone(), Vec::new());
            println!("Room {} created by {}", code, peer);
            let _ = writeln!(writer, "ROOM_CREATED");
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

            //if second peer joined -> exchange pubkeys
            if room.len()==2{
                let pk1=base64::engine::general_purpose::STANDARD.encode(&room[0].pubkey);
                let pk2=base64::engine::general_purpose::STANDARD.encode(&room[1].pubkey);

                let _=writeln!(room[0].stream, "PEER_PUBKEY {}",pk2);
                let _=writeln!(room[1].stream, "PEER_PUBKEY {}",pk1);
            }
        }

        _ => {
            let _ = writeln!(writer, "ERROR");
        }
    }
}

