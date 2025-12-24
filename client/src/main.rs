use std::env;

use vc_core::room::code::{generate_room_code,validate_room_code};

fn main(){
    let args:Vec<String>=env::args().collect();

    if args.len()<2{
        print_usage();
        return;
    }

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
            writeln!(stream, "JOIN {}",code).unwrap();

            let mut reader=BufReader::new(stream);
            let mut response=String::new();
            reader.read_line(&mut response).unwrap();

            let resp=response.trim().to_uppercase();

            match resp.as_str(){
                "ROOM_EXISTS"=>{
                    println!("Connected to room {}",code);
                }
                "ROOM_NOT_FOUND"=>{
                    println!("Error:room not found");
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


