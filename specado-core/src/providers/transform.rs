//! Transformation engine with lossiness tracking
//!
//! This module implements the core transformation logic that converts requests
//! between different provider formats while tracking what information is lost.

use crate::protocol::types::{ChatRequest, Message, MessageRole, MessageContent, ResponseFormat};
use crate::providers::adapter::Provider;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Result of a transformation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    /// The transformed request
    pub transformed: ChatRequest,
    
    /// Whether the transformation was lossy
    pub lossy: bool,
    
    /// Reasons for lossiness (human-readable)
    pub reasons: Vec<String>,
    
    /// Additional metadata about the transformation
    pub metadata: HashMap<String, Value>,
}

/// Reasons for lossy transformations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LossinessReason {
    SystemRoleMerged,
    JsonModeNotSupported,
    FunctionCallingNotSupported,
    StreamingNotSupported,
    MaxTokensExceeded,
    ConsecutiveSameRoleNotSupported,
    ParameterNotSupported(String),
    CustomReason(String),
}

impl LossinessReason {
    /// Convert to human-readable string
    pub fn as_str(&self) -> &str {
        match self {
            Self::SystemRoleMerged => "system_role_merged",
            Self::JsonModeNotSupported => "json_mode_not_supported",
            Self::FunctionCallingNotSupported => "function_calling_not_supported",
            Self::StreamingNotSupported => "streaming_not_supported",
            Self::MaxTokensExceeded => "max_tokens_exceeded",
            Self::ConsecutiveSameRoleNotSupported => "consecutive_same_role_not_supported",
            Self::ParameterNotSupported(param) => param,
            Self::CustomReason(reason) => reason,
        }
    }
}

/// Core transformation engine
pub struct TransformationEngine {
    /// Source provider (for future use in bidirectional transforms)
    #[allow(dead_code)]
    source_provider: Box<dyn Provider>,
    
    /// Target provider
    target_provider: Box<dyn Provider>,
}

impl TransformationEngine {
    /// Create a new transformation engine
    pub fn new(source: Box<dyn Provider>, target: Box<dyn Provider>) -> Self {
        Self {
            source_provider: source,
            target_provider: target,
        }
    }
    
    /// Transform a request from source to target format
    pub fn transform_request(&self, request: ChatRequest) -> TransformResult {
        let mut lossy = false;
        let mut reasons = Vec::new();
        let target_caps = self.target_provider.capabilities();
        
        // Clone the request for transformation
        let mut transformed = request.clone();
        
        // Check and transform system role if not supported
        if !target_caps.supports_system_role && self.has_system_message(&transformed) {
            self.merge_system_messages(&mut transformed);
            lossy = true;
            reasons.push(LossinessReason::SystemRoleMerged.as_str().to_string());
        }
        
        // Check JSON mode support
        if let Some(ResponseFormat::JsonObject) = &transformed.response_format {
            if !target_caps.supports_json_mode {
                transformed.response_format = None;
                lossy = true;
                reasons.push(LossinessReason::JsonModeNotSupported.as_str().to_string());
            }
        }
        
        // Check function calling support
        if transformed.tools.is_some() && !target_caps.supports_functions {
            transformed.tools = None;
            transformed.tool_choice = None;
            lossy = true;
            reasons.push(LossinessReason::FunctionCallingNotSupported.as_str().to_string());
        }
        
        // Check streaming support
        if transformed.stream == Some(true) && !target_caps.supports_streaming {
            transformed.stream = Some(false);
            lossy = true;
            reasons.push(LossinessReason::StreamingNotSupported.as_str().to_string());
        }
        
        // Check temperature support
        if transformed.temperature.is_some() && !target_caps.supports_temperature {
            transformed.temperature = None;
            lossy = true;
            reasons.push(LossinessReason::ParameterNotSupported("temperature".to_string()).as_str().to_string());
        }
        
        // Check top_p support
        if transformed.top_p.is_some() && !target_caps.supports_top_p {
            transformed.top_p = None;
            lossy = true;
            reasons.push(LossinessReason::ParameterNotSupported("top_p".to_string()).as_str().to_string());
        }
        
        // Check consecutive same role support
        if !target_caps.supports_consecutive_same_role && self.has_consecutive_same_role(&transformed) {
            self.merge_consecutive_same_role(&mut transformed);
            lossy = true;
            reasons.push(LossinessReason::ConsecutiveSameRoleNotSupported.as_str().to_string());
        }
        
        // Let the target provider do any additional transformations
        transformed = self.target_provider.transform_request(transformed);
        
        // Add lossiness metadata to the request
        if lossy {
            transformed.metadata.insert("lossy".to_string(), json!(true));
            transformed.metadata.insert("lossy_reasons".to_string(), json!(reasons.clone()));
        }
        
        TransformResult {
            transformed,
            lossy,
            reasons,
            metadata: HashMap::new(),
        }
    }
    
