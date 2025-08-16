//! Week 1 Demo Integration Test
//!
//! This test demonstrates the core functionality for Week 1:
//! - Provider abstraction with transformation
//! - Explicit lossiness tracking
//! - Same code working across different providers

use serde_json::json;
use specado_core::protocol::types::{ChatRequest, Message, ResponseFormat};
use specado_core::providers::transform_request;

#[test]
fn week_1_demo_core_magic() {
    println!("\n=== Week 1 Demo: The Core Magic ===\n");

    // Create an OpenAI format request with features that Anthropic doesn't support
    let mut request = ChatRequest::new(
        "gpt-4",
        vec![
            Message::system("You are a helpful assistant"),
            Message::user("Hello!"),
        ],
    );
    request.response_format = Some(ResponseFormat::JsonObject);

    println!("Original OpenAI Request:");
    println!("  Model: {}", request.model);
    println!(
        "  Messages: {} (including system role)",
        request.messages.len()
    );
    println!("  JSON Mode: Enabled");
    println!();

    // Transform for Anthropic (they don't support system role separately or JSON mode)
    let result = transform_request(request, "anthropic");

    println!("Transformed for Anthropic:");
    println!(
        "  Messages: {} (system merged into user)",
        result.transformed.messages.len()
    );
    println!("  JSON Mode: Removed");
    println!();

    println!("Lossiness Metadata:");
    println!("  Lossy: {}", result.lossy);
    println!("  Reasons: {:?}", result.reasons);
    println!();

    // Verify the transformation worked correctly
    assert!(result.lossy, "Transformation should be marked as lossy");
    assert_eq!(result.reasons.len(), 2, "Should have two lossiness reasons");
    assert!(result.reasons.contains(&"system_role_merged".to_string()));
    assert!(result
        .reasons
        .contains(&"json_mode_not_supported".to_string()));

    // Check the transformed message
    assert_eq!(result.transformed.messages.len(), 1);
    if let specado_core::protocol::types::MessageContent::Text(content) =
        &result.transformed.messages[0].content
    {
        assert!(content.contains("You are a helpful assistant"));
        assert!(content.contains("Hello!"));
        println!("Merged message content:");
        println!("  \"{}\"", content);
    }

    println!("\n✅ Week 1 Demo Complete: Same request works for different providers!");
    println!("✅ You KNOW when transformations lose fidelity!");
    println!("✅ No more silent failures from incompatible features!");
}

#[test]
fn week_1_demo_value_proposition() {
    println!("\n=== Demonstrating Value: Multi-Provider Compatibility ===\n");

    // A complex request that showcases various incompatibilities
    let mut request = ChatRequest::new(
        "gpt-4",
        vec![
            Message::system("You are an expert programmer. Be concise and accurate."),
            Message::user("Explain the concept of closures in Rust."),
            Message::assistant("Closures in Rust are anonymous functions that can capture variables from their enclosing scope..."),
            Message::user("Can you provide an example?"),
            Message::user("Specifically, show how to use move semantics."), // Consecutive user messages
        ],
    );
    request.response_format = Some(ResponseFormat::JsonObject);
    request.temperature = Some(0.7);
    request.max_tokens = Some(500);

    println!("Complex OpenAI Request:");
    println!("  - System message: ✓");
    println!("  - Multiple messages: {} messages", request.messages.len());
    println!("  - Consecutive user messages: ✓");
    println!("  - JSON response format: ✓");
    println!("  - Temperature: {:?}", request.temperature);
    println!("  - Max tokens: {:?}", request.max_tokens);
    println!();

    // Test transformation to different providers
    let providers = vec!["anthropic", "openai"];

    for provider in providers {
        println!("Transforming for {}:", provider);
        let result = transform_request(request.clone(), provider);

        if result.lossy {
            println!("  ⚠️  Transformation is LOSSY");
            println!("  Reasons for information loss:");
            for reason in &result.reasons {
                println!("    - {}", reason);
            }
        } else {
            println!("  ✅ Transformation is LOSSLESS");
        }

        println!(
            "  Final message count: {}",
            result.transformed.messages.len()
        );
        println!(
            "  Metadata added: {:?}",
            result.transformed.metadata.keys().collect::<Vec<_>>()
        );
        println!();
    }

    println!("🎯 Key Insight: Specado tells you EXACTLY what's incompatible!");
    println!("🎯 No more debugging mysterious API failures!");
}

#[test]
fn week_1_demo_python_interface_preview() {
    println!("\n=== Python Interface Preview (What Week 1 Enables) ===\n");

    // This test shows what the Python interface will look like
    // once we add the FFI bindings in Week 2

    println!("```python");
    println!("from specado_core import transform_request");
    println!();
    println!("# OpenAI format request");
    println!("request = {{");
    println!("    \"model\": \"gpt-4\",");
    println!("    \"messages\": [");
    println!("        {{\"role\": \"system\", \"content\": \"You are a helpful assistant\"}},");
    println!("        {{\"role\": \"user\", \"content\": \"Hello!\"}},");
    println!("    ],");
    println!("    \"response_format\": {{\"type\": \"json_object\"}}");
    println!("}}");
    println!();
    println!("# Transform for Anthropic");
    println!("result = transform_request(request, provider=\"anthropic\")");
    println!();
    println!("print(result[\"metadata\"])");
    println!("# Output: {{\"lossy\": true, \"reasons\": [\"system_role_merged\", \"json_mode_not_supported\"]}}");
    println!("```");
    println!();

    // Actually test the Rust version to ensure it works
    let mut request = ChatRequest::new(
        "gpt-4",
        vec![
            Message::system("You are a helpful assistant"),
            Message::user("Hello!"),
        ],
    );
    request.response_format = Some(ResponseFormat::JsonObject);

    let result = transform_request(request, "anthropic");

    assert!(result.lossy);
    assert_eq!(result.reasons.len(), 2);

    println!("✅ Core engine ready for Python bindings!");
}

#[test]
fn week_1_demo_edge_cases() {
    println!("\n=== Testing Edge Cases ===\n");

    // Test 1: Empty request
    let empty_request = ChatRequest::new("gpt-4", vec![]);
    let result = transform_request(empty_request, "anthropic");
    assert!(!result.lossy);
    println!("✅ Empty request handled gracefully");

    // Test 2: Only system messages
    let system_only = ChatRequest::new(
        "gpt-4",
        vec![Message::system("System 1"), Message::system("System 2")],
    );
    let result = transform_request(system_only, "anthropic");
    assert!(result.lossy);
    // System messages should be converted to a user message
    assert_eq!(result.transformed.messages.len(), 1);
    assert_eq!(
        result.transformed.messages[0].role,
        specado_core::protocol::types::MessageRole::User
    );
    println!("✅ System-only messages handled correctly");

    // Test 3: Already Anthropic-compatible request
    let compatible = ChatRequest::new(
        "claude-3",
        vec![
            Message::user("Question"),
            Message::assistant("Answer"),
            Message::user("Follow-up"),
        ],
    );
    let result = transform_request(compatible, "anthropic");
    assert!(!result.lossy);
    println!("✅ Already compatible requests pass through unchanged");

    println!("\nAll edge cases handled correctly!");
}
