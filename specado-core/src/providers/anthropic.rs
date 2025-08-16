//! Anthropic provider implementation
//!
//! Implements the Provider trait for Anthropic's Claude API.
//! Handles the differences in message format and capabilities.

use crate::http::CallKind;
use crate::protocol::types::{ChatRequest, ChatResponse, Message, MessageRole};
use crate::providers::adapter::{Provider, ProviderCapabilities};
use std::collections::HashMap;
use serde_json::Value;

/// Anthropic provider implementation
pub struct AnthropicProvider {
    capabilities: ProviderCapabilities,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new() -> Self {
        let capabilities = ProviderCapabilities {
            supports_system_role: false, // Anthropic handles system differently
            supports_json_mode: false, // No native JSON mode
            supports_functions: false, // No function calling in MVP
            supports_streaming: true,
            max_context_tokens: 200000, // Claude 3 context window
            supports_consecutive_same_role: false, // Anthropic requires alternating roles
            supports_temperature: true,
            supports_top_p: true,
            supports_max_tokens: true,
            custom: {
                let mut custom = HashMap::new();
                custom.insert("supports_system_prompt".to_string(), Value::Bool(true));
                custom.insert("model_prefix".to_string(), Value::String("claude-3".to_string()));
                custom
            },
        };
        
        Self { capabilities }
    }
    
    /// Convert messages to Anthropic format
    fn convert_messages(&self, messages: &[Message]) -> Vec<Message> {
        // Anthropic expects alternating user/assistant messages
        // System messages are handled separately in their API
        let mut converted = Vec::new();
        
        for message in messages {
            match message.role {
                MessageRole::System => {
                    // System messages are merged into user messages by the transformation engine
                    // So we shouldn't see them here, but if we do, convert to user
                    let mut user_msg = message.clone();
                    user_msg.role = MessageRole::User;
                    converted.push(user_msg);
                }
                _ => {
                    converted.push(message.clone());
                }
            }
        }
        
        converted
    }
}

impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }
    
    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }
    
    fn transform_request(&self, mut request: ChatRequest) -> ChatRequest {
        // Convert messages to Anthropic format
        request.messages = self.convert_messages(&request.messages);
        
        // Map provider-specific parameters
        // Anthropic uses "max_tokens_to_sample" instead of "max_tokens"
        if let Some(max_tokens) = request.max_tokens {
            request.metadata.insert(
                "max_tokens_to_sample".to_string(), 
                serde_json::json!(max_tokens)
            );
            // Keep the original max_tokens for now - actual HTTP client will use metadata
        }
        
        // Anthropic uses "top_k" parameter which OpenAI doesn't have
        // This would come from request.metadata if the user specified it
        
        request
    }
    
    fn transform_response(&self, response: ChatResponse) -> ChatResponse {
        // Anthropic responses need to be converted to our canonical format
        // For MVP, we assume the response is already normalized
        response
    }
    
    fn base_url(&self) -> &str {
        "https://api.anthropic.com/v1"
    }
    
    fn endpoint(&self, call_kind: CallKind) -> &str {
        match call_kind {
            CallKind::Chat => "/messages",  // Anthropic uses /messages, not /chat/completions
        }
    }
    
    fn headers(&self, api_key: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("x-api-key".to_string(), api_key.to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
        headers
    }
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::new()
    }
}