    /// Check if request has system messages
    fn has_system_message(&self, request: &ChatRequest) -> bool {
        request.messages.iter().any(|msg| msg.role == MessageRole::System)
    }
    
    /// Merge system messages into user messages
    fn merge_system_messages(&self, request: &mut ChatRequest) {
        let mut merged_messages = Vec::new();
        let mut system_content = String::new();
        
        for message in &request.messages {
            match message.role {
                MessageRole::System => {
                    if let MessageContent::Text(text) = &message.content {
                        if !system_content.is_empty() {
                            system_content.push_str("\n\n");
                        }
                        system_content.push_str(text);
                    }
                }
                MessageRole::User => {
                    // If we have accumulated system content, prepend it to this user message
                    if !system_content.is_empty() {
                        let mut new_message = message.clone();
                        if let MessageContent::Text(user_text) = &message.content {
                            new_message.content = MessageContent::Text(
                                format!("{}\n\n{}", system_content, user_text)
                            );
                        }
                        merged_messages.push(new_message);
                        system_content.clear();
                    } else {
                        merged_messages.push(message.clone());
                    }
                }
                _ => {
                    merged_messages.push(message.clone());
                }
            }
        }
        
        // If there's remaining system content and no user message to attach it to,
        // create a new user message
        if !system_content.is_empty() {
            merged_messages.push(Message {
                role: MessageRole::User,
                content: MessageContent::Text(system_content),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                metadata: HashMap::new(),
            });
        }
        
        request.messages = merged_messages;
    }
    
    /// Check if request has consecutive messages with the same role
    fn has_consecutive_same_role(&self, request: &ChatRequest) -> bool {
        let mut prev_role = None;
        for message in &request.messages {
            if let Some(prev) = prev_role {
                if prev == message.role && message.role != MessageRole::System {
                    return true;
                }
            }
            prev_role = Some(message.role);
        }
        false
    }
    
    /// Merge consecutive messages with the same role
    fn merge_consecutive_same_role(&self, request: &mut ChatRequest) {
        let mut merged_messages = Vec::new();
        let mut current_group: Option<Message> = None;
        
        for message in &request.messages {
            if let Some(mut group) = current_group.take() {
                if group.role == message.role && message.role != MessageRole::System {
                    // Merge the content
                    if let (MessageContent::Text(group_text), MessageContent::Text(msg_text)) = 
                        (&group.content, &message.content) {
                        group.content = MessageContent::Text(
                            format!("{}\n\n{}", group_text, msg_text)
                        );
                    }
                    current_group = Some(group);
                } else {
                    merged_messages.push(group);
                    current_group = Some(message.clone());
                }
            } else {
                current_group = Some(message.clone());
            }
        }
        
        if let Some(group) = current_group {
            merged_messages.push(group);
        }
        
        request.messages = merged_messages;
    }
}

/// Helper function for simple transformation (Week 1 demo interface)
pub fn transform_request(request: ChatRequest, target_provider: &str) -> TransformResult {
    use crate::providers::adapter::ProviderType;
    
    // For MVP, assume source is OpenAI format
    let source = ProviderType::OpenAI.create_provider();
    
    let target = match target_provider.to_lowercase().as_str() {
        "anthropic" => ProviderType::Anthropic.create_provider(),
        "openai" => ProviderType::OpenAI.create_provider(),
        _ => ProviderType::OpenAI.create_provider(), // Default to OpenAI
    };
    
    let engine = TransformationEngine::new(source, target);
    engine.transform_request(request)
}