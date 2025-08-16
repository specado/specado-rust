//! Core protocol types for LLM interactions
//!
//! This module contains the fundamental data structures used for communication
//! with LLM providers. The design prioritizes:
//! - Type safety through enums and strong typing
//! - Forward compatibility through optional fields and metadata
//! - Streaming support through response type variants
//! - Provider flexibility through extension points

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Role of a message in the conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System instructions that guide the model's behavior
    System,
    /// User input message
    User,
    /// Assistant (model) response
    Assistant,
    /// Function call result (for function calling support)
    Function,
    /// Tool response (for tool use support)
    Tool,
}

/// Content of a message - supports text and future structured content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Plain text content
    Text(String),
    /// Structured content parts (for multimodal support)
    Parts(Vec<ContentPart>),
}

/// Individual content part for multimodal messages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// Text content part
    Text { text: String },
    /// Image content (base64 encoded or URL)
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        base64: Option<String>,
    },
    /// Audio content (for future audio support)
    Audio {
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        base64: Option<String>,
    },
}

/// A message in the conversation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: MessageRole,

    /// Content of the message
    pub content: MessageContent,

    /// Optional name for the message sender
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Function call information (for assistant messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,

    /// Tool calls (for assistant messages with tool use)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Tool call ID (for tool response messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,

    /// Provider-specific metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Function call information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Name of the function to call
    pub name: String,

    /// Arguments to the function (usually JSON string)
    pub arguments: String,
}

/// Tool call information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call
    pub id: String,

    /// Type of tool (usually "function")
    #[serde(rename = "type")]
    pub tool_type: String,

    /// Function information
    pub function: FunctionCall,
}

/// Chat completion request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ChatRequest {
    /// Model identifier to use
    #[serde(default)]
    pub model: String,

    /// Messages in the conversation
    #[serde(default)]
    pub messages: Vec<Message>,

    /// Sampling temperature (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,

    /// Nucleus sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Number of completions to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<usize>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// Streaming configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Streaming options (for advanced streaming control)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,

    /// Frequency penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Presence penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// User identifier for abuse detection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Response format hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// Seed for deterministic generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// Tool definitions for function calling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,

    /// Tool choice configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Provider-specific parameters
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Streaming options for advanced streaming control
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamOptions {
    /// Include usage information in stream
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
}

/// Response format configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    /// Plain text response
    Text,
    /// JSON mode
    JsonObject,
    /// JSON with schema
    JsonSchema { schema: serde_json::Value },
}

/// Tool definition for function calling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Type of tool (usually "function")
    #[serde(rename = "type")]
    pub tool_type: String,

    /// Function definition
    pub function: FunctionDefinition,
}

/// Function definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,

    /// Function description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Parameters schema (JSON Schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Tool choice configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// Auto, none, required
    Mode(String),
    /// Specific function
    Function {
        #[serde(rename = "type")]
        choice_type: String,
        function: FunctionChoice,
    },
}

/// Function choice specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionChoice {
    /// Name of the function to use
    pub name: String,
}

/// Chat completion response type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseType {
    /// Complete response (non-streaming)
    Complete(ChatResponse),
    /// Streaming chunk
    Stream(ChatStreamChunk),
}

/// Complete chat response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Unique response ID
    pub id: String,

    /// Object type (usually "chat.completion")
    pub object: String,

    /// Creation timestamp
    pub created: i64,

    /// Model used for generation
    pub model: String,

    /// Response choices
    pub choices: Vec<ResponseChoice>,

    /// Token usage information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,

    /// System fingerprint for reproducibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,

    /// Provider-specific metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Response choice
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseChoice {
    /// Choice index
    pub index: usize,

    /// Generated message
    pub message: Message,

    /// Finish reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,

    /// Log probabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
}

