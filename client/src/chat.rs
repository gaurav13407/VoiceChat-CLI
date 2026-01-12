use crate::protocol::chat::{ChatMessage, ChatText};
use crate::state::secure_stream::SecureStream;


pub fn send_chat_message(
    stream: &mut SecureStream,
    sender_id: String,
    text: String,
) -> anyhow::Result<()> {
    let msg = ChatMessage::Text(ChatText {
        sender_id,
        body: text,
    });

    let encoded = bincode::serialize(&msg)?;
    stream.write_frame(&encoded)?;
    Ok(())
}

pub fn recv_chat_loop(mut stream:SecureStream)->anyhow::Result<()>{
    loop{
        let frame=stream.recv()?;
        let msg:ChatMessage=bincode::deserialize(&frame)?;

        match msg{
            ChatMessage::Text(chat)=>{
                println!("[{}] {}",chat.sender_id,chat.body);
            }
            ChatMessage::System(sys)=>{
                println!("[system] {}",sys.body);
            }
        }
    }
}

