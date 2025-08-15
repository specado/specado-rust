//! Protocol module for LLM request/response structures
//!
//! This module defines the canonical data models for interacting with LLM providers.
//! These structures are designed to be:
//! - Provider-agnostic
//! - Forward-compatible with streaming
//! - Extensible through metadata fields
//! - Type-safe and serializable

pub mod types;

pub use types::{
    ChatRequest, ChatResponse, ChatStreamChunk, CompletionUsage, ContentPart, FunctionCall,
    FunctionCallDelta, FunctionChoice, FunctionDefinition, Message, MessageContent, MessageDelta,
    MessageRole, ResponseChoice, ResponseFormat, ResponseType, StreamChoice, StreamOptions,
    ToolCall, ToolCallDelta, ToolChoice, ToolDefinition,
};

// Re-export common traits for convenience
pub use types::{IntoMessage, MessageBuilder};
