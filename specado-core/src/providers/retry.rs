//! Retry policy and error mapping for resilient provider operations
//!
//! This module implements configurable retry policies with exponential backoff,
//! jitter, and intelligent error mapping for provider operations.

use crate::providers::routing::ProviderError;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use rand::Rng;

/// Configuration for retry behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (not including the initial attempt)
    pub max_retries: u32,
    
    /// Initial delay before first retry (milliseconds)
    pub initial_delay_ms: u64,
    
    /// Maximum delay between retries (milliseconds)
    pub max_delay_ms: u64,
    
    /// Base for exponential backoff (e.g., 2.0 for doubling)
    pub exponential_base: f64,
    
    /// Jitter factor (0.0 to 1.0) to randomize delays
    pub jitter_factor: f64,
    
    /// Whether to respect retry-after headers
    pub respect_retry_after: bool,
    
    /// Maximum total time to spend retrying (milliseconds)
    pub timeout_ms: Option<u64>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            exponential_base: 2.0,
            jitter_factor: 0.1,
            respect_retry_after: true,
            timeout_ms: Some(30_000), // 30 seconds total
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy with custom configuration
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }
    
    /// Create an aggressive retry policy for critical operations
    pub fn aggressive() -> Self {
        Self {
            max_retries: 5,
            initial_delay_ms: 50,
            max_delay_ms: 5_000,
            exponential_base: 1.5,
            jitter_factor: 0.2,
            respect_retry_after: true,
            timeout_ms: Some(60_000), // 1 minute
        }
    }
    
    /// Create a conservative retry policy to minimize load
    pub fn conservative() -> Self {
        Self {
            max_retries: 2,
            initial_delay_ms: 500,
            max_delay_ms: 15_000,
            exponential_base: 3.0,
            jitter_factor: 0.3,
            respect_retry_after: true,
            timeout_ms: Some(20_000), // 20 seconds
        }
    }
    
    /// Create a policy with no retries
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            ..Default::default()
        }
    }
    
    /// Calculate the delay for a given retry attempt
    pub fn calculate_delay(&self, attempt: u32, error: &ProviderError) -> Duration {
        // Check for retry-after header first
        if self.respect_retry_after {
            if let Some(retry_after) = error.retry_delay() {
                return retry_after;
            }
        }
        
        // Calculate exponential backoff
        let base_delay = self.initial_delay_ms as f64 * self.exponential_base.powi(attempt as i32);
        let capped_delay = base_delay.min(self.max_delay_ms as f64);
        
        // Add jitter
        let delay_with_jitter = if self.jitter_factor > 0.0 {
            let mut rng = rand::thread_rng();
            let jitter_range = capped_delay * self.jitter_factor;
            let jitter = rng.gen_range(-jitter_range..=jitter_range);
            (capped_delay + jitter).max(0.0)
        } else {
            capped_delay
        };
        
        Duration::from_millis(delay_with_jitter as u64)
    }
    
    /// Check if we should retry based on the error and attempt count
    pub fn should_retry(&self, error: &ProviderError, attempt: u32) -> bool {
        // Check if we've exceeded max retries
        if attempt >= self.max_retries {
            return false;
        }
        
        // Check if the error is retryable
        error.is_retryable()
    }
}

/// Result of a retry operation
#[derive(Debug, Clone)]
pub struct RetryResult<T> {
    /// The successful result (if any)
    pub result: Option<T>,
    
    /// Number of retry attempts made
    pub attempts: u32,
    
    /// Total time spent retrying
    pub total_delay_ms: u64,
    
    /// The final error (if failed)
    pub final_error: Option<ProviderError>,
    
    /// All errors encountered during retries
    pub error_history: Vec<ProviderError>,
}

/// Executor for retry operations
pub struct RetryExecutor {
    policy: RetryPolicy,
}

