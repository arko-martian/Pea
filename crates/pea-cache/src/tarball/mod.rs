//! Tarball extraction and creation utilities
//!
//! This module provides functionality for working with npm-compatible
//! tarballs including extraction and creation operations.

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use pea_core::error::PeaError;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tar::{Archive, Builder};

use crate::CacheResult;

pub mod extract;
pub mod create;

// Re-export main functions
pub use extract::extract_tarball;
pub use create::create_tarball;