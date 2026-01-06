use std::env;
mod identity;
pub mod app;
pub mod host;
mod cli;
use base64::{engine::general_purpose,Engine};

use vc_core::room::code::{generate_room_code,validate_room_code};

fn main(){
    let args:Vec<String>=env::args().collect();

    if args.len()<2{
        print_usage();
        return;
    }

    let identity=identity::Identity::load_or_create();
    println!("My Public identity:{:?}",identity.public_key_bytes());

    match args[1].as_str(){
        "create"=>{
            let room_code=generate_room_code();
            let mut stream=
                std::net::TcpStream::connect("127.0.0.1:9000").expect("Cannot connect to the signaling");
            use std::io::Write;
            writeln!(stream, "CREATE {}",room_code).unwrap();

            println!("Room Created");
            println!("Room Code: {}",room_code);
        }
        "join"=>{
            if args.len()<3{
                println!("Error:room code missing");
                return;
            }
            let code=&args[2];

            if !validate_room_code(code){
                println!("Error:invalid room code format");
                return;
            }
            let mut stream=
                std::net::TcpStream::connect("127.0.0.1:9000").expect("Cannot connect to signaling");

            use std::io::{BufReader,BufRead,Write};
            let pubkey_b64=
                general_purpose::STANDARD.encode(identity.public_key_bytes());
            writeln!(stream, "JOIN {} {}",code,pubkey_b64).unwrap();

            let mut reader=BufReader::new(stream);
            let mut response=String::new();
            reader.read_line(&mut response).unwrap();

            let resp=response.trim().to_uppercase();

            match resp.as_str(){
                "ROOM_EXISTS" | "ROOM_JOINED"=>{
                    println!("Connected to room {}",code);
                
                loop{
                    let mut line=String::new();
                    if reader.read_line(&mut line).is_err(){
                        break;
                    }
                    let parts:Vec<&str>=line.trim().split_whitespace().collect();
                    if parts.len()==2 && parts[0]=="PEER_PUBKEY"{
                        let peer_pubkey=general_purpose::STANDARD
                            .decode(parts[1])
                            .expect("Invalid base64 peer key");

                        println!("Received peer public key ({} bytes)",peer_pubkey.len());

                        break;
                    }
                }
            }
                "ROOM_NOT_FOUND"=>{
                    println!("Error:room not found");
                }
                "ROOM_FULL"=>{
                    println!("Error:room full");
                }
                other=>{
                    println!("Unkown server response: {:?}",other);
                }
            }
        }

        _ =>{
            print_usage();
        }
    }
}

fn print_usage(){
    println!("Usage:");
    println!(" create");
    println!(" join <ROOM_CODE>");
}


