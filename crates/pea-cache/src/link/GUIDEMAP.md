# Link Module Structure

Hardlinking and file system operations.

## Files

- `mod.rs` - Module exports and common imports
- `linker.rs` - Linker implementation for hardlink operations

## Key Types

- **Linker**: Main interface for linking operations

## Key Functions

- **hardlink_recursive**: Create hardlinks recursively
- **copy_recursive**: Fallback copy operation

## Design Notes

- Prefer hardlinks for space efficiency
- Graceful fallback to copying when hardlinks fail
- Preserve file permissions and timestamps
- Handle cross-filesystem scenarios