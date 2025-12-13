//! Metadata caching with TTL support

use std::time::{Duration, SystemTime};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use crate::api::PackageMetadataResponse;

/// Cache entry with TTL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Cached metadata
    pub metadata: PackageMetadataResponse,
    /// When the entry was stored
    pub stored_at: SystemTime,
    /// Time-to-live duration
    pub ttl: Duration,
}

impl CacheEntry {
    /// Create new cache entry with default TTL (1 hour)
    pub fn new(metadata: PackageMetadataResponse) -> Self {
        Self::with_ttl(metadata, Duration::from_secs(3600))
    }

    /// Create cache entry with custom TTL
    pub fn with_ttl(metadata: PackageMetadataResponse, ttl: Duration) -> Self {
        Self {
            metadata,
            stored_at: SystemTime::now(),
            ttl,
        }
    }

    /// Check if cache entry is still fresh
    pub fn is_fresh(&self) -> bool {
        match self.stored_at.elapsed() {
            Ok(elapsed) => elapsed < self.ttl,
            Err(_) => false, // Clock went backwards, consider stale
        }
    }

    /// Get age of cache entry
    pub fn age(&self) -> Option<Duration> {
        self.stored_at.elapsed().ok()
    }
}

/// In-memory metadata cache with TTL
#[derive(Debug)]
pub struct MetadataCache {
    /// Cache storage
    cache: DashMap<String, CacheEntry>,
}

impl MetadataCache {
    /// Create new metadata cache
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }
}
impl MetadataCache {
    /// Get cached metadata if fresh
    pub fn get(&self, package_name: &str) -> Option<PackageMetadataResponse> {
        let entry = self.cache.get(package_name)?;
        if entry.is_fresh() {
            Some(entry.metadata.clone())
        } else {
            // Remove stale entry
            self.cache.remove(package_name);
            None
        }
    }

    /// Store metadata with default TTL
    pub fn insert(&self, package_name: String, metadata: PackageMetadataResponse) {
        let entry = CacheEntry::new(metadata);
        self.cache.insert(package_name, entry);
    }

    /// Store metadata with custom TTL
    pub fn insert_with_ttl(&self, package_name: String, metadata: PackageMetadataResponse, ttl: Duration) {
        let entry = CacheEntry::with_ttl(metadata, ttl);
        self.cache.insert(package_name, entry);
    }

    /// Check if package is cached and fresh
    pub fn contains_fresh(&self, package_name: &str) -> bool {
        self.cache.get(package_name)
            .map(|entry| entry.is_fresh())
            .unwrap_or(false)
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut fresh_count = 0;
        let mut stale_count = 0;
        
        for entry in self.cache.iter() {
            if entry.is_fresh() {
                fresh_count += 1;
            } else {
                stale_count += 1;
            }
        }

        CacheStats {
            total_entries: self.cache.len(),
            fresh_entries: fresh_count,
            stale_entries: stale_count,
        }
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Remove stale entries
    pub fn cleanup(&self) -> usize {
        let mut removed = 0;
        self.cache.retain(|_, entry| {
            if entry.is_fresh() {
                true
            } else {
                removed += 1;
                false
            }
        });
        removed
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of entries
    pub total_entries: usize,
    /// Number of fresh entries
    pub fresh_entries: usize,
    /// Number of stale entries
    pub stale_entries: usize,
}

impl Default for MetadataCache {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests;