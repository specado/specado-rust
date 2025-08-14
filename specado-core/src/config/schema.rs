//! Configuration schema structures with serde support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::error::{ValidationError, ValidationErrorKind};

/// Root configuration structure for Specado
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpecadoConfig {
    /// Schema version (required - no default)
    pub version: String,
    
    /// List of LLM providers
    #[serde(default)]
    pub providers: Vec<Provider>,
    
    /// Routing configuration
    #[serde(default)]
    pub routing: RoutingConfig,
    
    /// Global connection settings
    #[serde(default)]
    pub connection: ConnectionConfig,
    
    /// Global defaults
    #[serde(default)]
    pub defaults: DefaultConfig,
    
    /// Custom metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// LLM Provider configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Provider {
    /// Unique provider name
    pub name: String,
    
    /// Provider type (openai, anthropic, etc.)
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    
    /// API key (supports environment variable interpolation)
    pub api_key: String,
    
    /// Base URL for the provider API
    pub base_url: String,
    
    /// Available models for this provider
    #[serde(default)]
    pub models: Vec<Model>,
    
    /// Provider-specific rate limiting
    #[serde(default)]
    pub rate_limit: Option<RateLimitConfig>,
    
    /// Provider-specific retry policy
    #[serde(default)]
    pub retry_policy: Option<RetryPolicy>,
    
    /// Whether this provider is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Priority for routing (higher = preferred)
    #[serde(default = "default_priority")]
    pub priority: u32,
}

/// Supported provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Google,
    Azure,
    Cohere,
    Custom,
}

/// Model configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    /// Model identifier (e.g., "gpt-4", "claude-3")
    pub id: String,
    
    /// Maximum context tokens
    pub max_tokens: usize,
    
    /// Maximum output tokens
    #[serde(default)]
    pub max_output_tokens: Option<usize>,
    
    /// Default temperature for this model
    #[serde(default = "default_temperature")]
    pub default_temperature: f32,
    
    /// Whether this model supports streaming
    #[serde(default = "default_true")]
    pub supports_streaming: bool,
    
    /// Whether this model supports function calling
    #[serde(default)]
    pub supports_functions: bool,
    
    /// Cost per 1K input tokens (in USD)
    #[serde(default)]
    pub cost_per_1k_input: Option<f64>,
    
    /// Cost per 1K output tokens (in USD)
    #[serde(default)]
    pub cost_per_1k_output: Option<f64>,
}

/// Routing configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingConfig {
    /// Routing strategy
    #[serde(default)]
    pub strategy: RoutingStrategy,
    
    /// Fallback configuration
    #[serde(default)]
    pub fallback: FallbackPolicy,
    
    /// Load balancing weights (provider name -> weight)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub weights: HashMap<String, f32>,
}

/// Routing strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStrategy {
    /// Round-robin between providers
    RoundRobin,
    /// Random selection
    Random,
    /// Weighted random based on weights
    Weighted,
    /// Always use the highest priority provider
    Priority,
    /// Use the provider with lowest latency
    LowestLatency,
    /// Use the most cost-effective provider
    CostOptimized,
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        Self::Priority
    }
}

/// Fallback policy configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FallbackPolicy {
    /// Enable fallback to other providers
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Maximum number of retries across providers
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    
    /// Timeout before fallback (in milliseconds)
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    
    /// Whether to fallback on rate limit errors
    #[serde(default = "default_true")]
    pub on_rate_limit: bool,
    
    /// Whether to fallback on timeout errors
    #[serde(default = "default_true")]
    pub on_timeout: bool,
}

impl Default for FallbackPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            max_retries: 3,
            timeout_ms: 30000,
            on_rate_limit: true,
            on_timeout: true,
        }
    }
}

/// Retry policy configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RetryPolicy {
    /// Maximum number of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    
    /// Initial retry delay in milliseconds
    #[serde(default = "default_initial_delay")]
    pub initial_delay_ms: u64,
    
    /// Maximum retry delay in milliseconds
    #[serde(default = "default_max_delay")]
    pub max_delay_ms: u64,
    
    /// Backoff multiplier
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f32,
    
    /// Add jitter to retry delays
    #[serde(default = "default_true")]
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 60000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RateLimitConfig {
    /// Requests per minute limit
    #[serde(default)]
    pub requests_per_minute: Option<u32>,
    
    /// Tokens per minute limit
    #[serde(default)]
    pub tokens_per_minute: Option<u32>,
    
    /// Concurrent request limit
    #[serde(default)]
    pub max_concurrent: Option<u32>,
}

