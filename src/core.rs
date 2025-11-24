use crate::constant::STORE_COLLECTION;
use crate::file_system::{AegFileSystem, CollectionLock};
use crate::memory_engine::AegMemoryEngine;
use rand_core::TryRngCore;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct AegCore {
    pub active_collection: String,
    pub collections: Vec<String>,
}

impl AegCore {
    pub fn load() -> Self {
        let lock = AegFileSystem::read_collection_lock_obj();
        Self {
            active_collection: lock.active,
            collections: lock.collections,
        }
    }

    pub fn save(&self) {
        let lock = CollectionLock {
            active: self.active_collection.clone(),
            collections: self.collections.clone(),
        };
        let json = serde_json::to_string_pretty(&lock).expect("Serialize failed");
        let auth_key = AegFileSystem::read_authorization_key();

        let path = AegFileSystem::get_config_path().join(STORE_COLLECTION);
        fs::write(&path, json.clone()).expect("Write failed");

        AegFileSystem::write_collection_lock_json(&json, &auth_key);
    }

    pub fn get_active_collection(&self) -> &str {
        &self.active_collection
    }

    pub fn set_active_collection(&mut self, name: &str) -> Result<(), String> {
        if !self.collections.contains(&name.to_string()) {
            return Err(format!("Collection '{}' does not exist", name));
        }
        self.active_collection = name.to_string();
        self.save();
        Ok(())
    }

    pub fn create_collection(name: &str) -> String {
        let mut core = Self::load();
        if core.collections.contains(&name.to_string()) {
            return format!("✗ Collection '{}' already exists", name);
        }

        core.collections.push(name.to_string());
        core.save();

        let _ = Self::load();

        format!("✓ Collection '{}' created", name)
    }

    pub fn delete_collection(name: &str) -> String {
        let mut core = Self::load();
        if core.collections.len() == 1 {
            return "✗ Cannot delete the last collection".into();
        }
        if let Some(pos) = core.collections.iter().position(|x| x == name) {
            core.collections.remove(pos);
            if core.active_collection == name {
                core.active_collection = core.collections[0].clone();
            }
            core.save();
            format!("✓ Collection '{}' deleted", name)
        } else {
            format!("✗ Collection '{}' does not exist", name)
        }
    }

    pub fn rename_collection(name: &str, new_name: &str) -> String {
        let mut core = Self::load();
        if core.collections.contains(&new_name.to_string()) {
            return format!("✗ Collection '{}' already exists", new_name);
        }
        if let Some(pos) = core.collections.iter().position(|x| x == name) {
            core.collections[pos] = new_name.to_string();
            if core.active_collection == name {
                core.active_collection = new_name.to_string();
            }
            core.save();
            format!("✓ Collection '{}' renamed to '{}'", name, new_name)
        } else {
            format!("✗ Collection '{}' does not exist", name)
        }
    }

    /// Insert into memory (non-blocking). Does not perform immediate disk save.
    /// Background saver (if started) will persist this later.
    pub fn put_value(key: &str, value: &str) -> String {
        let mut engine = AegMemoryEngine::load();
        engine.insert(key, value);
        // no engine.save() here - background saver will persist
        format!(
            "✓ Key '{}' saved in collection '{}' (in-memory)",
            key, engine.collection_name
        )
    }

    /// Read from memory (plaintext in RAM).
    pub fn get_value(key: &str) -> Option<String> {
        let engine = AegMemoryEngine::load();
        engine.get(key)
    }

    /// Delete in-memory (non-blocking). Background saver will persist deletion later.
    pub fn delete_value(key: &str) -> String {
        let mut engine = AegMemoryEngine::load();
        if engine.get(key).is_some() {
            engine.delete(key);
            // no engine.save() here
            format!(
                "✓ Key '{}' deleted from collection '{}' (in-memory)",
                key, engine.collection_name
            )
        } else {
            format!(
                "✗ Key '{}' not found in collection '{}' (in-memory)",
                key, engine.collection_name
            )
        }
    }

    /// Clear in-memory values (non-blocking). Background saver will persist later.
    pub fn clear_values() -> String {
        let mut engine = AegMemoryEngine::load();
        engine.clear();
        format!(
            "✓ All keys cleared from collection '{}' (in-memory)",
            engine.collection_name
        )
    }

    /// Force immediate flush (saves all collections to disk synchronously).
    pub fn flush_now() {
        AegMemoryEngine::save_all();
    }

    /// Start background saver thread. Safe to call multiple times.
    /// interval_seconds: how often to persist (e.g. 1).
    pub fn start_background_saver(interval_seconds: u64) {
        AegMemoryEngine::start_background_saver(interval_seconds);
    }

    /// Signal background saver to stop. Returns immediately.
    pub fn stop_background_saver() {
        AegMemoryEngine::stop_background_saver();
    }
}
