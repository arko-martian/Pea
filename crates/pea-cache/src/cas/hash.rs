//! Content hashing utilities using Blake3
//!
//! This module provides the ContentHash type and related utilities
//! for content-addressable storage.

use blake3::Hasher;
use pea_core::error::PeaError;
use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use std::fmt;
use std::hash::{Hash, Hasher as StdHasher};

/// A Blake3 content hash for content-addressable storage
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord,
    SerdeSerialize, SerdeDeserialize
)]
pub struct ContentHash {
    /// The raw hash bytes (32 bytes for Blake3)
    bytes: [u8; 32],
}

impl ContentHash {
    /// Create a new ContentHash from raw bytes
    pub fn new(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Create a new ContentHash from a Vec<u8>
    pub fn from_vec(bytes: Vec<u8>) -> Result<Self, PeaError> {
        if bytes.len() != 32 {
            return Err(PeaError::IntegrityFailure {
                package: "hash".to_string(),
                expected: "32 bytes".to_string(),
                actual: format!("{} bytes", bytes.len()),
            });
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Self { bytes: array })
    }

    /// Convert hash to hexadecimal string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    /// Create ContentHash from hexadecimal string
    pub fn from_hex(hex_str: &str) -> Result<Self, PeaError> {
        let bytes = hex::decode(hex_str)
            .map_err(|e| PeaError::IntegrityFailure {
                package: "hash".to_string(),
                expected: "valid hex string".to_string(),
                actual: format!("invalid hex: {}", e),
            })?;
        Self::from_vec(bytes)
    }
}
impl Hash for ContentHash {
    fn hash<H: StdHasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Compute Blake3 hash of content
pub fn compute_hash(content: &[u8]) -> ContentHash {
    let mut hasher = Hasher::new();
    hasher.update(content);
    let hash = hasher.finalize();
    ContentHash::new(*hash.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_creation() {
        let bytes = [0u8; 32];
        let hash = ContentHash::new(bytes);
        assert_eq!(hash.bytes, bytes);
    }

    #[test]
    fn test_content_hash_invalid_length() {
        let bytes = vec![0u8; 16]; // Wrong length
        assert!(ContentHash::from_vec(bytes).is_err());
    }

    #[test]
    fn test_hex_conversion() {
        let bytes = [0u8; 32];
        let hash = ContentHash::new(bytes);
        let hex = hash.to_hex();
        let restored = ContentHash::from_hex(&hex).unwrap();
        assert_eq!(hash, restored);
    }

    #[test]
    fn test_compute_hash() {
        let content = b"hello world";
        let hash1 = compute_hash(content);
        let hash2 = compute_hash(content);
        assert_eq!(hash1, hash2); // Deterministic
        
        let different_content = b"hello world!";
        let hash3 = compute_hash(different_content);
        assert_ne!(hash1, hash3); // Different content = different hash
    }
}
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::Config as ProptestConfig;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        /// **Property 8: CAS Hash Determinism**
        /// **Validates: Requirements 7.1, 10.2**
        #[test]
        fn hash_determinism_property(content in prop::collection::vec(any::<u8>(), 0..1000)) {
            // Hash multiple times
            let hash1 = compute_hash(&content);
            let hash2 = compute_hash(&content);
            let hash3 = compute_hash(&content);
            
            // Verify all hashes are identical
            prop_assert_eq!(hash1, hash2);
            prop_assert_eq!(hash2, hash3);
            prop_assert_eq!(hash1, hash3);
            
            // Verify hex conversion is consistent
            let hex1 = hash1.to_hex();
            let hex2 = hash2.to_hex();
            prop_assert_eq!(hex1.clone(), hex2);
            
            // Verify round-trip through hex
            let restored = ContentHash::from_hex(&hex1).unwrap();
            prop_assert_eq!(hash1, restored);
        }
    }
}
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Hash a single file
pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<ContentHash, PeaError> {
    let content = fs::read(path.as_ref())
        .map_err(|e| PeaError::io("Failed to read file for hashing".to_string(), e))?;
    Ok(compute_hash(&content))
}

/// Hash multiple files in parallel
pub fn hash_files_parallel(paths: &[PathBuf]) -> Result<Vec<(String, ContentHash)>, PeaError> {
    paths
        .par_iter()
        .map(|path| {
            let path_str = path.to_string_lossy().to_string();
            let hash = hash_file(path)?;
            Ok((path_str, hash))
        })
        .collect()
}

/// Hash all files in a directory recursively with deterministic ordering
pub fn hash_directory<P: AsRef<Path>>(dir_path: P) -> Result<Vec<(String, ContentHash)>, PeaError> {
    // Collect all files first, then sort for deterministic hashing
    let mut file_paths: Vec<PathBuf> = WalkDir::new(dir_path.as_ref())
        .into_iter()
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                if e.file_type().is_file() {
                    Some(e.path().to_path_buf())
                } else {
                    None
                }
            })
        })
        .collect();
    
    // Sort entries for deterministic hashing
    file_paths.sort();
    
    // Hash in parallel
    hash_files_parallel(&file_paths)
}
    #[test]
    fn test_hash_file() {
        use tempfile::NamedTempFile;
        use std::io::Write;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        
        let hash = hash_file(temp_file.path()).unwrap();
        let expected = compute_hash(b"test content");
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_hash_files_parallel() {
        use tempfile::tempdir;
        use std::io::Write;
        
        let temp_dir = tempdir().unwrap();
        
        // Create test files
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.txt");
        
        fs::write(&file1_path, b"content1").unwrap();
        fs::write(&file2_path, b"content2").unwrap();
        
        let paths = vec![file1_path, file2_path];
        let results = hash_files_parallel(&paths).unwrap();
        
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|(path, _)| path.contains("file1.txt")));
        assert!(results.iter().any(|(path, _)| path.contains("file2.txt")));
    }

    #[test]
    fn test_hash_directory() {
        use tempfile::tempdir;
        
        let temp_dir = tempdir().unwrap();
        
        // Create test files
        fs::write(temp_dir.path().join("file1.txt"), b"content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), b"content2").unwrap();
        
        // Create subdirectory with file
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(sub_dir.join("file3.txt"), b"content3").unwrap();
        
        let results = hash_directory(temp_dir.path()).unwrap();
        assert_eq!(results.len(), 3);
        
        // Verify deterministic ordering by running twice
        let results2 = hash_directory(temp_dir.path()).unwrap();
        assert_eq!(results, results2);
    }