impl RetryExecutor {
    /// Create a new retry executor with the given policy
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy }
    }
    
    /// Execute an operation with retry logic
    pub async fn execute<F, T, Fut>(&self, mut operation: F) -> RetryResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, ProviderError>>,
    {
        let mut attempts = 0;
        let mut total_delay_ms = 0;
        let mut error_history = Vec::new();
        let start_time = std::time::Instant::now();
        
        loop {
            match operation().await {
                Ok(result) => {
                    return RetryResult {
                        result: Some(result),
                        attempts,
                        total_delay_ms,
                        final_error: None,
                        error_history,
                    };
                }
                Err(error) => {
                    error_history.push(error.clone());
                    
                    // Check if we should retry
                    if !self.policy.should_retry(&error, attempts) {
                        return RetryResult {
                            result: None,
                            attempts,
                            total_delay_ms,
                            final_error: Some(error),
                            error_history,
                        };
                    }
                    
                    // Check timeout
                    if let Some(timeout_ms) = self.policy.timeout_ms {
                        if start_time.elapsed().as_millis() > timeout_ms as u128 {
                            return RetryResult {
                                result: None,
                                attempts,
                                total_delay_ms,
                                final_error: Some(ProviderError::Timeout),
                                error_history,
                            };
                        }
                    }
                    
                    // Calculate and apply delay
                    let delay = self.policy.calculate_delay(attempts, &error);
                    total_delay_ms += delay.as_millis() as u64;
                    
                    tokio::time::sleep(delay).await;
                    attempts += 1;
                }
            }
        }
    }
}

/// Error mapper for converting provider-specific errors to common types
pub struct ErrorMapper;

impl ErrorMapper {
    /// Map an HTTP status code to a provider error
    pub fn from_status_code(status: u16, body: Option<&str>) -> ProviderError {
        match status {
            401 | 403 => ProviderError::AuthenticationError,
            429 => {
                // Try to parse retry-after from body or headers
                let retry_after = Self::parse_retry_after(body);
                ProviderError::RateLimit { retry_after }
            }
            400 => ProviderError::InvalidRequest {
                message: body.unwrap_or("Bad request").to_string(),
            },
            404 => ProviderError::ModelNotAvailable {
                model: body.unwrap_or("unknown").to_string(),
            },
            408 | 504 => ProviderError::Timeout,
            500..=599 => ProviderError::ServerError {
                status_code: status,
                message: body.unwrap_or("Internal server error").to_string(),
            },
            _ => ProviderError::Custom {
                code: status.to_string(),
                message: body.unwrap_or("Unknown error").to_string(),
            },
        }
    }
    
    /// Parse retry-after value from response body or headers
    fn parse_retry_after(body: Option<&str>) -> Option<Duration> {
        // Simple parsing - in production would parse JSON response
        if let Some(body) = body {
            if body.contains("retry_after") || body.contains("retry-after") {
                // Try to extract number of seconds
                if let Some(seconds) = Self::extract_retry_seconds(body) {
                    return Some(Duration::from_secs(seconds));
                }
            }
        }
        None
    }
    
    /// Extract retry seconds from text (simplified for MVP)
    fn extract_retry_seconds(text: &str) -> Option<u64> {
        // Look for patterns like "retry_after: 5" or "retry-after: 5"
        for part in text.split_whitespace() {
            if let Ok(num) = part.parse::<u64>() {
                if num > 0 && num < 3600 { // Reasonable retry delay
                    return Some(num);
                }
            }
        }
        None
    }
    
