//! OpenAI provider implementation
//!
//! Implements the Provider trait for OpenAI's API.
//! For MVP, capabilities are hardcoded based on GPT-4 and GPT-3.5-turbo.

use crate::protocol::types::{ChatRequest, ChatResponse};
use crate::providers::adapter::{Provider, ProviderCapabilities};
use std::collections::HashMap;
use serde_json::Value;

/// OpenAI provider implementation
pub struct OpenAIProvider {
    capabilities: ProviderCapabilities,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new() -> Self {
        let capabilities = ProviderCapabilities {
            supports_system_role: true,
            supports_json_mode: true,
            supports_functions: true,
            supports_streaming: true,
            max_context_tokens: 128000, // GPT-4 Turbo
            supports_consecutive_same_role: true,
            supports_temperature: true,
            supports_top_p: true,
            supports_max_tokens: true,
            custom: {
                let mut custom = HashMap::new();
                custom.insert("supports_response_format".to_string(), Value::Bool(true));
                custom.insert("supports_seed".to_string(), Value::Bool(true));
                custom.insert("supports_logprobs".to_string(), Value::Bool(true));
                custom.insert("supports_n".to_string(), Value::Bool(true));
                custom
            },
        };
        
        Self { capabilities }
    }
}

impl Provider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }
    
    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }
    
    fn transform_request(&self, request: ChatRequest) -> ChatRequest {
        // OpenAI uses its own format, so no transformation needed for MVP
        // In the future, this might handle OpenAI-specific adjustments
        request
    }
    
    fn transform_response(&self, response: ChatResponse) -> ChatResponse {
        // OpenAI response is already in our canonical format
        response
    }
    
    fn base_url(&self) -> &str {
        "https://api.openai.com/v1"
    }
    
    fn headers(&self, api_key: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }
}

impl Default for OpenAIProvider {
    fn default() -> Self {
        Self::new()
    }
}