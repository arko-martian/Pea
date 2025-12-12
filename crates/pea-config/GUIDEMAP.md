# pea-config Crate Guide

## Purpose
Configuration parsing and validation for Pea runtime, handling both pea.toml and package.json formats.

## Architecture

### Core Modules
- `toml/` - pea.toml parsing and serialization
- `json/` - package.json parsing and serialization  
- `merge/` - Configuration layering and fallback logic

### Key Types
- `PeaToml` - Complete pea.toml configuration
- `PackageJson` - Complete package.json configuration
- `ConfigLoader` - Unified configuration loading interface

## Code Quality Rules

### Maximum 4 Public Functions Per File
Each module file should expose at most 4 public functions to maintain focused interfaces.

### Error Handling
- Use `PeaError` from pea-core for all errors
- Provide actionable error messages with file locations
- Include suggestions for common mistakes

### Performance
- Cache parsed configurations
- Use zero-copy parsing where possible
- Minimize allocations during parsing

### Testing
- Property tests for round-trip serialization
- Unit tests for all parsing edge cases
- Integration tests for configuration layering

## Dependencies
- `toml_edit` - TOML parsing with error locations
- `serde_json` - JSON parsing and serialization
- `pea-core` - Shared types and error handling