//! Content-Addressable Storage implementation
//!
//! This module provides the core CAS functionality including
//! content hashing, storage, and retrieval operations.

use blake3::Hasher;
use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use pea_core::error::PeaError;
use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use std::fs;
use std::sync::Arc;
use tokio::fs as async_fs;

use crate::CacheResult;

pub mod store;
pub mod hash;
pub mod index;

// Re-export main types
pub use store::CasStore;
pub use hash::ContentHash;
pub use index::{CasIndex, CacheEntry};