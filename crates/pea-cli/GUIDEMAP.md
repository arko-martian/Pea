# pea-cli Crate Guide

## Purpose
Command-line interface for the Pea JavaScript/TypeScript runtime and package manager. This crate provides the user-facing CLI tool that orchestrates all Pea operations.

## Architecture

### Module Structure
- `main.rs` - Entry point with argument parsing and runtime setup
- `commands/` - Command implementations and dispatch logic
- `output/` - Terminal output formatting (colors, progress, errors)

### Key Components
- **CLI Parser** - Uses clap for robust argument parsing
- **Command Dispatcher** - Routes commands to appropriate handlers
- **Output System** - Handles colors, progress bars, and error formatting
- **Context Management** - Shared state and configuration across commands

## Code Quality Rules
- Maximum 4 public functions per file
- All commands are async and use Tokio runtime
- Comprehensive error handling with user-friendly messages
- Structured logging with tracing
- Graceful panic handling with bug reporting instructions

## Dependencies
- `clap` - Command-line argument parsing
- `tokio` - Async runtime for I/O operations
- `tracing` - Structured logging
- `pea-core` - Shared types and utilities

## Command Categories
- **Project**: new, init
- **Dependencies**: install, add, remove, update, check
- **Execution**: run, test, bench, build
- **Publishing**: publish, login
- **Meta**: upgrade, doc, clean

## Design Principles
- Fast startup (<5ms cold start target)
- User-friendly error messages with suggestions
- Consistent output formatting
- Cargo-inspired UX patterns