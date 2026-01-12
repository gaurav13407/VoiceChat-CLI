use std::fs;
use std::path::PathBuf;
use rand_core::OsRng;
use ed25519_dalek::{Keypair, PUBLIC_KEY_LENGTH};

fn get_identity_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".voicechat");
    path.push("identity.key");
    path
}

pub struct Identity {
    pub keypair: Keypair,
}

impl Identity {
    pub fn load_or_create() -> Self {
        let id_path = get_identity_path();
        
        if id_path.exists() {
            let bytes = fs::read(&id_path).expect("Failed to load identity");

            let keypair =
                Keypair::from_bytes(&bytes).expect("Invalid identity keypair");

            Self { keypair }
        } else {
            let mut rng = OsRng;
            let keypair = Keypair::generate(&mut rng);

            fs::create_dir_all(id_path.parent().unwrap()).unwrap();
            fs::write(&id_path, keypair.to_bytes()).unwrap();

            Self { keypair }
        }
    }

    pub fn public_key_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.keypair.public.to_bytes()
    }
}

