use std::io::{Read, Write};
use std::net::TcpStream;

use crate::state::secure_session::{SecureSession, SecureSessionError};
#[derive(Debug)]
pub enum SecureStreamError {
    Io(std::io::Error),
    Crypto(SecureSessionError),
    FrameTooLarge,
    UnexpectedEof,
}

impl From<std::io::Error> for SecureStreamError {
    fn from(e: std::io::Error) -> Self {
        SecureStreamError::Io(e)
    }
}

impl From<SecureSessionError> for SecureStreamError {
    fn from(e: SecureSessionError) -> Self {
        SecureStreamError::Crypto(e)
    }
}

pub struct SecureStream {
    stream: TcpStream,
    session: SecureSession,
}

impl SecureStream {
    pub fn new(stream: TcpStream, session: SecureSession) -> Self {
        // Set stream to non-blocking mode to prevent deadlocks
        stream.set_nonblocking(true).ok();
        Self { stream, session }
    }

    pub fn try_clone(&self) -> Result<TcpStream, std::io::Error> {
        self.stream.try_clone()
    }

    /// Send one encrypted frame
    pub fn send(&mut self, plaintext: &[u8]) -> Result<(), SecureStreamError> {
        eprintln!("[SecureStream] Encrypting {} bytes", plaintext.len());
        let encrypted = self.session.encrypt(plaintext);
        eprintln!("[SecureStream] Encrypted to {} bytes", encrypted.len());

        if encrypted.len() > u16::MAX as usize {
            return Err(SecureStreamError::FrameTooLarge);
        }

        let len = encrypted.len() as u16;
        let len_bytes = len.to_be_bytes();

        // LEN || ENCRYPTED_DATA
        eprintln!("[SecureStream] Writing {} byte frame (2 byte len + {} byte data)", len + 2, len);
        self.stream.write_all(&len_bytes)?;
        self.stream.write_all(&encrypted)?;
        self.stream.flush()?;
        eprintln!("[SecureStream] Frame sent successfully");

        Ok(())
    }

    /// Receive one encrypted frame
    pub fn recv(&mut self) -> Result<Vec<u8>, SecureStreamError> {
        // Read LEN (silently - no debug spam)
        let mut len_buf = [0u8; 2];
        self.stream.read_exact(&mut len_buf)?;
        let len = u16::from_be_bytes(len_buf) as usize;

        if len == 0 {
            return Err(SecureStreamError::UnexpectedEof);
        }

        // Read ENCRYPTED_DATA
        let mut enc_buf = vec![0u8; len];
        self.stream.read_exact(&mut enc_buf)?;
        eprintln!("[SecureStream] Received {} byte frame, decrypting...", len);

        let plaintext = self.session.decrypt(&enc_buf)?;
        eprintln!("[SecureStream] Decrypted to {} bytes", plaintext.len());
        Ok(plaintext)
    }

    pub fn into_inner(self) -> TcpStream {
        self.stream
    }

}
