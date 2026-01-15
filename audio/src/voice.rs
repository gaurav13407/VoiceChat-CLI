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
        
        for &sample in &self.samples{
            buf.extend_from_slice(&sample.to_le_bytes());
        }
        
        buf
    }
    pub fn decode(buf:&[u8])->Option<Self>{
        if buf.len()<12{
            return None;
        }
        
        let sender_id=u32::from_le_bytes([buf[0],buf[1],buf[2],buf[3]]);
        let seq=u32::from_le_bytes([buf[4],buf[5],buf[6],buf[7]]);
        let len=u32::from_le_bytes([buf[8],buf[9],buf[10],buf[11]]) as usize;
        
        if buf.len()<12+len*4{
            return None;
        }
        
        let mut samples=Vec::with_capacity(len);
        for i in 0..len{
            let offset=12+i*4;
            let sample=f32::from_le_bytes([
                buf[offset],
                buf[offset+1],
                buf[offset+2],
                buf[offset+3],
            ]);
            samples.push(sample);
        }
        
        Some(Self{sender_id,seq,samples})
    }
}