    /// Map provider-specific error messages to common types
    pub fn from_provider_error(provider: &str, error_msg: &str) -> ProviderError {
        let lower_msg = error_msg.to_lowercase();
        
        // Common patterns across providers
        if lower_msg.contains("rate limit") || lower_msg.contains("too many requests") {
            return ProviderError::RateLimit { retry_after: None };
        }
        
        if lower_msg.contains("timeout") || lower_msg.contains("timed out") {
            return ProviderError::Timeout;
        }
        
        if lower_msg.contains("unauthorized") || lower_msg.contains("authentication") {
            return ProviderError::AuthenticationError;
        }
        
        if lower_msg.contains("invalid request") || lower_msg.contains("bad request") {
            return ProviderError::InvalidRequest {
                message: error_msg.to_string(),
            };
        }
        
        if lower_msg.contains("model") && lower_msg.contains("not found") {
            return ProviderError::ModelNotAvailable {
                model: "unknown".to_string(),
            };
        }
        
        // Provider-specific patterns
        match provider {
            "openai" => {
                if lower_msg.contains("insufficient_quota") {
                    ProviderError::RateLimit { retry_after: None }
                } else if lower_msg.contains("server_error") {
                    ProviderError::ServerError {
                        status_code: 500,
                        message: error_msg.to_string(),
                    }
                } else {
                    ProviderError::Custom {
                        code: "openai_error".to_string(),
                        message: error_msg.to_string(),
                    }
                }
            }
            "anthropic" => {
                if lower_msg.contains("overloaded") {
                    ProviderError::ServerError {
                        status_code: 503,
                        message: error_msg.to_string(),
                    }
                } else if lower_msg.contains("invalid_api_key") {
                    ProviderError::AuthenticationError
                } else {
                    ProviderError::Custom {
                        code: "anthropic_error".to_string(),
                        message: error_msg.to_string(),
                    }
                }
            }
            _ => ProviderError::Custom {
                code: format!("{}_error", provider),
                message: error_msg.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_retry_policy_defaults() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.initial_delay_ms, 100);
        assert_eq!(policy.exponential_base, 2.0);
    }
    
    #[test]
    fn test_exponential_backoff_calculation() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            exponential_base: 2.0,
            jitter_factor: 0.0, // No jitter for predictable test
            respect_retry_after: false,
            timeout_ms: None,
        };
        
        let error = ProviderError::Timeout;
        
        // First retry: 100 * 2^0 = 100ms
        let delay0 = policy.calculate_delay(0, &error);
        assert_eq!(delay0.as_millis(), 100);
        
        // Second retry: 100 * 2^1 = 200ms
        let delay1 = policy.calculate_delay(1, &error);
        assert_eq!(delay1.as_millis(), 200);
        
        // Third retry: 100 * 2^2 = 400ms
        let delay2 = policy.calculate_delay(2, &error);
        assert_eq!(delay2.as_millis(), 400);
        
        // Fourth retry: 100 * 2^3 = 800ms (capped at max)
        let delay3 = policy.calculate_delay(3, &error);
        assert_eq!(delay3.as_millis(), 800);
        
        // Fifth retry: would be 1600ms but capped at 1000ms
        let delay4 = policy.calculate_delay(4, &error);
        assert_eq!(delay4.as_millis(), 1000);
    }
    
    #[test]
    fn test_retry_after_respected() {
        let policy = RetryPolicy {
            respect_retry_after: true,
            ..Default::default()
        };
        
        let error = ProviderError::RateLimit {
            retry_after: Some(Duration::from_secs(5)),
        };
        
        let delay = policy.calculate_delay(0, &error);
        assert_eq!(delay.as_secs(), 5);
    }
    
    #[test]
    fn test_should_retry_logic() {
        let policy = RetryPolicy::new(2);
        
        // Retryable errors should be retried
        let timeout = ProviderError::Timeout;
        assert!(policy.should_retry(&timeout, 0));
        assert!(policy.should_retry(&timeout, 1));
        assert!(!policy.should_retry(&timeout, 2)); // Exceeded max
        
        // Non-retryable errors should not be retried
        let auth_error = ProviderError::AuthenticationError;
        assert!(!policy.should_retry(&auth_error, 0));
    }
    
    #[test]
    fn test_error_mapper_status_codes() {
        assert!(matches!(
            ErrorMapper::from_status_code(401, None),
            ProviderError::AuthenticationError
        ));
        
        assert!(matches!(
            ErrorMapper::from_status_code(429, None),
            ProviderError::RateLimit { .. }
        ));
        
        assert!(matches!(
            ErrorMapper::from_status_code(500, Some("Server error")),
            ProviderError::ServerError { status_code: 500, .. }
        ));
        
        assert!(matches!(
            ErrorMapper::from_status_code(408, None),
            ProviderError::Timeout
        ));
    }
    
    #[test]
    fn test_error_mapper_provider_patterns() {
        let error = ErrorMapper::from_provider_error("openai", "Rate limit exceeded");
        assert!(matches!(error, ProviderError::RateLimit { .. }));
        
        let error = ErrorMapper::from_provider_error("anthropic", "overloaded");
        assert!(matches!(error, ProviderError::ServerError { .. }));
        
        let error = ErrorMapper::from_provider_error("openai", "Unauthorized");
        assert!(matches!(error, ProviderError::AuthenticationError));
    }
}