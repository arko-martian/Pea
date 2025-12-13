# pea-registry Crate Guide

## Purpose
HTTP client for fetching package metadata and tarballs from npm registry with high performance and reliability.

## Architecture

### Core Modules
- `client/` - HTTP client with connection pooling and retry logic
- `api/` - npm registry API types and response parsing
- `cache/` - Metadata caching with TTL support

### Key Types
- `RegistryClient` - Main HTTP client with connection pooling
- `PackageMetadataResponse` - npm registry package metadata
- `MetadataCache` - In-memory cache with TTL

## Code Quality Rules

### Maximum 4 Public Functions Per File
Each module file should expose at most 4 public functions to maintain focused interfaces.

### Performance Requirements
- Connection pooling with 50 max idle connections per host
- HTTP/2 support with prior knowledge
- Gzip compression enabled
- 30s timeout for requests
- Exponential backoff retry (3 attempts, 100ms-10s)

### Error Handling
- Use `PeaError` from pea-core for all errors
- Retry on network failures with exponential backoff
- Handle 404s as PackageNotFound errors
- Provide actionable error messages

### Testing
- Unit tests for all API parsing
- Integration tests with wiremock for HTTP behavior
- Property tests for retry logic
- Offline mode testing

## Dependencies
- `reqwest` - HTTP client with connection pooling
- `tokio` - Async runtime
- `dashmap` - Concurrent caching
- `blake3` - Integrity verification
## Implementation Status

### ✅ Completed Tasks
- **Task 9.1**: Initialize pea-registry crate structure
- **Task 9.2**: HTTP client with connection pooling (reqwest, HTTP/2, gzip)
- **Task 9.3**: Authentication (bearer token + basic auth)
- **Task 9.4**: Retry logic with exponential backoff (3 retries, 100ms-10s)
- **Task 9.5**: Package metadata response types (complete npm API)
- **Task 9.6**: Package metadata fetching with scoped package support
- **Task 9.7**: Tarball downloading with integrity verification (sha512/sha1)
- **Task 9.8**: Metadata caching with TTL (1 hour default, thread-safe)
- **Task 9.11**: Unit tests (21 tests total, 100% passing)

### 🔄 Remaining Tasks
- **Task 9.9**: Offline mode (detect network, use stale cache)
- **Task 9.10**: Custom registry support (scope-specific registries)

### Test Coverage
- **Client tests**: 13 tests (HTTP, auth, retry, integrity, scoped packages)
- **Cache tests**: 8 tests (TTL, cleanup, stats, freshness)
- **Total**: 21 tests with comprehensive scenarios and edge cases

### Performance Achieved
- Connection pooling: 50 max idle per host, 90s timeout
- HTTP/2 with prior knowledge for multiplexing
- Gzip compression for reduced bandwidth
- Exponential backoff prevents registry overload
- Thread-safe caching with DashMap for concurrent access