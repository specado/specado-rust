//! Provider adapter trait and capabilities
//!
//! Defines the core abstraction for LLM providers and their capabilities.
//! For MVP, capabilities are hardcoded. Future versions will load from manifests.

use crate::http::CallKind;
use crate::protocol::types::{ChatRequest, ChatResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Core provider trait that all LLM providers must implement
pub trait Provider: Send + Sync {
    /// Get the provider's name
    fn name(&self) -> &str;

    /// Get the provider's capabilities
    fn capabilities(&self) -> &ProviderCapabilities;

    /// Transform a request from canonical format to provider-specific format
    fn transform_request(&self, request: ChatRequest) -> ChatRequest;

    /// Transform a response from provider-specific format to canonical format
    fn transform_response(&self, response: ChatResponse) -> ChatResponse;

    /// Get the base URL for this provider
    fn base_url(&self) -> &str;

    /// Get the endpoint path for a specific call kind
    fn endpoint(&self, call_kind: CallKind) -> &str;

    /// Get headers required for this provider
    fn headers(&self, api_key: &str) -> HashMap<String, String>;
}

/// Provider capabilities (hardcoded for MVP, manifest-driven in future)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    /// Does the provider support system messages?
    pub supports_system_role: bool,

    /// Does the provider support JSON mode?
    pub supports_json_mode: bool,

    /// Does the provider support function calling?
    pub supports_functions: bool,

    /// Does the provider support streaming?
    pub supports_streaming: bool,

    /// Maximum context window size
    pub max_context_tokens: usize,

    /// Does the provider support multiple messages with the same role in sequence?
    pub supports_consecutive_same_role: bool,

    /// Does the provider support the temperature parameter?
    pub supports_temperature: bool,

    /// Does the provider support top_p parameter?
    pub supports_top_p: bool,

    /// Does the provider support max_tokens parameter?
    pub supports_max_tokens: bool,

    /// Provider-specific capability flags
    pub custom: HashMap<String, Value>,
}

impl Default for ProviderCapabilities {
    fn default() -> Self {
        Self {
            supports_system_role: true,
            supports_json_mode: false,
            supports_functions: false,
            supports_streaming: true,
            max_context_tokens: 4096,
            supports_consecutive_same_role: true,
            supports_temperature: true,
            supports_top_p: true,
            supports_max_tokens: true,
            custom: HashMap::new(),
        }
    }
}

/// Provider type enumeration for MVP
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    // Future: Generic for OpenAI-compatible providers
}

impl ProviderType {
    /// Create a provider instance for this type
    pub fn create_provider(&self) -> Box<dyn Provider> {
        match self {
            ProviderType::OpenAI => Box::new(crate::providers::OpenAIProvider::new()),
            ProviderType::Anthropic => Box::new(crate::providers::AnthropicProvider::new()),
        }
    }
}
