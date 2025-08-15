//! Comprehensive tests for the transformation engine
//!
//! Tests the core transformation logic including lossiness tracking
//! and provider-specific adaptations.

use specado_core::protocol::types::{ChatRequest, Message, MessageRole, ResponseFormat};
use specado_core::providers::{TransformationEngine, OpenAIProvider, AnthropicProvider, transform_request};
use serde_json::json;

#[test]
fn test_openai_to_anthropic_system_role_transformation() {
    // Create a request with a system message (supported by OpenAI, not by Anthropic)
    let request = ChatRequest::new(
        "gpt-4",
        vec![
            Message::system("You are a helpful assistant"),
            Message::user("Hello, how are you?"),
        ],
    );
    
    // Transform from OpenAI to Anthropic
    let result = transform_request(request, "anthropic");
    
    // Check that transformation is marked as lossy
    assert!(result.lossy);
    assert!(result.reasons.contains(&"system_role.merged".to_string()));
    
    // Check that system message was merged into user message
    assert_eq!(result.transformed.messages.len(), 1);
    assert_eq!(result.transformed.messages[0].role, MessageRole::User);
    
    // Verify the content was merged correctly
    if let specado_core::protocol::types::MessageContent::Text(content) = &result.transformed.messages[0].content {
        assert!(content.contains("You are a helpful assistant"));
        assert!(content.contains("Hello, how are you?"));
    } else {
        panic!("Expected text content");
    }
    
    // Check metadata
    assert_eq!(result.transformed.metadata.get("lossy"), Some(&json!(true)));
    assert!(result.transformed.metadata.contains_key("lossy_reasons"));
}

#[test]
fn test_json_mode_not_supported() {
    // Create a request with JSON mode (supported by OpenAI, not by Anthropic)
    let mut request = ChatRequest::new(
        "gpt-4",
        vec![Message::user("Generate a JSON object")],
    );
    request.response_format = Some(ResponseFormat::JsonObject);
    
    // Transform to Anthropic
    let result = transform_request(request, "anthropic");
    
    // Check lossiness
    assert!(result.lossy);
    assert!(result.reasons.contains(&"response_format.json_mode.unsupported".to_string()));
    
    // Check that JSON mode was removed
    assert_eq!(result.transformed.response_format, None);
}

#[test]
fn test_no_transformation_needed_openai_to_openai() {
    let request = ChatRequest::new(
        "gpt-4",
        vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
        ],
    );
    
    // Transform from OpenAI to OpenAI (no changes expected)
    let result = transform_request(request.clone(), "openai");
    
    // Should not be lossy
    assert!(!result.lossy);
    assert!(result.reasons.is_empty());
    
    // Messages should be unchanged
    assert_eq!(result.transformed.messages.len(), request.messages.len());
}

#[test]
fn test_multiple_lossiness_reasons() {
    // Create a request with multiple incompatible features
    let mut request = ChatRequest::new(
        "gpt-4",
        vec![
            Message::system("System prompt"),
            Message::user("First question"),
            Message::user("Second question"), // Consecutive same role
        ],
    );
    request.response_format = Some(ResponseFormat::JsonObject);
    request.tools = Some(vec![]); // Function calling
    
    // Transform to Anthropic
    let result = transform_request(request, "anthropic");
    
    // Should have multiple lossiness reasons
    assert!(result.lossy);
    assert!(result.reasons.len() >= 3);
    assert!(result.reasons.contains(&"system_role.merged".to_string()));
    assert!(result.reasons.contains(&"response_format.json_mode.unsupported".to_string()));
    assert!(result.reasons.contains(&"messages.consecutive_same_role.unsupported".to_string()));
}

#[test]
fn test_consecutive_same_role_merging() {
    let request = ChatRequest::new(
        "gpt-4",
        vec![
            Message::user("First message"),
            Message::user("Second message"),
            Message::assistant("Response"),
            Message::user("Third message"),
        ],
    );
    
    // Transform to Anthropic (doesn't support consecutive same role)
    let result = transform_request(request, "anthropic");
    
    assert!(result.lossy);
    assert!(result.reasons.contains(&"messages.consecutive_same_role.unsupported".to_string()));
    
    // Should have merged the consecutive user messages
    assert_eq!(result.transformed.messages.len(), 3);
    
    // First message should be the merged user messages
    if let specado_core::protocol::types::MessageContent::Text(content) = &result.transformed.messages[0].content {
        assert!(content.contains("First message"));
        assert!(content.contains("Second message"));
    }
}

#[test]
fn test_parameter_removal_tracking() {
    let mut request = ChatRequest::new(
        "claude-3-opus",
        vec![Message::user("Hello")],
    );
    
    // Add parameters that might not be supported
    request.temperature = Some(0.7);
    request.top_p = Some(0.9);
    request.stream = Some(true);
    
    // Both OpenAI and Anthropic support these in MVP, so should be no loss
    let result = transform_request(request.clone(), "anthropic");
    assert!(!result.lossy);
    
    // But if we had a provider that didn't support streaming...
    // (This is where future providers would be tested)
}

#[test]
fn test_transform_engine_direct_usage() {
    
    let source = Box::new(OpenAIProvider::new());
    let target = Box::new(AnthropicProvider::new());
    let engine = TransformationEngine::new(source, target);
    
    let request = ChatRequest::new(
        "gpt-4",
        vec![
            Message::system("Be concise"),
            Message::user("What is 2+2?"),
        ],
    );
    
    let result = engine.transform_request(request);
    
    assert!(result.lossy);
    assert_eq!(result.transformed.messages.len(), 1);
    assert_eq!(result.transformed.messages[0].role, MessageRole::User);
}

#[test]
fn test_metadata_preservation() {
    let mut request = ChatRequest::new(
        "gpt-4",
        vec![Message::user("Hello")],
    );
    
    // Add custom metadata
    request.metadata.insert("request_id".to_string(), json!("test-123"));
    request.metadata.insert("user_id".to_string(), json!("user-456"));
    
    let result = transform_request(request, "anthropic");
    
    // Original metadata should be preserved
    assert_eq!(
        result.transformed.metadata.get("request_id"),
        Some(&json!("test-123"))
    );
    assert_eq!(
        result.transformed.metadata.get("user_id"),
        Some(&json!("user-456"))
    );
}

#[test]
fn test_empty_request_handling() {
    let request = ChatRequest::new("gpt-4", vec![]);
    
    let result = transform_request(request, "anthropic");
    
    // Should handle empty messages gracefully
    assert!(!result.lossy);
    assert!(result.transformed.messages.is_empty());
}