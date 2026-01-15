pub struct VoicePacket{
    pub sender_id:u32,
    pub seq:u32,
    pub samples:Vec<f32>,
}

impl VoicePacket{
    pub fn encode(&self)->Vec<u8>{
        let mut buf=Vec::with_capacity(12+self.samples.len()*4);

        buf.extend_from_slice(&self.sender_id.to_le_bytes());
        buf.extend_from_slice(&self.seq.to_le_bytes());
        buf.extend_from_slice(&(self.samples.len() as u32).to_le_bytes());
    }
    pub fn decode(buf:&[u8])->Option<Self>{}
}
