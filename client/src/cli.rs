use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

use vc_core::net::secure_stream::SecureStream;
use vc_core::protocol::chat::{ChatMessage, ChatText};

pub fn input_loop(stream: SecureStream, sender_id: String) -> anyhow::Result<()> {
    eprintln!("[INFO] Chat ready! Type /msg <text> to send messages");

    // Split stream into Arc<Mutex<>> for sharing between threads
    let stream = Arc::new(Mutex::new(stream));
    let recv_stream = Arc::clone(&stream);

    // Channel to signal receiver to check for messages
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    // Spawn receiver thread
    let receiver_handle = thread::spawn(move || {
        loop {
            // Try to receive without blocking on the channel first
            match shutdown_rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    eprintln!("[RECV] Shutting down receiver");
                    break;
                }
                Err(TryRecvError::Empty) => {
                    // No shutdown signal, continue
                }
            }

            // Acquire lock, do the recv, then immediately release
            let recv_result = {
                let mut s = recv_stream.lock().unwrap();
                s.recv()
            };
            // Lock is released here!

            match recv_result {
                Ok(data) => {
                    eprintln!("[RECV] Received {} bytes", data.len());
                    if let Ok(msg) = bincode::deserialize::<ChatMessage>(&data) {
                        match msg {
                            ChatMessage::Text(txt) => {
                                println!("\n[{}]: {}", txt.sender_id, txt.body);
                                print!("> ");
                                io::stdout().flush().ok();
                            }
                            ChatMessage::System(sys) => {
                                println!("\n[SYSTEM]: {}", sys.body);
                                print!("> ");
                                io::stdout().flush().ok();
                            }
                        }
                    } else {
                        eprintln!("[RECV] Failed to deserialize message");
                    }
                }
                Err(e) => {
                    // Check if it's a timeout or WouldBlock error (no data available)
                    let e_str = format!("{:?}", e);
                    if e_str.contains("WouldBlock") || e_str.contains("TimedOut") {
                        // No data available, just continue without delay
                        continue;
                    }
                    
                    // Handle other errors but keep receiver alive
                    eprintln!("[RECV] Error: {:?}", e);
                    
                    // Only exit on fatal connection errors, not transient ones
                    if e_str.contains("ConnectionReset") || e_str.contains("BrokenPipe") {
                        eprintln!("[RECV] Fatal connection error, exiting receiver");
                        break;
                    }
                    
                    // For UnexpectedEof and other errors, just retry after a delay
                    eprintln!("[RECV] Retrying after error...");
                    thread::sleep(Duration::from_millis(200));
                    continue;
                }
            }
        }
    });

    // Input loop  
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "/exit" {
            shutdown_tx.send(()).ok();
            break;
        }

        if let Some(text) = input.strip_prefix("/msg ") {
            eprintln!("[SEND] Sending message: {}", text);
            // Acquire lock just for sending
            let result = {
                let mut s = stream.lock().unwrap();
                send_chat_messgae(&mut s, sender_id.clone(), text.to_string())
            };
            // Lock released here!
            
            match result {
                Ok(_) => eprintln!("[SEND] Message sent successfully"),
                Err(e) => eprintln!("[SEND] ERROR: {:?}", e),
            }
        } else {
            println!("Usage: /msg <text>");
        }
    }

    // Wait for receiver to finish
    receiver_handle.join().ok();
    Ok(())
}

fn send_chat_messgae(
    stream: &mut SecureStream,
    sender_id: String,
    body: String,
) -> anyhow::Result<()> {
    eprintln!("[send_chat_messgae] Creating message...");
    let msg = ChatMessage::Text(ChatText { sender_id, body });
    eprintln!("[send_chat_messgae] Serializing...");
    let data = bincode::serialize(&msg)?;
    eprintln!("[send_chat_messgae] Serialized to {} bytes, calling stream.send()...", data.len());
    stream.send(&data).map_err(|e| anyhow::anyhow!("Failed to send: {:?}", e))?;
    eprintln!("[send_chat_messgae] Done!");
    Ok(())
}
