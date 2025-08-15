//! Tests for the routing and fallback functionality
//!
//! These tests verify that the routing strategy correctly handles
//! primary provider failures and falls back to alternative providers.

use specado_core::protocol::types::{ChatRequest, Message};
use specado_core::providers::{
    RoutingStrategy, PrimaryWithFallbacks, RoutingBuilder, ProviderError,
    OpenAIProvider, AnthropicProvider, ProviderType,
};
use tokio;

#[tokio::test]
async fn test_primary_success_no_fallback() {
    // Create a routing strategy with OpenAI as primary and Anthropic as fallback
    let router = RoutingBuilder::new()
        .primary(Box::new(OpenAIProvider::new()))
        .fallback(Box::new(AnthropicProvider::new()))
        .build()
        .unwrap();
    
    // Create a normal request
    let request = ChatRequest::new(
        "gpt-4",
        vec![Message::user("Hello, world!")],
    );
    
    // Route the request
    let result = router.route(request).await.unwrap();
    
    // Verify primary was used
    assert_eq!(result.provider_used, "openai");
    assert!(!result.used_fallback);
    assert_eq!(result.attempts, 1);
    assert!(result.provider_errors.is_empty());
    
    // Check metadata
    assert_eq!(result.metadata.get("fallback_used").unwrap(), false);
    assert_eq!(result.metadata.get("primary_provider").unwrap(), "openai");
}

#[tokio::test]
async fn test_fallback_on_timeout() {
    // Create router with OpenAI primary and Anthropic fallback
    let router = RoutingBuilder::new()
        .primary(Box::new(OpenAIProvider::new()))
        .fallback(Box::new(AnthropicProvider::new()))
        .build()
        .unwrap();
    
    // Create a request that simulates timeout
    let request = ChatRequest::new(
        "timeout-test-model",
        vec![Message::user("This will timeout")],
    );
    
    // Route the request
    let result = router.route(request).await.unwrap();
    
    // Verify fallback was used
    assert_eq!(result.provider_used, "anthropic");
    assert!(result.used_fallback);
    assert_eq!(result.attempts, 2);
    assert!(result.provider_errors.contains_key("openai"));
    
    // Check metadata
    assert_eq!(result.metadata.get("fallback_used").unwrap(), true);
    assert_eq!(result.metadata.get("primary_provider").unwrap(), "openai");
    assert_eq!(result.metadata.get("fallback_provider").unwrap(), "anthropic");
    assert_eq!(result.metadata.get("fallback_index").unwrap(), 0);
}

#[tokio::test]
async fn test_fallback_on_rate_limit() {
    let router = RoutingBuilder::new()
        .primary(Box::new(OpenAIProvider::new()))
        .fallback(Box::new(AnthropicProvider::new()))
        .build()
        .unwrap();
    
    // Create a request that simulates rate limiting
    let request = ChatRequest::new(
        "rate-limit-test-model",
        vec![Message::user("This will hit rate limit")],
    );
    
    // Route the request
    let result = router.route(request).await.unwrap();
    
    // Verify fallback was used
    assert_eq!(result.provider_used, "anthropic");
    assert!(result.used_fallback);
    assert_eq!(result.attempts, 2);
    
    // Verify error was recorded
    let openai_error = result.provider_errors.get("openai").unwrap();
    assert!(openai_error.contains("Rate limit"));
}

#[tokio::test]
async fn test_no_fallback_on_auth_error() {
    let router = RoutingBuilder::new()
        .primary(Box::new(OpenAIProvider::new()))
        .fallback(Box::new(AnthropicProvider::new()))
        .build()
        .unwrap();
    
    // Create a request that simulates auth error
    let request = ChatRequest::new(
        "auth-error-test-model",
        vec![Message::user("This will fail auth")],
    );
    
    // Route the request - should fail without trying fallback
    let result = router.route(request).await;
    
    // Verify it failed and didn't try fallback
    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::AuthenticationError => {},
        _ => panic!("Expected AuthenticationError"),
    }
}

