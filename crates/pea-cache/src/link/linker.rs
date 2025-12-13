//! Linker implementation for hardlink operations and node_modules creation
//!
//! This module provides functionality for creating hardlinks with fallback
//! to copying, and managing node_modules directory structure.

use camino::{Utf8Path, Utf8PathBuf};
use pea_core::error::PeaError;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

use super::super::cas::CasStore;
use crate::CacheResult;

/// Linker for creating hardlinks with fallback to copying
#[derive(Debug)]
pub struct Linker {
    /// Reference to the CAS store
    cas_store: std::sync::Arc<CasStore>,
}

impl Linker {
    /// Create a new linker
    pub fn new(cas_store: std::sync::Arc<CasStore>) -> Self {
        Self { cas_store }
    }

    /// Get reference to the CAS store
    pub fn cas_store(&self) -> &CasStore {
        &self.cas_store
    }

    pub fn hardlink_recursive(&self, source_dir: &Path, dest_dir: &Path) -> CacheResult<LinkResult> {
        let mut result = LinkResult::default();
        
        fs::create_dir_all(dest_dir)
            .map_err(|e| PeaError::io("Failed to create destination directory".to_string(), e))?;
        
        for entry in WalkDir::new(source_dir) {
            let entry = entry.map_err(|e| PeaError::io("Failed to walk directory".to_string(), 
                io::Error::new(io::ErrorKind::Other, e.to_string())))?;
            let source_path = entry.path();
            
            let relative_path = source_path.strip_prefix(source_dir)
                .map_err(|e| PeaError::io(format!("Failed to strip prefix: {}", e),
                    io::Error::new(io::ErrorKind::Other, e.to_string())))?;
            
            if relative_path.as_os_str().is_empty() {
                continue;
            }
            
            let dest_path = dest_dir.join(relative_path);
            
            if entry.file_type().is_dir() {
                fs::create_dir_all(&dest_path)
                    .map_err(|e| PeaError::io("Failed to create directory".to_string(), e))?;
                result.directories_created += 1;
            } else if entry.file_type().is_file() {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| PeaError::io("Failed to create parent directory".to_string(), e))?;
                }
                
                match fs::hard_link(source_path, &dest_path) {
                    Ok(_) => result.hardlinks_created += 1,
                    Err(_) => {
                        fs::copy(source_path, &dest_path)
                            .map_err(|e| PeaError::io("Failed to copy file".to_string(), e))?;
                        result.files_copied += 1;
                    }
                }
            }
        }
        
        Ok(result)
    }

    pub fn copy_recursive(&self, source_dir: &Path, dest_dir: &Path) -> CacheResult<LinkResult> {
        let mut result = LinkResult::default();
        
        fs::create_dir_all(dest_dir)
            .map_err(|e| PeaError::io("Failed to create destination directory".to_string(), e))?;
        
        for entry in WalkDir::new(source_dir) {
            let entry = entry.map_err(|e| PeaError::io("Failed to walk directory".to_string(),
                io::Error::new(io::ErrorKind::Other, e.to_string())))?;
            let source_path = entry.path();
            
            let relative_path = source_path.strip_prefix(source_dir)
                .map_err(|e| PeaError::io(format!("Failed to strip prefix: {}", e),
                    io::Error::new(io::ErrorKind::Other, e.to_string())))?;
            
            if relative_path.as_os_str().is_empty() {
                continue;
            }
            
            let dest_path = dest_dir.join(relative_path);
            
            if entry.file_type().is_dir() {
                fs::create_dir_all(&dest_path)
                    .map_err(|e| PeaError::io("Failed to create directory".to_string(), e))?;
                result.directories_created += 1;
            } else if entry.file_type().is_file() {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| PeaError::io("Failed to create parent directory".to_string(), e))?;
                }
                
                fs::copy(source_path, &dest_path)
                    .map_err(|e| PeaError::io("Failed to copy file".to_string(), e))?;
                result.files_copied += 1;
            }
        }
        
        Ok(result)
    }

    /// Create node_modules structure with hardlinks
    pub fn create_node_modules(
        &self,
        packages: &[PackageInfo],
        node_modules_dir: &Utf8Path,
    ) -> CacheResult<NodeModulesResult> {
        let mut result = NodeModulesResult::default();
        
        // Create node_modules directory
        fs::create_dir_all(node_modules_dir)
            .map_err(|e| PeaError::io("Failed to create node_modules directory".to_string(), e))?;
        
        // Create .bin directory
        let bin_dir = node_modules_dir.join(".bin");
        fs::create_dir_all(&bin_dir)
            .map_err(|e| PeaError::io("Failed to create .bin directory".to_string(), e))?;
        
        // Process each package
        for package in packages {
            let package_dir = if package.name.starts_with('@') {
                // Scoped package: @org/name -> @org/name
                node_modules_dir.join(&package.name)
            } else {
                // Regular package: name -> name
                node_modules_dir.join(&package.name)
            };
            
            // Create scoped directory if needed
            if package.name.starts_with('@') {
                if let Some(scope_dir) = package_dir.parent() {
                    fs::create_dir_all(scope_dir)
                        .map_err(|e| PeaError::io("Failed to create scope directory".to_string(), e))?;
                }
            }
            
            // Link package content
            let link_result = self.hardlink_recursive(
                package.source_path.as_std_path(),
                package_dir.as_std_path(),
            )?;
            
            result.packages_linked += 1;
            result.hardlinks_created += link_result.hardlinks_created;
            result.files_copied += link_result.files_copied;
            
            // Create .bin symlinks
            for (bin_name, bin_path) in &package.bin_entries {
                let bin_target = package_dir.join(bin_path);
                let bin_link = bin_dir.join(bin_name);
                
                if bin_target.exists() {
                    self.create_bin_symlink(&bin_target, &bin_link)?;
                    result.bin_links_created += 1;
                }
            }
        }
        
        Ok(result)
    }

    /// Create a .bin symlink
    fn create_bin_symlink(&self, target: &Utf8Path, link: &Utf8Path) -> CacheResult<()> {
        // Remove existing symlink if it exists
        if link.exists() || link.is_symlink() {
            fs::remove_file(link)
                .map_err(|e| PeaError::io("Failed to remove existing bin link".to_string(), e))?;
        }
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(target.as_std_path(), link.as_std_path())
                .map_err(|e| PeaError::io("Failed to create bin symlink".to_string(), e))?;
            
            // Make symlink executable
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(target) {
                let mut perms = metadata.permissions();
                perms.set_mode(perms.mode() | 0o111); // Add execute permission
                let _ = fs::set_permissions(target, perms); // Ignore errors
            }
        }
        
        #[cfg(windows)]
        {
            // On Windows, create a .cmd wrapper script
            let cmd_content = format!(
                "@echo off\nnode \"{}\" %*\n",
                target.as_str().replace('/', "\\")
            );
            let cmd_path = link.with_extension("cmd");
            fs::write(&cmd_path, cmd_content)
                .map_err(|e| PeaError::io("Failed to create bin wrapper".to_string(), e))?;
        }
        
        Ok(())
    }

    /// Remove node_modules directory while preserving CAS entries
    pub fn cleanup_node_modules(&self, node_modules_dir: &Utf8Path) -> CacheResult<CleanupResult> {
        let mut result = CleanupResult::default();
        
        if !node_modules_dir.exists() {
            return Ok(result);
        }
        
        // Count entries before removal
        for entry in WalkDir::new(node_modules_dir) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    result.files_removed += 1;
                } else if entry.file_type().is_dir() {
                    result.directories_removed += 1;
                }
            }
        }
        
        // Remove the entire node_modules directory
        fs::remove_dir_all(node_modules_dir)
            .map_err(|e| PeaError::io("Failed to remove node_modules directory".to_string(), e))?;
        
        Ok(result)
    }
}

