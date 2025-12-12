# commands/ Module Guide

## Purpose
Command implementations and dispatch logic for all Pea CLI commands.

## Files
- `mod.rs` - Command dispatcher and shared context
- `new.rs` - `pea new` command implementation
- `init.rs` - `pea init` command implementation
- `install.rs` - `pea install` command implementation
- `add.rs` - `pea add` command implementation
- `remove.rs` - `pea remove` command implementation
- `run.rs` - `pea run` command implementation
- `build.rs` - `pea build` command implementation
- `test.rs` - `pea test` command implementation

## Design Principles
- All commands are async functions
- Commands receive a shared CommandContext
- Maximum 4 public functions per file
- Comprehensive error handling with user-friendly messages
- Structured logging for debugging

## Command Context
The `CommandContext` provides:
- Current working directory
- Output handler for consistent formatting
- Shared configuration (when implemented)
- Async runtime access

## Error Handling
- Use `PeaResult<()>` for all command functions
- Provide actionable error messages
- Log errors with appropriate levels
- Suggest fixes when possible