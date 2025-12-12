# output/ Module Guide

## Purpose
Terminal output formatting and utilities for consistent user experience.

## Files
- `mod.rs` - Main OutputHandler and common formatting
- `colors.rs` - Terminal color detection and formatting
- `progress.rs` - Progress bar implementations
- `errors.rs` - Error message formatting with suggestions

## Design Principles
- Consistent visual hierarchy across all output
- Automatic color detection (respects NO_COLOR)
- Emoji-based status indicators for quick scanning
- Accessible output that works in all terminals
- Maximum 4 public functions per file

## Color Scheme
- **Green (✓)**: Success messages and completed operations
- **Yellow (⚠)**: Warnings and non-critical issues
- **Red (✗)**: Errors and failures
- **Dim**: Informational messages and details
- **Bold**: Important highlights and headers

## Output Categories
- `info()` - General information (dim text)
- `success()` - Successful operations (green checkmark)
- `warn()` - Warnings (yellow warning symbol)
- `error()` - Errors (red X symbol)
- `step()` - Process steps with custom emoji