/// Connection configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionConfig {
    /// Connection timeout in milliseconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_ms: u64,
    
    /// Request timeout in milliseconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout_ms: u64,
    
    /// Maximum idle connections per host
    #[serde(default = "default_max_idle")]
    pub max_idle_per_host: usize,
    
    /// Keep-alive timeout in seconds
    #[serde(default = "default_keepalive")]
    pub keepalive_secs: u64,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            connect_timeout_ms: 10000,
            request_timeout_ms: 60000,
            max_idle_per_host: 10,
            keepalive_secs: 90,
        }
    }
}

/// Default configuration values
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultConfig {
    /// Default temperature for all models
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    
    /// Default max tokens for responses
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
    
    /// Default top_p value
    #[serde(default)]
    pub top_p: Option<f32>,
    
    /// Default frequency penalty
    #[serde(default)]
    pub frequency_penalty: Option<f32>,
    
    /// Default presence penalty
    #[serde(default)]
    pub presence_penalty: Option<f32>,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 2048,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        }
    }
}

// Default value functions for serde
fn default_true() -> bool { true }
fn default_priority() -> u32 { 100 }
fn default_temperature() -> f32 { 0.7 }
fn default_max_retries() -> u32 { 3 }
fn default_timeout() -> u64 { 30000 }
fn default_initial_delay() -> u64 { 1000 }
fn default_max_delay() -> u64 { 60000 }
fn default_backoff_multiplier() -> f32 { 2.0 }
fn default_connect_timeout() -> u64 { 10000 }
fn default_request_timeout() -> u64 { 60000 }
fn default_max_idle() -> usize { 10 }
fn default_keepalive() -> u64 { 90 }
fn default_max_tokens() -> usize { 2048 }

impl SpecadoConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate version
        if self.version.is_empty() {
            return Err(ValidationError::required("version"));
        }
        
        // Currently support only version 0.1
        if self.version != "0.1" {
            return Err(ValidationError::new(
                "version",
                ValidationErrorKind::InvalidVersion {
                    expected: "0.1".to_string(),
                    actual: self.version.clone(),
                },
            ));
        }
        
        // Validate providers
        if self.providers.is_empty() {
            return Err(ValidationError::required("providers")
                .with_context("At least one provider must be configured"));
        }
        
        // Check for duplicate provider names
        let mut seen_names = std::collections::HashSet::new();
        for (i, provider) in self.providers.iter().enumerate() {
            if !seen_names.insert(&provider.name) {
                return Err(ValidationError::new(
                    format!("providers[{}].name", i),
                    ValidationErrorKind::DuplicateValue {
                        value: provider.name.clone(),
                    },
                ));
            }
            
            // Validate provider
            provider.validate(&format!("providers[{}]", i))?;
        }
        
        // Validate routing weights reference existing providers
        for (name, weight) in &self.routing.weights {
            if !self.providers.iter().any(|p| &p.name == name) {
                return Err(ValidationError::new(
                    format!("routing.weights.{}", name),
                    ValidationErrorKind::InvalidValue {
                        expected: "existing provider name".to_string(),
                        actual: name.clone(),
                    },
                ));
            }
            
            if *weight <= 0.0 {
                return Err(ValidationError::out_of_range(
                    format!("routing.weights.{}", name),
                    "Weight must be positive",
                ));
            }
        }
        
        Ok(())
    }
}

