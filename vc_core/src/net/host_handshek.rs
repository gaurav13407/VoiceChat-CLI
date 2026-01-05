use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519Public};
use rand::rngs::OsRng;
use crate::handshake::*;
use crate::crypto::{random_nonce, derive_session_key};

pub struct HostHandshake{
    pub identity:SigningKey,
    pub eph_secret:EphemeralSecret,
    pub eph_public:X25519Public,
    pub nonce_h:[u8;32],
}

impl HostHandshake{
    pub fn new(identity: SigningKey)->Self {
        let mut rng = OsRng;
        let eph_secret = EphemeralSecret::random_from_rng(&mut rng);
        let eph_public = X25519Public::from(&eph_secret);

        Self{
            identity,
            eph_secret,
            eph_public,
            nonce_h: random_nonce(),
        }
    }

    pub fn challenge(&self,hello:&ClientHello)->HostChallenge{
        let signed=[
            hello.nonce_c.as_slice(),
            self.nonce_h.as_slice(),
            &hello.client_ephemeral_pub,
            self.eph_public.as_bytes(),
        ].concat();

        let sig_h=self.identity.sign(&signed);

        HostChallenge{
            host_id: VerifyingKey::from(&self.identity).to_bytes(),
            host_ephemeral_pub:self.eph_public.to_bytes(),
            nonce_h:self.nonce_h,
            sig_h: sig_h.to_bytes(),
        }
    }

    pub fn verify_response(
        self,
        hello:&ClientHello,
        response:ClientResponse,
    )->[u8;32]{
        let sig = Signature::from_bytes(&response.sig_c);
        let client_pub = VerifyingKey::from_bytes(&hello.client_id).unwrap();
        
        // Verify client's signature
        let signed = [
            self.nonce_h.as_slice(),
            hello.nonce_c.as_slice(),
            self.eph_public.as_bytes(),
            &hello.client_ephemeral_pub,
        ].concat();
        
        client_pub.verify(&signed, &sig)
            .expect("Client signature invalid");
        
        let client_eph = X25519Public::from(hello.client_ephemeral_pub);
        let shared = self.eph_secret.diffie_hellman(&client_eph);

        derive_session_key(
            *shared.as_bytes(),
            &hello.nonce_c,
            &self.nonce_h,
        )
    }
}
