use crate::constant::{STORE_AUTHORIZATION_KEY, STORE_COLLECTION, STORE_CONFIG_AEG, STORE_DIR};
use crate::crypto::AegCrypto;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::{Engine as _, engine::general_purpose};
use dirs_next::home_dir;
use rand_core::TryRngCore;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fs;
use std::path::PathBuf;

pub struct AegFileSystem;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectionLock {
    pub active: String,
    pub collections: Vec<String>,
}

impl AegFileSystem {
    pub fn get_config_path() -> PathBuf {
        let mut config_path = home_dir().expect("Failed to get home directory");
        config_path.push(STORE_DIR);
        if !config_path.exists() {
            fs::create_dir_all(&config_path).expect("Failed to create config directory");
        }
        config_path
    }

    pub fn reset_files() {
        let path = Self::get_config_path();
        if path.exists() {
            fs::remove_dir_all(&path).expect("Failed to delete .aegisr configuration directory");
        }
        fs::create_dir_all(&path).expect("Failed to recreate config directory");
    }

    pub fn validate_files() {
        let path = Self::get_config_path();
        let collection_lock: PathBuf = path.join(STORE_COLLECTION);
        let config_file = path.join(STORE_CONFIG_AEG);
        let auth_file = path.join(STORE_AUTHORIZATION_KEY);
        if !config_file.exists() || !auth_file.exists() || !collection_lock.exists() {
            println!("Missing file. Running initialize config.");
            Self::initialize_config(None, None);
        } else {
            if let Err(e) = Self::maybe_migrate_collection_lock() {
                eprintln!("Migration failed: {}. Reinitializing.", e);
                Self::initialize_config(None, None);
            }
        }
    }

    pub fn initialize_config(overwrite: Option<bool>, verbose_mode: Option<bool>) -> PathBuf {
        let overwrite_mode = overwrite.unwrap_or(false);
        let _verbose_mode = verbose_mode.unwrap_or(false);
        let dir = Self::get_config_path();

        if overwrite_mode && dir.exists() {
            fs::remove_dir_all(&dir).expect("Failed to remove existing config directory");
        }

        if !dir.exists() {
            fs::create_dir_all(&dir).expect("Failed to create config directory");
        }

        let key_path = dir.join(STORE_AUTHORIZATION_KEY);
        let auth_key = if key_path.exists() {
            fs::read_to_string(&key_path).expect("Failed to read AUTHORIZATION_KEY")
        } else {
            let k = AegCrypto::create_authorization_key(Some(_verbose_mode));
            fs::write(&key_path, &k).expect("Failed to write AUTHORIZATION_KEY");
            k
        };

        let collection_path = dir.join(STORE_COLLECTION);
        if !collection_path.exists() {
            Self::write_collection_lock_default(&auth_key);
        }

        dir
    }

    pub fn write_collection_lock_json(data: &str, auth_key: &str) {
        let key_bytes = general_purpose::STANDARD
            .decode(auth_key)
            .expect("Invalid base64");
        let key_arr: [u8; 32] = key_bytes
            .as_slice()
            .try_into()
            .expect("Auth key must be 32 bytes");
        let key: &aes_gcm::Key<Aes256Gcm> = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&key_arr[..12]);

        let encrypted = cipher
            .encrypt(nonce, data.as_bytes())
            .expect("Encrypt failed");
        let encoded = general_purpose::STANDARD.encode(&encrypted);

        let path = Self::get_config_path().join(STORE_COLLECTION);
        let mut file = fs::File::create(&path).expect("Failed to open file");
        use std::io::Write;
        file.write_all(encoded.as_bytes()).expect("Write failed");
        file.sync_all().expect("Flush failed");
    }

    pub fn read_collection_lock() -> String {
        let path = Self::get_config_path().join(STORE_COLLECTION);
        if !path.exists() {
            return String::new();
        }

        let auth_key = Self::read_authorization_key();
        let key_bytes = general_purpose::STANDARD
            .decode(auth_key)
            .expect("Invalid auth key");

        let key_arr: [u8; 32] = key_bytes
            .as_slice()
            .try_into()
            .expect("Auth key must be 32 bytes");
        let key: &aes_gcm::Key<Aes256Gcm> = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&key_arr[..12]);

        let encrypted = fs::read_to_string(&path).unwrap_or_default();
        if encrypted.is_empty() {
            return String::new();
        }

        let encrypted_bytes = general_purpose::STANDARD
            .decode(encrypted)
            .expect("Invalid base64 content");

        let decrypted = cipher
            .decrypt(nonce, encrypted_bytes.as_ref())
            .expect("Decrypt failed");

        String::from_utf8(decrypted).expect("Invalid UTF-8")
    }

    pub fn read_collection_lock_obj() -> CollectionLock {
        let json_str = Self::read_collection_lock();
        if json_str.trim().is_empty() {
            return CollectionLock {
                active: "default".to_string(),
                collections: vec!["default".to_string()],
            };
        }

        match serde_json::from_str::<CollectionLock>(&json_str) {
            Ok(lock) => lock,
            Err(_) => {
                let s = json_str.trim().trim_matches('"').to_string();
                let lock = CollectionLock {
                    active: s.clone(),
                    collections: vec![s],
                };

                let auth_key = Self::read_authorization_key();
                let serialized = serde_json::to_string_pretty(&lock).expect("Serialize failed");
                Self::write_collection_lock_json(&serialized, &auth_key);
                lock
            }
        }
    }

    fn maybe_migrate_collection_lock() -> Result<(), String> {
        let _ = Self::read_collection_lock_obj();
        Ok(())
    }

    pub fn write_collection_lock_default(auth_key: &str) {
        let lock = CollectionLock {
            active: "default".to_string(),
            collections: vec!["default".to_string()],
        };
        let serialized = serde_json::to_string_pretty(&lock).expect("Serialize failed");
        Self::write_collection_lock_json(&serialized, auth_key);
    }

    pub fn read_authorization_key() -> String {
        let path = Self::get_config_path().join(STORE_AUTHORIZATION_KEY);
        fs::read_to_string(&path).expect("Failed to read authorization key")
    }
}
