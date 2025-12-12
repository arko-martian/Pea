# utils/ Module Guide

## Purpose
Common utility functions used across multiple Pea crates.

## Files
- `hash.rs` - Blake3 hashing and integrity verification
- `path.rs` - Path normalization and security checks

## Design Principles
- Maximum 4 public functions per file
- Pure functions where possible
- Comprehensive error handling
- Security-first approach for path operations

## Key Functions
- `blake3_hash()` - Fast content hashing
- `verify_integrity()` - Hash verification
- `normalize_path()` - Cross-platform path normalization
- `is_safe_path()` - Path traversal security checks