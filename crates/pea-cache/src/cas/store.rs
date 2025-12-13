//! CAS store implementation
//!
//! This module provides the main CasStore interface for
//! content-addressable storage operations.

use camino::{Utf8Path, Utf8PathBuf};
use pea_core::error::PeaError;
use std::fs;
use std::sync::Arc;

use super::{ContentHash, CasIndex, CacheEntry};
use super::hash::compute_hash;
use crate::CacheResult;

/// Content-addressable storage
#[derive(Debug)]
pub struct CasStore {
    /// Root directory for storage (~/.pea/store)
    root_path: Utf8PathBuf,
    /// Index for metadata
    index: Arc<CasIndex>,
}

impl CasStore {
    /// Create a new CAS store
    pub fn new<P: AsRef<Utf8Path>>(root_path: P) -> CacheResult<Self> {
        let root_path = root_path.as_ref().to_path_buf();
        
        // Create root directory if it doesn't exist
        fs::create_dir_all(&root_path)
            .map_err(|e| PeaError::io("Failed to create store directory".to_string(), e))?;
        
        // Load or create index
        let index_path = root_path.join("index.json");
        let index = Arc::new(CasIndex::load_or_create(index_path)?);
        
        Ok(Self {
            root_path,
            index,
        })
    }

    /// Get the storage path for a hash
    fn hash_to_path(&self, hash: &ContentHash) -> Utf8PathBuf {
        let hex = hash.to_hex();
        // Store as store/ab/cd/abcd...
        let prefix1 = &hex[0..2];
        let prefix2 = &hex[2..4];
        self.root_path.join(prefix1).join(prefix2).join(&hex)
    }

    /// Check if content exists in store
    pub fn contains(&self, hash: &ContentHash) -> bool {
        let path = self.hash_to_path(hash);
        path.exists()
    }

    /// Get the root path of the store
    pub fn root_path(&self) -> &Utf8Path {
        &self.root_path
    }
}
impl CasStore {
    /// Store content and return its hash
    pub fn store(&self, content: &[u8]) -> CacheResult<ContentHash> {
        // Compute hash
        let hash = compute_hash(content);
        let path = self.hash_to_path(&hash);
        
        // Check if already exists (skip if so)
        if path.exists() {
            // Update index with access time
            let key = hash.to_hex();
            if let Some(mut entry) = self.index.get(&key) {
                entry.touch();
                self.index.insert(key, entry);
            }
            return Ok(hash);
        }
        
        // Create parent directories
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PeaError::io("Failed to create content directory".to_string(), e))?;
        }
        
        // Write content to path
        fs::write(&path, content)
            .map_err(|e| PeaError::io("Failed to write content file".to_string(), e))?;
        
        // Update index
        let entry = CacheEntry::new(hash.clone(), content.len() as u64);
        self.index.insert(hash.to_hex(), entry);
        
        Ok(hash)
    }

    /// Get content by hash
    pub fn get(&self, hash: &ContentHash) -> CacheResult<Vec<u8>> {
        let path = self.hash_to_path(hash);
        
        // Check if path exists
        if !path.exists() {
            return Err(PeaError::IntegrityFailure {
                package: "unknown".to_string(),
                expected: hash.to_hex(),
                actual: "not found".to_string(),
            });
        }
        
        // Read content
        let content = fs::read(&path)
            .map_err(|e| PeaError::io("Failed to read content file".to_string(), e))?;
        
        // Update access time in index
        let key = hash.to_hex();
        if let Some(mut entry) = self.index.get(&key) {
            entry.touch();
            self.index.insert(key, entry);
        }
        
        Ok(content)
    }

    /// Verify content integrity
    pub fn verify(&self, hash: &ContentHash) -> CacheResult<bool> {
        match self.get(hash) {
            Ok(content) => {
                let computed_hash = compute_hash(&content);
                Ok(computed_hash == *hash)
            }
            Err(_) => Ok(false),
        }
    }

    /// Save index to disk
    pub fn save_index(&self) -> CacheResult<()> {
        self.index.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cas_store_creation() {
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        let store = CasStore::new(&store_path).unwrap();
        assert_eq!(store.root_path(), &store_path);
    }

    #[test]
    fn test_store_and_retrieve() {
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        let store = CasStore::new(&store_path).unwrap();
        
        let content = b"hello world";
        let hash = store.store(content).unwrap();
        let retrieved = store.get(&hash).unwrap();
        assert_eq!(content, retrieved.as_slice());
    }

    #[test]
    fn test_duplicate_store() {
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        let store = CasStore::new(&store_path).unwrap();
        
        let content = b"hello world";
        let hash1 = store.store(content).unwrap();
        let hash2 = store.store(content).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_verify_integrity() {
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        let store = CasStore::new(&store_path).unwrap();
        
        let content = b"hello world";
        let hash = store.store(content).unwrap();
        assert!(store.verify(&hash).unwrap());
    }

    #[test]
    fn test_contains() {
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        let store = CasStore::new(&store_path).unwrap();
        
        let content = b"hello world";
        let hash = store.store(content).unwrap();
        assert!(store.contains(&hash));
        
        let fake_hash = ContentHash::from_hex("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        assert!(!store.contains(&fake_hash));
    }
}
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::Config as ProptestConfig;
    use tempfile::tempdir;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        /// **Property 7: Content-Addressable Storage Integrity**
        /// **Validates: Requirements 7.1, 7.3, 10.4**
        #[test]
        fn cas_integrity_property(content in prop::collection::vec(any::<u8>(), 0..1000)) {
            let temp_dir = tempdir().unwrap();
            let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
            let store = CasStore::new(&store_path).unwrap();
            
            // Store content and verify hash is deterministic
            let hash1 = store.store(&content).unwrap();
            let hash2 = store.store(&content).unwrap();
            prop_assert_eq!(hash1, hash2);
            
            // Retrieve content and verify it matches original
            let retrieved = store.get(&hash1).unwrap();
            prop_assert_eq!(content, retrieved);
            
            // Verify integrity check passes
            prop_assert!(store.verify(&hash1).unwrap());
        }
    }
}
use chrono::Utc;

