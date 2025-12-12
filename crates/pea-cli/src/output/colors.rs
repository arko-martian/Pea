//! Terminal color support detection and formatting.
//!
//! Provides automatic color detection that respects NO_COLOR environment variable
//! and TTY detection for consistent output across different environments.

use std::env;
use std::io::{self, IsTerminal};

/// Color support detection and formatting
pub struct ColorSupport {
    enabled: bool,
}

impl ColorSupport {
    /// Detect color support automatically
    pub fn detect() -> Self {
        let enabled = Self::should_use_colors();
        Self { enabled }
    }

    /// Force enable colors
    pub fn enabled() -> Self {
        Self { enabled: true }
    }

    /// Force disable colors
    pub fn disabled() -> Self {
        Self { enabled: false }
    }

    /// Check if colors should be used
    fn should_use_colors() -> bool {
        // Respect NO_COLOR environment variable
        if env::var("NO_COLOR").is_ok() {
            return false;
        }

        // Check if we're in a TTY
        io::stderr().is_terminal() && io::stdout().is_terminal()
    }
}

impl ColorSupport {
    /// Format text in green
    pub fn green(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[32m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    /// Format text in yellow
    pub fn yellow(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[33m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    /// Format text in red
    pub fn red(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[31m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    /// Format text as dim/gray
    pub fn dim(&self, text: &str) -> String {
        if self.enabled {
            format!("\x1b[2m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }
}