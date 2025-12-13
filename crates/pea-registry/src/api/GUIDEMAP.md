# API Module Guide

## Purpose
npm registry API types and response parsing with comprehensive error handling.

## Key Types
- `PackageMetadataResponse` - Complete package metadata from registry
- `VersionMetadata` - Individual version information
- `DistInfo` - Distribution tarball information with integrity
- `RepositoryInfo` - Repository metadata

## Functions (Max 4 Public)
1. `PackageMetadataResponse` - Deserialize registry JSON response
2. `VersionMetadata` - Individual version with dependencies
3. `DistInfo` - Tarball URL, shasum, and integrity
4. `RepositoryInfo` - Git repository information

## Registry Compatibility
- Support for scoped packages (@org/pkg → @org%2fpkg)
- Handle abbreviated metadata responses
- Parse both sha512 and shasum integrity formats
- Support custom registry URLs

## Implementation Status
✅ Complete npm registry API types
✅ Serde serialization/deserialization
✅ Support for all dependency types (normal, dev, peer)
✅ Repository and distribution metadata
✅ Compatible with npm registry JSON format