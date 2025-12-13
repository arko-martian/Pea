# CAS Module Structure

Content-Addressable Storage implementation.

## Files

- `mod.rs` - Module exports and common imports
- `store.rs` - CasStore implementation for storage operations
- `hash.rs` - ContentHash type and hashing utilities
- `index.rs` - CasIndex for metadata management

## Key Types

- **CasStore**: Main storage interface
- **ContentHash**: Blake3 hash wrapper
- **CasIndex**: Metadata index for cache entries
- **CacheEntry**: Individual cache entry metadata

## Design Notes

- Files stored in nested directories based on hash prefix (ab/cd/abcd...)
- Blake3 used for fast, secure hashing
- Atomic operations for thread safety
- Memory-mapped files for large content when possible