# error/ Module Guide

## Purpose
Unified error handling for all Pea operations with actionable error messages.

## Files
- `mod.rs` - PeaError enum and PeaResult type alias

## Design Principles
- Single error type for the entire ecosystem
- Structured error data for programmatic handling
- Human-readable messages with actionable suggestions
- Source error chaining for debugging
- Categorized by operation type (config, registry, resolution, etc.)

## Error Categories
- **Config**: TOML/JSON parsing and validation errors
- **Registry**: Package discovery and network errors  
- **Resolution**: Dependency conflicts and circular dependencies
- **Cache**: Integrity failures and corruption
- **Runtime**: JavaScript execution and permission errors
- **IO**: File system and network I/O errors

## Usage
- Use `PeaResult<T>` for all fallible operations
- Include context in error messages
- Provide suggestions when possible
- Chain source errors for debugging