/// Result of node_modules creation
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeModulesResult {
    /// Number of packages linked
    pub packages_linked: usize,
    /// Number of hardlinks created
    pub hardlinks_created: usize,
    /// Number of files copied (fallback)
    pub files_copied: usize,
    /// Number of .bin symlinks created
    pub bin_links_created: usize,
}

/// Result of cleanup operation
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CleanupResult {
    /// Number of files removed
    pub files_removed: usize,
    /// Number of directories removed
    pub directories_removed: usize,
}

/// Result of linking operations
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LinkResult {
    /// Number of hardlinks created
    pub hardlinks_created: usize,
    /// Number of files copied (fallback)
    pub files_copied: usize,
    /// Number of directories created
    pub directories_created: usize,
}

/// Package information for node_modules creation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInfo {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Path to extracted package content
    pub source_path: Utf8PathBuf,
    /// Binary entries from package.json
    pub bin_entries: HashMap<String, String>,
    /// Whether this is a workspace package
    pub is_workspace: bool,
}

impl PackageInfo {
    /// Create new package info
    pub fn new(name: String, version: String, source_path: Utf8PathBuf) -> Self {
        Self {
            name,
            version,
            source_path,
            bin_entries: HashMap::new(),
            is_workspace: false,
        }
    }

    /// Add binary entry
    pub fn with_bin(mut self, name: String, path: String) -> Self {
        self.bin_entries.insert(name, path);
        self
    }

