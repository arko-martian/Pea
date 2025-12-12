//! Package metadata types.
//!
//! Defines structures for package information from pea.toml and package.json.

use super::Version;
use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

/// Package metadata from pea.toml or package.json
#[derive(
    Debug, Clone, PartialEq, Eq, Archive, Deserialize, Serialize, SerdeDeserialize, SerdeSerialize,
)]
#[archive(check_bytes)]
pub struct PackageMetadata {
    pub name: String,
    pub version: Version,
    pub description: Option<String>,
    pub main: Option<String>,
    pub license: Option<String>,
    pub repository: Option<Repository>,
    pub keywords: Vec<String>,
}

/// Repository information
#[derive(
    Debug, Clone, PartialEq, Eq, Archive, Deserialize, Serialize, SerdeDeserialize, SerdeSerialize,
)]
#[archive(check_bytes)]
pub struct Repository {
    pub url: String,
    pub directory: Option<String>,
}

impl PackageMetadata {
    /// Create new package metadata with required fields
    pub fn new(name: String, version: Version) -> Self {
        Self {
            name,
            version,
            description: None,
            main: None,
            license: None,
            repository: None,
            keywords: Vec::new(),
        }
    }

    /// Check if this is a valid package name
    pub fn is_valid_name(name: &str) -> bool {
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            && !name.starts_with('-')
            && !name.ends_with('-')
    }

    /// Get the main entry point (defaults to "index.js")
    pub fn main_entry(&self) -> &str {
        self.main.as_deref().unwrap_or("index.js")
    }

    /// Check if this package has a specific keyword
    pub fn has_keyword(&self, keyword: &str) -> bool {
        self.keywords.iter().any(|k| k == keyword)
    }
}

impl Repository {
    /// Create a new repository reference
    pub fn new(url: String) -> Self {
        Self {
            url,
            directory: None,
        }
    }

    /// Create a repository reference with subdirectory
    pub fn with_directory(url: String, directory: String) -> Self {
        Self {
            url,
            directory: Some(directory),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_metadata_creation() {
        let version = Version::new(1, 0, 0);
        let pkg = PackageMetadata::new("test-package".to_string(), version.clone());

        assert_eq!(pkg.name, "test-package");
        assert_eq!(pkg.version, version);
        assert_eq!(pkg.description, None);
        assert_eq!(pkg.main_entry(), "index.js");
    }

    #[test]
    fn test_valid_package_names() {
        assert!(PackageMetadata::is_valid_name("my-package"));
        assert!(PackageMetadata::is_valid_name("my_package"));
        assert!(PackageMetadata::is_valid_name("package123"));

        assert!(!PackageMetadata::is_valid_name(""));
        assert!(!PackageMetadata::is_valid_name("-invalid"));
        assert!(!PackageMetadata::is_valid_name("invalid-"));
        assert!(!PackageMetadata::is_valid_name("invalid@name"));
    }

    #[test]
    fn test_keywords() {
        let version = Version::new(1, 0, 0);
        let mut pkg = PackageMetadata::new("test".to_string(), version);
        pkg.keywords = vec!["web".to_string(), "framework".to_string()];

        assert!(pkg.has_keyword("web"));
        assert!(pkg.has_keyword("framework"));
        assert!(!pkg.has_keyword("database"));
    }

    #[test]
    fn test_repository() {
        let repo = Repository::new("https://github.com/user/repo".to_string());
        assert_eq!(repo.url, "https://github.com/user/repo");
        assert_eq!(repo.directory, None);

        let repo_with_dir = Repository::with_directory(
            "https://github.com/user/monorepo".to_string(),
            "packages/core".to_string(),
        );
        assert_eq!(repo_with_dir.directory, Some("packages/core".to_string()));
    }
}
