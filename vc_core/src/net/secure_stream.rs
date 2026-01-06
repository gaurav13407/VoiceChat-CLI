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
        Self { stream, session }
    }

    /// Send one encrypted frame
    pub fn send(&mut self, plaintext: &[u8]) -> Result<(), SecureStreamError> {
        let encrypted = self.session.encrypt(plaintext);

        if encrypted.len() > u16::MAX as usize {
            return Err(SecureStreamError::FrameTooLarge);
        }

        let len = encrypted.len() as u16;
        let len_bytes = len.to_be_bytes();

        // LEN || ENCRYPTED_DATA
        self.stream.write_all(&len_bytes)?;
        self.stream.write_all(&encrypted)?;
        self.stream.flush()?;

        Ok(())
    }

    /// Receive one encrypted frame
    pub fn recv(&mut self) -> Result<Vec<u8>, SecureStreamError> {
        // Read LEN
        let mut len_buf = [0u8; 2];
        self.stream.read_exact(&mut len_buf)?;
        let len = u16::from_be_bytes(len_buf) as usize;

        if len == 0 {
            return Err(SecureStreamError::UnexpectedEof);
        }

        // Read ENCRYPTED_DATA
        let mut enc_buf = vec![0u8; len];
        self.stream.read_exact(&mut enc_buf)?;

        let plaintext = self.session.decrypt(&enc_buf)?;
        Ok(plaintext)
    }

    pub fn into_inner(self) -> TcpStream {
        self.stream
    }
}
