# Cache Module Guide

## Purpose
In-memory metadata caching with TTL support and offline mode capabilities.

## Key Types
- `MetadataCache` - Thread-safe cache with DashMap
- `CacheEntry` - Cached metadata with timestamp and TTL
- `CacheStats` - Cache hit/miss statistics

## Functions (Max 4 Public)
1. `new()` - Create cache with default TTL (1 hour)
2. `get()` - Get cached metadata if fresh
3. `insert()` - Store metadata with default or custom TTL
4. `cleanup()` - Remove stale entries and return count

## Caching Strategy
- Default TTL of 1 hour for metadata
- Thread-safe with DashMap for concurrent access
- Automatic stale entry removal on access
- Cache statistics for monitoring
- Support for custom TTL per entry

## Implementation Status
✅ Thread-safe in-memory cache with DashMap
✅ TTL-based freshness checking
✅ Automatic stale entry cleanup
✅ Cache statistics and monitoring
✅ Custom TTL support
✅ Comprehensive unit tests (8 tests)