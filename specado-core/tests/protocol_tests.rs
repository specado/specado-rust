//! Tests for the protocol module

use serde_json;
use specado_core::protocol::*;

#[test]
fn test_message_construction() {
    // Test system message
    let sys_msg = Message::system("You are a helpful assistant");
    assert_eq!(sys_msg.role, MessageRole::System);
    assert_eq!(
        sys_msg.content.as_text(),
        Some("You are a helpful assistant")
    );

    // Test user message
    let user_msg = Message::user("Hello!");
    assert_eq!(user_msg.role, MessageRole::User);
    assert_eq!(user_msg.content.as_text(), Some("Hello!"));

    // Test assistant message
    let asst_msg = Message::assistant("Hi there! How can I help?");
    assert_eq!(asst_msg.role, MessageRole::Assistant);
    assert_eq!(
        asst_msg.content.as_text(),
        Some("Hi there! How can I help?")
    );

    // Test function message
    let func_msg = Message::function("get_weather", "{\"temp\": 72}");
    assert_eq!(func_msg.role, MessageRole::Function);
    assert_eq!(func_msg.name, Some("get_weather".to_string()));

    // Test tool message
    let tool_msg = Message::tool("call_123", "Tool result");
    assert_eq!(tool_msg.role, MessageRole::Tool);
    assert_eq!(tool_msg.tool_call_id, Some("call_123".to_string()));
}

#[test]
fn test_message_builder() {
    let msg = MessageBuilder::new(MessageRole::Assistant, "Response")
        .with_name("assistant_1")
        .with_function_call("calculate", "{\"x\": 5, \"y\": 3}")
        .with_metadata("provider", serde_json::json!("openai"))
        .build();

    assert_eq!(msg.role, MessageRole::Assistant);
    assert_eq!(msg.name, Some("assistant_1".to_string()));
    assert!(msg.function_call.is_some());
    let fc = msg.function_call.unwrap();
    assert_eq!(fc.name, "calculate");
    assert_eq!(fc.arguments, "{\"x\": 5, \"y\": 3}");
    assert_eq!(
        msg.metadata.get("provider"),
        Some(&serde_json::json!("openai"))
    );
}

#[test]
fn test_chat_request_construction() {
    let messages = vec![
        Message::system("You are helpful"),
        Message::user("What is 2+2?"),
    ];

    let request = ChatRequest::new("gpt-4", messages.clone())
        .with_temperature(0.7)
        .with_max_tokens(1000)
        .with_streaming(true);

    assert_eq!(request.model, "gpt-4");
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.max_tokens, Some(1000));
    assert_eq!(request.stream, Some(true));
    assert!(request.stream_options.is_some());
}

#[test]
fn test_message_serialization() {
    let msg = Message::user("Test message");
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Message = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.role, MessageRole::User);
    assert_eq!(parsed.content.as_text(), Some("Test message"));
}

#[test]
fn test_chat_request_serialization() {
    let request = ChatRequest::new("gpt-3.5-turbo", vec![Message::user("Hello")])
        .with_temperature(0.5)
        .with_max_tokens(500);

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"model\":\"gpt-3.5-turbo\""));
    assert!(json.contains("\"temperature\":0.5"));
    assert!(json.contains("\"max_tokens\":500"));

    // Test that optional fields are omitted
    assert!(!json.contains("\"top_p\""));
    assert!(!json.contains("\"stream\""));

    let parsed: ChatRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.model, "gpt-3.5-turbo");
    assert_eq!(parsed.temperature, Some(0.5));
    assert_eq!(parsed.max_tokens, Some(500));
}

#[test]
fn test_chat_response_structure() {
    let response = ChatResponse {
        id: "chatcmpl-123".to_string(),
        object: "chat.completion".to_string(),
        created: 1234567890,
        model: "gpt-4".to_string(),
        choices: vec![ResponseChoice {
            index: 0,
            message: Message::assistant("The answer is 4"),
            finish_reason: Some("stop".to_string()),
            logprobs: None,
        }],
        usage: Some(CompletionUsage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        }),
        system_fingerprint: Some("fp_123".to_string()),
        metadata: Default::default(),
    };

    assert_eq!(response.id, "chatcmpl-123");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(
        response.choices[0].message.content.as_text(),
        Some("The answer is 4")
    );
    assert_eq!(response.usage.as_ref().unwrap().total_tokens, 15);
}

