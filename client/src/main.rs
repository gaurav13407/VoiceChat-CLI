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
            println!("Room Created");
            println!("Room code:{}",room_code);
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
            println!("Joining room{}...",code);
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


