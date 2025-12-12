//! Path utilities for safe file system operations.
//!
//! Provides path normalization and security checks to prevent directory traversal.

use crate::error::{PeaError, PeaResult};
use std::path::{Path, PathBuf};

/// Normalize a path by resolving . and .. components
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {
                // Skip current directory
            },
            std::path::Component::ParentDir => {
                // Pop last component if possible, but track if we go negative
                if components.is_empty() {
                    // This would escape the base directory
                    components.push(component);
                } else {
                    components.pop();
                }
            },
            other => {
                components.push(other);
            },
        }
    }

    components.iter().collect()
}

/// Check if a path is safe (no directory traversal)
pub fn is_safe_path(path: &Path) -> bool {
    // Check for absolute paths
    if path.is_absolute() {
        return false;
    }

    // Track depth to detect escaping
    let mut depth = 0i32;

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {
                // Current directory is safe
            },
            std::path::Component::ParentDir => {
                depth -= 1;
                // If we go negative, we're escaping the base directory
                if depth < 0 {
                    return false;
                }
            },
            std::path::Component::Normal(_) => {
                depth += 1;
            },
            _ => {
                // Other components (like RootDir) are not safe in relative paths
                return false;
            },
        }
    }

    true
}

/// Safely join paths, preventing directory traversal
pub fn safe_join(base: &Path, path: &Path) -> PeaResult<PathBuf> {
    if !is_safe_path(path) {
        return Err(PeaError::PermissionDenied {
            permission: "path traversal".to_string(),
            resource: path.display().to_string(),
        });
    }

    Ok(base.join(normalize_path(path)))
}

/// Get the file extension as a lowercase string
pub fn get_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        let path = Path::new("./src/../lib/./file.rs");
        let normalized = normalize_path(path);
        assert_eq!(normalized, Path::new("lib/file.rs"));
    }

    #[test]
    fn test_is_safe_path() {
        assert!(is_safe_path(Path::new("src/lib.rs")));
        assert!(is_safe_path(Path::new("./src/lib.rs")));
        assert!(!is_safe_path(Path::new("../../../etc/passwd")));
        assert!(!is_safe_path(Path::new("/absolute/path")));
    }

    #[test]
    fn test_safe_join() {
        let base = Path::new("/home/user");

        // Safe path
        let result = safe_join(base, Path::new("project/src/main.rs")).unwrap();
        assert_eq!(result, Path::new("/home/user/project/src/main.rs"));

        // Unsafe path
        let result = safe_join(base, Path::new("../../../etc/passwd"));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension(Path::new("file.rs")), Some("rs".to_string()));
        assert_eq!(
            get_extension(Path::new("file.tar.gz")),
            Some("gz".to_string())
        );
        assert_eq!(get_extension(Path::new("FILE.JS")), Some("js".to_string()));
        assert_eq!(get_extension(Path::new("no_extension")), None);
    }
}