#[test]
fn test_streaming_chunk() {
    let chunk = ChatStreamChunk {
        id: "chatcmpl-123".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1234567890,
        model: "gpt-4".to_string(),
        choices: vec![StreamChoice {
            index: 0,
            delta: MessageDelta {
                role: Some(MessageRole::Assistant),
                content: Some("The".to_string()),
                function_call: None,
                tool_calls: None,
            },
            finish_reason: None,
            logprobs: None,
        }],
        usage: None,
        system_fingerprint: None,
    };

    assert_eq!(chunk.choices[0].delta.content, Some("The".to_string()));
    assert_eq!(chunk.choices[0].delta.role, Some(MessageRole::Assistant));
}

#[test]
fn test_response_type_variants() {
    // Test complete response variant
    let complete = ResponseType::Complete(ChatResponse {
        id: "resp-1".to_string(),
        object: "chat.completion".to_string(),
        created: 1234567890,
        model: "gpt-4".to_string(),
        choices: vec![],
        usage: None,
        system_fingerprint: None,
        metadata: Default::default(),
    });

    match complete {
        ResponseType::Complete(resp) => assert_eq!(resp.id, "resp-1"),
        _ => panic!("Expected Complete variant"),
    }

    // Test stream chunk variant
    let stream = ResponseType::Stream(ChatStreamChunk {
        id: "chunk-1".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1234567890,
        model: "gpt-4".to_string(),
        choices: vec![],
        usage: None,
        system_fingerprint: None,
    });

    match stream {
        ResponseType::Stream(chunk) => assert_eq!(chunk.id, "chunk-1"),
        _ => panic!("Expected Stream variant"),
    }
}

#[test]
fn test_message_content_variants() {
    // Test text content
    let text_content = MessageContent::Text("Hello world".to_string());
    assert_eq!(text_content.as_text(), Some("Hello world"));
    assert!(!text_content.is_empty());

    // Test parts content
    let parts_content = MessageContent::Parts(vec![
        ContentPart::Text {
            text: "Check out this image:".to_string(),
        },
        ContentPart::Image {
            url: Some("https://example.com/image.png".to_string()),
            base64: None,
        },
    ]);
    assert_eq!(parts_content.as_text(), None);
    assert!(!parts_content.is_empty());

    // Test empty content
    let empty_text = MessageContent::Text(String::new());
    assert!(empty_text.is_empty());

    let empty_parts = MessageContent::Parts(vec![]);
    assert!(empty_parts.is_empty());
}

#[test]
fn test_tool_and_function_calls() {
    // Test function call in message
    let mut msg = Message::assistant("");
    msg.function_call = Some(FunctionCall {
        name: "get_weather".to_string(),
        arguments: "{\"location\": \"San Francisco\"}".to_string(),
    });

    assert!(msg.function_call.is_some());
    let fc = msg.function_call.as_ref().unwrap();
    assert_eq!(fc.name, "get_weather");

    // Test tool calls in message
    msg.tool_calls = Some(vec![ToolCall {
        id: "tool_1".to_string(),
        tool_type: "function".to_string(),
        function: FunctionCall {
            name: "search".to_string(),
            arguments: "{\"query\": \"rust programming\"}".to_string(),
        },
    }]);

    assert!(msg.tool_calls.is_some());
    let tools = msg.tool_calls.as_ref().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].id, "tool_1");
}

#[test]
fn test_request_with_tools() {
    let mut request = ChatRequest::new("gpt-4", vec![Message::user("Search for info")]);

    // Add tool definitions
    request.tools = Some(vec![ToolDefinition {
        tool_type: "function".to_string(),
        function: FunctionDefinition {
            name: "search".to_string(),
            description: Some("Search the web".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                }
            })),
        },
    }]);

    // Set tool choice
    request.tool_choice = Some(ToolChoice::Mode("auto".to_string()));

    assert!(request.tools.is_some());
    assert_eq!(request.tools.as_ref().unwrap().len(), 1);
    assert!(request.tool_choice.is_some());
}

