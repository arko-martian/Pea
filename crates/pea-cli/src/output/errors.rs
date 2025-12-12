//! Error message formatting with actionable suggestions.
//!
//! Provides user-friendly error formatting that includes context,
//! suggestions for fixes, and relevant file locations when available.

use pea_core::error::PeaError;
use super::colors::ColorSupport;
use std::error::Error;

/// Error formatter with suggestions
pub struct ErrorFormatter {
    colors: ColorSupport,
}

impl ErrorFormatter {
    /// Create a new error formatter
    pub fn new() -> Self {
        Self {
            colors: ColorSupport::detect(),
        }
    }

    /// Format an error with context and suggestions
    pub fn format_error(&self, error: &PeaError) -> String {
        let mut output = String::new();
        
        // Main error message
        output.push_str(&self.colors.red("error"));
        output.push_str(": ");
        output.push_str(&error.to_string());
        output.push('\n');
        
        // Add suggestion if available
        if let Some(suggestion) = error.suggestion() {
            output.push('\n');
            output.push_str(&self.colors.dim("help"));
            output.push_str(": ");
            output.push_str(suggestion);
            output.push('\n');
        }
        
        // Add source chain if available
        let mut source = error.source();
        while let Some(err) = source {
            output.push('\n');
            output.push_str(&self.colors.dim("caused by"));
            output.push_str(": ");
            output.push_str(&err.to_string());
            source = err.source();
        }
        
        output
    }

    /// Format a simple error message
    pub fn format_simple(&self, message: &str) -> String {
        format!("{}: {}", self.colors.red("error"), message)
    }

    /// Format a warning message
    pub fn format_warning(&self, message: &str) -> String {
        format!("{}: {}", self.colors.yellow("warning"), message)
    }

    /// Format file location context
    pub fn format_location(&self, file: &str, line: usize, column: usize) -> String {
        format!(
            "{} {}:{}:{}",
            self.colors.dim("-->"),
            file,
            line,
            column
        )
    }
}

impl Default for ErrorFormatter {
    fn default() -> Self {
        Self::new()
    }
}