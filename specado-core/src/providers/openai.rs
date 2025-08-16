//! OpenAI provider implementation
//!
//! Implements the Provider trait for OpenAI's API.
//! For MVP, capabilities are hardcoded based on GPT-4 and GPT-3.5-turbo.

use crate::http::CallKind;
use crate::protocol::types::{ChatRequest, ChatResponse};
use crate::providers::adapter::{Provider, ProviderCapabilities};
use serde_json::Value;
use std::collections::HashMap;

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
        // OpenAI uses its own format natively
        // Map any provider-specific parameters from metadata

        // OpenAI supports seed parameter for reproducible outputs
        if let Some(_seed) = request.metadata.get("seed") {
            // The seed parameter is already in metadata, HTTP client will use it
        }

        // OpenAI supports logprobs parameter
        if let Some(_logprobs) = request.metadata.get("logprobs") {
            // The logprobs parameter is already in metadata, HTTP client will use it
        }

        // OpenAI uses standard "max_tokens" naming, no transformation needed

        request
    }

    fn transform_response(&self, response: ChatResponse) -> ChatResponse {
        // OpenAI response is already in our canonical format
        response
    }

    fn base_url(&self) -> &str {
        "https://api.openai.com/v1"
    }

    fn endpoint(&self, call_kind: CallKind) -> &str {
        match call_kind {
            CallKind::Chat => "/chat/completions",
        }
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
