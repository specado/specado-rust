//! Routing strategies for provider selection and fallback
//!
//! This module implements routing logic that enables automatic fallback
//! to alternative providers when the primary provider fails with retryable errors.

use crate::protocol::types::{ChatRequest, ChatResponse};
use crate::providers::adapter::Provider;
use crate::providers::transform::TransformResult;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use async_trait::async_trait;

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
            Self::ServerError { status_code, message } => {
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
}

impl PrimaryWithFallbacks {
    /// Create a new primary with fallbacks router
    pub fn new(primary: Box<dyn Provider>, fallbacks: Vec<Box<dyn Provider>>) -> Self {
        Self {
            primary,
            fallbacks,
            track_metadata: true,
            retry_policy: Some(crate::providers::retry::RetryPolicy::default()),
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
    /// Returns: (result, errors, provider_tried_count, delay_ms)
    /// Note: provider_tried_count is always 1 (we tried this provider once)
    async fn execute_with_retry(
        &self,
        request: &ChatRequest,
        provider: &Box<dyn Provider>,
    ) -> (Option<TransformResult>, Vec<ProviderError>, u32, u64) {
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
            match self.execute_with_provider(request, provider) {
                Ok(result) => {
                    return (Some(result), error_history, 1, total_delay_ms);
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
        
        (None, error_history, 1, total_delay_ms)
    }
    
    /// Execute request against a specific provider (single attempt)
    fn execute_with_provider(
        &self,
        request: &ChatRequest,
        provider: &Box<dyn Provider>,
    ) -> Result<TransformResult, ProviderError> {
        // Use the transformation engine to prepare the request
        use crate::providers::TransformationEngine;
        use crate::providers::adapter::ProviderType;
        
        // For MVP, we assume source is OpenAI format
        let source = ProviderType::OpenAI.create_provider();
        
        // Clone the provider box to create a new owned box
        // This is a workaround since we can't clone trait objects directly
        // In production, we'd use Arc<dyn Provider> for shared ownership
        let target = match provider.name() {
            "openai" => ProviderType::OpenAI.create_provider(),
            "anthropic" => ProviderType::Anthropic.create_provider(),
            _ => ProviderType::OpenAI.create_provider(),
        };
        
        // Create transformation engine
        let engine = TransformationEngine::new(source, target);
        
        // Transform the request
        let transform_result = engine.transform_request(request.clone());
        
        // TODO: Execute actual HTTP request and populate response field
        // For MVP, we simulate success if transformation worked
        // In Week 3, this will make real HTTP calls and return ChatResponse
        
        // Simulate some failure scenarios for testing
        // Only fail on the first provider to test fallback behavior
        if provider.name() == "openai" {
            if request.model.contains("timeout-test") {
                return Err(ProviderError::Timeout);
            }
            
            if request.model.contains("rate-limit-test") {
                return Err(ProviderError::RateLimit {
                    retry_after: Some(Duration::from_secs(5)),
                });
            }
            
            if request.model.contains("server-error-test") {
                return Err(ProviderError::ServerError {
                    status_code: 503,
                    message: "Service temporarily unavailable".to_string(),
                });
            }
        }
        
        // Auth errors fail on all providers
        if request.model.contains("auth-error-test") {
            return Err(ProviderError::AuthenticationError);
        }
        
        Ok(transform_result)
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
        let (transform_opt, errors, attempts, delay_ms) = self.execute_with_retry(&request, &self.primary).await;
        
        result.attempts += attempts as usize;
        
        if let Some(transform_result) = transform_opt {
                // Primary succeeded
                result.transform_result = Some(transform_result.clone());
                result.provider_used = primary_name.clone();
                
                if self.track_metadata {
                    result.metadata.insert("primary_provider".to_string(), json!(primary_name));
                    result.metadata.insert("fallback_used".to_string(), json!(false));
                    result.metadata.insert("attempts".to_string(), json!(attempts));
                    result.metadata.insert("retry_delay_ms".to_string(), json!(delay_ms));
                    
                    // Add transform metadata if lossy
                    if transform_result.lossy {
                        result.metadata.insert("transformation_lossy".to_string(), json!(true));
                        result.metadata.insert(
                            "lossy_reasons".to_string(),
                            json!(transform_result.reasons),
                        );
                    }
                }
                
                return Ok(result);
            } else {
                // Primary failed after retries, record errors
                if let Some(last_error) = errors.last() {
                    result.provider_errors.insert(primary_name.clone(), last_error.to_string());
                    
                    // Check if error is retryable for fallback
                    if !last_error.is_retryable() {
                        return Err(last_error.clone());
                    }
                }
                
                // Try fallbacks
                for (idx, fallback) in self.fallbacks.iter().enumerate() {
                    let fallback_name = fallback.name().to_string();
                    let (transform_opt, fb_errors, fb_attempts, _fb_delay_ms) = self.execute_with_retry(&request, fallback).await;
                    
                    result.attempts += fb_attempts as usize;
                    
                    if let Some(transform_result) = transform_opt {
                            // Fallback succeeded
                            result.transform_result = Some(transform_result.clone());
                            result.provider_used = fallback_name.clone();
                            result.used_fallback = true;
                            
                            if self.track_metadata {
                                result.metadata.insert("primary_provider".to_string(), json!(primary_name));
                                result.metadata.insert("fallback_provider".to_string(), json!(fallback_name));
                                result.metadata.insert("fallback_used".to_string(), json!(true));
                                result.metadata.insert("fallback_index".to_string(), json!(idx));
                                result.metadata.insert("attempts".to_string(), json!(result.attempts));
                                result.metadata.insert(
                                    "provider_errors".to_string(),
                                    json!(result.provider_errors),
                                );
                                
                                // Add transform metadata if lossy
                                if transform_result.lossy {
                                    result.metadata.insert("transformation_lossy".to_string(), json!(true));
                                    result.metadata.insert(
                                        "lossy_reasons".to_string(),
                                        json!(transform_result.reasons),
                                    );
                                }
                            }
                            
                            return Ok(result);
                        } else {
                            // Fallback failed after retries, record errors
                            if let Some(last_fb_error) = fb_errors.last() {
                                result.provider_errors.insert(fallback_name, last_fb_error.to_string());
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
        let primary = self.primary.ok_or_else(|| "Primary provider required".to_string())?;
        
        Ok(PrimaryWithFallbacks::new(primary, self.fallbacks))
    }
}

impl Default for RoutingBuilder {
    fn default() -> Self {
        Self::new()
    }
}