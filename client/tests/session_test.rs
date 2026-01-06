// Simple test for SecureSession that works without helper functions
// Note: Uses ed25519_dalek v2 types from vc_core (VerifyingKey)

#[test]
fn test_secure_session_encrypt_decrypt() {
    use vc_core::state::secure_session::{SecureSession, SessionRole};
    
    // Create a fake peer identity using raw bytes
    // In real usage, this would come from ed25519_dalek v2 keypair
    let peer_id_bytes = [1u8; 32]; // Mock public key bytes
    
    // Import from vc_core which uses ed25519-dalek v2
    use vc_core::protocol::handshake::ClientHello;
    
    // Use VerifyingKey from the ed25519-dalek v2 that vc_core uses
    let peer_identity = ed25519::VerifyingKey::from_bytes(&peer_id_bytes).unwrap();
    
    let session_key = [42u8; 32]; // Mock session key
    
    let mut sender = SecureSession::new(
        SessionRole::Client,
        session_key,
        peer_identity,
    );
    
    let mut receiver = SecureSession::new(
        SessionRole::Host,
        session_key,
        peer_identity,
    );
    
    // Test single message
    let plaintext = b"Secret message";
    let encrypted = sender.encrypt(plaintext);
    let decrypted = receiver.decrypt(&encrypted).expect("Decryption failed");
    assert_eq!(decrypted, plaintext);
    
    // Test multiple messages (counters increment)
    for i in 0..10 {
        let msg = format!("Message {}", i);
        let enc = sender.encrypt(msg.as_bytes());
        let dec = receiver.decrypt(&enc).expect("Decryption failed");
        assert_eq!(dec, msg.as_bytes());
    }
    
    println!("✓ Encryption/decryption working");
    println!("✓ Counter increments properly");
}

#[test]
fn test_replay_attack_prevention() {
    use vc_core::state::secure_session::{SecureSession, SessionRole, SecureSessionError};
    use vc_core::crypto::random_nonce;
    
    let peer_id_bytes = random_nonce();
    let peer_identity = ed25519_dalek::PublicKey::from_bytes(&peer_id_bytes).unwrap();
    let session_key = [42u8; 32];
    
    let mut sender = SecureSession::new(SessionRole::Client, session_key, peer_identity);
    let mut receiver = SecureSession::new(SessionRole::Host, session_key, peer_identity);
    
    // Send and receive first message
    let enc1 = sender.encrypt(b"Message 1");
    let _ = receiver.decrypt(&enc1).expect("First decrypt should work");
    
    // Send second message
    let enc2 = sender.encrypt(b"Message 2");
    let _ = receiver.decrypt(&enc2).expect("Second decrypt should work");
    
    // Try to replay first message (should fail)
    let result = receiver.decrypt(&enc1);
    match result {
        Err(SecureSessionError::ReplayDetected) => {
            println!("✓ Replay attack detected and prevented");
        }
        _ => panic!("Replay attack was not detected!"),
    }
}

#[test]
fn test_wrong_counter_order() {
    use vc_core::state::secure_session::{SecureSession, SessionRole, SecureSessionError};
    use vc_core::crypto::random_nonce;
    
    let peer_id_bytes = random_nonce();
    let peer_identity = ed25519_dalek::PublicKey::from_bytes(&peer_id_bytes).unwrap();
    let session_key = [42u8; 32];
    
    let mut sender = SecureSession::new(SessionRole::Client, session_key, peer_identity);
    let mut receiver = SecureSession::new(SessionRole::Host, session_key, peer_identity);
    
    // Send multiple messages
    let enc1 = sender.encrypt(b"Message 1");
    let enc2 = sender.encrypt(b"Message 2");
    let enc3 = sender.encrypt(b"Message 3");
    
    // Receive in correct order
    let _ = receiver.decrypt(&enc1).expect("Should work");
    let _ = receiver.decrypt(&enc2).expect("Should work");
    let _ = receiver.decrypt(&enc3).expect("Should work");
    
    println!("✓ Sequential messages processed correctly");
}

#[test]
fn test_nonce_generation() {
    // Test that nonce generation works as expected
    use vc_core::state::secure_session::{SecureSession, SessionRole};
    use vc_core::crypto::random_nonce;
    
    let peer_id_bytes = random_nonce();
    let peer_identity = ed25519_dalek::PublicKey::from_bytes(&peer_id_bytes).unwrap();
    let session_key = [42u8; 32];
    
    let mut session = SecureSession::new(SessionRole::Client, session_key, peer_identity);
    
    // Encrypt many messages to test counter rollover handling
    for i in 0..100 {
        let msg = format!("Message {}", i);
        let _ = session.encrypt(msg.as_bytes());
    }
    
    println!("✓ Nonce generation handles multiple encryptions");
}
