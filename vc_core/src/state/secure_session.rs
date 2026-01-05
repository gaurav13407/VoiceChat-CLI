use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Key, Nonce,
};
use ed25519_dalek::VerifyingKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRole {
    Client,
    Host,
}

#[derive(Debug)]
pub enum SecureSessionError {
    DecryptionFailed,
    ReplayDetected,
    MalformedPacket,
}

pub struct SecureSession {
    role: SessionRole,
    peer_identity: VerifyingKey,
    cipher: ChaCha20Poly1305,
    send_ctr: u64,
    recv_ctr: u64,
}

impl SecureSession {
    pub fn new(
        role: SessionRole,
        session_key: [u8; 32],
        peer_identity: VerifyingKey,
    ) -> Self {
        let key = Key::from_slice(&session_key);
        let cipher = ChaCha20Poly1305::new(key);

        Self {
            role,
            peer_identity,
            cipher,
            send_ctr: 0,
            recv_ctr: 0,
        }
    }

    pub fn encrypt(&mut self, plaintext: &[u8]) -> Vec<u8> {
        let ctr = self.send_ctr;
        self.send_ctr += 1;

        let nonce = nonce_from_ctr(ctr);

        let ciphertext = self
            .cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: plaintext,
                    aad: &ctr.to_be_bytes(),
                },
            )
            .expect("encryption failure");

        let mut out = Vec::with_capacity(8 + ciphertext.len());
        out.extend_from_slice(&ctr.to_be_bytes());
        out.extend_from_slice(&ciphertext);
        out
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, SecureSessionError> {
        if data.len() < 8 {
            return Err(SecureSessionError::MalformedPacket);
        }

        let (ctr_bytes, ciphertext) = data.split_at(8);
        let ctr = u64::from_be_bytes(ctr_bytes.try_into().unwrap());

        if ctr < self.recv_ctr {
            return Err(SecureSessionError::ReplayDetected);
        }

        let nonce = nonce_from_ctr(ctr);

        let plaintext = self
            .cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: ciphertext,
                    aad: ctr_bytes,
                },
            )
            .map_err(|_| SecureSessionError::DecryptionFailed)?;

        self.recv_ctr = ctr + 1;
        Ok(plaintext)
    }

    pub fn peer_identity(&self) -> &VerifyingKey {
        &self.peer_identity
    }
}

fn nonce_from_ctr(ctr: u64) -> Nonce {
    let mut nonce = [0u8; 12];
    nonce[4..].copy_from_slice(&ctr.to_be_bytes());
    Nonce::from_slice(&nonce).clone()
}

