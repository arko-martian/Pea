//! CAS index for metadata management
//!
//! This module provides the CasIndex for tracking cached entries
//! and their metadata.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use pea_core::error::PeaError;
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use super::ContentHash;
use crate::CacheResult;

/// Metadata for a cached entry
#[derive(
    Debug, Clone, PartialEq, Eq,
    SerdeSerialize, SerdeDeserialize
)]
pub struct CacheEntry {
    /// Content hash
    pub hash: ContentHash,
    /// Size in bytes
    pub size: u64,
    /// When the entry was stored (as timestamp)
    pub stored_at: i64,
    /// When the entry was last accessed (as timestamp)
    pub last_accessed: i64,
}

impl CacheEntry {
    /// Create a new cache entry
    pub fn new(hash: ContentHash, size: u64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            hash,
            size,
            stored_at: now,
            last_accessed: now,
        }
    }

    /// Update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = Utc::now().timestamp();
    }

    /// Get age of entry in seconds
    pub fn age_seconds(&self) -> i64 {
        Utc::now().timestamp() - self.stored_at
    }

    /// Get stored_at as DateTime
    pub fn stored_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.stored_at, 0).unwrap_or_else(|| Utc::now())
    }

    /// Get last_accessed as DateTime
    pub fn last_accessed_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.last_accessed, 0).unwrap_or_else(|| Utc::now())
    }
}
/// Index for managing CAS entries
#[derive(Debug)]
pub struct CasIndex {
    /// In-memory index of entries
    entries: Arc<DashMap<String, CacheEntry>>,
    /// Path to the index file
    index_path: std::path::PathBuf,
}

impl CasIndex {
    /// Load existing index or create new one
    pub fn load_or_create<P: AsRef<Path>>(index_path: P) -> CacheResult<Self> {
        let index_path = index_path.as_ref().to_path_buf();
        let entries = Arc::new(DashMap::new());
        
        // Try to load existing index
        if index_path.exists() {
            match fs::read_to_string(&index_path) {
                Ok(content) => {
                    if let Ok(loaded_entries) = serde_json::from_str::<Vec<(String, CacheEntry)>>(&content) {
                        for (key, entry) in loaded_entries {
                            entries.insert(key, entry);
                        }
                    }
                }
                Err(_) => {
                    // If we can't read the index, start fresh
                }
            }
        }
        
        Ok(Self {
            entries,
            index_path,
        })
    }

    /// Insert a new entry
    pub fn insert(&self, key: String, entry: CacheEntry) {
        self.entries.insert(key, entry);
    }

    /// Get an entry by key
    pub fn get(&self, key: &str) -> Option<CacheEntry> {
        if let Some(entry_ref) = self.entries.get(key) {
            let mut entry = entry_ref.clone();
            entry.touch();
            // Update the entry in the map (drop the reference first)
            drop(entry_ref);
            self.entries.insert(key.to_string(), entry.clone());
            Some(entry)
        } else {
            None
        }
    }

    /// Save index to disk
    pub fn save(&self) -> CacheResult<()> {
        let entries: Vec<(String, CacheEntry)> = self.entries
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        
        let content = serde_json::to_string_pretty(&entries)
            .map_err(|e| PeaError::io(
                format!("Failed to serialize index: {}", e),
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            ))?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.index_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PeaError::io("Failed to create index directory".to_string(), e))?;
        }
        
        fs::write(&self.index_path, content)
            .map_err(|e| PeaError::io("Failed to write index file".to_string(), e))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_entry_creation() {
        let hash = ContentHash::from_hex("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let entry = CacheEntry::new(hash.clone(), 1024);
        assert_eq!(entry.hash, hash);
        assert_eq!(entry.size, 1024);
    }

    #[test]
    fn test_cas_index_operations() {
        let temp_dir = tempdir().unwrap();
        let index_path = temp_dir.path().join("index.json");
        
        let index = CasIndex::load_or_create(&index_path).unwrap();
        let hash = ContentHash::from_hex("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let entry = CacheEntry::new(hash, 1024);
        
        index.insert("test_key".to_string(), entry.clone());
        let retrieved = index.get("test_key").unwrap();
        assert_eq!(retrieved.hash, entry.hash);
        assert_eq!(retrieved.size, entry.size);
    }

    #[test]
    fn test_index_persistence() {
        let temp_dir = tempdir().unwrap();
        let index_path = temp_dir.path().join("index.json");
        
        // Create and save index
        {
            let index = CasIndex::load_or_create(&index_path).unwrap();
            let hash = ContentHash::from_hex("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
            let entry = CacheEntry::new(hash, 1024);
            index.insert("test_key".to_string(), entry);
            index.save().unwrap();
        }
        
        // Load index and verify
        {
            let index = CasIndex::load_or_create(&index_path).unwrap();
            let retrieved = index.get("test_key").unwrap();
            assert_eq!(retrieved.size, 1024);
        }
    }
}
impl CasIndex {
    /// Remove an entry by key
    pub fn remove(&self, key: &str) -> Option<CacheEntry> {
        self.entries.remove(key).map(|(_, entry)| entry)
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}