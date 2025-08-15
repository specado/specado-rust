//! Provider error types and handling

use thiserror::Error;

/// Result type for provider operations
pub type ProviderResult<T> = Result<T, ProviderError>;

/// Errors that can occur when interacting with LLM providers
#[derive(Debug, Error)]
pub enum ProviderError {
    /// Network or connection error
    #[error("Network error: {0}")]
    Network(String),
    
    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        retry_after_secs: Option<u64>,
    },
    
    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    /// Provider returned an error
    #[error("Provider error: {code}: {message}")]
    ProviderError {
        code: String,
        message: String,
    },
    
    /// Timeout occurred
    #[error("Request timed out after {0} seconds")]
    Timeout(u64),
    
    /// Response parsing error
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    
    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    /// Insufficient quota
    #[error("Insufficient quota: {0}")]
    InsufficientQuota(String),
    
    /// Service unavailable
    #[error("Service temporarily unavailable: {0}")]
    ServiceUnavailable(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ProviderError::Timeout(30) // Default timeout value
        } else if err.is_connect() {
            ProviderError::Network(format!("Connection failed: {}", err))
        } else if err.is_status() {
            if let Some(status) = err.status() {
                match status.as_u16() {
                    401 => ProviderError::Authentication(err.to_string()),
                    429 => ProviderError::RateLimit {
                        message: "Too many requests".to_string(),
                        retry_after_secs: None,
                    },
                    500..=599 => ProviderError::ServiceUnavailable(err.to_string()),
                    _ => ProviderError::ProviderError {
                        code: status.to_string(),
                        message: err.to_string(),
                    },
                }
            } else {
                ProviderError::Other(err.to_string())
            }
        } else {
            ProviderError::Network(err.to_string())
        }
    }
}

impl From<serde_json::Error> for ProviderError {
    fn from(err: serde_json::Error) -> Self {
        ProviderError::ParseError(err.to_string())
    }
}