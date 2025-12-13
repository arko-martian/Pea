//! Dependency resolution engine for Pea runtime
//!
//! This crate provides high-performance dependency resolution with cycle detection,
//! parallel processing, and comprehensive conflict resolution for JavaScript packages.

pub mod graph;
pub mod sat;
pub mod semver;

// Re-export main types
pub use graph::{DependencyGraph, PackageNode, DependencyEdge, PackageId};
pub use sat::{Resolver, ResolutionResult, ConflictError};

use pea_core::error::PeaError;

/// Result type for resolver operations
pub type ResolverResult<T> = Result<T, PeaError>;