//! Routing strategies for provider selection and fallback
//!
//! This module implements routing logic that enables automatic fallback
//! to alternative providers when the primary provider fails with retryable errors.

use crate::protocol::types::{ChatRequest, ChatResponse};
use crate::providers::adapter::Provider;
use crate::providers::transform::TransformResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

/// Errors that can occur during provider operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderError {
    /// Rate limit exceeded, retry after specified duration
    RateLimit { retry_after: Option<Duration> },

    /// Request timeout
    Timeout,

    /// Temporary server error (5xx)
    ServerError { status_code: u16, message: String },

    /// Invalid request that should not be retried (4xx)
    InvalidRequest { message: String },

    /// Authentication failure
    AuthenticationError,

    /// Model not available or unsupported
    ModelNotAvailable { model: String },

    /// Generic network error
    NetworkError { message: String },

    /// Provider-specific error
    Custom { code: String, message: String },
}

impl ProviderError {
    /// Determine if this error is retryable with a different provider
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::RateLimit { .. } => true,
            Self::Timeout => true,
            Self::ServerError { .. } => true,
            Self::NetworkError { .. } => true,
            Self::ModelNotAvailable { .. } => true,
            Self::InvalidRequest { .. } => false,
            Self::AuthenticationError => false,
            Self::Custom { .. } => false, // Conservative: don't retry custom errors
        }
    }

    /// Get suggested retry delay for this error
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            Self::RateLimit { retry_after } => *retry_after,
            Self::Timeout => Some(Duration::from_secs(1)),
            Self::ServerError { .. } => Some(Duration::from_secs(2)),
            Self::NetworkError { .. } => Some(Duration::from_secs(1)),
            _ => None,
        }
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RateLimit { retry_after } => {
                if let Some(duration) = retry_after {
                    write!(f, "Rate limit exceeded, retry after {:?}", duration)
                } else {
                    write!(f, "Rate limit exceeded")
                }
            }
            Self::Timeout => write!(f, "Request timeout"),
            Self::ServerError {
                status_code,
                message,
            } => {
                write!(f, "Server error ({}): {}", status_code, message)
            }
            Self::InvalidRequest { message } => write!(f, "Invalid request: {}", message),
            Self::AuthenticationError => write!(f, "Authentication failed"),
            Self::ModelNotAvailable { model } => write!(f, "Model '{}' not available", model),
            Self::NetworkError { message } => write!(f, "Network error: {}", message),
            Self::Custom { code, message } => write!(f, "Error [{}]: {}", code, message),
        }
    }
}

impl std::error::Error for ProviderError {}

/// Result of a routing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResult {
    /// The successful response (if any)
    pub response: Option<ChatResponse>,

    /// The transform result from the successful provider
    pub transform_result: Option<TransformResult>,

    /// Which provider ultimately succeeded
    pub provider_used: String,

    /// Whether a fallback was used
    pub used_fallback: bool,

    /// Number of attempts made
    pub attempts: usize,

    /// Errors encountered per provider
    pub provider_errors: HashMap<String, String>,

    /// Additional routing metadata
    pub metadata: HashMap<String, Value>,
}

/// Trait for routing strategies
#[async_trait]
pub trait RoutingStrategy: Send + Sync {
    /// Select a provider and execute the request
    async fn route(&self, request: ChatRequest) -> Result<RoutingResult, ProviderError>;

    /// Get the name of this routing strategy
    fn name(&self) -> &str;

    /// Get available providers in order of preference
    fn providers(&self) -> Vec<String>;
}

/// Primary with fallbacks routing strategy
pub struct PrimaryWithFallbacks {
    /// Primary provider to try first
    primary: Box<dyn Provider>,

    /// Ordered list of fallback providers
    fallbacks: Vec<Box<dyn Provider>>,

    /// Whether to add routing metadata to responses
    track_metadata: bool,

    /// Retry policy for transient errors
    retry_policy: Option<crate::providers::retry::RetryPolicy>,

    /// HTTP client for making requests
    http_client: crate::http::client::HttpClient,
}

impl PrimaryWithFallbacks {
    /// Create a new primary with fallbacks router
    pub fn new(primary: Box<dyn Provider>, fallbacks: Vec<Box<dyn Provider>>) -> Self {
        let http_client =
            crate::http::client::HttpClient::new().expect("Failed to create HTTP client");

        Self {
            primary,
            fallbacks,
            track_metadata: true,
            retry_policy: Some(crate::providers::retry::RetryPolicy::default()),
            http_client,
        }
    }