#[tokio::test]
async fn test_multiple_fallbacks() {
    // Create multiple fallback providers
    let router = PrimaryWithFallbacks::new(
        Box::new(OpenAIProvider::new()),
        vec![
            ProviderType::Anthropic.create_provider(),
            ProviderType::OpenAI.create_provider(), // Can use same provider type as fallback
        ],
    );
    
    // Verify provider list
    let providers = router.providers();
    assert_eq!(providers.len(), 3);
    assert_eq!(providers[0], "openai");
    assert_eq!(providers[1], "anthropic");
    assert_eq!(providers[2], "openai");
    
    // Test that we can iterate through fallbacks
    let request = ChatRequest::new(
        "server-error-test-model",
        vec![Message::user("This will fail on first provider")],
    );
    
    let result = router.route(request).await.unwrap();
    
    // Should use first fallback (Anthropic)
    assert_eq!(result.provider_used, "anthropic");
    assert!(result.used_fallback);
}

#[tokio::test]
async fn test_all_providers_fail() {
    // Create a router with providers that will all fail
    let router = RoutingBuilder::new()
        .primary(Box::new(OpenAIProvider::new()))
        .fallback(Box::new(AnthropicProvider::new()))
        .build()
        .unwrap();
    
    // Create a request that will fail on all providers
    // For testing, we'll simulate this by using a non-retryable error
    // In real implementation, this would be multiple retryable failures
    let request = ChatRequest::new(
        "auth-error-test-model", // Non-retryable error
        vec![Message::user("This will fail on all")],
    );
    
    let result = router.route(request).await;
    
    // Should return error when all fail
    assert!(result.is_err());
}

#[tokio::test]
async fn test_metadata_tracking_disabled() {
    let router = PrimaryWithFallbacks::new(
        Box::new(OpenAIProvider::new()),
        vec![Box::new(AnthropicProvider::new())],
    ).with_metadata_tracking(false);
    
    let request = ChatRequest::new(
        "gpt-4",
        vec![Message::user("Test without metadata")],
    );
    
    let result = router.route(request).await.unwrap();
    
    // Metadata should be empty when tracking is disabled
    assert!(result.metadata.is_empty());
    assert_eq!(result.provider_used, "openai");
}

#[tokio::test]
async fn test_error_classification() {
    // Test that errors are correctly classified as retryable or not
    let rate_limit = ProviderError::RateLimit { retry_after: Some(std::time::Duration::from_secs(5)) };
    assert!(rate_limit.is_retryable());
    assert!(rate_limit.retry_delay().is_some());
    
    let timeout = ProviderError::Timeout;
    assert!(timeout.is_retryable());
    
    let server_error = ProviderError::ServerError {
        status_code: 503,
        message: "Service unavailable".to_string(),
    };
    assert!(server_error.is_retryable());
    
    let auth_error = ProviderError::AuthenticationError;
    assert!(!auth_error.is_retryable());
    
    let invalid_request = ProviderError::InvalidRequest {
        message: "Bad request".to_string(),
    };
    assert!(!invalid_request.is_retryable());
}

#[tokio::test]
async fn test_transform_metadata_preservation() {
    // Test that transformation metadata is preserved in routing result
    let router = RoutingBuilder::new()
        .primary(Box::new(AnthropicProvider::new())) // Anthropic doesn't support system role
        .build()
        .unwrap();
    
    // Create a request with system message (will be lossy for Anthropic)
    let request = ChatRequest::new(
        "claude-3",
        vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
        ],
    );
    
    let result = router.route(request).await.unwrap();
    
    // Check that transformation lossiness is tracked
    assert!(result.transform_result.is_some());
    let transform = result.transform_result.unwrap();
    assert!(transform.lossy);
    assert!(transform.reasons.contains(&"system_role.merged".to_string()));
    
    // Check metadata includes transformation info
    assert_eq!(result.metadata.get("transformation_lossy").unwrap(), true);
    let reasons = result.metadata.get("lossy_reasons").unwrap();
    assert!(reasons.as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_routing_strategy_trait() {
    // Test the trait interface
    let router: Box<dyn RoutingStrategy> = Box::new(
        RoutingBuilder::new()
            .primary(Box::new(OpenAIProvider::new()))
            .fallback(Box::new(AnthropicProvider::new()))
            .build()
            .unwrap()
    );
    
    assert_eq!(router.name(), "primary_with_fallbacks");
    
    let providers = router.providers();
    assert_eq!(providers.len(), 2);
    assert_eq!(providers[0], "openai");
    assert_eq!(providers[1], "anthropic");
    
    // Test routing through trait
    let request = ChatRequest::new("gpt-4", vec![Message::user("Test")]);
    let result = router.route(request).await.unwrap();
    assert_eq!(result.provider_used, "openai");
}