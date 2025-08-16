//! Comprehensive tests for HTTP client functionality with mocking

use specado_core::http::client::HttpClient;
use specado_core::http::{CallKind, HttpExecutor, RequestOptions};
use specado_core::protocol::types::{
    ChatRequest, ChatResponse, Message, MessageContent, MessageRole, ResponseChoice,
};
use specado_core::providers::adapter::{Provider, ProviderCapabilities, ProviderType};
use specado_core::providers::routing::ProviderError;
use std::collections::HashMap;
use std::time::Duration;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Create a test provider for mocking
struct TestProvider {
    base_url: String,
    capabilities: ProviderCapabilities,
}

impl TestProvider {
    fn new(base_url: String) -> Self {
        Self {
            base_url,
            capabilities: ProviderCapabilities::default(),
        }
    }
}

impl Provider for TestProvider {
    fn name(&self) -> &str {
        "test"
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    fn transform_request(&self, request: ChatRequest) -> ChatRequest {
        request
    }

    fn transform_response(&self, response: ChatResponse) -> ChatResponse {
        response
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn endpoint(&self, call_kind: CallKind) -> &str {
        match call_kind {
            CallKind::Chat => "/chat/completions",
        }
    }

    fn headers(&self, api_key: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }
}

/// Helper to create a test chat request
fn test_chat_request() -> ChatRequest {
    ChatRequest {
        model: "test-model".to_string(),
        messages: vec![Message {
            role: MessageRole::User,
            content: MessageContent::Text("Test message".to_string()),
            name: None,
            function_call: None,
            tool_calls: None,
            tool_call_id: None,
            metadata: HashMap::new(),
        }],
        temperature: Some(0.7),
        max_tokens: Some(100),
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
    }
}

/// Helper to create a test chat response
fn test_chat_response() -> ChatResponse {
    ChatResponse {
        id: "test-id".to_string(),
        object: "chat.completion".to_string(),
        created: 1234567890,
        model: "test-model".to_string(),
        choices: vec![ResponseChoice {
            index: 0,
            message: Message {
                role: MessageRole::Assistant,
                content: MessageContent::Text("Test response".to_string()),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                metadata: HashMap::new(),
            },
            finish_reason: Some("stop".to_string()),
            logprobs: None,
        }],
        usage: None,
        system_fingerprint: None,
        metadata: HashMap::new(),
    }
}

/// Test successful JSON response
#[tokio::test]
async fn test_success_json_response() {
    // Set up mock server
    let mock_server = MockServer::start().await;

    // Create mock for successful response
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&test_chat_response())
                .insert_header("content-type", "application/json"),
        )
        .mount(&mock_server)
        .await;

    // Set API key in environment for test
    std::env::set_var("TEST_API_KEY", "test-key");

    // Create test provider with mock server URL
    let provider = TestProvider::new(mock_server.uri());

    // Create HTTP client and execute request
    let client = HttpClient::new().expect("Failed to create client");
    let request = test_chat_request();
    let options = RequestOptions::new(CallKind::Chat);

    let result = client.execute_json(&provider, request, options).await;

    assert!(
        result.is_ok(),
        "Expected successful response, got: {:?}",
        result
    );
    let response = result.unwrap();
    assert_eq!(response.id, "test-id");
    assert_eq!(response.choices.len(), 1);
}

/// Test deserialization failure with malformed JSON
#[tokio::test]
async fn test_deserialization_failure() {
    let mock_server = MockServer::start().await;

    // Create mock that returns invalid JSON
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("{ invalid json }")
                .insert_header("content-type", "application/json"),
        )
        .mount(&mock_server)
        .await;

    std::env::set_var("TEST_API_KEY", "test-key");

    let provider = TestProvider::new(mock_server.uri());
    let client = HttpClient::new().expect("Failed to create client");
    let request = test_chat_request();
    let options = RequestOptions::new(CallKind::Chat);

    let result = client.execute_json(&provider, request, options).await;

    assert!(result.is_err(), "Expected deserialization error");
    match result.unwrap_err() {
        ProviderError::Custom { code, message } => {
            assert_eq!(code, "PARSE_ERROR");
            assert!(message.contains("Invalid response format"));
        }
        _ => panic!("Expected Custom error with PARSE_ERROR code"),
    }
}

/// Test request timeout
#[tokio::test]
async fn test_request_timeout() {
    let mock_server = MockServer::start().await;

    // Create mock that delays response beyond timeout
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(2)) // Delay longer than timeout
                .set_body_json(&test_chat_response()),
        )
        .mount(&mock_server)
        .await;

    std::env::set_var("TEST_API_KEY", "test-key");

    let provider = TestProvider::new(mock_server.uri());

    // Create client with very short timeout
    let client = HttpClient::with_config(
        Duration::from_millis(100), // Very short connect timeout
        Duration::from_millis(500), // Very short request timeout
        10,
    )
    .expect("Failed to create client");

    let request = test_chat_request();
    let options = RequestOptions::new(CallKind::Chat).with_timeout(Duration::from_millis(500));

    let result = client.execute_json(&provider, request, options).await;

    assert!(result.is_err(), "Expected timeout error");
    assert!(matches!(result.unwrap_err(), ProviderError::Timeout));
}