#[test]
fn test_response_format() {
    let mut request = ChatRequest::new("gpt-4", vec![Message::user("Generate JSON")]);

    // Test JSON object format
    request.response_format = Some(ResponseFormat::JsonObject);
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"response_format\""));

    // Test JSON schema format
    request.response_format = Some(ResponseFormat::JsonSchema {
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            }
        }),
    });

    let json2 = serde_json::to_string(&request).unwrap();
    assert!(json2.contains("\"json_schema\""));
    assert!(json2.contains("\"schema\""));
}

#[test]
fn test_streaming_delta_accumulation() {
    // Simulate accumulating streaming deltas
    let mut accumulated_content = String::new();

    let deltas = vec![
        MessageDelta {
            role: Some(MessageRole::Assistant),
            content: Some("The ".to_string()),
            function_call: None,
            tool_calls: None,
        },
        MessageDelta {
            role: None,
            content: Some("answer ".to_string()),
            function_call: None,
            tool_calls: None,
        },
        MessageDelta {
            role: None,
            content: Some("is 42.".to_string()),
            function_call: None,
            tool_calls: None,
        },
    ];

    for delta in deltas {
        if let Some(content) = delta.content {
            accumulated_content.push_str(&content);
        }
    }

    assert_eq!(accumulated_content, "The answer is 42.");
}

#[test]
fn test_metadata_fields() {
    // Test message metadata
    let mut msg = Message::user("Test");
    msg.metadata
        .insert("custom_field".to_string(), serde_json::json!("value"));

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"metadata\""));
    assert!(json.contains("\"custom_field\""));

    // Test request metadata
    let mut request = ChatRequest::new("model", vec![]);
    request.metadata.insert(
        "provider_specific".to_string(),
        serde_json::json!({"option": true}),
    );

    let req_json = serde_json::to_string(&request).unwrap();
    assert!(req_json.contains("\"provider_specific\""));

    // Test response metadata
    let mut response = ChatResponse {
        id: "id".to_string(),
        object: "chat.completion".to_string(),
        created: 0,
        model: "model".to_string(),
        choices: vec![],
        usage: None,
        system_fingerprint: None,
        metadata: Default::default(),
    };
    response
        .metadata
        .insert("latency_ms".to_string(), serde_json::json!(123));

    let resp_json = serde_json::to_string(&response).unwrap();
    assert!(resp_json.contains("\"latency_ms\""));
}

#[test]
fn test_into_message_trait() {
    // Test string conversion
    let msg1: Message = "Hello".into_message();
    assert_eq!(msg1.role, MessageRole::User);
    assert_eq!(msg1.content.as_text(), Some("Hello"));

    // Test String conversion
    let msg2: Message = String::from("World").into_message();
    assert_eq!(msg2.role, MessageRole::User);
    assert_eq!(msg2.content.as_text(), Some("World"));

    // Test Message pass-through
    let original = Message::system("System");
    let msg3: Message = original.clone().into_message();
    assert_eq!(msg3.role, MessageRole::System);
    assert_eq!(msg3.content.as_text(), Some("System"));
}

#[test]
fn test_multimodal_content() {
    let parts = vec![
        ContentPart::Text {
            text: "What's in this image?".to_string(),
        },
        ContentPart::Image {
            url: Some("https://example.com/image.jpg".to_string()),
            base64: None,
        },
        ContentPart::Audio {
            url: None,
            base64: Some("base64audiodata".to_string()),
        },
    ];

    let content = MessageContent::Parts(parts);
    let msg = Message {
        role: MessageRole::User,
        content,
        name: None,
        function_call: None,
        tool_calls: None,
        tool_call_id: None,
        metadata: Default::default(),
    };

    // Serialize and verify structure
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"type\":\"text\""));
    assert!(json.contains("\"type\":\"image\""));
    assert!(json.contains("\"type\":\"audio\""));

    // Deserialize back
    let parsed: Message = serde_json::from_str(&json).unwrap();
    match parsed.content {
        MessageContent::Parts(parts) => assert_eq!(parts.len(), 3),
        _ => panic!("Expected Parts variant"),
    }
}
