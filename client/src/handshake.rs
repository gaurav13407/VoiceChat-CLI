use x25519_dalek::{EphemeralSecret, PublicKey};
use rand_core::OsRng;

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

    pub fn derive_shared(&self, peer_pub: PublicKey) -> [u8; 32] {
        let shared = self.secret.diffie_hellman(&peer_pub);
        *shared.as_bytes()
    }
}

