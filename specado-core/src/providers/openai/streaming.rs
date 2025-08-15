//! Streaming support for OpenAI responses

use crate::providers::{ChatStream, ProviderError};
use super::converter::from_openai_stream_chunk;
use super::types::OpenAIStreamChunk;
use bytes::Bytes;
use eventsource_stream::Eventsource;
use futures::{Stream, StreamExt};

/// Parse Server-Sent Events stream from OpenAI
pub fn parse_stream(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> ChatStream {
    let event_stream = stream.eventsource();
    
    Box::pin(event_stream.filter_map(|result| async move {
        match result {
            Ok(event) => {
                // OpenAI sends data as "data: {...json...}"
                // The last message is "data: [DONE]"
                if event.data == "[DONE]" {
                    return None;
                }
                
                // Parse the JSON data
                match serde_json::from_str::<OpenAIStreamChunk>(&event.data) {
                    Ok(chunk) => {
                        // Convert to our format
                        let chat_chunk = from_openai_stream_chunk(chunk);
                        Some(Ok(chat_chunk))
                    }
                    Err(e) => {
                        // Log parsing error but continue stream
                        tracing::warn!("Failed to parse stream chunk: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                // Convert error to our type
                Some(Err(ProviderError::ParseError(format!("Stream error: {}", e))))
            }
        }
    }))
}