# Tarball Module Structure

Tarball extraction and creation utilities for npm compatibility.

## Files

- `mod.rs` - Module exports and common imports
- `extract.rs` - Tarball extraction functionality
- `create.rs` - Tarball creation functionality

## Key Functions

- **extract_tarball**: Extract gzipped tarballs safely
- **create_tarball**: Create npm-compatible tarballs

## Design Notes

- Stream-based processing to avoid loading large files into memory
- Safe extraction with path validation to prevent directory traversal
- Preserve file permissions and handle symlinks safely
- npm-compatible format with package/ prefix for created tarballs