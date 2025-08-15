//! Tests for protocol builder enhancements

use specado_core::protocol::*;

#[test]
fn test_message_builder_with_parts() {
    // Test creating a message with multimodal parts
    let parts = vec![
        ContentPart::Text {
            text: "Analyze this image:".to_string(),
        },
        ContentPart::Image {
            url: Some("https://example.com/chart.png".to_string()),
            base64: None,
        },
    ];
    
    let msg = MessageBuilder::with_parts(MessageRole::User, parts.clone()).build();
    
    assert_eq!(msg.role, MessageRole::User);
    match msg.content {
        MessageContent::Parts(p) => assert_eq!(p.len(), 2),
        _ => panic!("Expected Parts content"),
    }
}

#[test]
fn test_message_builder_with_parts_and_metadata() {
    let parts = vec![
        ContentPart::Text {
            text: "Process this audio:".to_string(),
        },
        ContentPart::Audio {
            url: None,
            base64: Some("base64data".to_string()),
        },
    ];
    
    let msg = MessageBuilder::with_parts(MessageRole::User, parts)
        .with_name("user_123")
        .with_metadata("timestamp", serde_json::json!(1234567890))
        .build();
    
    assert_eq!(msg.role, MessageRole::User);
    assert_eq!(msg.name, Some("user_123".to_string()));
    assert!(msg.metadata.contains_key("timestamp"));
    match msg.content {
        MessageContent::Parts(p) => {
            assert_eq!(p.len(), 2);
            match &p[1] {
                ContentPart::Audio { base64, .. } => {
                    assert_eq!(base64.as_ref().unwrap(), "base64data");
                }
                _ => panic!("Expected Audio part"),
            }
        }
        _ => panic!("Expected Parts content"),
    }
}

#[test]
fn test_chat_request_builder_with_top_p() {
    let request = ChatRequest::new("gpt-4", vec![Message::user("Hello")])
        .with_temperature(0.7)
        .with_top_p(0.9)
        .with_max_tokens(1000);
    
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.top_p, Some(0.9));
    assert_eq!(request.max_tokens, Some(1000));
}

#[test]
fn test_chat_request_builder_with_stop_sequences() {
    let request = ChatRequest::new("gpt-4", vec![Message::user("Count to 10")])
        .with_stop(vec!["5".to_string(), "STOP".to_string()]);
    
    assert!(request.stop.is_some());
    let stops = request.stop.unwrap();
    assert_eq!(stops.len(), 2);
    assert!(stops.contains(&"5".to_string()));
    assert!(stops.contains(&"STOP".to_string()));
}

#[test]
fn test_chat_request_builder_with_stop_sequence() {
    let request = ChatRequest::new("gpt-4", vec![Message::user("Generate text")])
        .with_stop_sequence("END")
        .with_stop_sequence("DONE");
    
    assert!(request.stop.is_some());
    let stops = request.stop.unwrap();
    assert_eq!(stops.len(), 2);
    assert!(stops.contains(&"END".to_string()));
    assert!(stops.contains(&"DONE".to_string()));
}

#[test]
fn test_completion_usage_u32() {
    // Test that CompletionUsage uses u32
    let usage = CompletionUsage {
        prompt_tokens: 100,
        completion_tokens: 50,
        total_tokens: 150,
    };
    
    // This would fail to compile if the fields weren't u32
    let _prompt: u32 = usage.prompt_tokens;
    let _completion: u32 = usage.completion_tokens;
    let _total: u32 = usage.total_tokens;
    
    assert_eq!(usage.prompt_tokens, 100u32);
    assert_eq!(usage.completion_tokens, 50u32);
    assert_eq!(usage.total_tokens, 150u32);
}

#[test]
fn test_combined_builder_methods() {
    let request = ChatRequest::new("claude-3", vec![
        Message::system("You are a helpful assistant"),
        Message::user("Write a poem"),
    ])
    .with_temperature(0.8)
    .with_max_tokens(500)
    .with_top_p(0.95)
    .with_stop_sequence("END")
    .with_stop_sequence("THE END")
    .with_streaming(true);
    
    assert_eq!(request.model, "claude-3");
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.temperature, Some(0.8));
    assert_eq!(request.max_tokens, Some(500));
    assert_eq!(request.top_p, Some(0.95));
    assert_eq!(request.stream, Some(true));
    
    let stops = request.stop.unwrap();
    assert_eq!(stops.len(), 2);
}

#[test]
fn test_message_builder_combinations() {
    // Test that original text builder still works
    let text_msg = MessageBuilder::new(MessageRole::Assistant, "Hello")
        .with_name("assistant_1")
        .build();
    
    assert_eq!(text_msg.role, MessageRole::Assistant);
    assert_eq!(text_msg.content.as_text(), Some("Hello"));
    
    // Test parts builder with all options
    let parts_msg = MessageBuilder::with_parts(
        MessageRole::User,
        vec![ContentPart::Text { text: "Test".to_string() }]
    )
    .with_name("user_1")
    .with_metadata("custom", serde_json::json!(true))
    .build();
    
    assert_eq!(parts_msg.role, MessageRole::User);
    assert_eq!(parts_msg.name, Some("user_1".to_string()));
    assert!(parts_msg.metadata.contains_key("custom"));
}