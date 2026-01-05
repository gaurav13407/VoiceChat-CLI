use x25519_dalek::{EphemeralSecret, PublicKey};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use serde_bytes;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientHello {
    pub client_id: [u8; 32],             // Ed25519 public key
    pub client_ephemeral_pub: [u8; 32],  // X25519 public key
    pub nonce_c: [u8; 32],
}


#[derive(Serialize, Deserialize, Debug)]
pub struct HostChallenge {
    pub host_id: [u8; 32],
    pub host_ephemeral_pub: [u8; 32],
    pub nonce_h: [u8; 32],
    #[serde(with = "serde_bytes")] 
    pub sig_h: [u8; 64],
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ClientResponse {
    #[serde(with = "serde_bytes")]
    pub sig_c: [u8; 64],
}

pub struct Handshake {
    secret: EphemeralSecret,
    pub public: PublicKey,
}

impl Handshake {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let secret = EphemeralSecret::random_from_rng(&mut rng);
        let public = PublicKey::from(&secret);

        Self { secret, public }
    }

    pub fn derive_shared(self, peer_pub: &PublicKey) -> [u8; 32] {
        let shared = self.secret.diffie_hellman(peer_pub);
        *shared.as_bytes()
    }
}

