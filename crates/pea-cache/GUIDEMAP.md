# pea-cache Crate Structure

This crate implements content-addressable storage for the Pea package manager.

## Directory Structure

- `src/cas/` - Content-addressable storage implementation
- `src/tarball/` - Tarball extraction and creation utilities  
- `src/link/` - Hardlinking and file system operations

## Key Components

- **CasStore**: Main storage interface for content-addressable files
- **ContentHash**: Blake3-based content hashing
- **CasIndex**: Metadata index for cached entries
- **Tarball utilities**: Extract and create npm-compatible tarballs
- **Linker**: Hardlink creation with fallback to copying

## Design Principles

- Content-addressable: Files stored by their Blake3 hash
- Integrity verification: All content verified on retrieval
- Parallel operations: Use Rayon for concurrent hashing
- Zero-copy when possible: Memory-mapped files and shared buffers
- Atomic operations: Safe concurrent access to cache

## File Limits

Maximum 4 functions per file to maintain readability.