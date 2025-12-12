# pea-core Crate Guide

## Purpose
Core types and utilities shared across all Pea crates. This crate provides the fundamental data structures and error types that other crates depend on.

## Architecture

### Module Structure
- `types/` - Core data types with rkyv serialization
- `error/` - Error types and result aliases  
- `utils/` - Utility functions and helpers

### Key Types
- `Version` - Semantic version with comparison and parsing
- `VersionReq` - Version requirements with matching logic
- `PackageMetadata` - Package information from pea.toml/package.json
- `Dependency` - Dependency specification with features and kind
- `PeaError` - Unified error type for all operations

## Code Quality Rules
- Maximum 4 public functions per file
- All types must support rkyv serialization for performance
- Comprehensive error messages with actionable suggestions
- Property tests for critical invariants (version parsing, semver matching)

## Dependencies
- `rkyv` - Zero-copy serialization for performance
- `serde` - JSON/TOML serialization compatibility
- `thiserror` - Ergonomic error handling

## Testing Strategy
- Unit tests for individual type behavior
- Property tests for mathematical properties (version ordering, semver matching)
- Round-trip tests for serialization formats