    /// Set whether to track routing metadata
    pub fn with_metadata_tracking(mut self, enabled: bool) -> Self {
        self.track_metadata = enabled;
        self
    }

    /// Set the retry policy for transient errors
    pub fn with_retry_policy(mut self, policy: crate::providers::retry::RetryPolicy) -> Self {
        self.retry_policy = Some(policy);
        self
    }

    /// Disable retry logic
    pub fn without_retry(mut self) -> Self {
        self.retry_policy = None;
        self
    }

    /// Execute request against a specific provider with retry logic (simplified for MVP)
    /// Returns: (response, transform_result, errors, provider_tried_count, delay_ms)
    /// Note: provider_tried_count is always 1 (we tried this provider once)
    async fn execute_with_retry(
        &self,
        request: &ChatRequest,
        provider: &Box<dyn Provider>,
    ) -> (
        Option<ChatResponse>,
        Option<TransformResult>,
        Vec<ProviderError>,
        u32,
        u64,
    ) {
        let mut attempts = 0;
        let mut total_delay_ms = 0;
        let mut error_history = Vec::new();

        // Determine max attempts based on retry policy
        let max_attempts = if let Some(ref policy) = self.retry_policy {
            policy.max_retries + 1 // Initial attempt + retries
        } else {
            1 // No retry, just one attempt
        };

        while attempts < max_attempts {
            match self.execute_with_provider(request, provider).await {
                Ok((response, transform_result)) => {
                    return (
                        Some(response),
                        Some(transform_result),
                        error_history,
                        1,
                        total_delay_ms,
                    );
                }
                Err(error) => {
                    error_history.push(error.clone());
                    attempts += 1;

                    // Check if we should retry
                    if let Some(ref policy) = self.retry_policy {
                        if attempts < max_attempts && error.is_retryable() {
                            // Calculate delay
                            let delay = policy.calculate_delay(attempts - 1, &error);
                            total_delay_ms += delay.as_millis() as u64;

                            // Sleep for the calculated delay
                            tokio::time::sleep(delay).await;
                        } else {
                            break; // Don't retry non-retryable errors
                        }
                    }
                }
            }
        }

        (None, None, error_history, 1, total_delay_ms)
    }

    /// Execute request against a specific provider (single attempt)
    async fn execute_with_provider(
        &self,
        request: &ChatRequest,
        provider: &Box<dyn Provider>,
    ) -> Result<(ChatResponse, TransformResult), ProviderError> {
        use crate::http::{CallKind, HttpExecutor, RequestOptions};

        // Create request options with a new request ID
        let options = RequestOptions::new(CallKind::Chat);

        // Execute the HTTP request
        let response = self
            .http_client
            .execute_json(provider.as_ref(), request.clone(), options)
            .await?;

        // Create a TransformResult for tracking
        // Note: In a full implementation, we'd track actual transformations done by the provider
        let transform_result = TransformResult {
            transformed: request.clone(),
            lossy: false,
            reasons: vec![],
        };

        Ok((response, transform_result))
    }
}

