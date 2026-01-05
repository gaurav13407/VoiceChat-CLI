
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use rand::RngCore;

use vc_core::client_handshake::ClientHandshake;
use vc_core::host_handshek::HostHandshake;

#[test]
fn handshake_produces_same_session_key() {
    // identities
    let mut client_secret = [0u8; 32];
    OsRng.fill_bytes(&mut client_secret);
    let client_id = SigningKey::from_bytes(&client_secret);

    let mut host_secret = [0u8; 32];
    OsRng.fill_bytes(&mut host_secret);
    let host_id = SigningKey::from_bytes(&host_secret);

    // init handshakes
    let client = ClientHandshake::new(client_id);
    let host   = HostHandshake::new(host_id);

    // step 1: client -> host
    let hello = client.hello();

    // step 2: host -> client
    let challenge = host.challenge(&hello);

    // step 3: client verifies + responds
    let (client_key, response) = client.handle_challenge(challenge);

    // step 4: host verifies
    let host_key = host.verify_response(&hello, response);

    // FINAL ASSERTION
    assert_eq!(client_key, host_key);
}
