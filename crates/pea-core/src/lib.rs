//! # pea-core
//!
//! Core types and utilities shared across all Pea crates.
//!
//! This crate provides:
//! - Version and VersionReq types with rkyv serialization support
//! - PackageMetadata and Dependency types for package management
//! - PeaError enum for unified error handling
//! - Utility functions for common operations
//!
//! ## Architecture
//!
//! The crate is organized into modules:
//! - `types`: Core data types (Version, PackageMetadata, etc.)
//! - `error`: Error types and result aliases
//! - `utils`: Utility functions and helpers

pub mod error;
pub mod types;
pub mod utils;

// Re-export commonly used types
pub use error::{PeaError, PeaResult};
pub use types::{Dependency, DependencyKind, PackageMetadata, Version, VersionReq};
