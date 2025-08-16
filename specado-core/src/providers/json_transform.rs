//! JSON transformation module for actual API compatibility
//!
//! This module implements the actual JSON structure transformations needed
//! to convert between OpenAI and Anthropic API formats.

use crate::protocol::types::{
    ChatRequest, ChatResponse, Message, MessageContent, MessageRole, ResponseChoice,
};
use serde_json::{json, Map, Value};
use std::collections::HashMap;

/// Transform an OpenAI-format request to Anthropic JSON format
pub fn openai_request_to_anthropic_json(request: ChatRequest) -> Value {
    let mut anthropic_request = Map::new();

    // Model name stays the same
    anthropic_request.insert("model".to_string(), json!(request.model));

    // CRITICAL: Anthropic uses "max_tokens", NOT "max_tokens_to_sample" in modern API
    // This was a misconception - let me check the latest Anthropic docs
    if let Some(max_tokens) = request.max_tokens {
        anthropic_request.insert("max_tokens".to_string(), json!(max_tokens));
    }

    // Temperature stays the same
    if let Some(temperature) = request.temperature {
        anthropic_request.insert("temperature".to_string(), json!(temperature));
    }

    // Top-p stays the same
    if let Some(top_p) = request.top_p {
        anthropic_request.insert("top_p".to_string(), json!(top_p));
    }

    // Handle messages transformation
    let mut system_prompt = None;
    let mut messages = Vec::new();

    for message in request.messages {
        match message.role {
            MessageRole::System => {
                // Anthropic handles system messages as a separate "system" field
                if let MessageContent::Text(text) = message.content {
                    system_prompt = Some(text);
                }
            }
            MessageRole::User | MessageRole::Assistant => {
                // Convert message to Anthropic format
                let mut msg = Map::new();
                // Convert role to string representation
                let role_str = match message.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    _ => "user", // Default fallback
                };
                msg.insert("role".to_string(), json!(role_str));

                // Handle content - Anthropic expects content array for structured content
                match message.content {
                    MessageContent::Text(text) => {
                        // For simple text, Anthropic accepts string content
                        msg.insert("content".to_string(), json!(text));
                    }
                    MessageContent::Parts(parts) => {
                        // Convert parts to Anthropic content blocks
                        let mut content_blocks = Vec::new();
                        for part in parts {
                            match part {
                                crate::protocol::types::ContentPart::Text { text } => {
                                    content_blocks.push(json!({
                                        "type": "text",
                                        "text": text
                                    }));
                                }
                                crate::protocol::types::ContentPart::Image { url, base64 } => {
                                    // Anthropic expects base64 images
                                    if let Some(base64_data) = base64 {
                                        content_blocks.push(json!({
                                            "type": "image",
                                            "source": {
                                                "type": "base64",
                                                "media_type": "image/jpeg",
                                                "data": base64_data.replace("data:image/jpeg;base64,", "")
                                            }
                                        }));
                                    } else if let Some(url_data) = url {
                                        content_blocks.push(json!({
                                            "type": "image",
                                            "source": {
                                                "type": "base64",
                                                "media_type": "image/jpeg",
                                                "data": url_data.replace("data:image/jpeg;base64,", "")
                                            }
                                        }));
                                    }
                                    // Skip if no data
                                }
                                crate::protocol::types::ContentPart::Audio { .. } => {
                                    // Audio not supported in MVP, skip
                                }
                            }
                        }
                        msg.insert("content".to_string(), json!(content_blocks));
                    }
                }

                messages.push(json!(msg));
            }
            _ => {
                // Skip other roles for now (Tool, Function not supported in MVP)
            }
        }
    }

    // Add system prompt if present
    if let Some(system) = system_prompt {
        anthropic_request.insert("system".to_string(), json!(system));
    }

    // Add messages
    anthropic_request.insert("messages".to_string(), json!(messages));

    // Handle streaming
    if let Some(stream) = request.stream {
        anthropic_request.insert("stream".to_string(), json!(stream));
    }

    // Handle stop sequences
    if let Some(stop) = request.stop {
        anthropic_request.insert("stop_sequences".to_string(), json!(stop));
    }

    // Add any custom metadata from the request that Anthropic might support
    for (key, value) in request.metadata {
        // Skip keys we've already handled or that are OpenAI-specific
        if ![
            "max_tokens",
            "temperature",
            "top_p",
            "stream",
            "stop",
            "n",
            "presence_penalty",
            "frequency_penalty",
            "seed",
            "logprobs",
            "response_format",
        ]
        .contains(&key.as_str())
        {
            anthropic_request.insert(key, value);
        }
    }

    Value::Object(anthropic_request)
}

