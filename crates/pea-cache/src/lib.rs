//! Content-Addressable Storage for Pea
//!
//! This crate provides a content-addressable storage system for caching
//! packages and their contents. It uses Blake3 hashing for integrity
//! and supports tarball extraction/creation.

pub mod cas;
pub mod tarball;
pub mod link;

// Re-export main types
pub use cas::{CasStore, ContentHash, CasIndex, CacheEntry};
pub use tarball::{extract_tarball, create_tarball};
pub use link::Linker;

use pea_core::error::PeaError;

/// Result type for cache operations
pub type CacheResult<T> = Result<T, PeaError>;