impl CasStore {
    /// Find unreferenced entries older than threshold
    pub fn find_unreferenced_entries(&self, max_age_days: i64) -> CacheResult<Vec<String>> {
        let threshold = Utc::now().timestamp() - (max_age_days * 24 * 60 * 60);
        let mut unreferenced = Vec::new();
        
        // We need to add a method to CasIndex to iterate entries
        // For now, return empty list as this is a placeholder
        // TODO: Add proper iteration method to CasIndex
        
        Ok(unreferenced)
    }

    /// Remove entries and return freed space
    pub fn remove_entries(&self, keys: &[String]) -> CacheResult<u64> {
        let mut freed_space = 0u64;
        
        for key in keys {
            if let Some(entry) = self.index.get(key) {
                let hash = &entry.hash;
                let path = self.hash_to_path(hash);
                
                if path.exists() {
                    // Get file size before removal
                    if let Ok(metadata) = fs::metadata(&path) {
                        freed_space += metadata.len();
                    }
                    
                    // Remove the file
                    if let Err(e) = fs::remove_file(&path) {
                        eprintln!("Warning: Failed to remove {}: {}", path, e);
                    }
                    
                    // Try to remove empty parent directories
                    if let Some(parent) = path.parent() {
                        let _ = fs::remove_dir(parent); // Ignore errors (dir might not be empty)
                        if let Some(grandparent) = parent.parent() {
                            let _ = fs::remove_dir(grandparent);
                        }
                    }
                }
                
                // Remove from index (we need to add a remove method)
                // TODO: Add remove method to CasIndex
            }
        }
        
        Ok(freed_space)
    }

    /// Perform garbage collection
    pub fn garbage_collect(&self, max_age_days: i64) -> CacheResult<GcResult> {
        let unreferenced = self.find_unreferenced_entries(max_age_days)?;
        let entries_removed = unreferenced.len();
        let freed_space = self.remove_entries(&unreferenced)?;
        
        // Save updated index
        self.save_index()?;
        
        Ok(GcResult {
            entries_removed,
            freed_space,
        })
    }
}

/// Result of garbage collection operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcResult {
    /// Number of entries removed
    pub entries_removed: usize,
    /// Bytes freed
    pub freed_space: u64,
}

impl GcResult {
    /// Format freed space in human-readable format
    pub fn format_freed_space(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = self.freed_space as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", self.freed_space, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}
    #[test]
    fn test_garbage_collection() {
        use tempfile::tempdir;
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        let store = CasStore::new(&store_path).unwrap();
        
        // Store some content
        let content1 = b"content1";
        let content2 = b"content2";
        let hash1 = store.store(content1).unwrap();
        let hash2 = store.store(content2).unwrap();
        
        // Verify both exist
        assert!(store.contains(&hash1));
        assert!(store.contains(&hash2));
        
        // Run garbage collection (should not remove anything since entries are new)
        let result = store.garbage_collect(0).unwrap();
        assert_eq!(result.entries_removed, 0);
        assert_eq!(result.freed_space, 0);
        
        // Both should still exist
        assert!(store.contains(&hash1));
        assert!(store.contains(&hash2));
    }

    #[test]
    fn test_gc_result_formatting() {
        let result = GcResult {
            entries_removed: 5,
            freed_space: 1536, // 1.5 KB
        };
        
        let formatted = result.format_freed_space();
        assert_eq!(formatted, "1.5 KB");
        
        let large_result = GcResult {
            entries_removed: 100,
            freed_space: 1024 * 1024 * 1024 + 512 * 1024 * 1024, // 1.5 GB
        };
        
        let formatted_large = large_result.format_freed_space();
        assert_eq!(formatted_large, "1.5 GB");
    }