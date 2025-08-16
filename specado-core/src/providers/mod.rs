//! Provider abstraction and transformation engine
//!
//! This module implements the core provider abstraction layer that enables
//! transparent transformation between different LLM provider formats while
//! tracking lossiness of transformations.

pub mod adapter;
pub mod anthropic;
pub mod json_transform;
pub mod openai;
pub mod retry;
pub mod routing;
pub mod transform;

pub use adapter::{Provider, ProviderCapabilities, ProviderType};
pub use transform::{transform_request, LossinessReason, TransformResult, TransformationEngine};

// Re-export concrete providers
pub use anthropic::AnthropicProvider;
pub use openai::OpenAIProvider;

// Re-export routing types
pub use routing::{
    PrimaryWithFallbacks, ProviderError, RoutingBuilder, RoutingResult, RoutingStrategy,
};

// Re-export retry types
pub use retry::{ErrorMapper, RetryExecutor, RetryPolicy, RetryResult};
