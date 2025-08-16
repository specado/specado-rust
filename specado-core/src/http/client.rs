//! HTTP client implementation using reqwest

use crate::http::{CallKind, HttpExecutor, RequestOptions, StreamDelta};
use crate::protocol::types::{ChatRequest, ChatResponse};
use crate::providers::adapter::Provider;
use crate::providers::routing::ProviderError;
use async_trait::async_trait;
use reqwest::{Client, ClientBuilder, Response};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Maximum response size (10MB for MVP)
const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024;

/// Default user agent
const USER_AGENT: &str = "specado/0.1.0";

/// Shared HTTP client with connection pooling
#[derive(Clone)]
pub struct HttpClient {
    /// The underlying reqwest client
    client: Arc<Client>,
    
    /// Maximum response size to prevent OOM
    max_response_size: usize,
}

impl HttpClient {
    /// Create a new HTTP client with default settings
    pub fn new() -> Result<Self, String> {
        let client = ClientBuilder::new()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .user_agent(USER_AGENT)
            .gzip(true)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        
        Ok(Self {
            client: Arc::new(client),
            max_response_size: MAX_RESPONSE_SIZE,
        })
    }
    
    /// Create a new HTTP client with custom configuration
    pub fn with_config(
        connect_timeout: Duration,
        request_timeout: Duration,
        max_idle_per_host: usize,
    ) -> Result<Self, String> {
        let client = ClientBuilder::new()
            .pool_max_idle_per_host(max_idle_per_host)
            .pool_idle_timeout(Duration::from_secs(90))
            .connect_timeout(connect_timeout)
            .timeout(request_timeout)
            .user_agent(USER_AGENT)
            .gzip(true)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        
        Ok(Self {
            client: Arc::new(client),
            max_response_size: MAX_RESPONSE_SIZE,
        })
    }
    
    /// Build the full URL for a provider and call kind
    fn build_url(&self, provider: &dyn Provider, call_kind: CallKind) -> String {
        format!("{}{}", provider.base_url(), provider.endpoint(call_kind))
    }
    
