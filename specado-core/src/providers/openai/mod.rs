//! OpenAI provider implementation
//!
//! This module provides an adapter for the OpenAI API, translating between
//! Specado's canonical protocol and OpenAI's specific format.

mod client;
pub mod converter;
mod streaming;
pub mod types;

pub use client::OpenAIProvider;
pub use types::{OpenAIRequest, OpenAIResponse, OpenAIStreamChunk};

