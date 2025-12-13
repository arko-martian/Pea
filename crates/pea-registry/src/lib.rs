//! npm registry client for Pea runtime
//!
//! This crate provides HTTP client functionality for fetching package metadata
//! and tarballs from npm registry with connection pooling, retry logic, and caching.

pub mod client;
pub mod api;
pub mod cache;

// Re-export main types
pub use client::{RegistryClient, RetryConfig, AuthConfig};
pub use api::{PackageMetadataResponse, VersionMetadata, DistInfo, RepositoryInfo};
pub use cache::{MetadataCache, CacheEntry, CacheStats};

use pea_core::error::PeaError;

/// Result type for registry operations
pub type RegistryResult<T> = Result<T, PeaError>;