    /// Mark as workspace package
    pub fn as_workspace(mut self) -> Self {
        self.is_workspace = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[test]
    fn test_copy_recursive() {
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        let cas_store = Arc::new(CasStore::new(&store_path).unwrap());
        let linker = Linker::new(cas_store);
        
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");
        
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("file1.txt"), "content1").unwrap();
        
        let result = linker.copy_recursive(&source_dir, &dest_dir).unwrap();
        assert_eq!(result.files_copied, 1);
    }

    #[test]
    fn test_hardlink_creation() {
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        let cas_store = Arc::new(CasStore::new(&store_path).unwrap());
        let linker = Linker::new(cas_store);
        
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");
        
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("file1.txt"), "content1").unwrap();
        fs::write(source_dir.join("file2.txt"), "content2").unwrap();
        
        let result = linker.hardlink_recursive(&source_dir, &dest_dir).unwrap();
        
        // Should create hardlinks or fall back to copy
        assert!(result.hardlinks_created > 0 || result.files_copied > 0);
        assert_eq!(result.hardlinks_created + result.files_copied, 2);
        
        // Verify files exist and have correct content
        assert!(dest_dir.join("file1.txt").exists());
        assert!(dest_dir.join("file2.txt").exists());
        
        let content1 = fs::read_to_string(dest_dir.join("file1.txt")).unwrap();
        let content2 = fs::read_to_string(dest_dir.join("file2.txt")).unwrap();
        assert_eq!(content1, "content1");
        assert_eq!(content2, "content2");
    }

    #[test]
    fn test_node_modules_creation() {
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        let cas_store = Arc::new(CasStore::new(&store_path).unwrap());
        let linker = Linker::new(cas_store);
        
        // Create package source
        let pkg_source = temp_dir.path().join("pkg_source");
        fs::create_dir_all(&pkg_source).unwrap();
        fs::write(pkg_source.join("index.js"), "console.log('hello');").unwrap();
        fs::write(pkg_source.join("bin.js"), "#!/usr/bin/env node\nconsole.log('bin');").unwrap();
        
        let pkg_source_utf8 = Utf8PathBuf::from_path_buf(pkg_source).unwrap();
        let node_modules_dir = Utf8PathBuf::from_path_buf(temp_dir.path().join("node_modules")).unwrap();
        
        // Create package info
        let package = PackageInfo::new("test-pkg".to_string(), "1.0.0".to_string(), pkg_source_utf8)
            .with_bin("test-bin".to_string(), "bin.js".to_string());
        
        let result = linker.create_node_modules(&[package], &node_modules_dir).unwrap();
        
        assert_eq!(result.packages_linked, 1);
        assert_eq!(result.bin_links_created, 1);
        
        // Verify structure
        assert!(node_modules_dir.join("test-pkg").exists());
        assert!(node_modules_dir.join("test-pkg").join("index.js").exists());
        assert!(node_modules_dir.join(".bin").exists());
        
        #[cfg(unix)]
        {
            assert!(node_modules_dir.join(".bin").join("test-bin").exists());
        }
        
        #[cfg(windows)]
        {
            assert!(node_modules_dir.join(".bin").join("test-bin.cmd").exists());
        }
    }

    #[test]
    fn test_scoped_package() {
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        let cas_store = Arc::new(CasStore::new(&store_path).unwrap());
        let linker = Linker::new(cas_store);
        
        // Create package source
        let pkg_source = temp_dir.path().join("pkg_source");
        fs::create_dir_all(&pkg_source).unwrap();
        fs::write(pkg_source.join("index.js"), "console.log('scoped');").unwrap();
        
        let pkg_source_utf8 = Utf8PathBuf::from_path_buf(pkg_source).unwrap();
        let node_modules_dir = Utf8PathBuf::from_path_buf(temp_dir.path().join("node_modules")).unwrap();
        
        // Create scoped package info
        let package = PackageInfo::new("@org/scoped-pkg".to_string(), "1.0.0".to_string(), pkg_source_utf8);
        
        let result = linker.create_node_modules(&[package], &node_modules_dir).unwrap();
        
        assert_eq!(result.packages_linked, 1);
        
        // Verify scoped structure
        assert!(node_modules_dir.join("@org").exists());
        assert!(node_modules_dir.join("@org").join("scoped-pkg").exists());
        assert!(node_modules_dir.join("@org").join("scoped-pkg").join("index.js").exists());
    }

    #[test]
    fn test_cleanup_node_modules() {
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        let cas_store = Arc::new(CasStore::new(&store_path).unwrap());
        let linker = Linker::new(cas_store);
        
        let node_modules_dir = Utf8PathBuf::from_path_buf(temp_dir.path().join("node_modules")).unwrap();
        
        // Create some content
        fs::create_dir_all(&node_modules_dir).unwrap();
        fs::write(node_modules_dir.join("file1.txt"), "content").unwrap();
        fs::create_dir_all(node_modules_dir.join("subdir")).unwrap();
        fs::write(node_modules_dir.join("subdir").join("file2.txt"), "content").unwrap();
        
        assert!(node_modules_dir.exists());
        
        let result = linker.cleanup_node_modules(&node_modules_dir).unwrap();
        
        assert!(result.files_removed > 0);
        assert!(result.directories_removed > 0);
        assert!(!node_modules_dir.exists());
    }

    #[test]
    fn test_cleanup_nonexistent_directory() {
        let temp_dir = tempdir().unwrap();
        let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        let cas_store = Arc::new(CasStore::new(&store_path).unwrap());
        let linker = Linker::new(cas_store);
        
        let node_modules_dir = Utf8PathBuf::from_path_buf(temp_dir.path().join("nonexistent")).unwrap();
        
        let result = linker.cleanup_node_modules(&node_modules_dir).unwrap();
        
        assert_eq!(result.files_removed, 0);
        assert_eq!(result.directories_removed, 0);
    }
}
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::Config as ProptestConfig;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tempfile::tempdir;

    // Strategy for generating file structures
    fn file_structure_strategy() -> impl Strategy<Value = HashMap<String, String>> {
        prop::collection::hash_map(
            // File paths (no directory traversal)
            "[a-zA-Z0-9_-]+(/[a-zA-Z0-9_-]+){0,2}\\.[a-z]{1,4}",
            // File contents
            prop::collection::vec(any::<u8>(), 0..100)
                .prop_map(|bytes| String::from_utf8_lossy(&bytes).to_string()),
            1..5
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(5))]
        /// **Property 17: Hardlink Equivalence**
        /// **Validates: Requirements 11.1**
        #[test]
        fn hardlink_equivalence_property(files in file_structure_strategy()) {
            let temp_dir = tempdir().unwrap();
            let store_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
            let cas_store = Arc::new(CasStore::new(&store_path).unwrap());
            let linker = Linker::new(cas_store);
            
            let source_dir = temp_dir.path().join("source");
            let dest_dir = temp_dir.path().join("dest");
            
            // Create source files
            fs::create_dir_all(&source_dir).unwrap();
            for (file_path, content) in &files {
                let full_path = source_dir.join(file_path);
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                fs::write(&full_path, content).unwrap();
            }
            
            // Create hardlinks (or copies as fallback)
            let _result = linker.hardlink_recursive(&source_dir, &dest_dir).unwrap();
            
            // Verify content matches
            for (file_path, expected_content) in &files {
                let source_path = source_dir.join(file_path);
                let dest_path = dest_dir.join(file_path);
                
                prop_assert!(dest_path.exists(), "Destination file {} should exist", file_path);
                
                let source_content = fs::read_to_string(&source_path).unwrap();
                let dest_content = fs::read_to_string(&dest_path).unwrap();
                
                prop_assert_eq!(&source_content, expected_content, "Source content mismatch for {}", file_path);
                prop_assert_eq!(&dest_content, expected_content, "Destination content mismatch for {}", file_path);
                prop_assert_eq!(source_content, dest_content, "Source and destination content should match for {}", file_path);
            }
        }
    }
}