impl Provider {
    /// Validate provider configuration
    pub fn validate(&self, path: &str) -> Result<(), ValidationError> {
        // Validate name
        if self.name.is_empty() {
            return Err(ValidationError::required(format!("{}.name", path)));
        }
        
        // Validate API key (can be env var placeholder)
        if self.api_key.is_empty() {
            return Err(ValidationError::required(format!("{}.api_key", path)));
        }
        
        // Validate base URL
        if self.base_url.is_empty() {
            return Err(ValidationError::required(format!("{}.base_url", path)));
        }
        
        // Proper URL validation using url crate
        match url::Url::parse(&self.base_url) {
            Ok(url) => {
                // Ensure it's HTTP or HTTPS
                if url.scheme() != "http" && url.scheme() != "https" {
                    return Err(ValidationError::new(
                        format!("{}.base_url", path),
                        ValidationErrorKind::InvalidUrl {
                            message: format!("URL scheme must be http or https, got: {}", url.scheme()),
                        },
                    ));
                }
            }
            Err(e) => {
                return Err(ValidationError::new(
                    format!("{}.base_url", path),
                    ValidationErrorKind::InvalidUrl {
                        message: e.to_string(),
                    },
                ));
            }
        }
        
        // Validate models
        let mut seen_model_ids = std::collections::HashSet::new();
        for (i, model) in self.models.iter().enumerate() {
            let model_path = format!("{}.models[{}]", path, i);
            
            if model.id.is_empty() {
                return Err(ValidationError::required(format!("{}.id", model_path)));
            }
            
            if !seen_model_ids.insert(&model.id) {
                return Err(ValidationError::new(
                    format!("{}.id", model_path),
                    ValidationErrorKind::DuplicateValue {
                        value: model.id.clone(),
                    },
                ));
            }
            
            // Validate model parameters
            model.validate(&model_path)?;
        }
        
        // Validate rate limit config if present
        if let Some(rate_limit) = &self.rate_limit {
            rate_limit.validate(&format!("{}.rate_limit", path))?;
        }
        
        // Validate retry policy if present
        if let Some(retry) = &self.retry_policy {
            retry.validate(&format!("{}.retry_policy", path))?;
        }
        
        Ok(())
    }
}

impl Model {
    /// Validate model configuration
    pub fn validate(&self, path: &str) -> Result<(), ValidationError> {
        // Validate max_tokens
        if self.max_tokens == 0 {
            return Err(ValidationError::out_of_range(
                format!("{}.max_tokens", path),
                "Must be greater than 0",
            ));
        }
        
        // Validate max_output_tokens if present
        if let Some(max_output) = self.max_output_tokens {
            if max_output == 0 {
                return Err(ValidationError::out_of_range(
                    format!("{}.max_output_tokens", path),
                    "Must be greater than 0",
                ));
            }
            
            if max_output > self.max_tokens {
                return Err(ValidationError::new(
                    format!("{}.max_output_tokens", path),
                    ValidationErrorKind::Incompatible {
                        message: "Cannot exceed max_tokens".to_string(),
                    },
                ));
            }
        }
        
        // Validate temperature
        if !(0.0..=2.0).contains(&self.default_temperature) {
            return Err(ValidationError::out_of_range(
                format!("{}.default_temperature", path),
                "Must be between 0.0 and 2.0",
            ));
        }
        
        // Validate costs if present
        if let Some(cost) = self.cost_per_1k_input {
            if cost < 0.0 {
                return Err(ValidationError::out_of_range(
                    format!("{}.cost_per_1k_input", path),
                    "Must be non-negative",
                ));
            }
        }
        
        if let Some(cost) = self.cost_per_1k_output {
            if cost < 0.0 {
                return Err(ValidationError::out_of_range(
                    format!("{}.cost_per_1k_output", path),
                    "Must be non-negative",
                ));
            }
        }
        
        Ok(())
    }
}

impl RateLimitConfig {
    /// Validate rate limit configuration
    pub fn validate(&self, path: &str) -> Result<(), ValidationError> {
        // At least one limit should be specified
        if self.requests_per_minute.is_none() 
            && self.tokens_per_minute.is_none() 
            && self.max_concurrent.is_none() {
            return Err(ValidationError::new(
                path,
                ValidationErrorKind::Custom {
                    message: "At least one rate limit must be specified".to_string(),
                },
            ));
        }
        
        Ok(())
    }
}

impl RetryPolicy {
    /// Validate retry policy
    pub fn validate(&self, path: &str) -> Result<(), ValidationError> {
        if self.initial_delay_ms == 0 {
            return Err(ValidationError::out_of_range(
                format!("{}.initial_delay_ms", path),
                "Must be greater than 0",
            ));
        }
        
        if self.max_delay_ms < self.initial_delay_ms {
            return Err(ValidationError::new(
                format!("{}.max_delay_ms", path),
                ValidationErrorKind::Incompatible {
                    message: "Must be >= initial_delay_ms".to_string(),
                },
            ));
        }
        
        if self.backoff_multiplier <= 1.0 {
            return Err(ValidationError::out_of_range(
                format!("{}.backoff_multiplier", path),
                "Must be greater than 1.0",
            ));
        }
        
        Ok(())
    }
}