#[async_trait]
impl RoutingStrategy for PrimaryWithFallbacks {
    async fn route(&self, request: ChatRequest) -> Result<RoutingResult, ProviderError> {
        let mut result = RoutingResult {
            response: None,
            transform_result: None,
            provider_used: String::new(),
            used_fallback: false,
            attempts: 0,
            provider_errors: HashMap::new(),
            metadata: HashMap::new(),
        };

        // Try primary provider first with retry logic
        let primary_name = self.primary.name().to_string();
        let (response_opt, transform_opt, errors, attempts, delay_ms) =
            self.execute_with_retry(&request, &self.primary).await;

        result.attempts += attempts as usize;

        if let Some(response) = response_opt {
            // Primary succeeded
            result.response = Some(response);
            result.transform_result = transform_opt.clone();
            result.provider_used = primary_name.clone();

            if self.track_metadata {
                result
                    .metadata
                    .insert("primary_provider".to_string(), json!(primary_name));
                result
                    .metadata
                    .insert("fallback_used".to_string(), json!(false));
                result
                    .metadata
                    .insert("attempts".to_string(), json!(attempts));
                result
                    .metadata
                    .insert("retry_delay_ms".to_string(), json!(delay_ms));

                // Add transform metadata if lossy
                if let Some(ref transform_result) = result.transform_result {
                    if transform_result.lossy {
                        result
                            .metadata
                            .insert("transformation_lossy".to_string(), json!(true));
                        result
                            .metadata
                            .insert("lossy_reasons".to_string(), json!(transform_result.reasons));
                    }
                }
            }

            return Ok(result);
        } else {
            // Primary failed after retries, record errors
            if let Some(last_error) = errors.last() {
                result
                    .provider_errors
                    .insert(primary_name.clone(), last_error.to_string());

                // Check if error is retryable for fallback
                if !last_error.is_retryable() {
                    return Err(last_error.clone());
                }
            }

            // Try fallbacks
            for (idx, fallback) in self.fallbacks.iter().enumerate() {
                let fallback_name = fallback.name().to_string();
                let (response_opt, transform_opt, fb_errors, fb_attempts, _fb_delay_ms) =
                    self.execute_with_retry(&request, fallback).await;

                result.attempts += fb_attempts as usize;

                if let Some(response) = response_opt {
                    // Fallback succeeded
                    result.response = Some(response);
                    result.transform_result = transform_opt.clone();
                    result.provider_used = fallback_name.clone();
                    result.used_fallback = true;

                    if self.track_metadata {
                        result
                            .metadata
                            .insert("primary_provider".to_string(), json!(primary_name));
                        result
                            .metadata
                            .insert("fallback_provider".to_string(), json!(fallback_name));
                        result
                            .metadata
                            .insert("fallback_used".to_string(), json!(true));
                        result
                            .metadata
                            .insert("fallback_index".to_string(), json!(idx));
                        result
                            .metadata
                            .insert("attempts".to_string(), json!(result.attempts));
                        result
                            .metadata
                            .insert("provider_errors".to_string(), json!(result.provider_errors));

                        // Add transform metadata if lossy
                        if let Some(ref transform_result) = result.transform_result {
                            if transform_result.lossy {
                                result
                                    .metadata
                                    .insert("transformation_lossy".to_string(), json!(true));
                                result.metadata.insert(
                                    "lossy_reasons".to_string(),
                                    json!(transform_result.reasons),
                                );
                            }
                        }
                    }

                    return Ok(result);
                } else {
                    // Fallback failed after retries, record errors
                    if let Some(last_fb_error) = fb_errors.last() {
                        result
                            .provider_errors
                            .insert(fallback_name, last_fb_error.to_string());
                    }
                    // Continue to next fallback
                }
            }

            // All providers failed
            Err(ProviderError::Custom {
                code: "ALL_PROVIDERS_FAILED".to_string(),
                message: format!(
                    "All {} providers failed. Errors: {:?}",
                    result.attempts, result.provider_errors
                ),
            })
        }
    }

    fn name(&self) -> &str {
        "primary_with_fallbacks"
    }

    fn providers(&self) -> Vec<String> {
        let mut providers = vec![self.primary.name().to_string()];
        for fallback in &self.fallbacks {
            providers.push(fallback.name().to_string());
        }
        providers
    }
}

/// Builder for creating routing strategies
pub struct RoutingBuilder {
    primary: Option<Box<dyn Provider>>,
    fallbacks: Vec<Box<dyn Provider>>,
}

impl RoutingBuilder {
    /// Create a new routing builder
    pub fn new() -> Self {
        Self {
            primary: None,
            fallbacks: Vec::new(),
        }
    }

    /// Set the primary provider
    pub fn primary(mut self, provider: Box<dyn Provider>) -> Self {
        self.primary = Some(provider);
        self
    }

    /// Add a fallback provider
    pub fn fallback(mut self, provider: Box<dyn Provider>) -> Self {
        self.fallbacks.push(provider);
        self
    }

    /// Build the routing strategy
    pub fn build(self) -> Result<PrimaryWithFallbacks, String> {
        let primary = self
            .primary
            .ok_or_else(|| "Primary provider required".to_string())?;

        Ok(PrimaryWithFallbacks::new(primary, self.fallbacks))
    }
}

impl Default for RoutingBuilder {
    fn default() -> Self {
        Self::new()
    }
}
