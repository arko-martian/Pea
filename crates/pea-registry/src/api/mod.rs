//! npm registry API response types

use std::collections::HashMap;
use serde::{Deserialize, Serialize};


/// Package metadata response from npm registry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageMetadataResponse {
    /// Package name
    pub name: String,
    /// Package description
    pub description: Option<String>,
    /// Latest version
    #[serde(rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,
    /// All versions metadata
    pub versions: HashMap<String, VersionMetadata>,
    /// Package creation time
    pub time: HashMap<String, String>,
}

/// Metadata for a specific package version
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionMetadata {
    /// Version string
    pub version: String,
    /// Package description
    pub description: Option<String>,
    /// Main entry point
    pub main: Option<String>,
    /// License
    pub license: Option<String>,
    /// Repository information
    pub repository: Option<RepositoryInfo>,
    /// Keywords
    pub keywords: Option<Vec<String>>,
    /// Dependencies
    pub dependencies: Option<HashMap<String, String>>,
    /// Dev dependencies
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
    /// Peer dependencies
    #[serde(rename = "peerDependencies")]
    pub peer_dependencies: Option<HashMap<String, String>>,
    /// Distribution information
    pub dist: DistInfo,
}

/// Repository information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepositoryInfo {
    /// Repository type (usually "git")
    #[serde(rename = "type")]
    pub repo_type: Option<String>,
    /// Repository URL
    pub url: Option<String>,
}

/// Distribution information for package tarball
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DistInfo {
    /// Tarball download URL
    pub tarball: String,
    /// SHA-1 checksum (legacy)
    pub shasum: String,
    /// Subresource integrity hash (preferred)
    pub integrity: Option<String>,
    /// Unpackaged size in bytes
    #[serde(rename = "unpackedSize")]
    pub unpacked_size: Option<u64>,
    /// File count
    #[serde(rename = "fileCount")]
    pub file_count: Option<u32>,
}