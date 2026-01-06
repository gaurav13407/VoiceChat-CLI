// use std::net::{TcpListener, TcpStream};
// use ed25519_dalek::{Signature, PublicKey};
// use vc_core::net::host_handshek::HostHandshake;
// use vc_core::net::secure_stream::SecureStream;
// use vc_core::state::secure_session::{SecureSession, SessionRole};

// pub type HostError = Box<dyn std::error::Error + Send + Sync>;

// TODO: Implement these helper functions:
// - recv_client_hello(&mut TcpStream) -> Result<ClientHello, Error>
// - send_host_challenge(&mut TcpStream, &HostChallenge) -> Result<(), Error>
// - recv_client_response(&mut TcpStream) -> Result<ClientResponse, Error>
//
// Example using bincode:
//   fn recv_client_hello(stream: &mut TcpStream) -> std::io::Result<ClientHello> {
//       let mut len_buf = [0u8; 4];
//       stream.read_exact(&mut len_buf)?;
//       let len = u32::from_be_bytes(len_buf) as usize;
//       let mut buf = vec![0u8; len];
//       stream.read_exact(&mut buf)?;
//       Ok(bincode::deserialize(&buf).unwrap())
//   }

/*
pub fn run_host(
    bind_addr: &str,
    identity: ed25519_dalek::Keypair,
) -> Result<SecureStream, HostError> {
    // 1. Listen for peer
    let listener = TcpListener::bind(bind_addr)?;
    println!("Hosting on {}", bind_addr);

    let (mut stream, peer_addr) = listener.accept()?;
    println!("Peer connected from {}", peer_addr);

    // 2. Receive ClientHello
    let hello = recv_client_hello(&mut stream)?;

    // 3. Run handshake
    let host_hs = HostHandshake::new(identity);

    let challenge = host_hs.challenge(&hello);
    send_host_challenge(&mut stream, &challenge)?;

    let response = recv_client_response(&mut stream)?;

    // 4. Extract peer identity
    let peer_identity =
        PublicKey::from_bytes(&hello.client_id)?;

    let session_key =
        host_hs.verify_response(&hello, response);

    // 5. Upgrade to SecureSession
    let session = SecureSession::new(
        SessionRole::Host,
        session_key,
        peer_identity,
    );

    // 6. Upgrade TCP → SecureStream
    let secure_stream = SecureStream::new(stream, session);

    Ok(secure_stream)
}
*/

