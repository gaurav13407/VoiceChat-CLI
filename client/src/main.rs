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
            let pubkey_b64=
                general_purpose::STANDARD.encode(identity.public_key_bytes());
            writeln!(stream, "CREATE {} {}",room_code,pubkey_b64).unwrap();

            println!("Room Created");
            println!("Room Code: {}",room_code);
            println!("Waiting for peer to join...");
            
            // Wait for PEER_PUBKEY message just like JOIN does
            let mut line=String::new();
            read_line_unbuffered(&mut stream, &mut line)?;
            
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
                        stream,
                        identity.public_key_bytes(),
                        peer_pubkey_array,
                    )?
                } else {
                    vc_core::protocol::handshake::run_as_host(
                        stream,
                        identity.public_key_bytes(),
                        peer_pubkey_array,
                    )?
                };

                println!("Secure connection established!");

                // Generate UDP ports for voice based on role
                let (my_voice_port, peer_voice_port, my_sender_id) = if role == "CLIENT" {
                    (9001u16, 9002u16, 1u32)  // Client = ID 1
                } else {
                    (9002u16, 9001u16, 2u32)  // Host = ID 2
                };

                // Start voice session
                let voice_send_addr = format!("127.0.0.1:{}", peer_voice_port);
                let voice_recv_bind = format!("0.0.0.0:{}", my_voice_port);
                
                let _voice_session = audio::VoiceSession::start(
                    my_sender_id,
                    &voice_send_addr,
                    &voice_recv_bind,
                )?;

                //2. Start Chat Here
                let sender_id = general_purpose::STANDARD.encode(identity.public_key_bytes());
                cli::input_loop(secure_stream, sender_id)?;
                return Ok(());
            }
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

            // Read responses line by line WITHOUT BufReader to avoid buffering issues
            let mut response=String::new();
            read_line_unbuffered(&mut stream, &mut response)?;

            let resp=response.trim().to_uppercase();

            match resp.as_str(){
                "ROOM_EXISTS" | "ROOM_JOINED"=>{
                    println!("Connected to room {}",code);
                
                    let mut line=String::new();
                    read_line_unbuffered(&mut stream, &mut line)?;
                    
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
                                stream,
                                identity.public_key_bytes(),
                                peer_pubkey_array,
                            )?
                        } else {
                            vc_core::protocol::handshake::run_as_host(
                                stream,
                                identity.public_key_bytes(),
                                peer_pubkey_array,
                            )?
                        };

                        println!("Secure connection established!");

                        // Generate UDP ports for voice based on role
                        let (my_voice_port, peer_voice_port, my_sender_id) = if role == "CLIENT" {
                            (9001u16, 9002u16, 1u32)  // Client = ID 1
                        } else {
                            (9002u16, 9001u16, 2u32)  // Host = ID 2
                        };

                        // Start voice session
                        let voice_send_addr = format!("127.0.0.1:{}", peer_voice_port);
                        let voice_recv_bind = format!("0.0.0.0:{}", my_voice_port);
                        
                        let _voice_session = audio::VoiceSession::start(
                            my_sender_id,
                            &voice_send_addr,
                            &voice_recv_bind,
                        )?;

                        //2. Start Chat Here
                        let sender_id = general_purpose::STANDARD.encode(identity.public_key_bytes());
                        cli::input_loop(secure_stream, sender_id)?;
                        return Ok(());
                    }
                }
                "ROOM_NOT_FOUND"=>{
                    println!("Error:room not found");
                }
                "ROOM_FULL"=>{
                    println!("Error:room full");
                }
                _=>{}
            }
        }
        _ =>{
            print_usage();
        }
    }
    Ok(())
}

fn read_line_unbuffered(stream: &mut std::net::TcpStream, line: &mut String) -> anyhow::Result<()> {
    use std::io::Read;
    let mut byte = [0u8; 1];
    loop {
        stream.read_exact(&mut byte)?;
        let ch = byte[0] as char;
        if ch == '\n' {
            break;
        }
        line.push(ch);
    }
    Ok(())
}

fn print_usage(){
    println!("Usage:");
    println!(" create");
    println!(" join <ROOM_CODE>");
}


