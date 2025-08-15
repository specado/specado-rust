//! Week 2 Demo - Retry Policy and Error Mapping
//!
//! This example demonstrates the retry policy features:
//! - Exponential backoff with jitter
//! - Respecting retry-after headers
//! - Error classification and mapping
//! - Integration with routing strategy
//!
//! Run with: cargo run --example week2_retry_demo

use specado_core::protocol::types::{ChatRequest, Message};
use specado_core::providers::{
    RoutingStrategy, RoutingBuilder, RetryPolicy, ErrorMapper, ProviderError,
    OpenAIProvider, AnthropicProvider,
};
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("\n🚀 Specado Week 2 Demo - Retry Policy & Error Mapping\n");
    println!("======================================================\n");
    
    // Example 1: Default retry policy
    println!("📝 Example 1: Default Retry Policy");
    println!("──────────────────────────────────");
    
    let default_policy = RetryPolicy::default();
    println!("Default configuration:");
    println!("  Max retries: {}", default_policy.max_retries);
    println!("  Initial delay: {}ms", default_policy.initial_delay_ms);
    println!("  Max delay: {}ms", default_policy.max_delay_ms);
    println!("  Exponential base: {}", default_policy.exponential_base);
    println!("  Jitter factor: {}", default_policy.jitter_factor);
    println!();
    
    // Example 2: Different retry strategies
    println!("📝 Example 2: Retry Strategy Presets");
    println!("────────────────────────────────────");
    
    let aggressive = RetryPolicy::aggressive();
    println!("Aggressive (for critical operations):");
    println!("  Max retries: {} | Initial delay: {}ms", 
        aggressive.max_retries, aggressive.initial_delay_ms);
    
    let conservative = RetryPolicy::conservative();
    println!("Conservative (to minimize load):");
    println!("  Max retries: {} | Initial delay: {}ms", 
        conservative.max_retries, conservative.initial_delay_ms);
    
    let no_retry = RetryPolicy::no_retry();
    println!("No retry (fail fast):");
    println!("  Max retries: {}", no_retry.max_retries);
    println!();
    
    // Example 3: Exponential backoff calculation
    println!("📝 Example 3: Exponential Backoff Visualization");
    println!("────────────────────────────────────────────────");
    
    let policy = RetryPolicy {
        max_retries: 5,
        initial_delay_ms: 100,
        max_delay_ms: 5000,
        exponential_base: 2.0,
        jitter_factor: 0.0, // No jitter for clear visualization
        respect_retry_after: false,
        timeout_ms: None,
    };
    
    let error = ProviderError::Timeout;
    println!("Backoff progression (base 2.0, no jitter):");
    for attempt in 0..6 {
        let delay = policy.calculate_delay(attempt, &error);
        println!("  Attempt {}: {}ms delay", attempt + 1, delay.as_millis());
    }
    println!();
    
    // Example 4: Jitter effect
    println!("📝 Example 4: Jitter for Load Distribution");
    println!("───────────────────────────────────────────");
    
    let jittery_policy = RetryPolicy {
        initial_delay_ms: 1000,
        jitter_factor: 0.3, // 30% jitter
        ..Default::default()
    };
    
    println!("With 30% jitter on 1000ms base delay:");
    for i in 0..3 {
        let delay = jittery_policy.calculate_delay(0, &error);
        println!("  Sample {}: {}ms", i + 1, delay.as_millis());
    }
    println!("  (Notice the variation around 1000ms ± 300ms)");
    println!();
    
    // Example 5: Respecting retry-after headers
    println!("📝 Example 5: Respecting Retry-After Headers");
    println!("─────────────────────────────────────────────");
    
    let rate_limit = ProviderError::RateLimit {
        retry_after: Some(Duration::from_secs(5)),
    };
    
    let respectful_policy = RetryPolicy {
        respect_retry_after: true,
        ..Default::default()
    };
    
    let delay = respectful_policy.calculate_delay(0, &rate_limit);
    println!("Rate limit with retry-after: 5 seconds");
    println!("Calculated delay: {} seconds", delay.as_secs());
    println!("  ✅ Respects the server's requested delay");
    println!();
    
    // Example 6: Error classification
    println!("📝 Example 6: Error Classification");
    println!("──────────────────────────────────");
    
    let errors = vec![
        ProviderError::Timeout,
        ProviderError::RateLimit { retry_after: None },
        ProviderError::ServerError { status_code: 503, message: "Unavailable".to_string() },
        ProviderError::AuthenticationError,
        ProviderError::InvalidRequest { message: "Bad JSON".to_string() },
    ];
    
    for error in &errors {
        println!("  {:?}", error);
        println!("    Retryable: {}", if error.is_retryable() { "✅ Yes" } else { "❌ No" });
    }
    println!();
    
    // Example 7: Error mapping from HTTP status codes
    println!("📝 Example 7: HTTP Status Code Mapping");
    println!("───────────────────────────────────────");
    
    let status_codes = vec![
        (401, "Unauthorized"),
        (429, "Too Many Requests"),
        (500, "Internal Server Error"),
        (503, "Service Unavailable"),
        (408, "Request Timeout"),
    ];
    
    for (code, description) in status_codes {
        let error = ErrorMapper::from_status_code(code, Some(description));
        println!("  {} {}: → {:?}", code, description, error);
    }
    println!();
    
    // Example 8: Router with retry policy
    println!("📝 Example 8: Router with Custom Retry Policy");
    println!("──────────────────────────────────────────────");
    
    let custom_policy = RetryPolicy {
        max_retries: 2,
        initial_delay_ms: 50,
        max_delay_ms: 500,
        exponential_base: 1.5,
        jitter_factor: 0.1,
        respect_retry_after: true,
        timeout_ms: Some(5000), // 5 second total timeout
    };
    
    let router = RoutingBuilder::new()
        .primary(Box::new(OpenAIProvider::new()))
        .fallback(Box::new(AnthropicProvider::new()))
        .build()
        .unwrap()
        .with_retry_policy(custom_policy);
    
    println!("Router configuration:");
    println!("  Primary: OpenAI");
    println!("  Fallback: Anthropic");
    println!("  Retry policy: 2 retries, 50ms initial, 1.5x backoff");
    println!();
    
    // Test with a request
    let request = ChatRequest::new(
        "gpt-4",
        vec![Message::user("Test message")],
    );
    
    match router.route(request).await {
        Ok(result) => {
            println!("✅ Request succeeded!");
            println!("  Provider used: {}", result.provider_used);
            println!("  Total attempts: {}", result.attempts);
            
            if let Some(retry_delay) = result.metadata.get("retry_delay_ms") {
                println!("  Total retry delay: {}ms", retry_delay);
            }
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
        }
    }
    
    println!();
    
    // Example 9: Simulating transient failures
    println!("📝 Example 9: Handling Transient Failures");
    println!("──────────────────────────────────────────");
    
    // This will trigger retries on the primary before falling back
    let flaky_request = ChatRequest::new(
        "timeout-test-model", // Simulates timeout on primary
        vec![Message::user("This might fail transiently")],
    );
    
    match router.route(flaky_request).await {
        Ok(result) => {
            println!("✅ Request eventually succeeded!");
            println!("  Provider used: {}", result.provider_used);
            println!("  Used fallback: {}", result.used_fallback);
            println!("  Total attempts: {}", result.attempts);
            
            if !result.provider_errors.is_empty() {
                println!("\n  Errors encountered:");
                for (provider, error) in &result.provider_errors {
                    println!("    {}: {}", provider, error);
                }
            }
        }
        Err(e) => {
            println!("❌ Request failed after all retries: {}", e);
        }
    }
    
    println!("\n{}", "=".repeat(50));
    println!("\n✨ Key Benefits Demonstrated:");
    println!("  1. Configurable retry policies for different scenarios");
    println!("  2. Exponential backoff with jitter to prevent thundering herd");
    println!("  3. Respects server retry-after headers");
    println!("  4. Smart error classification (retryable vs non-retryable)");
    println!("  5. Seamless integration with routing strategy");
    println!("  6. Comprehensive error mapping from HTTP codes");
    
    println!("\n🎯 Production Ready Features:");
    println!("  - Prevents cascading failures");
    println!("  - Reduces unnecessary load on struggling services");
    println!("  - Improves success rate for transient failures");
    println!("  - Provides detailed metrics for observability");
    
    println!("\n🏁 Week 2 Retry Demo Complete!");
}