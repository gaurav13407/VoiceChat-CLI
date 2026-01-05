use rand::rngs::OsRng;
use rand::RngCore;
use hkdf::Hkdf;
use sha2::Sha256;

pub fn random_nonce() -> [u8; 32] {
    let mut n = [0u8; 32];
    OsRng.fill_bytes(&mut n);
    n
}

pub fn derive_session_key(
    shared_secret: [u8; 32],
    nonce_c: &[u8; 32],
    nonce_h: &[u8; 32],
) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, &shared_secret);
    let mut key = [0u8; 32];

    let info = [nonce_c.as_slice(), nonce_h.as_slice()].concat();

    hk.expand(&info, &mut key)
        .expect("HKDF expand failed");

    key
}

