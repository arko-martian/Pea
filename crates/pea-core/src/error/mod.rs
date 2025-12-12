//! Error types and result aliases for Pea operations.
//!
//! Provides a unified error type that covers all possible error conditions
//! across the Pea ecosystem with actionable error messages.

use thiserror::Error;

/// Unified error type for all Pea operations
#[derive(Error, Debug)]
pub enum PeaError {
    // Config errors
    #[error("Failed to parse pea.toml: {message} at line {line}, column {column}")]
    TomlParse {
        message: String,
        line: usize,
        column: usize,
    },

    #[error("Failed to parse package.json: {message}")]
    JsonParse { message: String },

    #[error("Configuration field '{field}' is invalid: {reason}")]
    ConfigValidation { field: String, reason: String },

    // Registry errors
    #[error("Package '{name}' not found in registry")]
    PackageNotFound { name: String },

    #[error("Network error: {message}")]
    Network {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    // Resolution errors
    #[error(
        "Version conflict: {package} requires {required}, but {conflicting} requires {conflict}"
    )]
    VersionConflict {
        package: String,
        required: String,
        conflicting: String,
        conflict: String,
    },

    #[error("Circular dependency detected: {cycle}")]
    CircularDependency { cycle: String },

    // Cache errors
    #[error("Integrity check failed for {package}: expected {expected}, got {actual}")]
    IntegrityFailure {
        package: String,
        expected: String,
        actual: String,
    },

    // Runtime errors
    #[error("JavaScript error: {message}\n{stack}")]
    JavaScript { message: String, stack: String },

    #[error("Permission denied: {permission} access to {resource}")]
    PermissionDenied {
        permission: String,
        resource: String,
    },

    #[error("Module not found: {specifier}")]
    ModuleNotFound { specifier: String },

    // IO errors
    #[error("IO error: {message}")]
    Io {
        message: String,
        #[source]
        source: std::io::Error,
    },
}

/// Result type alias for Pea operations
pub type PeaResult<T> = Result<T, PeaError>;

impl PeaError {
    /// Create a network error from any error type
    pub fn network<E>(message: String, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Network {
            message,
            source: Some(Box::new(source)),
        }
    }

    /// Create an IO error from std::io::Error
    pub fn io(message: String, source: std::io::Error) -> Self {
        Self::Io { message, source }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(self, PeaError::Network { .. } | PeaError::Io { .. })
    }

    /// Get a user-friendly suggestion for fixing this error
    pub fn suggestion(&self) -> Option<&'static str> {
        match self {
            PeaError::PackageNotFound { .. } => {
                Some("Check the package name spelling or try searching the registry")
            },
            PeaError::Network { .. } => Some("Check your internet connection and try again"),
            PeaError::VersionConflict { .. } => {
                Some("Try updating dependencies or use 'pea update' to resolve conflicts")
            },
            PeaError::CircularDependency { .. } => {
                Some("Remove circular dependencies by restructuring your packages")
            },
            PeaError::PermissionDenied { .. } => {
                Some("Run with appropriate permissions or use --allow-* flags")
            },
            _ => None,
        }
    }
}
