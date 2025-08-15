//! Provider implementations for various LLM services
//!
//! This module contains adapters for different LLM providers, translating
//! between the canonical Specado protocol and provider-specific APIs.

pub mod openai;
pub mod error;
pub mod rate_limit;

pub use error::{ProviderError, ProviderResult};
pub use rate_limit::{RateLimitInfo, RateLimitTracker};

use crate::protocol::{ChatRequest, ChatResponse, ChatStreamChunk};
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

/// Stream type for chat responses
pub type ChatStream = Pin<Box<dyn Stream<Item = Result<ChatStreamChunk, ProviderError>> + Send>>;

/// Trait that all LLM providers must implement
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;
    
    /// Check if the provider is available and properly configured
    async fn health_check(&self) -> ProviderResult<()>;
    
    /// Send a chat completion request (non-streaming)
    async fn chat_completion(&self, request: ChatRequest) -> ProviderResult<ChatResponse>;
    
    /// Send a chat completion request (streaming)
    async fn chat_completion_stream(&self, request: ChatRequest) -> ProviderResult<ChatStream>;
    
    /// Get current rate limit information if available
    fn rate_limit_info(&self) -> Option<RateLimitInfo>;
}

/// Provider configuration common to all providers
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// API key for authentication
    pub api_key: String,
    
    /// Base URL for the API
    pub base_url: String,
    
    /// Organization ID (optional)
    pub organization_id: Option<String>,
    
    /// Request timeout in seconds
    pub timeout_secs: u64,
    
    /// Maximum retries for failed requests
    pub max_retries: u32,
    
    /// Enable rate limit tracking
    pub track_rate_limits: bool,
}