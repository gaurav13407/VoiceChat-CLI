#[derive(Debug)]
pub enum ClientMessage{
    Create{room_code:String},
    Join{room_code:String},
}

#[derive(Debug)]

pub enum ServerMessage {
    RoomCreated,
    RoomExists,
    RoomNotFound,
}

