//! Hardlinking and file system operations
//!
//! This module provides functionality for creating hardlinks
//! with fallback to copying when hardlinks are not supported.

use camino::{Utf8Path, Utf8PathBuf};
use pea_core::error::PeaError;
use std::fs;
use std::io;

use crate::CacheResult;

pub mod linker;

// Re-export main types
pub use linker::{Linker, LinkResult, PackageInfo, NodeModulesResult, CleanupResult};