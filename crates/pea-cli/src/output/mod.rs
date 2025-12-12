//! Terminal output formatting and utilities.
//!
//! This module provides consistent output formatting across all commands,
//! including colors, progress bars, and error messages.

pub mod colors;
pub mod progress;
pub mod errors;

// Re-export modules

/// Output handler for consistent terminal formatting
pub struct OutputHandler {
    colors: colors::ColorSupport,
}

impl OutputHandler {
    /// Create a new output handler
    pub fn new() -> Self {
        Self {
            colors: colors::ColorSupport::detect(),
        }
    }

    /// Print an info message
    pub fn info(&self, message: &str) {
        println!("{}", self.colors.dim(message));
    }

    /// Print a success message
    pub fn success(&self, message: &str) {
        println!("{} {}", self.colors.green("✓"), message);
    }

    /// Print a warning message
    pub fn warn(&self, message: &str) {
        println!("{} {}", self.colors.yellow("⚠"), message);
    }

    /// Print an error message
    pub fn error(&self, message: &str) {
        eprintln!("{} {}", self.colors.red("✗"), message);
    }

    /// Print a step message with emoji
    pub fn step(&self, emoji: &str, message: &str) {
        println!("{} {}", emoji, message);
    }
}

impl Default for OutputHandler {
    fn default() -> Self {
        Self::new()
    }
}