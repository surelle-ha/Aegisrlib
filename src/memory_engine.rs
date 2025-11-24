use crate::core::AegCore;
use crate::file_system::AegFileSystem;
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::{Engine as _, engine::general_purpose};
use rand_core::TryRngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

/// IN-MEMORY KEY-VALUE STORE ENGINE
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AegMemoryEngine {
    pub store: HashMap<String, String>,
    pub collection_name: String,
}

/// SAFE GLOBAL IN-MEMORY CACHE (OnceLock + Mutex)
static MEMORY_CACHE: OnceLock<Mutex<HashMap<String, AegMemoryEngine>>> = OnceLock::new();

/// Background saver control
static SAVER_RUNNING: OnceLock<AtomicBool> = OnceLock::new();
static SAVER_STARTED: OnceLock<AtomicBool> = OnceLock::new();

impl AegMemoryEngine {
    /// Returns a reference to the global Mutex<HashMap<...>>.
    fn global_memory_mutex() -> &'static Mutex<HashMap<String, AegMemoryEngine>> {
        MEMORY_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
    }

    pub fn new(collection_name: &str) -> Self {
        Self {
            store: HashMap::new(),
            collection_name: collection_name.to_string(),
        }
    }

    fn engine_file_path(collection_name: &str) -> PathBuf {
        let mut path = AegFileSystem::get_config_path();
        path.push(format!("collection_{}.aekv", collection_name));
        path
    }

    /// Insert into current engine and update global in-memory cache (fast).
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.store.insert(key.into(), value.into());
        // persist to global in-memory cache (only memory)
        let mutex = Self::global_memory_mutex();
        let mut guard = mutex.lock().expect("Failed to lock global memory mutex");
        guard.insert(self.collection_name.clone(), self.clone());
        // intentionally not calling self.save() here
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.store.get(key).cloned()
    }

    pub fn delete(&mut self, key: &str) {
        self.store.remove(key);
        let mutex = Self::global_memory_mutex();
        let mut guard = mutex.lock().expect("Failed to lock global memory mutex");
        guard.insert(self.collection_name.clone(), self.clone());
    }

    pub fn list(&self) -> Vec<(String, String)> {
        self.store
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn clear(&mut self) {
        self.store.clear();
        let mutex = Self::global_memory_mutex();
        let mut guard = mutex.lock().expect("Failed to lock global memory mutex");
        guard.insert(self.collection_name.clone(), self.clone());
    }

    /// Persist single engine to disk (synchronous) — same encryption as before.
    pub fn save_to_disk(engine: &AegMemoryEngine) -> Result<(), String> {
        let path = Self::engine_file_path(&engine.collection_name);

        let json =
            serde_json::to_string_pretty(engine).map_err(|e| format!("serialize error: {}", e))?;

        let auth_key = AegFileSystem::read_authorization_key();
        let key_bytes = general_purpose::STANDARD
            .decode(auth_key)
            .map_err(|e| format!("base64 decode auth key: {}", e))?;

        let key: &aes_gcm::Key<Aes256Gcm> = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&key_bytes[..12]);

        let encrypted = cipher
            .encrypt(nonce, json.as_bytes())
            .map_err(|e| format!("encrypt error: {:?}", e))?;

        let encoded = general_purpose::STANDARD.encode(&encrypted);

        fs::write(&path, encoded).map_err(|e| format!("write error: {}", e))?;

        Ok(())
    }

    /// Save ALL collections currently in memory to disk.
    /// This function clones the cache under the mutex and performs expensive work outside the lock.
    pub fn save_all() {
        // 1) Clone the memory map under the lock (minimize lock time)
        let snapshot: HashMap<String, AegMemoryEngine> = {
            let mutex = Self::global_memory_mutex();
            let guard = mutex.lock().expect("Failed to lock global memory mutex");
            guard.clone()
        };

        // 2) For each collection, perform serialization/encryption/write outside the lock
        for (_name, engine) in snapshot.into_iter() {
            // best-effort: log errors but continue
            if let Err(e) = Self::save_to_disk(&engine) {
                eprintln!(
                    "Failed to save collection '{}': {}",
                    engine.collection_name, e
                );
            }
        }
    }

    /// Load engine from memory cache; otherwise load from disk; otherwise fresh engine.
    pub fn load() -> Self {
        let core = AegCore::load();
        let collection_name = core.active_collection.clone();

        // First try in-memory (global cache)
        {
            let mutex = Self::global_memory_mutex();
            let guard = mutex.lock().expect("Failed to lock global memory mutex");
            if let Some(engine) = guard.get(&collection_name).cloned() {
                return engine;
            }
        }

        // If not in memory, load from disk
        let path = Self::engine_file_path(&collection_name);

        if path.exists() {
            let encrypted = fs::read_to_string(&path).unwrap_or_default();
            if encrypted.trim().is_empty() {
                let engine = Self::new(&collection_name);
                // store in memory
                let mutex = Self::global_memory_mutex();
                let mut guard = mutex.lock().expect("Failed to lock global memory mutex");
                guard.insert(collection_name.clone(), engine.clone());
                return engine;
            }

            let auth_key = AegFileSystem::read_authorization_key();
            let key_bytes = general_purpose::STANDARD
                .decode(auth_key)
                .expect("Invalid base64");

            let key: &aes_gcm::Key<Aes256Gcm> = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
            let cipher = Aes256Gcm::new(key);

            let nonce = Nonce::from_slice(&key_bytes[..12]);

            let decoded = general_purpose::STANDARD
                .decode(encrypted)
                .expect("Invalid base64");

            let decrypted = cipher
                .decrypt(nonce, decoded.as_ref())
                .expect("Decrypt failed");

            let engine: AegMemoryEngine =
                serde_json::from_slice(&decrypted).unwrap_or(Self::new(&collection_name));

            // Store to in-memory cache
            let mutex = Self::global_memory_mutex();
            let mut guard = mutex.lock().expect("Failed to lock global memory mutex");
            guard.insert(collection_name.clone(), engine.clone());

            return engine;
        }

        // Fresh engine
        let engine = Self::new(&collection_name);
        let mutex = Self::global_memory_mutex();
        let mut guard = mutex.lock().expect("Failed to lock global memory mutex");
        guard.insert(collection_name.clone(), engine.clone());
        engine
    }

    /// Start a background thread to periodically save memory to disk.
    /// If already started, this is a no-op.
    pub fn start_background_saver(interval_seconds: u64) {
        // initialize the running flag (if not already)
        let running = SAVER_RUNNING.get_or_init(|| AtomicBool::new(false));
        let started_flag = SAVER_STARTED.get_or_init(|| AtomicBool::new(false));

        // if already started, do nothing
        if started_flag.load(Ordering::SeqCst) {
            return;
        }

        // mark running
        running.store(true, Ordering::SeqCst);
        // mark started
        started_flag.store(true, Ordering::SeqCst);

        // spawn detached thread
        let running_ref: &'static AtomicBool = running;
        thread::spawn(move || {
            let interval = Duration::from_secs(interval_seconds.max(1));
            while running_ref.load(Ordering::SeqCst) {
                // save snapshot
                Self::save_all();
                // sleep for interval (cooperative)
                sleep(interval);
            }
            // final flush on exit attempt
            Self::save_all();
        });
    }

    /// Signal the background saver to stop. Thread is detached so we can't join; this just signals termination.
    pub fn stop_background_saver() {
        if let Some(running) = SAVER_RUNNING.get() {
            running.store(false, Ordering::SeqCst);
        }
        if let Some(started) = SAVER_STARTED.get() {
            started.store(false, Ordering::SeqCst);
        }
    }
}

// ===================== USAGE GUIDE =====================
//
// During startup:
// AegFileSystem::initialize_config(None, None);   // prepares configuration files
// AegCore::start_background_saver(1);             // enables automatic persistence (1-second interval)
//
// Normal operations use:
// AegCore::put_value(...);
// AegCore::get_value(...);
//
// For an immediate write to disk:
// AegCore::flush_now();
//
// At application shutdown:
// AegCore::stop_background_saver();               // stops the background thread
// AegCore::flush_now();                           // optional final save
