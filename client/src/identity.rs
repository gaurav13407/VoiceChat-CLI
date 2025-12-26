use std::fs;
use std::path::Path;
use rand_core::OsRng;
use ed25519_dalek::{Keypair, PUBLIC_KEY_LENGTH};

const ID_PATH: &str = "config/identity.key";

pub struct Identity {
    pub keypair: Keypair,
}

impl Identity {
    pub fn load_or_create() -> Self {
        if Path::new(ID_PATH).exists() {
            let bytes = fs::read(ID_PATH).expect("Failed to load identity");

            let keypair =
                Keypair::from_bytes(&bytes).expect("Invalid identity keypair");

            Self { keypair }
        } else {
            let mut rng = OsRng;
            let keypair = Keypair::generate(&mut rng);

            fs::create_dir_all("config").unwrap();
            fs::write(ID_PATH, keypair.to_bytes()).unwrap();

            Self { keypair }
        }
    }

    pub fn public_key_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.keypair.public.to_bytes()
    }
}