/// Transform an Anthropic JSON response to OpenAI-compatible ChatResponse format
pub fn anthropic_response_to_openai(response_json: Value) -> Result<ChatResponse, String> {
    let obj = response_json
        .as_object()
        .ok_or("Response is not an object")?;

    // Extract fields from Anthropic response
    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("msg_unknown")
        .to_string();

    let model = obj
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("claude-unknown")
        .to_string();

    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Transform Anthropic content to OpenAI choices format
    let mut choices = Vec::new();

    // Anthropic has a "content" array with content blocks
    if let Some(content_value) = obj.get("content") {
        let mut combined_text = String::new();

        if let Some(content_array) = content_value.as_array() {
            // Handle array of content blocks
            for content_block in content_array {
                if let Some(block_obj) = content_block.as_object() {
                    if let Some(block_type) = block_obj.get("type").and_then(|v| v.as_str()) {
                        match block_type {
                            "text" => {
                                if let Some(text) = block_obj.get("text").and_then(|v| v.as_str()) {
                                    combined_text.push_str(text);
                                }
                            }
                            _ => {
                                // Handle other content types if needed
                            }
                        }
                    }
                }
            }
        } else if let Some(content_str) = content_value.as_str() {
            // Handle simple string content (some Anthropic responses)
            combined_text = content_str.to_string();
        }

        // Create an OpenAI-style choice
        let message = Message {
            role: MessageRole::Assistant,
            content: MessageContent::Text(combined_text),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
            metadata: HashMap::new(),
        };

        let choice = ResponseChoice {
            index: 0,
            message,
            finish_reason: obj
                .get("stop_reason")
                .and_then(|v| v.as_str())
                .map(|s| match s {
                    "end_turn" => "stop",
                    "max_tokens" => "length",
                    "stop_sequence" => "stop",
                    _ => s,
                })
                .map(|s| s.to_string()),
            logprobs: None,
        };

        choices.push(choice);
    }

    // Transform usage statistics
    let usage = if let Some(usage_obj) = obj.get("usage").and_then(|v| v.as_object()) {
        // Map Anthropic usage fields to OpenAI format
        let prompt_tokens = usage_obj
            .get("input_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let completion_tokens = usage_obj
            .get("output_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let total_tokens = prompt_tokens + completion_tokens;

        Some(crate::protocol::types::CompletionUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens,
        })
    } else {
        None
    };

    // Build the OpenAI-compatible response
    let response = ChatResponse {
        id,
        object: "chat.completion".to_string(),
        created,
        model,
        choices,
        usage,
        system_fingerprint: None,
        metadata: HashMap::new(),
    };

    Ok(response)
}

/// Transform an OpenAI JSON response to our internal format (it's already compatible)
pub fn openai_response_to_internal(response_json: Value) -> Result<ChatResponse, String> {
    serde_json::from_value(response_json)
        .map_err(|e| format!("Failed to parse OpenAI response: {}", e))
}

/// Transform request from internal format to provider-specific JSON
pub fn request_to_provider_json(request: ChatRequest, provider_name: &str) -> Value {
    match provider_name {
        "anthropic" => openai_request_to_anthropic_json(request),
        "openai" => {
            // OpenAI format is our internal format, just serialize
            serde_json::to_value(request).unwrap_or_else(|_| json!({}))
        }
        _ => serde_json::to_value(request).unwrap_or_else(|_| json!({})),
    }
}

/// Transform response from provider-specific JSON to internal format
pub fn provider_response_to_internal(
    response_json: Value,
    provider_name: &str,
) -> Result<ChatResponse, String> {
    match provider_name {
        "anthropic" => anthropic_response_to_openai(response_json),
        "openai" => openai_response_to_internal(response_json),
        _ => openai_response_to_internal(response_json),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_to_anthropic_transformation() {
        let mut request = ChatRequest::new(
            "gpt-4",
            vec![
                Message::system("You are a helpful assistant"),
                Message::user("Hello, what's the weather?"),
            ],
        );
        request.max_tokens = Some(100);
        request.temperature = Some(0.7);

        let transformed = openai_request_to_anthropic_json(request);
        let obj = transformed.as_object().unwrap();

        // Check that max_tokens is correctly named
        assert_eq!(obj.get("max_tokens").unwrap(), &json!(100));

        // Check that system message became a system field
        assert_eq!(
            obj.get("system").unwrap(),
            &json!("You are a helpful assistant")
        );

        // Check that messages array only has user message
        let messages = obj.get("messages").unwrap().as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "Hello, what's the weather?");

        // Check temperature (compare as float to handle precision)
        let temp = obj.get("temperature").unwrap().as_f64().unwrap();
        assert!(
            (temp - 0.7).abs() < 0.0001,
            "Temperature should be approximately 0.7"
        );
    }

    #[test]
    fn test_anthropic_response_transformation() {
        let anthropic_response = json!({
            "id": "msg_01XYZ",
            "model": "claude-3-opus-20240229",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "I don't have access to real-time weather data."
                }
            ],
            "stop_reason": "end_turn",
            "usage": {
                "input_tokens": 15,
                "output_tokens": 12
            }
        });

        let transformed = anthropic_response_to_openai(anthropic_response).unwrap();

        // Check basic fields
        assert_eq!(transformed.id, "msg_01XYZ");
        assert_eq!(transformed.model, "claude-3-opus-20240229");
        assert_eq!(transformed.object, "chat.completion");

        // Check choices
        assert_eq!(transformed.choices.len(), 1);
        let choice = &transformed.choices[0];
        assert_eq!(choice.index, 0);
        assert_eq!(choice.finish_reason, Some("stop".to_string()));

        // Check message content
        if let MessageContent::Text(text) = &choice.message.content {
            assert_eq!(text, "I don't have access to real-time weather data.");
        } else {
            panic!("Expected text content");
        }

        // Check usage transformation
        let usage = transformed.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 15);
        assert_eq!(usage.completion_tokens, 12);
        assert_eq!(usage.total_tokens, 27);
    }

    #[test]
    fn test_multimodal_content_transformation() {
        let request = ChatRequest::new(
            "gpt-4-vision",
            vec![Message {
                role: MessageRole::User,
                content: MessageContent::Parts(vec![
                    crate::protocol::types::ContentPart::Text {
                        text: "What's in this image?".to_string(),
                    },
                    crate::protocol::types::ContentPart::Image {
                        url: None,
                        base64: Some("data:image/jpeg;base64,/9j/4AAQ...".to_string()),
                    },
                ]),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                metadata: HashMap::new(),
            }],
        );

        let transformed = openai_request_to_anthropic_json(request);
        let obj = transformed.as_object().unwrap();

        let messages = obj.get("messages").unwrap().as_array().unwrap();
        assert_eq!(messages.len(), 1);

        let content = messages[0]["content"].as_array().unwrap();
        assert_eq!(content.len(), 2);

        // Check text block
        assert_eq!(content[0]["type"], "text");
        assert_eq!(content[0]["text"], "What's in this image?");

        // Check image block
        assert_eq!(content[1]["type"], "image");
        assert!(content[1]["source"]["data"]
            .as_str()
            .unwrap()
            .starts_with("/9j/4AAQ"));
    }
}