/// Streaming response chunk
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatStreamChunk {
    /// Unique response ID
    pub id: String,

    /// Object type (usually "chat.completion.chunk")
    pub object: String,

    /// Creation timestamp
    pub created: i64,

    /// Model used
    pub model: String,

    /// Delta choices
    pub choices: Vec<StreamChoice>,

    /// Usage (only in final chunk if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,

    /// System fingerprint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}

/// Streaming choice with delta
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamChoice {
    /// Choice index
    pub index: usize,

    /// Delta message content
    pub delta: MessageDelta,

    /// Finish reason (only in final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,

    /// Log probabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<serde_json::Value>,
}

/// Delta message for streaming
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MessageDelta {
    /// Role (only in first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<MessageRole>,

    /// Content delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Function call delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCallDelta>,

    /// Tool calls delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

/// Function call delta for streaming
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCallDelta {
    /// Function name (only in first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Arguments delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// Tool call delta for streaming
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallDelta {
    /// Index in the tool calls array
    pub index: usize,

    /// Tool call ID (only in first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Tool type (only in first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub tool_type: Option<String>,

    /// Function delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionCallDelta>,
}

/// Token usage information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionUsage {
    /// Tokens in the prompt
    pub prompt_tokens: u32,

    /// Tokens in the completion
    pub completion_tokens: u32,

    /// Total tokens used
    pub total_tokens: u32,
}

// ============================================================================
// Builder and convenience traits
// ============================================================================

/// Trait for converting types into messages
pub trait IntoMessage {
    /// Convert self into a Message
    fn into_message(self) -> Message;
}

impl IntoMessage for Message {
    fn into_message(self) -> Message {
        self
    }
}

impl IntoMessage for String {
    fn into_message(self) -> Message {
        Message::user(self)
    }
}

impl IntoMessage for &str {
    fn into_message(self) -> Message {
        Message::user(self)
    }
}

/// Builder for constructing messages
pub struct MessageBuilder {
    role: MessageRole,
    content: MessageContent,
    name: Option<String>,
    function_call: Option<FunctionCall>,
    tool_calls: Option<Vec<ToolCall>>,
    tool_call_id: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl MessageBuilder {
    /// Create a new message builder with role and text content
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: MessageContent::Text(content.into()),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new message builder with role and multimodal parts
    pub fn with_parts(role: MessageRole, parts: Vec<ContentPart>) -> Self {
        Self {
            role,
            content: MessageContent::Parts(parts),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the name field
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set function call
    pub fn with_function_call(
        mut self,
        name: impl Into<String>,
        arguments: impl Into<String>,
    ) -> Self {
        self.function_call = Some(FunctionCall {
            name: name.into(),
            arguments: arguments.into(),
        });
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Build the message
    pub fn build(self) -> Message {
        Message {
            role: self.role,
            content: self.content,
            name: self.name,
            function_call: self.function_call,
            tool_calls: self.tool_calls,
            tool_call_id: self.tool_call_id,
            metadata: self.metadata,
        }
    }
}

// ============================================================================
// Convenience constructors
// ============================================================================

impl Message {
    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        MessageBuilder::new(MessageRole::System, content).build()
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        MessageBuilder::new(MessageRole::User, content).build()
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        MessageBuilder::new(MessageRole::Assistant, content).build()
    }

    /// Create a function response message
    pub fn function(name: impl Into<String>, content: impl Into<String>) -> Self {
        MessageBuilder::new(MessageRole::Function, content)
            .with_name(name)
            .build()
    }

    /// Create a tool response message
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        let mut msg = MessageBuilder::new(MessageRole::Tool, content).build();
        msg.tool_call_id = Some(tool_call_id.into());
        msg
    }
}

impl ChatRequest {
    /// Create a new chat request with model and messages
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            top_p: None,
            n: None,
            stop: None,
            stream: None,
            stream_options: None,
            frequency_penalty: None,
            presence_penalty: None,
            user: None,
            response_format: None,
            seed: None,
            tools: None,
            tool_choice: None,
            metadata: HashMap::new(),
        }
    }

    /// Enable streaming
    pub fn with_streaming(mut self, include_usage: bool) -> Self {
        self.stream = Some(true);
        if include_usage {
            self.stream_options = Some(StreamOptions {
                include_usage: Some(true),
            });
        }
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set top_p for nucleus sampling
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set stop sequences
    pub fn with_stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }

    /// Add a single stop sequence
    pub fn with_stop_sequence(mut self, stop: impl Into<String>) -> Self {
        self.stop.get_or_insert_with(Vec::new).push(stop.into());
        self
    }
}

impl MessageContent {
    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        match self {
            MessageContent::Text(s) => s.is_empty(),
            MessageContent::Parts(parts) => parts.is_empty(),
        }
    }

    /// Get text representation
    pub fn as_text(&self) -> Option<&str> {
        match self {
            MessageContent::Text(s) => Some(s.as_str()),
            MessageContent::Parts(_) => None,
        }
    }
}
