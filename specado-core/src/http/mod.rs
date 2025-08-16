//! HTTP client module for making API requests to LLM providers
//!
//! This module implements the HTTP layer for Specado, handling:
//! - Connection pooling and client management
//! - Request/response transformation
//! - Error mapping and retry hints
//! - Request ID generation and correlation

pub mod client;
pub mod error;

use crate::protocol::types::{ChatRequest, ChatResponse};
use crate::providers::adapter::Provider;
use crate::providers::routing::ProviderError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Type of API call being made
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallKind {
    /// Chat completion request
    Chat,
    // Future additions:
    // Completions,
    // Embeddings,
    // Images,
    // Audio,
}

impl CallKind {
    /// Get the endpoint path for this call kind
    pub fn endpoint(&self) -> &str {
        match self {
            CallKind::Chat => "/chat/completions",
        }
    }
}

/// Options for an HTTP request
#[derive(Debug, Clone)]
pub struct RequestOptions {
    /// Type of API call
    pub call_kind: CallKind,

    /// Unique request ID for correlation
    pub request_id: Uuid,

    /// Request timeout
    pub timeout: Duration,

    /// Optional idempotency key for safe retries
    pub idempotency_key: Option<String>,

    /// Optional context ID for correlation across requests
    pub context_id: Option<String>,
}

impl Default for RequestOptions {
    fn default() -> Self {
        Self {
            call_kind: CallKind::Chat,
            request_id: Uuid::new_v4(),
            timeout: Duration::from_secs(30),
            idempotency_key: None,
            context_id: None,
        }
    }
}

impl RequestOptions {
    /// Create new request options with a generated request ID
    pub fn new(call_kind: CallKind) -> Self {
        Self {
            call_kind,
            request_id: Uuid::new_v4(),
            ..Default::default()
        }
    }

    /// Set the timeout for this request
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set an idempotency key for safe retries
    pub fn with_idempotency_key(mut self, key: String) -> Self {
        self.idempotency_key = Some(key);
        self
    }

    /// Set a context ID for correlation
    pub fn with_context_id(mut self, id: String) -> Self {
        self.context_id = Some(id);
        self
    }
}

/// Stream delta for streaming responses (Phase 2 stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDelta {
    /// Delta content
    pub content: String,

    /// Whether this is the final delta
    pub is_final: bool,
}

/// Trait for HTTP executors
#[async_trait]
pub trait HttpExecutor: Send + Sync {
    /// Execute a non-streaming JSON request
    async fn execute_json(
        &self,
        provider: &dyn Provider,
        request: ChatRequest,
        options: RequestOptions,
    ) -> Result<ChatResponse, ProviderError>;

    /// Execute a streaming request (Phase 2 stub)
    async fn execute_stream(
        &self,
        provider: &dyn Provider,
        request: ChatRequest,
        options: RequestOptions,
    ) -> Result<Vec<StreamDelta>, ProviderError> {
        // Phase 2: Will return BoxStream<'static, Result<StreamDelta, ProviderError>>
        // For now, return error indicating streaming not yet implemented
        let _ = (provider, request, options);
        Err(ProviderError::Custom {
            code: "STREAMING_NOT_IMPLEMENTED".to_string(),
            message: "Streaming support will be implemented in Phase 2".to_string(),
        })
    }
}
