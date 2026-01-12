use x25519_dalek::{EphemeralSecret, PublicKey};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use serde_bytes;
use std::net::TcpStream;
use std::io::{Read,Write};

use crate::state::secure_session::{SecureSession,SessionRole};
use crate::net::secure_stream::SecureStream;

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
pub fn run(
    mut stream:TcpStream,
    my_pubkey:[u8;32],
    peer_pubkey:[u8;32],
)->anyhow::Result<SecureStream>{
    eprintln!("[CLIENT] Starting handshake as initiator...");
    //1.create ephermal handshake state 
    let hs=Handshake::new();

    //2.Send client hello 
    let hello=ClientHello{
        client_id:my_pubkey,
        client_ephemeral_pub:hs.public.as_bytes().clone(),
        nonce_c:rand::random(),
    };

    let msg=bincode::serialize(&hello)?;
    eprintln!("[CLIENT] Sending ClientHello ({} bytes)...", msg.len());
    stream.write_all(&msg)?;
    stream.flush()?;
    eprintln!("[CLIENT] ClientHello sent, waiting for HostChallenge...");

    //3.recive HostChallenge 
    
    let mut buf=vec![0u8;1024];
    let n=stream.read(&mut buf)?;
    eprintln!("[CLIENT] Received {} bytes", n);

    let challenge:HostChallenge=bincode::deserialize(&buf[..n])?;
    eprintln!("[CLIENT] HostChallenge decoded successfully");

    //4.Verify the host identity 

    //5.Derive shared secret 
    let peer_ephemeral=
        x25519_dalek::PublicKey::from(challenge.host_ephemeral_pub);

    let shared=hs.derive_shared(&peer_ephemeral);

    //6.Crate SecureSession
    let peer_verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&peer_pubkey)
        .map_err(|e| anyhow::anyhow!("Invalid peer public key: {}", e))?;
    let session=SecureSession::new(SessionRole::Client, shared, peer_verifying_key);

    eprintln!("[CLIENT] Handshake complete!");
    Ok(SecureStream::new(stream,session))
}

pub fn run_as_host(
    mut stream: TcpStream,
    my_pubkey: [u8; 32],
    peer_pubkey: [u8; 32],
) -> anyhow::Result<SecureStream> {
    eprintln!("[HOST] Starting handshake as responder...");
    // 1. Create ephemeral handshake state
    let hs = Handshake::new();

    // 2. Receive ClientHello
    eprintln!("[HOST] Waiting for ClientHello...");
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf)?;
    eprintln!("[HOST] Received {} bytes", n);
    let hello: ClientHello = bincode::deserialize(&buf[..n])?;
    eprintln!("[HOST] ClientHello decoded successfully");

    // 3. Send HostChallenge
    let challenge = HostChallenge {
        host_id: my_pubkey,
        host_ephemeral_pub: hs.public.as_bytes().clone(),
        nonce_h: rand::random(),
        sig_h: [0u8; 64], // Simplified: no signature verification yet
    };

    let msg = bincode::serialize(&challenge)?;
    eprintln!("[HOST] Sending HostChallenge ({} bytes)...", msg.len());
    stream.write_all(&msg)?;
    stream.flush()?;
    eprintln!("[HOST] HostChallenge sent");

    // 4. Derive shared secret
    let peer_ephemeral = x25519_dalek::PublicKey::from(hello.client_ephemeral_pub);
    let shared = hs.derive_shared(&peer_ephemeral);

    // 5. Create SecureSession
    let peer_verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&peer_pubkey)
        .map_err(|e| anyhow::anyhow!("Invalid peer public key: {}", e))?;
    let session = SecureSession::new(SessionRole::Host, shared, peer_verifying_key);

    eprintln!("[HOST] Handshake complete!");
    Ok(SecureStream::new(stream, session))
}
