//! Core data types for Pea package management.
//!
//! This module provides the fundamental types used throughout the Pea ecosystem:
//! - Version types for semantic versioning
//! - Package metadata structures
//! - Dependency specifications

pub mod dependency;
pub mod package;
pub mod version;

// Re-export all public types
pub use dependency::{Dependency, DependencyKind};
pub use package::{PackageMetadata, Repository};
pub use version::{Comparator, Op, PartialVersion, Version, VersionError, VersionReq};
