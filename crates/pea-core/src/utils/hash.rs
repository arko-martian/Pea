//! Blake3 hashing utilities for content integrity.
//!
//! Provides fast hashing functions for package integrity verification.

use crate::error::{PeaError, PeaResult};

/// Compute Blake3 hash of data
pub fn blake3_hash(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hash.to_hex().to_string()
}

/// Compute Blake3 hash of a file
pub fn blake3_hash_file(path: &std::path::Path) -> PeaResult<String> {
    let data = std::fs::read(path)
        .map_err(|e| PeaError::io(format!("Failed to read file: {}", path.display()), e))?;
    Ok(blake3_hash(&data))
}

/// Verify data integrity against expected hash
pub fn verify_integrity(data: &[u8], expected_hash: &str) -> PeaResult<()> {
    let actual_hash = blake3_hash(data);
    if actual_hash == expected_hash {
        Ok(())
    } else {
        Err(PeaError::IntegrityFailure {
            package: "unknown".to_string(),
            expected: expected_hash.to_string(),
            actual: actual_hash,
        })
    }
}

/// Verify file integrity against expected hash
pub fn verify_file_integrity(path: &std::path::Path, expected_hash: &str) -> PeaResult<()> {
    let actual_hash = blake3_hash_file(path)?;
    if actual_hash == expected_hash {
        Ok(())
    } else {
        Err(PeaError::IntegrityFailure {
            package: path.display().to_string(),
            expected: expected_hash.to_string(),
            actual: actual_hash,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_blake3_hash() {
        let data = b"hello world";
        let hash = blake3_hash(data);

        // Blake3 hash of "hello world" should be consistent
        assert_eq!(hash.len(), 64); // 32 bytes = 64 hex chars
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_verify_integrity_success() {
        let data = b"test data";
        let hash = blake3_hash(data);

        assert!(verify_integrity(data, &hash).is_ok());
    }

    #[test]
    fn test_verify_integrity_failure() {
        let data = b"test data";
        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";

        assert!(verify_integrity(data, wrong_hash).is_err());
    }

    #[test]
    fn test_blake3_hash_file() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_hash_file.txt");

        // Create test file
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"test file content").unwrap();

        // Hash the file
        let hash = blake3_hash_file(&file_path).unwrap();
        assert_eq!(hash.len(), 64);

        // Verify integrity
        assert!(verify_file_integrity(&file_path, &hash).is_ok());

        // Clean up
        std::fs::remove_file(&file_path).unwrap();
    }
}
