//! Tests for retry policy and error mapping functionality

use specado_core::protocol::types::{ChatRequest, Message};
use specado_core::providers::{
    AnthropicProvider, ErrorMapper, OpenAIProvider, ProviderError, RetryExecutor, RetryPolicy,
    RoutingBuilder, RoutingStrategy,
};
use std::time::Duration;
use tokio;

#[test]
fn test_retry_policy_configurations() {
    // Test default policy
    let default_policy = RetryPolicy::default();
    assert_eq!(default_policy.max_retries, 3);
    assert_eq!(default_policy.initial_delay_ms, 100);
    assert_eq!(default_policy.exponential_base, 2.0);
    assert!(default_policy.respect_retry_after);

    // Test aggressive policy
    let aggressive = RetryPolicy::aggressive();
    assert_eq!(aggressive.max_retries, 5);
    assert_eq!(aggressive.initial_delay_ms, 50);
    assert_eq!(aggressive.exponential_base, 1.5);

    // Test conservative policy
    let conservative = RetryPolicy::conservative();
    assert_eq!(conservative.max_retries, 2);
    assert_eq!(conservative.initial_delay_ms, 500);
    assert_eq!(conservative.exponential_base, 3.0);

    // Test no retry policy
    let no_retry = RetryPolicy::no_retry();
    assert_eq!(no_retry.max_retries, 0);
}

#[test]
fn test_exponential_backoff_without_jitter() {
    let policy = RetryPolicy {
        max_retries: 5,
        initial_delay_ms: 100,
        max_delay_ms: 2000,
        exponential_base: 2.0,
        jitter_factor: 0.0, // No jitter for predictable testing
        respect_retry_after: false,
        timeout_ms: None,
    };

    let error = ProviderError::Timeout;

    // Test exponential growth
    assert_eq!(policy.calculate_delay(0, &error).as_millis(), 100); // 100 * 2^0
    assert_eq!(policy.calculate_delay(1, &error).as_millis(), 200); // 100 * 2^1
    assert_eq!(policy.calculate_delay(2, &error).as_millis(), 400); // 100 * 2^2
    assert_eq!(policy.calculate_delay(3, &error).as_millis(), 800); // 100 * 2^3
    assert_eq!(policy.calculate_delay(4, &error).as_millis(), 1600); // 100 * 2^4
    assert_eq!(policy.calculate_delay(5, &error).as_millis(), 2000); // Capped at max
}

#[test]
fn test_jitter_application() {
    let policy = RetryPolicy {
        max_retries: 3,
        initial_delay_ms: 1000,
        max_delay_ms: 10000,
        exponential_base: 2.0,
        jitter_factor: 0.5, // 50% jitter
        respect_retry_after: false,
        timeout_ms: None,
    };

    let error = ProviderError::ServerError {
        status_code: 503,
        message: "Service unavailable".to_string(),
    };

    // With 50% jitter, delay should be within ±500ms of 1000ms
    let delay = policy.calculate_delay(0, &error);
    assert!(delay.as_millis() >= 500);
    assert!(delay.as_millis() <= 1500);
}

#[test]
fn test_retry_after_header_respected() {
    let policy = RetryPolicy {
        respect_retry_after: true,
        ..Default::default()
    };

    let error = ProviderError::RateLimit {
        retry_after: Some(Duration::from_secs(10)),
    };

    // Should use the retry_after value instead of calculated delay
    let delay = policy.calculate_delay(2, &error);
    assert_eq!(delay.as_secs(), 10);
}

#[test]
fn test_should_retry_logic() {
    let policy = RetryPolicy::new(2);

    // Test retryable errors
    let timeout = ProviderError::Timeout;
    assert!(policy.should_retry(&timeout, 0));
    assert!(policy.should_retry(&timeout, 1));
    assert!(!policy.should_retry(&timeout, 2)); // Exceeded max retries

    let rate_limit = ProviderError::RateLimit { retry_after: None };
    assert!(policy.should_retry(&rate_limit, 0));

    let server_error = ProviderError::ServerError {
        status_code: 503,
        message: "Unavailable".to_string(),
    };
    assert!(policy.should_retry(&server_error, 1));

    // Test non-retryable errors
    let auth_error = ProviderError::AuthenticationError;
    assert!(!policy.should_retry(&auth_error, 0));

    let invalid_request = ProviderError::InvalidRequest {
        message: "Bad request".to_string(),
    };
    assert!(!policy.should_retry(&invalid_request, 0));
}

#[tokio::test]
async fn test_retry_executor_success_on_retry() {
    let policy = RetryPolicy {
        max_retries: 3,
        initial_delay_ms: 10, // Short delay for testing
        max_delay_ms: 100,
        exponential_base: 2.0,
        jitter_factor: 0.0,
        respect_retry_after: false,
        timeout_ms: None,
    };

    let executor = RetryExecutor::new(policy);

    let mut attempt_count = 0;
    let result = executor
        .execute(|| {
            attempt_count += 1;
            async move {
                if attempt_count <= 2 {
                    Err(ProviderError::Timeout)
                } else {
                    Ok("Success".to_string())
                }
            }
        })
        .await;

    assert!(result.result.is_some());
    assert_eq!(result.result.unwrap(), "Success");
    assert_eq!(result.attempts, 2); // Failed twice, succeeded on third
    assert_eq!(result.error_history.len(), 2);
}

