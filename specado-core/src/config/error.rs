//! Configuration error types with detailed error reporting

use thiserror::Error;
use std::fmt;

/// Main configuration error type with detailed context
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error reading config from '{path}': {source}")]
    IoError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Parse error in '{path}' at line {}, column {}: {message}", 
            .line.unwrap_or(0), .column.unwrap_or(0))]
    ParseError {
        path: String,
        line: Option<usize>,
        column: Option<usize>,
        message: String,
    },

    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),

    #[error("Environment variable '{var}' not found")]
    EnvVarNotFound { var: String },

    #[error("Invalid configuration: {message}")]
    Invalid { message: String },
}

/// Validation error with field path for precise error reporting
#[derive(Debug, Error)]
pub struct ValidationError {
    /// Path to the field that failed validation (e.g., "providers[0].api_key")
    pub field_path: String,
    /// The validation error kind
    pub kind: ValidationErrorKind,
    /// Optional additional context
    pub context: Option<String>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation failed at '{}': {}", self.field_path, self.kind)?;
        if let Some(ctx) = &self.context {
            write!(f, " ({})", ctx)?;
        }
        Ok(())
    }
}

/// Specific validation error types
#[derive(Debug, Error)]
pub enum ValidationErrorKind {
    #[error("required field is missing")]
    RequiredFieldMissing,

    #[error("invalid value: expected {expected}, got {actual}")]
    InvalidValue { expected: String, actual: String },

    #[error("value out of range: {message}")]
    OutOfRange { message: String },

    #[error("invalid format: {message}")]
    InvalidFormat { message: String },

    #[error("duplicate value: {value}")]
    DuplicateValue { value: String },

    #[error("incompatible configuration: {message}")]
    Incompatible { message: String },

    #[error("invalid URL: {message}")]
    InvalidUrl { message: String },

    #[error("invalid version: expected {expected}, got {actual}")]
    InvalidVersion { expected: String, actual: String },

    #[error("custom validation failed: {message}")]
    Custom { message: String },
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(field_path: impl Into<String>, kind: ValidationErrorKind) -> Self {
        Self {
            field_path: field_path.into(),
            kind,
            context: None,
        }
    }

    /// Add context to the validation error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Helper to create a required field error
    pub fn required(field_path: impl Into<String>) -> Self {
        Self::new(field_path, ValidationErrorKind::RequiredFieldMissing)
    }

    /// Helper to create an invalid value error
    pub fn invalid_value(
        field_path: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::new(
            field_path,
            ValidationErrorKind::InvalidValue {
                expected: expected.into(),
                actual: actual.into(),
            },
        )
    }

    /// Helper to create an out of range error
    pub fn out_of_range(field_path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            field_path,
            ValidationErrorKind::OutOfRange {
                message: message.into(),
            },
        )
    }

    /// Helper to create an invalid format error
    pub fn invalid_format(field_path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(
            field_path,
            ValidationErrorKind::InvalidFormat {
                message: message.into(),
            },
        )
    }
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;