    /// Get API key from environment (temporary for MVP)
    fn get_api_key(&self, provider: &dyn Provider) -> Result<String, ProviderError> {
        // For MVP, we'll use environment variables
        // This will be replaced with proper config in Phase 2
        let env_var = match provider.name() {
            "openai" => "OPENAI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            _ => return Err(ProviderError::Custom {
                code: "UNKNOWN_PROVIDER".to_string(),
                message: format!("Unknown provider: {}", provider.name()),
            }),
        };
        
        std::env::var(env_var).map_err(|_| ProviderError::AuthenticationError)
    }
    
    /// Validate response content type
    fn validate_content_type(response: &Response) -> Result<(), ProviderError> {
        if let Some(content_type) = response.headers().get("content-type") {
            let content_type_str = content_type
                .to_str()
                .unwrap_or("")
                .to_lowercase();
            
            if !content_type_str.contains("application/json") {
                return Err(ProviderError::Custom {
                    code: "INVALID_CONTENT_TYPE".to_string(),
                    message: format!("Expected application/json, got: {}", content_type_str),
                });
            }
        }
        
        Ok(())
    }
    
    /// Check response size to prevent OOM
    fn check_content_length(&self, response: &Response) -> Result<(), ProviderError> {
        if let Some(content_length) = response.content_length() {
            if content_length as usize > self.max_response_size {
                return Err(ProviderError::Custom {
                    code: "RESPONSE_TOO_LARGE".to_string(),
                    message: format!(
                        "Response size {} exceeds maximum {}",
                        content_length, self.max_response_size
                    ),
                });
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl HttpExecutor for HttpClient {
    async fn execute_json(
        &self,
        provider: &dyn Provider,
        request: ChatRequest,
        options: RequestOptions,
    ) -> Result<ChatResponse, ProviderError> {
        let request_id = options.request_id;
        
        info!(
            "Executing HTTP request to {} [request_id: {}]",
            provider.name(),
            request_id
        );
        
        // Get API key
        let api_key = self.get_api_key(provider)?;
        
        // Build URL
        let url = self.build_url(provider, options.call_kind);
        debug!("Request URL: {}", url);
        
        // Transform request to provider format
        let transformed_request = provider.transform_request(request);
        
        // Serialize request body
        let body = serde_json::to_value(&transformed_request)
            .map_err(|e| ProviderError::Custom {
                code: "SERIALIZATION_ERROR".to_string(),
                message: format!("Failed to serialize request: {} [request_id: {}]", e, request_id),
            })?;
        
        // Build HTTP request
        let mut req_builder = self.client
            .post(&url)
            .timeout(options.timeout)
            .json(&body);
        
        // Add headers
        for (key, value) in provider.headers(&api_key) {
            req_builder = req_builder.header(key, value);
        }
        
        // Add request ID header for correlation
        req_builder = req_builder.header("X-Request-ID", request_id.to_string());
        
        // Add idempotency key if provided
        if let Some(ref idempotency_key) = options.idempotency_key {
            req_builder = req_builder.header("Idempotency-Key", idempotency_key);
        }
        
        // Execute request
        let response = req_builder
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    warn!("Request timeout for {} [request_id: {}]", provider.name(), request_id);
                    ProviderError::Timeout
                } else if e.is_connect() {
                    error!("Connection error for {} [request_id: {}]: {}", provider.name(), request_id, e);
                    ProviderError::NetworkError {
                        message: format!("Connection failed: {} [request_id: {}]", e, request_id),
                    }
                } else {
                    error!("Request error for {} [request_id: {}]: {}", provider.name(), request_id, e);
                    ProviderError::NetworkError {
                        message: format!("{} [request_id: {}]", e, request_id),
                    }
                }
            })?;
        
        let status = response.status();
        debug!("Response status: {} [request_id: {}]", status, request_id);
        
        // Check for non-success status codes
        if !status.is_success() {
            // Capture headers for retry-after parsing
            let headers = response.headers().clone();
            
            // Try to get response body for error details
            let body = response
                .text()
                .await
                .ok();
            
            warn!(
                "Request failed with status {} for {} [request_id: {}]",
                status, provider.name(), request_id
            );
            
            return Err(crate::http::error::map_http_error(status, Some(&headers), body, request_id));
        }
        
        // Validate content type
        Self::validate_content_type(&response)?;
        
        // Check content length
        self.check_content_length(&response)?;
        
        // Parse response body
        let response_text = response
            .text()
            .await
            .map_err(|e| ProviderError::NetworkError {
                message: format!("Failed to read response body: {} [request_id: {}]", e, request_id),
            })?;
        
        // Check response size after reading
        if response_text.len() > self.max_response_size {
            return Err(ProviderError::Custom {
                code: "RESPONSE_TOO_LARGE".to_string(),
                message: format!(
                    "Response size {} exceeds maximum {} [request_id: {}]",
                    response_text.len(), self.max_response_size, request_id
                ),
            });
        }
        
        // Parse JSON response
        let response_json: ChatResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                error!(
                    "Failed to parse response from {} [request_id: {}]: {}",
                    provider.name(), request_id, e
                );
                ProviderError::Custom {
                    code: "PARSE_ERROR".to_string(),
                    message: format!("Invalid response format: {} [request_id: {}]", e, request_id),
                }
            })?;
        
        // Transform response to canonical format
        let canonical_response = provider.transform_response(response_json);
        
        info!(
            "Request completed successfully for {} [request_id: {}]",
            provider.name(), request_id
        );
        
        Ok(canonical_response)
    }
    
    async fn execute_stream(
        &self,
        provider: &dyn Provider,
        request: ChatRequest,
        options: RequestOptions,
    ) -> Result<Vec<StreamDelta>, ProviderError> {
        // Phase 2 implementation stub
        // For now, return error indicating streaming not yet implemented
        let _ = (provider, request);
        
        Err(ProviderError::Custom {
            code: "STREAMING_NOT_IMPLEMENTED".to_string(),
            message: format!(
                "Streaming support will be implemented in Phase 2 [request_id: {}]",
                options.request_id
            ),
        })
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}