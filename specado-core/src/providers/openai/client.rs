//! OpenAI client implementation

use crate::protocol::{ChatRequest, ChatResponse};
use crate::providers::{
    ChatStream, Provider, ProviderConfig, ProviderError, ProviderResult, 
    RateLimitInfo, RateLimitTracker,
};
use super::converter::{to_openai_request, from_openai_response};
use super::streaming::parse_stream;
use super::types::{OpenAIResponse, OpenAIError};
use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use std::time::Duration;

/// OpenAI provider implementation
pub struct OpenAIProvider {
    config: ProviderConfig,
    client: Client,
    rate_limiter: RateLimitTracker,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(config: ProviderConfig) -> ProviderResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| ProviderError::Configuration(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            config,
            client,
            rate_limiter: RateLimitTracker::new(),
        })
    }
    
    /// Build request headers
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        
        // API Key
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.config.api_key)
                .parse()
                .unwrap_or_else(|_| "Bearer invalid".parse().unwrap()),
        );
        
        // Organization ID if provided
        if let Some(org_id) = &self.config.organization_id {
            headers.insert(
                "OpenAI-Organization",
                org_id.parse().unwrap_or_else(|_| "".parse().unwrap()),
            );
        }
        
        // Content type
        headers.insert(
            "Content-Type",
            "application/json".parse().unwrap(),
        );
        
        headers
    }
    
    /// Handle API errors
    fn handle_error_response(&self, status: StatusCode, body: String) -> ProviderError {
        // Try to parse OpenAI error format
        if let Ok(error) = serde_json::from_str::<OpenAIError>(&body) {
            match error.error.error_type.as_str() {
                "invalid_api_key" => ProviderError::Authentication(error.error.message),
                "rate_limit_exceeded" => ProviderError::RateLimit {
                    message: error.error.message,
                    retry_after_secs: None, // OpenAI provides this in headers
                },
                "model_not_found" => ProviderError::ModelNotFound(error.error.message),
                "insufficient_quota" => ProviderError::InsufficientQuota(error.error.message),
                "invalid_request_error" => ProviderError::InvalidRequest(error.error.message),
                _ => ProviderError::ProviderError {
                    code: error.error.error_type,
                    message: error.error.message,
                },
            }
        } else {
            // Fallback to status code-based error
            match status {
                StatusCode::UNAUTHORIZED => ProviderError::Authentication(body),
                StatusCode::TOO_MANY_REQUESTS => ProviderError::RateLimit {
                    message: body,
                    retry_after_secs: None,
                },
                StatusCode::BAD_REQUEST => ProviderError::InvalidRequest(body),
                StatusCode::NOT_FOUND => ProviderError::ModelNotFound(body),
                StatusCode::INTERNAL_SERVER_ERROR |
                StatusCode::BAD_GATEWAY |
                StatusCode::SERVICE_UNAVAILABLE |
                StatusCode::GATEWAY_TIMEOUT => ProviderError::ServiceUnavailable(body),
                _ => ProviderError::ProviderError {
                    code: status.to_string(),
                    message: body,
                },
            }
        }
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }
    
    async fn health_check(&self) -> ProviderResult<()> {
        // Make a simple models list request to check connectivity
        let url = format!("{}/models", self.config.base_url);
        let response = self.client
            .get(&url)
            .headers(self.build_headers())
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(self.handle_error_response(status, body))
        }
    }
    
    async fn chat_completion(&self, request: ChatRequest) -> ProviderResult<ChatResponse> {
        // Check rate limits
        if let Some(wait_duration) = self.rate_limiter.should_wait() {
            return Err(ProviderError::RateLimit {
                message: format!("Rate limit reached, retry after {} seconds", wait_duration.as_secs()),
                retry_after_secs: Some(wait_duration.as_secs()),
            });
        }
        
        // Convert request to OpenAI format
        let mut openai_request = to_openai_request(&request);
        openai_request.stream = Some(false); // Ensure non-streaming
        
        // Make the API request
        let url = format!("{}/chat/completions", self.config.base_url);
        let response = self.client
            .post(&url)
            .headers(self.build_headers())
            .json(&openai_request)
            .send()
            .await?;
        
        // Track rate limits from headers
        if self.config.track_rate_limits {
            self.rate_limiter.update_from_headers(response.headers());
        }
        
        // Handle response
        if response.status().is_success() {
            let openai_response: OpenAIResponse = response.json().await?;
            
            // Track token usage
            if let Some(usage) = &openai_response.usage {
                self.rate_limiter.record_request(usage.total_tokens);
            }
            
            // Convert to our format
            Ok(from_openai_response(openai_response))
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(self.handle_error_response(status, body))
        }
    }
    
    async fn chat_completion_stream(&self, request: ChatRequest) -> ProviderResult<ChatStream> {
        // Check rate limits
        if let Some(wait_duration) = self.rate_limiter.should_wait() {
            return Err(ProviderError::RateLimit {
                message: format!("Rate limit reached, retry after {} seconds", wait_duration.as_secs()),
                retry_after_secs: Some(wait_duration.as_secs()),
            });
        }
        
        // Convert request to OpenAI format
        let mut openai_request = to_openai_request(&request);
        openai_request.stream = Some(true); // Enable streaming
        
        // Make the API request
        let url = format!("{}/chat/completions", self.config.base_url);
        let response = self.client
            .post(&url)
            .headers(self.build_headers())
            .json(&openai_request)
            .send()
            .await?;
        
        // Track rate limits from headers
        if self.config.track_rate_limits {
            self.rate_limiter.update_from_headers(response.headers());
        }
        
        // Handle response
        if response.status().is_success() {
            // Record the request (we'll update tokens later from stream)
            self.rate_limiter.record_request(0);
            
            // Return the stream
            let stream = response.bytes_stream();
            Ok(parse_stream(stream))
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(self.handle_error_response(status, body))
        }
    }
    
    fn rate_limit_info(&self) -> Option<RateLimitInfo> {
        Some(self.rate_limiter.get_info())
    }
}