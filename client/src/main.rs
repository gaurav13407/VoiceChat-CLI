use std::env;
mod identity;
pub mod app;
pub mod host;
mod cli;
use base64::{engine::general_purpose,Engine};

use vc_core::{room::code::{generate_room_code,validate_room_code}};

fn main() -> anyhow::Result<()> {
    let args:Vec<String>=env::args().collect();

    if args.len()<2{
        print_usage();
        return Ok(());
    }

    // Get server address from environment variable or use default
    let server_addr = std::env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:9000".to_string());
    println!("Using server: {}", server_addr);

    let identity=identity::Identity::load_or_create();
    println!("My Public identity:{:?}",identity.public_key_bytes());

    match args[1].as_str(){
        "create"=>{
            let room_code=generate_room_code();
            let mut stream=
                std::net::TcpStream::connect(&server_addr).expect("Cannot connect to the signaling");
            use std::io::Write;
            writeln!(stream, "CREATE {}",room_code).unwrap();

            println!("Room Created");
            println!("Room Code: {}",room_code);
        }
        "join"=>{
            if args.len()<3{
                println!("Error:room code missing");
                return Ok(());
            }
            let code=&args[2];

            if !validate_room_code(code){
                println!("Error:invalid room code format");
                return Ok(());
            }
            let mut stream=
                std::net::TcpStream::connect(&server_addr).expect("Cannot connect to signaling");

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
                    if parts.len()==3 && parts[0]=="PEER_PUBKEY"{
                        let peer_pubkey=general_purpose::STANDARD
                            .decode(parts[1])
                            .expect("Invalid base64 peer key");

                        let role = parts[2];
                        println!("Received peer public key ({} bytes), role: {}",peer_pubkey.len(), role);

                        // Convert peer_pubkey to [u8; 32]
                        let peer_pubkey_array: [u8; 32] = peer_pubkey.as_slice()
                            .try_into()
                            .map_err(|_| anyhow::anyhow!("Peer public key must be 32 bytes"))?;

                        //1.Perform secure handshake based on role
                        let secure_stream = if role == "CLIENT" {
                            vc_core::protocol::handshake::run(
                                reader.into_inner(),
                                identity.public_key_bytes(),
                                peer_pubkey_array,
                            )?
                        } else {
                            vc_core::protocol::handshake::run_as_host(
                                reader.into_inner(),
                                identity.public_key_bytes(),
                                peer_pubkey_array,
                            )?
                        };

                        println!("Secure connection established!");

                        //2. Start Chat Here
                        let sender_id = general_purpose::STANDARD.encode(identity.public_key_bytes());
                        cli::input_loop(secure_stream, sender_id)?;
                        return Ok(());
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
    Ok(())
}

fn print_usage(){
    println!("Usage:");
    println!(" create");
    println!(" join <ROOM_CODE>");
}


