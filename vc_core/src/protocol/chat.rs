use serde::{Deserialize,Serialize};

#[derive(Debug,Serialize,Deserialize)]
pub enum ChatMessage{
    Text(ChatText),
    System(SystemMessage),
}

#[derive(Debug,Serialize,Deserialize)]
pub struct ChatText{
    pub sender_id:String,
    pub body:String,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct SystemMessage{
    pub body:String,
}
