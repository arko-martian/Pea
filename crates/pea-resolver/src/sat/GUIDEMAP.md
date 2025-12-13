# SAT Module Guide

## Purpose
SAT-based dependency resolution engine with parallel processing and conflict detection.

## Key Types
- `Resolver` - Main resolution engine with registry client
- `ResolutionResult` - Complete resolution with graph and metadata
- `ConflictError` - Version conflict with detailed information

## Functions (Max 4 Public)
1. `new()` - Create resolver with registry client and cache
2. `resolve()` - Resolve dependencies with parallel processing
3. `detect_conflicts()` - Find version conflicts in requirements
4. `select_version()` - Choose best version matching constraints

## Resolution Strategy
- Parallel processing with Rayon for concurrent resolution
- Version selection using highest compatible version
- Conflict detection with clear error messages
- Workspace dependency linking for local packages

## Performance Features
- Concurrent resolution of dependency batches
- Cached package metadata for repeated lookups
- Thread-safe resolved package storage
- Efficient version constraint satisfaction