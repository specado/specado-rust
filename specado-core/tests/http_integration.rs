//! Integration tests for HTTP client functionality

use specado_core::http::client::HttpClient;
use specado_core::http::{CallKind, RequestOptions};
use specado_core::protocol::types::{ChatRequest, Message, MessageContent, MessageRole};
use specado_core::providers::adapter::ProviderType;
use specado_core::providers::routing::{PrimaryWithFallbacks, ProviderError, RoutingStrategy};
use std::collections::HashMap;
use std::env;

/// Test that the HTTP client can be created successfully
#[test]
fn test_http_client_creation() {
    let client = HttpClient::new();
    assert!(client.is_ok(), "Failed to create HTTP client");
}

/// Test that request options can be created and configured
#[test]
fn test_request_options() {
    let options = RequestOptions::new(CallKind::Chat);
    assert_eq!(options.call_kind, CallKind::Chat);
    assert!(!options.request_id.to_string().is_empty());

    let options_with_timeout = options.with_timeout(std::time::Duration::from_secs(60));
    assert_eq!(
        options_with_timeout.timeout,
        std::time::Duration::from_secs(60)
    );
}

/// Test that CallKind provides correct endpoints
#[test]
fn test_call_kind_endpoints() {
    assert_eq!(CallKind::Chat.endpoint(), "/chat/completions");
}

/// Test routing with HTTP client (requires API key to run)
#[tokio::test]
#[ignore] // Ignore by default since it requires API keys
async fn test_routing_with_http_client() {
    // This test requires OPENAI_API_KEY to be set
    if env::var("OPENAI_API_KEY").is_err() {
        eprintln!("Skipping test: OPENAI_API_KEY not set");
        return;
    }

    // Create providers
    let primary = ProviderType::OpenAI.create_provider();
    let fallbacks = vec![];

    // Create router with HTTP client
    let router = PrimaryWithFallbacks::new(primary, fallbacks);

    // Create a simple chat request
    let request = ChatRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: MessageContent::Text("Say 'Hello, Specado!' and nothing else.".to_string()),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
            metadata: HashMap::new(),
        }],
        temperature: Some(0.7),
        max_tokens: Some(10),
        top_p: None,
        n: None,
        stop: None,
        stream: Some(false),
        stream_options: None,
        presence_penalty: None,
        frequency_penalty: None,
        user: None,
        response_format: None,
        tools: None,
        tool_choice: None,
        seed: None,
        metadata: HashMap::new(),
    };

    // Execute the request
    match router.route(request).await {
        Ok(result) => {
            assert!(result.response.is_some(), "Expected response to be present");
            assert_eq!(result.provider_used, "openai");
            assert!(!result.used_fallback);
            println!("Successfully made HTTP request to OpenAI");
        }
        Err(e) => {
            // If it's an auth error, that's expected without a valid key
            if matches!(e, ProviderError::AuthenticationError) {
                println!("Authentication error (expected without valid API key)");
            } else {
                panic!("Unexpected error: {:?}", e);
            }
        }
    }
}

/// Test error mapping for common HTTP status codes
#[test]
fn test_error_mapping() {
    use reqwest::StatusCode;
    use specado_core::http::error::map_http_error;
    use uuid::Uuid;

    let request_id = Uuid::new_v4();

    // Test 401 Unauthorized
    let error = map_http_error(StatusCode::UNAUTHORIZED, None, None, request_id);
    assert!(matches!(error, ProviderError::AuthenticationError));

    // Test 429 Too Many Requests
    let error = map_http_error(StatusCode::TOO_MANY_REQUESTS, None, None, request_id);
    assert!(matches!(error, ProviderError::RateLimit { .. }));

    // Test 500 Internal Server Error
    let error = map_http_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        Some("Server error".to_string()),
        request_id,
    );
    assert!(matches!(error, ProviderError::ServerError { .. }));

    // Test 400 Bad Request
    let error = map_http_error(
        StatusCode::BAD_REQUEST,
        None,
        Some("Invalid request".to_string()),
        request_id,
    );
    assert!(matches!(error, ProviderError::InvalidRequest { .. }));
}

/// Test that the router initializes with HTTP client
#[test]
fn test_router_with_http_client() {
    let primary = ProviderType::OpenAI.create_provider();
    let fallback = ProviderType::Anthropic.create_provider();

    let router = PrimaryWithFallbacks::new(primary, vec![fallback]);

    assert_eq!(router.name(), "primary_with_fallbacks");
    assert_eq!(router.providers(), vec!["openai", "anthropic"]);
}