/// Test content-type validation (reject non-JSON)
#[tokio::test]
async fn test_content_type_rejection() {
    let mock_server = MockServer::start().await;

    // Create mock that returns HTML instead of JSON
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("<html><body>Not JSON</body></html>")
                .insert_header("content-type", "text/html"),
        )
        .mount(&mock_server)
        .await;

    std::env::set_var("TEST_API_KEY", "test-key");

    let provider = TestProvider::new(mock_server.uri());
    let client = HttpClient::new().expect("Failed to create client");
    let request = test_chat_request();
    let options = RequestOptions::new(CallKind::Chat);

    let result = client.execute_json(&provider, request, options).await;

    assert!(result.is_err(), "Expected content-type validation error");
    match result.unwrap_err() {
        ProviderError::Custom { code, message } => {
            assert_eq!(code, "INVALID_CONTENT_TYPE");
            assert!(message.contains("Expected application/json"));
        }
        _ => panic!("Expected Custom error with INVALID_CONTENT_TYPE code"),
    }
}

/// Test response size limit
#[tokio::test]
async fn test_response_size_limit() {
    let mock_server = MockServer::start().await;

    // Create a very large response (over 10MB)
    let large_text = "x".repeat(11 * 1024 * 1024); // 11MB of text
    let mut large_response = test_chat_response();
    large_response.choices[0].message.content = MessageContent::Text(large_text);

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&large_response)
                .insert_header("content-type", "application/json")
                .insert_header("content-length", (11 * 1024 * 1024).to_string()),
        )
        .mount(&mock_server)
        .await;

    std::env::set_var("TEST_API_KEY", "test-key");

    let provider = TestProvider::new(mock_server.uri());
    let client = HttpClient::new().expect("Failed to create client");
    let request = test_chat_request();
    let options = RequestOptions::new(CallKind::Chat);

    let result = client.execute_json(&provider, request, options).await;

    assert!(result.is_err(), "Expected response size limit error");
    match result.unwrap_err() {
        ProviderError::Custom { code, message } => {
            assert_eq!(code, "RESPONSE_TOO_LARGE");
            assert!(message.contains("exceeds maximum"));
        }
        _ => panic!("Expected Custom error with RESPONSE_TOO_LARGE code"),
    }
}

/// Test 401 Unauthorized error mapping
#[tokio::test]
async fn test_401_unauthorized() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;

    std::env::set_var("TEST_API_KEY", "test-key");

    let provider = TestProvider::new(mock_server.uri());
    let client = HttpClient::new().expect("Failed to create client");
    let request = test_chat_request();
    let options = RequestOptions::new(CallKind::Chat);

    let result = client.execute_json(&provider, request, options).await;

    assert!(matches!(
        result.unwrap_err(),
        ProviderError::AuthenticationError
    ));
}

/// Test 429 Rate Limit with Retry-After header
#[tokio::test]
async fn test_429_with_retry_after_header() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "60"))
        .mount(&mock_server)
        .await;

    std::env::set_var("TEST_API_KEY", "test-key");

    let provider = TestProvider::new(mock_server.uri());
    let client = HttpClient::new().expect("Failed to create client");
    let request = test_chat_request();
    let options = RequestOptions::new(CallKind::Chat);

    let result = client.execute_json(&provider, request, options).await;

    match result.unwrap_err() {
        ProviderError::RateLimit { retry_after } => {
            assert!(retry_after.is_some());
            assert_eq!(retry_after.unwrap(), Duration::from_secs(60));
        }
        _ => panic!("Expected RateLimit error with retry_after"),
    }
}

/// Test 500 Internal Server Error
#[tokio::test]
async fn test_500_server_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(500).set_body_string("{\"error\": \"Internal server error\"}"),
        )
        .mount(&mock_server)
        .await;

    std::env::set_var("TEST_API_KEY", "test-key");

    let provider = TestProvider::new(mock_server.uri());
    let client = HttpClient::new().expect("Failed to create client");
    let request = test_chat_request();
    let options = RequestOptions::new(CallKind::Chat);

    let result = client.execute_json(&provider, request, options).await;

    match result.unwrap_err() {
        ProviderError::ServerError {
            status_code,
            message,
        } => {
            assert_eq!(status_code, 500);
            assert!(message.contains("Internal server error"));
        }
        _ => panic!("Expected ServerError"),
    }
}

/// Test request ID is included in error messages
#[tokio::test]
async fn test_request_id_in_errors() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(400))
        .mount(&mock_server)
        .await;

    std::env::set_var("TEST_API_KEY", "test-key");

    let provider = TestProvider::new(mock_server.uri());
    let client = HttpClient::new().expect("Failed to create client");
    let request = test_chat_request();
    let options = RequestOptions::new(CallKind::Chat);
    let request_id = options.request_id;

    let result = client.execute_json(&provider, request, options).await;

    match result.unwrap_err() {
        ProviderError::InvalidRequest { message } => {
            assert!(message.contains(&request_id.to_string()));
        }
        _ => panic!("Expected InvalidRequest error"),
    }
}

/// Test that provider-specific endpoints work correctly
#[tokio::test]
async fn test_provider_specific_endpoints() {
    // Test OpenAI endpoint
    let openai = ProviderType::OpenAI.create_provider();
    assert_eq!(openai.endpoint(CallKind::Chat), "/chat/completions");

    // Test Anthropic endpoint
    let anthropic = ProviderType::Anthropic.create_provider();
    assert_eq!(anthropic.endpoint(CallKind::Chat), "/messages");
}
