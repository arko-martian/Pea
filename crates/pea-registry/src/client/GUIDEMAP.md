# Client Module Guide

## Purpose
HTTP client implementation with connection pooling, retry logic, and authentication.

## Key Types
- `RegistryClient` - Main HTTP client with reqwest
- `RetryConfig` - Exponential backoff configuration
- `AuthConfig` - Authentication credentials

## Functions (Max 4 Public)
1. `new()` - Create client with connection pooling
2. `with_auth()` - Create client with authentication
3. `fetch_metadata()` - Fetch package metadata with retry
4. `download_tarball()` - Download package tarball with integrity check

## Performance Features
- Connection pooling (50 max idle per host, 90s timeout)
- HTTP/2 with prior knowledge
- Gzip compression
- 30s request timeout
- Exponential backoff (3 retries, 100ms-10s)

## Implementation Status
✅ HTTP client with connection pooling
✅ Bearer token and basic authentication
✅ Exponential backoff retry logic
✅ Package metadata fetching
✅ Tarball downloading with integrity verification
✅ Scoped package URL encoding
✅ Comprehensive unit tests (13 tests)