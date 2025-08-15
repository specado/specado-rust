//! Week 2 Demo - Routing and Fallback Strategy
//!
//! This example demonstrates the routing and resilience features:
//! - Primary provider selection
//! - Automatic fallback on failure
//! - Metadata tracking for observability
//! - Error classification and retry logic
//!
//! Run with: cargo run --example week2_routing_demo

use specado_core::protocol::types::{ChatRequest, Message, ResponseFormat};
use specado_core::providers::{
    RoutingStrategy, RoutingBuilder, OpenAIProvider, AnthropicProvider,
};

#[tokio::main]
async fn main() {
    println!("\n🚀 Specado Week 2 Demo - Routing & Resilience\n");
    println!("=============================================\n");
    
    // Create a routing strategy with fallback
    println!("📋 Setting up routing strategy:");
    println!("  Primary: OpenAI (GPT-4)");
    println!("  Fallback: Anthropic (Claude)");
    println!();
    
    let router = RoutingBuilder::new()
        .primary(Box::new(OpenAIProvider::new()))
        .fallback(Box::new(AnthropicProvider::new()))
        .build()
        .unwrap();
    
    // Example 1: Successful primary provider
    println!("📝 Example 1: Normal Request (Primary Success)");
    println!("───────────────────────────────────────────────");
    
    let request1 = ChatRequest::new(
        "gpt-4",
        vec![
            Message::system("You are a helpful coding assistant"),
            Message::user("What is a monad in functional programming?"),
        ],
    );
    
    match router.route(request1).await {
        Ok(result) => {
            println!("✅ Request succeeded!");
            println!("  Provider used: {}", result.provider_used);
            println!("  Fallback used: {}", result.used_fallback);
            println!("  Attempts made: {}", result.attempts);
            
            if let Some(transform) = &result.transform_result {
                if transform.lossy {
                    println!("  ⚠️ Transformation was lossy:");
                    for reason in &transform.reasons {
                        println!("    - {}", reason);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
        }
    }
    
    println!();
    
    // Example 2: Primary fails with timeout, fallback succeeds
    println!("📝 Example 2: Timeout Scenario (Fallback Success)");
    println!("───────────────────────────────────────────────────");
    
    let request2 = ChatRequest::new(
        "timeout-test-model", // Simulated timeout
        vec![Message::user("This request will timeout on primary")],
    );
    
    match router.route(request2).await {
        Ok(result) => {
            println!("✅ Request succeeded after fallback!");
            println!("  Provider used: {}", result.provider_used);
            println!("  Fallback used: {}", result.used_fallback);
            println!("  Attempts made: {}", result.attempts);
            println!("  Primary error: {:?}", result.provider_errors.get("openai"));
            
            // Show detailed metadata
            if result.used_fallback {
                println!("\n📊 Fallback Metadata:");
                if let Some(primary) = result.metadata.get("primary_provider") {
                    println!("  Primary provider: {}", primary);
                }
                if let Some(fallback) = result.metadata.get("fallback_provider") {
                    println!("  Fallback provider: {}", fallback);
                }
                if let Some(errors) = result.metadata.get("provider_errors") {
                    println!("  Provider errors: {}", errors);
                }
            }
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
        }
    }
    
    println!();
    
    // Example 3: Rate limit scenario
    println!("📝 Example 3: Rate Limit (Automatic Fallback)");
    println!("──────────────────────────────────────────────");
    
    let request3 = ChatRequest::new(
        "rate-limit-test-model", // Simulated rate limit
        vec![Message::user("This will hit rate limits")],
    );
    
    match router.route(request3).await {
        Ok(result) => {
            println!("✅ Request succeeded via fallback!");
            println!("  Provider used: {}", result.provider_used);
            println!("  Fallback used: {}", result.used_fallback);
            println!("  Attempts made: {}", result.attempts);
            
            // Show which errors occurred
            println!("\n🔍 Error Details:");
            for (provider, error) in &result.provider_errors {
                println!("  {}: {}", provider, error);
            }
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
        }
    }
    
    println!();
    
    // Example 4: Non-retryable error (no fallback)
    println!("📝 Example 4: Authentication Error (No Fallback)");
    println!("─────────────────────────────────────────────────");
    
    let request4 = ChatRequest::new(
        "auth-error-test-model", // Simulated auth error
        vec![Message::user("This will fail authentication")],
    );
    
    match router.route(request4).await {
        Ok(_) => {
            println!("✅ Unexpected success");
        }
        Err(e) => {
            println!("❌ Request failed (as expected): {}", e);
            println!("  Note: Auth errors don't trigger fallback");
        }
    }
    
    println!();
    
    // Example 5: Complex request with transformation
    println!("📝 Example 5: Complex Request with Transformation");
    println!("──────────────────────────────────────────────────");
    
    let mut complex_request = ChatRequest::new(
        "server-error-test-model", // Will fail on primary
        vec![
            Message::system("You are an expert Rust developer"),
            Message::user("Explain async/await in Rust"),
        ],
    );
    complex_request.response_format = Some(ResponseFormat::JsonObject);
    
    match router.route(complex_request).await {
        Ok(result) => {
            println!("✅ Request succeeded via fallback!");
            println!("  Provider used: {}", result.provider_used);
            println!("  Attempts made: {}", result.attempts);
            
            if let Some(transform) = &result.transform_result {
                if transform.lossy {
                    println!("\n⚠️ Transformation Details:");
                    println!("  Lossy transformation occurred");
                    println!("  Reasons:");
                    for reason in &transform.reasons {
                        println!("    - {}", reason);
                    }
                }
            }
            
            println!("\n📊 Full Routing Metadata:");
            for (key, value) in &result.metadata {
                println!("  {}: {}", key, value);
            }
        }
        Err(e) => {
            println!("❌ Request failed: {}", e);
        }
    }
    
    println!("\n{}", "=".repeat(50));
    println!("\n✨ Key Benefits Demonstrated:");
    println!("  1. Automatic failover to backup providers");
    println!("  2. Smart error classification (retry vs fail-fast)");
    println!("  3. Complete observability via metadata");
    println!("  4. Transformation tracking across providers");
    println!("  5. No code changes needed for resilience!");
    
    println!("\n🎯 Next Steps:");
    println!("  - Add retry policies with exponential backoff");
    println!("  - Implement circuit breakers");
    println!("  - Add health checks for providers");
    println!("  - Create Python bindings for easy adoption");
    
    println!("\n🏁 Week 2 Demo Complete!");
}