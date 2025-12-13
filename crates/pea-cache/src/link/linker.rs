//! Linker implementation for hardlink operations

use pea_core::error::PeaError;
use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

use super::super::cas::CasStore;
use crate::CacheResult;

/// Linker for creating hardlinks with fallback to copying
#[derive(Debug)]
pub struct Linker {
    cas_store: std::sync::Arc<CasStore>,
}

impl Linker {
    pub fn new(cas_store: std::sync::Arc<CasStore>) -> Self {
        Self { cas_store }
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
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LinkResult {
    pub hardlinks_created: usize,
    pub files_copied: usize,
    pub directories_created: usize,
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
}