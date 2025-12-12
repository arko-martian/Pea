//! Configuration parsing for Pea runtime
//!
//! This crate handles parsing and validation of pea.toml and package.json files,
//! providing a unified configuration interface for the Pea runtime.

pub mod toml;
pub mod json;
pub mod merge;

// Re-export main types
pub use toml::{PeaToml, PackageSection, DependencySpec, WorkspaceSection, ProfileSection};
pub use json::PackageJson;
pub use merge::{ConfigLoader, ConfigLayering};

use pea_core::error::PeaError;

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, PeaError>;