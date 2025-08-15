//! Provider abstraction and transformation engine
//!
//! This module implements the core provider abstraction layer that enables
//! transparent transformation between different LLM provider formats while
//! tracking lossiness of transformations.

pub mod adapter;
pub mod transform;
pub mod openai;
pub mod anthropic;

pub use adapter::{Provider, ProviderCapabilities, ProviderType};
pub use transform::{TransformResult, TransformationEngine, LossinessReason, transform_request};

// Re-export concrete providers
pub use openai::OpenAIProvider;
pub use anthropic::AnthropicProvider;