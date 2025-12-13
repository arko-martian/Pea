//! Tarball creation functionality
//!
//! This module provides functionality for creating npm-compatible
//! tarballs with the package/ prefix.

use flate2::write::GzEncoder;
use flate2::Compression;
use pea_core::error::PeaError;
use std::fs;
use std::io::Write;
use std::path::Path;
use tar::Builder;
use walkdir::WalkDir;

use crate::CacheResult;

/// Create an npm-compatible tarball from a directory
pub fn create_tarball<W: Write>(
    writer: W,
    source_dir: &Path,
) -> CacheResult<()> {
    // Create gzip encoder
    let gz_encoder = GzEncoder::new(writer, Compression::default());
    let mut tar_builder = Builder::new(gz_encoder);
    
    // Walk the source directory
    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let relative_path = path.strip_prefix(source_dir)
            .map_err(|e| PeaError::io(
                format!("Failed to strip prefix: {}", e),
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            ))?;
        
        // Skip the root directory itself
        if relative_path.as_os_str().is_empty() {
            continue;
        }
        
        // Create npm-compatible path with package/ prefix
        let npm_path = Path::new("package").join(relative_path);
        
        if entry.file_type().is_file() {
            // Add regular file
            tar_builder.append_path_with_name(path, &npm_path)
                .map_err(|e| PeaError::io("IO operation failed".to_string(), e))?;
        } else if entry.file_type().is_dir() {
            // Add directory
            tar_builder.append_dir(&npm_path, path)
                .map_err(|e| PeaError::io("IO operation failed".to_string(), e))?;
        }
        // Skip symlinks and other special files for npm compatibility
    }
    
    // Finish the tarball
    tar_builder.finish()
        .map_err(|e| PeaError::io("IO operation failed".to_string(), e))?;
    
    Ok(())
}

/// Create tarball and return as bytes
pub fn create_tarball_bytes(source_dir: &Path) -> CacheResult<Vec<u8>> {
    let mut buffer = Vec::new();
    create_tarball(&mut buffer, source_dir)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tarball::extract::extract_tarball;
    use std::io::Cursor;
    use tempfile::tempdir;

    #[test]
    fn test_create_simple_tarball() {
        let temp_dir = tempdir().unwrap();
        let source_dir = temp_dir.path().join("source");
        let extract_dir = temp_dir.path().join("extract");
        
        // Create source files
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("file1.txt"), "content1").unwrap();
        fs::write(source_dir.join("file2.txt"), "content2").unwrap();
        
        let subdir = source_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("file3.txt"), "content3").unwrap();
        
        // Create tarball
        let tarball_bytes = create_tarball_bytes(&source_dir).unwrap();
        
        // Extract and verify
        let cursor = Cursor::new(tarball_bytes);
        extract_tarball(cursor, &extract_dir).unwrap();
        
        // Verify npm-compatible structure (package/ prefix)
        let package_dir = extract_dir.join("package");
        assert!(package_dir.exists());
        assert!(package_dir.join("file1.txt").exists());
        assert!(package_dir.join("file2.txt").exists());
        assert!(package_dir.join("subdir").join("file3.txt").exists());
        
        // Verify content
        let content1 = fs::read_to_string(package_dir.join("file1.txt")).unwrap();
        assert_eq!(content1, "content1");
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let source_dir = temp_dir.path().join("empty");
        fs::create_dir_all(&source_dir).unwrap();
        
        // Should create valid tarball even for empty directory
        let tarball_bytes = create_tarball_bytes(&source_dir).unwrap();
        assert!(!tarball_bytes.is_empty());
    }
}
#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::tarball::extract::extract_tarball;
    use proptest::prelude::*;
    use proptest::test_runner::Config as ProptestConfig;
    use std::collections::HashMap;
    use std::io::Cursor;
    use tempfile::tempdir;

    // Strategy for generating file structures
    fn file_structure_strategy() -> impl Strategy<Value = HashMap<String, String>> {
        prop::collection::hash_map(
            // File paths (no directory traversal)
            "[a-zA-Z0-9_-]+(/[a-zA-Z0-9_-]+){0,3}\\.[a-z]{1,4}",
            // File contents
            prop::collection::vec(any::<u8>(), 0..1000)
                .prop_map(|bytes| String::from_utf8_lossy(&bytes).to_string()),
            0..10
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]
        /// **Property 9: Tarball Round-Trip**
        /// **Validates: Requirements 50.7**
        #[test]
        fn tarball_round_trip_property(files in file_structure_strategy()) {
            let temp_dir = tempdir().unwrap();
            let source_dir = temp_dir.path().join("source");
            let extract_dir = temp_dir.path().join("extract");
            
            // Create source files
            fs::create_dir_all(&source_dir).unwrap();
            for (file_path, content) in &files {
                let full_path = source_dir.join(file_path);
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                fs::write(&full_path, content).unwrap();
            }
            
            // Create tarball
            let tarball_bytes = create_tarball_bytes(&source_dir).unwrap();
            
            // Extract tarball
            let cursor = Cursor::new(tarball_bytes);
            extract_tarball(cursor, &extract_dir).unwrap();
            
            // Verify contents match (accounting for package/ prefix)
            let package_dir = extract_dir.join("package");
            for (file_path, expected_content) in &files {
                let extracted_path = package_dir.join(file_path);
                prop_assert!(extracted_path.exists(), "File {} should exist after round-trip", file_path);
                
                let actual_content = fs::read_to_string(&extracted_path).unwrap();
                prop_assert_eq!(&actual_content, expected_content, "Content mismatch for {}", file_path);
            }
        }
    }
}