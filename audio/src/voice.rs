/// VoicePacket represents one encoded Opus frame
pub struct VoicePacket {
    pub sender_id: u32,
    pub seq: u32,
    pub payload: Vec<u8>, // Opus data
}

impl VoicePacket {
    /// Encode packet for UDP
    ///
    /// Format:
    /// [u32 sender_id]
    /// [u32 seq]
    /// [u32 payload_len]
    /// [u8 payload...]
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(12 + self.payload.len());

        buf.extend_from_slice(&self.sender_id.to_le_bytes());
        buf.extend_from_slice(&self.seq.to_le_bytes());
        buf.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.payload);

        buf
    }

    pub fn decode(buf: &[u8]) -> Option<Self> {
        if buf.len() < 12 {
            return None;
        }

        let mut off = 0;

        let sender_id =
            u32::from_le_bytes(buf[off..off + 4].try_into().ok()?);
        off += 4;

        let seq =
            u32::from_le_bytes(buf[off..off + 4].try_into().ok()?);
        off += 4;

        let len =
            u32::from_le_bytes(buf[off..off + 4].try_into().ok()?) as usize;
        off += 4;

        if buf.len() < off + len {
            return None;
        }

        let payload = buf[off..off + len].to_vec();

        Some(Self {
            sender_id,
            seq,
            payload,
        })
    }
}
