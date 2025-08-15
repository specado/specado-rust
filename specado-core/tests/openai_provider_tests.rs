//! Tests for the OpenAI provider implementation

use specado_core::protocol::{ChatRequest, Message, MessageRole, ContentPart, MessageContent};
use specado_core::providers::openai::OpenAIProvider;
use specado_core::providers::{Provider, ProviderConfig};

#[test]
fn test_request_conversion() {
    use specado_core::providers::openai::converter::to_openai_request;
    
    let request = ChatRequest::new("gpt-4", vec![
        Message::system("You are a helpful assistant"),
        Message::user("Hello, how are you?"),
    ])
    .with_temperature(0.7)
    .with_max_tokens(100)
    .with_top_p(0.9);
    
    let openai_req = to_openai_request(&request);
    
    assert_eq!(openai_req.model, "gpt-4");
    assert_eq!(openai_req.messages.len(), 2);
    assert_eq!(openai_req.messages[0].role, "system");
    assert_eq!(openai_req.messages[1].role, "user");
    assert_eq!(openai_req.temperature, Some(0.7));
    assert_eq!(openai_req.max_tokens, Some(100));
    assert_eq!(openai_req.top_p, Some(0.9));
}

#[test]
fn test_multimodal_message_conversion() {
    use specado_core::providers::openai::converter::to_openai_request;
    
    let message = Message {
        role: MessageRole::User,
        content: MessageContent::Parts(vec![
            ContentPart::Text { text: "What's in this image?".to_string() },
            ContentPart::Image {
                url: Some("https://example.com/image.jpg".to_string()),
                base64: None,
            },
        ]),
        name: None,
        function_call: None,
        tool_calls: None,
        tool_call_id: None,
        metadata: Default::default(),
    };
    
    let request = ChatRequest::new("gpt-4-vision", vec![message]);
    let openai_req = to_openai_request(&request);
    
    assert_eq!(openai_req.messages.len(), 1);
    // The conversion should handle multimodal content properly
    match &openai_req.messages[0].content {
        Some(specado_core::providers::openai::types::OpenAIContent::Parts(parts)) => {
            assert_eq!(parts.len(), 2);
        }
        _ => panic!("Expected Parts content"),
    }
}

#[test]
fn test_response_conversion() {
    use specado_core::providers::openai::converter::from_openai_response;
    use specado_core::providers::openai::types::{
        OpenAIResponse, OpenAIChoice, OpenAIMessage, OpenAIContent, OpenAIUsage,
    };
    
    let openai_resp = OpenAIResponse {
        id: "chatcmpl-123".to_string(),
        object: "chat.completion".to_string(),
        created: 1234567890,
        model: "gpt-4".to_string(),
        choices: vec![
            OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: Some(OpenAIContent::Text("Hello! I'm doing well.".to_string())),
                    name: None,
                    function_call: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: Some("stop".to_string()),
                logprobs: None,
            }
        ],
        usage: Some(OpenAIUsage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        }),
        system_fingerprint: Some("fp_123".to_string()),
    };
    
    let response = from_openai_response(openai_resp);
    
    assert_eq!(response.id, "chatcmpl-123");
    assert_eq!(response.model, "gpt-4");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].message.role, MessageRole::Assistant);
    assert_eq!(
        response.choices[0].message.content.as_text(),
        Some("Hello! I'm doing well.")
    );
    assert!(response.usage.is_some());
    assert_eq!(response.usage.unwrap().total_tokens, 15);
}

#[test]
fn test_stream_chunk_conversion() {
    use specado_core::providers::openai::converter::from_openai_stream_chunk;
    use specado_core::providers::openai::types::{
        OpenAIStreamChunk, OpenAIStreamChoice, OpenAIDelta,
    };
    
    let chunk = OpenAIStreamChunk {
        id: "chatcmpl-123".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1234567890,
        model: "gpt-4".to_string(),
        choices: vec![
            OpenAIStreamChoice {
                index: 0,
                delta: OpenAIDelta {
                    role: Some("assistant".to_string()),
                    content: Some("Hello".to_string()),
                    function_call: None,
                    tool_calls: None,
                },
                finish_reason: None,
                logprobs: None,
            }
        ],
        usage: None,
        system_fingerprint: None,
    };
    
    let stream_chunk = from_openai_stream_chunk(chunk);
    
    assert_eq!(stream_chunk.id, "chatcmpl-123");
    assert_eq!(stream_chunk.choices.len(), 1);
    assert_eq!(stream_chunk.choices[0].delta.role, Some(MessageRole::Assistant));
    assert_eq!(stream_chunk.choices[0].delta.content, Some("Hello".to_string()));
}

#[test]
fn test_error_handling() {
    use specado_core::providers::openai::types::{OpenAIError, OpenAIErrorDetail};
    
    let error = OpenAIError {
        error: OpenAIErrorDetail {
            message: "Invalid API key".to_string(),
            error_type: "invalid_api_key".to_string(),
            code: Some("401".to_string()),
            param: None,
        },
    };
    
    let json = serde_json::to_string(&error).unwrap();
    let parsed: OpenAIError = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.error.message, "Invalid API key");
    assert_eq!(parsed.error.error_type, "invalid_api_key");
}

