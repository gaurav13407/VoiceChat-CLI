use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519Public};
use rand::rngs::OsRng;
use crate::handshake::*;
use crate::crypto::{random_nonce, derive_session_key};

pub struct ClientHandshake {
    pub identity: SigningKey,
    pub eph_secret: EphemeralSecret,
    pub eph_public: X25519Public,
    pub nonce_c: [u8; 32],
}

impl ClientHandshake {
    pub fn new(identity: SigningKey) -> Self {
        let mut rng = OsRng;
        let eph_secret = EphemeralSecret::random_from_rng(&mut rng);
        let eph_public = X25519Public::from(&eph_secret);

        Self {
            identity,
            eph_secret,
            eph_public,
            nonce_c: random_nonce(),
        }
    }

    pub fn hello(&self) -> ClientHello {
        ClientHello {
            client_id: VerifyingKey::from(&self.identity).to_bytes(),
            client_ephemeral_pub: self.eph_public.to_bytes(),
            nonce_c: self.nonce_c,
        }
    }

    pub fn handle_challenge(
        self,
        challenge: HostChallenge,
    ) -> ([u8; 32], ClientResponse) {

        let host_pub = VerifyingKey::from_bytes(&challenge.host_id).unwrap();

        let signed = [
            self.nonce_c.as_slice(),
            challenge.nonce_h.as_slice(),
            self.eph_public.as_bytes(),
            &challenge.host_ephemeral_pub,
        ].concat();

        host_pub.verify(
            &signed,
            &Signature::from_bytes(&challenge.sig_h),
        ).expect("Host signature invalid");

        let host_eph = X25519Public::from(challenge.host_ephemeral_pub);
        let shared_secret = self.eph_secret.diffie_hellman(&host_eph);

        let session_key = derive_session_key(
            *shared_secret.as_bytes(),
            &self.nonce_c,
            &challenge.nonce_h,
        );

        let sig_c = self.identity.sign(&[
            challenge.nonce_h.as_slice(),
            self.nonce_c.as_slice(),
            &challenge.host_ephemeral_pub,
            self.eph_public.as_bytes(),
        ].concat());

        (
            session_key,
            ClientResponse {
                sig_c: sig_c.to_bytes(),
            }
        )
    }
}

