//! Utility functions and helpers.
//!
//! Common functionality used across multiple Pea crates.

pub mod hash;
pub mod path;

// Re-export commonly used utilities
pub use hash::{blake3_hash, verify_integrity};
pub use path::{is_safe_path, normalize_path};