#[tokio::test]
async fn test_retry_executor_all_attempts_fail() {
    let policy = RetryPolicy {
        max_retries: 2,
        initial_delay_ms: 10,
        max_delay_ms: 100,
        exponential_base: 2.0,
        jitter_factor: 0.0,
        respect_retry_after: false,
        timeout_ms: None,
    };

    let executor = RetryExecutor::new(policy);

    let result = executor
        .execute(|| async {
            Err::<String, ProviderError>(ProviderError::ServerError {
                status_code: 500,
                message: "Internal error".to_string(),
            })
        })
        .await;

    assert!(result.result.is_none());
    assert!(result.final_error.is_some());
    assert_eq!(result.attempts, 2); // Initial + 2 retries
    assert_eq!(result.error_history.len(), 3);
}

#[tokio::test]
async fn test_retry_executor_non_retryable_error() {
    let policy = RetryPolicy::default();
    let executor = RetryExecutor::new(policy);

    let result = executor
        .execute(|| async { Err::<String, ProviderError>(ProviderError::AuthenticationError) })
        .await;

    assert!(result.result.is_none());
    assert_eq!(result.attempts, 0); // No retries for auth errors
    assert_eq!(result.error_history.len(), 1);
    assert!(matches!(
        result.final_error,
        Some(ProviderError::AuthenticationError)
    ));
}

#[test]
fn test_error_mapper_http_status_codes() {
    // Test authentication errors
    let error = ErrorMapper::from_status_code(401, None);
    assert!(matches!(error, ProviderError::AuthenticationError));

    let error = ErrorMapper::from_status_code(403, None);
    assert!(matches!(error, ProviderError::AuthenticationError));

    // Test rate limiting
    let error = ErrorMapper::from_status_code(429, None);
    assert!(matches!(error, ProviderError::RateLimit { .. }));

    // Test invalid request
    let error = ErrorMapper::from_status_code(400, Some("Invalid JSON"));
    assert!(matches!(error, ProviderError::InvalidRequest { .. }));

    // Test model not found
    let error = ErrorMapper::from_status_code(404, Some("Model not found"));
    assert!(matches!(error, ProviderError::ModelNotAvailable { .. }));

    // Test timeouts
    let error = ErrorMapper::from_status_code(408, None);
    assert!(matches!(error, ProviderError::Timeout));

    let error = ErrorMapper::from_status_code(504, None);
    assert!(matches!(error, ProviderError::Timeout));

    // Test server errors
    let error = ErrorMapper::from_status_code(500, Some("Internal error"));
    assert!(matches!(
        error,
        ProviderError::ServerError {
            status_code: 500,
            ..
        }
    ));

    let error = ErrorMapper::from_status_code(503, None);
    assert!(matches!(
        error,
        ProviderError::ServerError {
            status_code: 503,
            ..
        }
    ));
}

#[test]
fn test_error_mapper_provider_patterns() {
    // Test common patterns
    let error = ErrorMapper::from_provider_error("openai", "Rate limit exceeded");
    assert!(matches!(error, ProviderError::RateLimit { .. }));

    let error = ErrorMapper::from_provider_error("anthropic", "Request timed out");
    assert!(matches!(error, ProviderError::Timeout));

    let error = ErrorMapper::from_provider_error("openai", "Unauthorized access");
    assert!(matches!(error, ProviderError::AuthenticationError));

    // Test provider-specific patterns
    let error = ErrorMapper::from_provider_error("openai", "insufficient_quota");
    assert!(matches!(error, ProviderError::RateLimit { .. }));

    let error = ErrorMapper::from_provider_error("anthropic", "overloaded");
    assert!(matches!(error, ProviderError::ServerError { .. }));

    let error = ErrorMapper::from_provider_error("anthropic", "invalid_api_key");
    assert!(matches!(error, ProviderError::AuthenticationError));

    // Test unknown errors
    let error = ErrorMapper::from_provider_error("custom", "Unknown error");
    assert!(matches!(error, ProviderError::Custom { .. }));
}

#[tokio::test]
async fn test_routing_with_retry_policy() {
    // Create router with custom retry policy
    let router = RoutingBuilder::new()
        .primary(Box::new(OpenAIProvider::new()))
        .fallback(Box::new(AnthropicProvider::new()))
        .build()
        .unwrap()
        .with_retry_policy(RetryPolicy {
            max_retries: 2,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            exponential_base: 2.0,
            jitter_factor: 0.0,
            respect_retry_after: false,
            timeout_ms: Some(1000),
        });

    // Test with a normal request (should succeed without retry)
    let request = ChatRequest::new("gpt-4", vec![Message::user("Hello")]);

    let result = router.route(request).await.unwrap();
    assert_eq!(result.provider_used, "openai");
    assert!(!result.used_fallback);
}

#[tokio::test]
async fn test_routing_without_retry_policy() {
    // Create router without retry policy
    let router = RoutingBuilder::new()
        .primary(Box::new(OpenAIProvider::new()))
        .fallback(Box::new(AnthropicProvider::new()))
        .build()
        .unwrap()
        .without_retry();

    // Test that it doesn't retry on failure
    let request = ChatRequest::new(
        "timeout-test-model",
        vec![Message::user("This will timeout")],
    );

    let result = router.route(request).await.unwrap();
    // Should fallback immediately without retries
    assert_eq!(result.provider_used, "anthropic");
    assert!(result.used_fallback);
}

#[test]
fn test_retry_delay_calculation_with_cap() {
    let policy = RetryPolicy {
        max_retries: 10,
        initial_delay_ms: 100,
        max_delay_ms: 1000, // Cap at 1 second
        exponential_base: 2.0,
        jitter_factor: 0.0,
        respect_retry_after: false,
        timeout_ms: None,
    };

    let error = ProviderError::Timeout;

    // Test that delays are capped
    for attempt in 0..10 {
        let delay = policy.calculate_delay(attempt, &error);
        assert!(delay.as_millis() <= 1000);
    }
}