#[test]
fn test_provider_config() {
    let config = ProviderConfig {
        api_key: "sk-test".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        organization_id: Some("org-123".to_string()),
        timeout_secs: 30,
        max_retries: 3,
        track_rate_limits: true,
    };
    
    assert_eq!(config.api_key, "sk-test");
    assert_eq!(config.base_url, "https://api.openai.com/v1");
    assert_eq!(config.organization_id, Some("org-123".to_string()));
    assert_eq!(config.timeout_secs, 30);
    assert_eq!(config.max_retries, 3);
    assert!(config.track_rate_limits);
}

#[test]
fn test_rate_limit_tracker() {
    use specado_core::providers::RateLimitTracker;
    use reqwest::header::{HeaderMap, HeaderValue};
    
    let tracker = RateLimitTracker::new();
    
    // Simulate headers from OpenAI
    let mut headers = HeaderMap::new();
    headers.insert("x-ratelimit-limit-requests", HeaderValue::from_static("60"));
    headers.insert("x-ratelimit-remaining-requests", HeaderValue::from_static("59"));
    headers.insert("x-ratelimit-limit-tokens", HeaderValue::from_static("10000"));
    headers.insert("x-ratelimit-remaining-tokens", HeaderValue::from_static("9500"));
    
    tracker.update_from_headers(&headers);
    
    let info = tracker.get_info();
    assert_eq!(info.requests_per_minute, Some(60));
    assert_eq!(info.requests_remaining, Some(59));
    assert_eq!(info.tokens_per_minute, Some(10000));
    assert_eq!(info.tokens_remaining, Some(9500));
    
    // Record a request
    tracker.record_request(100);
    
    let info = tracker.get_info();
    assert_eq!(info.requests_used, 1);
    assert_eq!(info.tokens_used, 100);
    assert_eq!(info.requests_remaining, Some(58));
    assert_eq!(info.tokens_remaining, Some(9400));
}

#[test]
fn test_rate_limit_should_wait() {
    use specado_core::providers::RateLimitTracker;
    use reqwest::header::{HeaderMap, HeaderValue};
    use chrono::Utc;
    
    let tracker = RateLimitTracker::new();
    
    // Simulate being at the limit
    let mut headers = HeaderMap::new();
    headers.insert("x-ratelimit-remaining-requests", HeaderValue::from_static("0"));
    
    // Set reset time to 10 seconds from now
    let reset_time = Utc::now().timestamp() + 10;
    headers.insert(
        "x-ratelimit-reset-requests",
        HeaderValue::from_str(&reset_time.to_string()).unwrap(),
    );
    
    tracker.update_from_headers(&headers);
    
    // Should need to wait
    let wait_duration = tracker.should_wait();
    assert!(wait_duration.is_some());
    
    if let Some(duration) = wait_duration {
        // Should be around 10 seconds (allow some variance)
        assert!(duration.as_secs() >= 9 && duration.as_secs() <= 11);
    }
}

#[test]
fn test_tool_and_function_conversion() {
    use specado_core::protocol::{ToolDefinition, FunctionDefinition};
    use specado_core::providers::openai::converter::to_openai_request;
    
    let mut request = ChatRequest::new("gpt-4", vec![Message::user("Call a function")]);
    request.tools = Some(vec![
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "get_weather".to_string(),
                description: Some("Get the weather".to_string()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    }
                })),
            },
        }
    ]);
    
    let openai_req = to_openai_request(&request);
    
    assert!(openai_req.tools.is_some());
    let tools = openai_req.tools.unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].function.name, "get_weather");
    assert_eq!(tools[0].function.description, Some("Get the weather".to_string()));
}

#[tokio::test]
async fn test_provider_creation() {
    let config = ProviderConfig {
        api_key: "sk-test".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        organization_id: None,
        timeout_secs: 30,
        max_retries: 3,
        track_rate_limits: true,
    };
    
    let provider = OpenAIProvider::new(config);
    assert!(provider.is_ok());
    
    let provider = provider.unwrap();
    assert_eq!(provider.name(), "openai");
}

// Integration test (requires valid API key to run)
#[tokio::test]
#[ignore] // Ignore by default since it requires API key
async fn test_real_api_call() {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    
    let config = ProviderConfig {
        api_key,
        base_url: "https://api.openai.com/v1".to_string(),
        organization_id: None,
        timeout_secs: 30,
        max_retries: 3,
        track_rate_limits: true,
    };
    
    let provider = OpenAIProvider::new(config).unwrap();
    
    // Test health check
    let health = provider.health_check().await;
    assert!(health.is_ok());
    
    // Test chat completion
    let request = ChatRequest::new("gpt-3.5-turbo", vec![
        Message::system("You are a helpful assistant"),
        Message::user("Say 'test successful' and nothing else"),
    ])
    .with_max_tokens(10);
    
    let response = provider.chat_completion(request).await;
    assert!(response.is_ok());
    
    let response = response.unwrap();
    assert!(!response.choices.is_empty());
    
    // Check rate limit info
    let rate_info = provider.rate_limit_info();
    assert!(rate_info.is_some());
}