# types/ Module Guide

## Purpose
Core data types for Pea package management with rkyv serialization support.

## Files
- `version.rs` - Semantic version types (Version, VersionReq, Comparator, Op)
- `package.rs` - Package metadata types (PackageMetadata, Repository)
- `dependency.rs` - Dependency specification types (Dependency, DependencyKind)

## Design Principles
- All types derive rkyv traits for zero-copy serialization
- Comprehensive parsing with detailed error messages
- Mathematical properties verified by property tests
- Display implementations for human-readable output

## Key Invariants
- Version comparison is transitive (a < b && b < c → a < c)
- Version parsing round-trips (parse(display(v)) == v)
- VersionReq matching is consistent with semver specification