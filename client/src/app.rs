// use std::net::TcpStream;
// use ed25519_dalek::{Signature, PublicKey};
// use vc_core::net::secure_stream::SecureStream;
// use vc_core::state::secure_session::{SecureSession, SessionRole};
// use vc_core::net::client_handshake::ClientHandshake;

// TEMP error type (can improve later)
// pub type ClientError = Box<dyn std::error::Error + Send + Sync>;

// TODO: Implement these helper functions:
// - send_client_hello(&mut TcpStream, ClientHello) -> Result<(), Error>
// - recv_host_challenge(&mut TcpStream) -> Result<HostChallenge, Error>
// - send_client_response(&mut TcpStream, ClientResponse) -> Result<(), Error>
//
// Example using bincode:
//   fn send_client_hello(stream: &mut TcpStream, hello: ClientHello) -> std::io::Result<()> {
//       let data = bincode::serialize(&hello).unwrap();
//       stream.write_all(&(data.len() as u32).to_be_bytes())?;
//       stream.write_all(&data)?;
//       Ok(())
//   }

/*
pub fn connect_to_host(
    addr: &str,
    identity: ed25519_dalek::Keypair,
) -> Result<SecureStream, ClientError> {
    // 1. TCP connect
    let mut stream = TcpStream::connect(addr)?;

    // 2. Handshake
    let client_hs = ClientHandshake::new(identity);

    send_client_hello(&mut stream, client_hs.hello())?;
    let challenge = recv_host_challenge(&mut stream)?;

    // ⬅ Extract peer identity BEFORE moving challenge
    let peer_identity =
        PublicKey::from_bytes(&challenge.host_id)?;

    let (session_key, response) =
        client_hs.handle_challenge(challenge);

    send_client_response(&mut stream, response)?;

    // 3. Create SecureSession
    let session = SecureSession::new(
        SessionRole::Client,
        session_key,
        peer_identity,
    );

    // 4. Upgrade TCP → SecureStream
    let secure_stream = SecureStream::new(stream, session);

    Ok(secure_stream)
}
*/

