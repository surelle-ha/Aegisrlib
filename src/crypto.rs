use base64::{Engine as _, engine::general_purpose};
use rand_core::{OsRng, TryRngCore};
use zeroize::Zeroize;

pub struct AegCrypto;

impl AegCrypto {
    pub fn generate_random_bytes(_verbose: Option<bool>) -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.try_fill_bytes(&mut key).unwrap();
        key
    }

    pub fn encode_base64(input: impl AsRef<[u8]>, _verbose: Option<bool>) -> String {
        general_purpose::STANDARD.encode(input.as_ref())
    }

    pub fn create_authorization_key(_verbose: Option<bool>) -> String {
        let mut bytes = Self::generate_random_bytes(None);
        let hash = blake3::hash(&bytes);
        bytes.zeroize();
        Self::encode_base64(hash.as_bytes(), None)
    }
}
