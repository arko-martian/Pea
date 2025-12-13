//! Unit tests for metadata cache

use super::*;
use std::time::Duration;
use std::collections::HashMap;

fn create_test_metadata() -> PackageMetadataResponse {
    PackageMetadataResponse {
        name: "test-package".to_string(),
        description: Some("A test package".to_string()),
        dist_tags: {
            let mut tags = HashMap::new();
            tags.insert("latest".to_string(), "1.0.0".to_string());
            tags
        },
        versions: HashMap::new(),
        time: HashMap::new(),
    }
}

#[test]
fn test_cache_entry_creation() {
    let metadata = create_test_metadata();
    let entry = CacheEntry::new(metadata.clone());
    
    assert_eq!(entry.metadata.name, "test-package");
    assert_eq!(entry.ttl, Duration::from_secs(3600)); // 1 hour default
    assert!(entry.is_fresh());
}

#[test]
fn test_cache_entry_with_custom_ttl() {
    let metadata = create_test_metadata();
    let ttl = Duration::from_secs(300); // 5 minutes
    let entry = CacheEntry::with_ttl(metadata, ttl);
    
    assert_eq!(entry.ttl, ttl);
    assert!(entry.is_fresh());
}

#[test]
fn test_cache_entry_age() {
    let metadata = create_test_metadata();
    let entry = CacheEntry::new(metadata);
    
    let age = entry.age();
    assert!(age.is_some());
    assert!(age.unwrap() < Duration::from_millis(100)); // Should be very recent
}

#[test]
fn test_metadata_cache_insert_and_get() {
    let cache = MetadataCache::new();
    let metadata = create_test_metadata();
    
    cache.insert("test-package".to_string(), metadata.clone());
    
    let retrieved = cache.get("test-package");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test-package");
}

#[test]
fn test_metadata_cache_get_nonexistent() {
    let cache = MetadataCache::new();
    
    let retrieved = cache.get("nonexistent-package");
    assert!(retrieved.is_none());
}

#[test]
fn test_metadata_cache_contains_fresh() {
    let cache = MetadataCache::new();
    let metadata = create_test_metadata();
    
    assert!(!cache.contains_fresh("test-package"));
    
    cache.insert("test-package".to_string(), metadata);
    assert!(cache.contains_fresh("test-package"));
}

#[test]
fn test_metadata_cache_insert_with_ttl() {
    let cache = MetadataCache::new();
    let metadata = create_test_metadata();
    let ttl = Duration::from_secs(300);
    
    cache.insert_with_ttl("test-package".to_string(), metadata, ttl);
    
    let retrieved = cache.get("test-package");
    assert!(retrieved.is_some());
}

#[test]
fn test_cache_stats() {
    let cache = MetadataCache::new();
    let metadata1 = create_test_metadata();
    let mut metadata2 = create_test_metadata();
    metadata2.name = "another-package".to_string();
    
    // Initially empty
    let stats = cache.stats();
    assert_eq!(stats.total_entries, 0);
    assert_eq!(stats.fresh_entries, 0);
    assert_eq!(stats.stale_entries, 0);
    
    // Add some entries
    cache.insert("test-package".to_string(), metadata1);
    cache.insert("another-package".to_string(), metadata2);
    
    let stats = cache.stats();
    assert_eq!(stats.total_entries, 2);
    assert_eq!(stats.fresh_entries, 2);
    assert_eq!(stats.stale_entries, 0);
}

#[test]
fn test_cache_clear() {
    let cache = MetadataCache::new();
    let metadata = create_test_metadata();
    
    cache.insert("test-package".to_string(), metadata);
    assert!(cache.contains_fresh("test-package"));
    
    cache.clear();
    assert!(!cache.contains_fresh("test-package"));
    
    let stats = cache.stats();
    assert_eq!(stats.total_entries, 0);
}

#[test]
fn test_cache_cleanup() {
    let cache = MetadataCache::new();
    let metadata = create_test_metadata();
    
    // Insert with very short TTL
    cache.insert_with_ttl("test-package".to_string(), metadata, Duration::from_nanos(1));
    
    // Wait a bit to ensure it's stale
    std::thread::sleep(Duration::from_millis(1));
    
    let removed = cache.cleanup();
    assert_eq!(removed, 1);
    
    let stats = cache.stats();
    assert_eq!(stats.total_entries, 0);
}

#[test]
fn test_cache_default() {
    let cache = MetadataCache::default();
    let stats = cache.stats();
    assert_eq!(stats.total_entries, 0);
}