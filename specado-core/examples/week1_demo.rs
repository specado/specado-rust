//! Week 1 Demo - Core Transformation Engine
//!
//! This example demonstrates the core value proposition of Specado:
//! - Provider abstraction with transformation
//! - Explicit lossiness tracking  
//! - No more silent failures from incompatible features
//!
//! Run with: cargo run --example week1_demo

use specado_core::protocol::types::{ChatRequest, Message, ResponseFormat};
use specado_core::providers::transform_request;

fn main() {
    println!("\n🚀 Specado Week 1 Demo - The Core Magic\n");
    println!("=========================================\n");
    
    // Create an OpenAI-style request with features that not all providers support
    let mut request = ChatRequest::new(
        "gpt-4",
        vec![
            Message::system("You are a helpful assistant specialized in Rust programming"),
            Message::user("Explain closures in Rust with an example"),
        ],
    );
    
    // Add JSON mode (supported by OpenAI, not by Anthropic)
    request.response_format = Some(ResponseFormat::JsonObject);
    
    println!("📝 Original Request (OpenAI Format):");
    println!("  - Model: {}", request.model);
    println!("  - Messages: {} (including system role)", request.messages.len());
    println!("  - JSON Mode: Enabled");
    println!("  - System Message: \"{}\"", 
        if let specado_core::protocol::types::MessageContent::Text(text) = &request.messages[0].content {
            text
        } else { "" }
    );
    println!();
    
    // Transform for Anthropic
    println!("🔄 Transforming for Anthropic...");
    let result = transform_request(request.clone(), "anthropic");
    
    println!("\n📊 Transformation Results:");
    println!("  - Lossy: {}", if result.lossy { "⚠️ YES" } else { "✅ NO" });
    
    if result.lossy {
        println!("  - Information Lost:");
        for reason in &result.reasons {
            println!("    • {}", reason);
        }
    }
    
    println!("\n📤 Transformed Request (Anthropic Format):");
    println!("  - Messages: {} (system merged into user)", result.transformed.messages.len());
    println!("  - JSON Mode: {}", 
        if result.transformed.response_format.is_some() { "Enabled" } else { "Removed" }
    );
    
    // Show the merged message content
    if let specado_core::protocol::types::MessageContent::Text(content) = 
        &result.transformed.messages[0].content {
        println!("  - Merged Content Preview:");
        let preview = if content.len() > 100 {
            format!("{}...", &content[..100])
        } else {
            content.clone()
        };
        println!("    \"{}\"", preview.replace('\n', " "));
    }
    
    println!("\n✨ Key Benefits Demonstrated:");
    println!("  1. Same code works for different providers");
    println!("  2. Explicit tracking of what gets lost in transformation");
    println!("  3. No silent failures - you KNOW what's incompatible");
    
    println!("\n🎯 Next Steps:");
    println!("  - Week 2: Add routing and Python bindings");
    println!("  - Week 3: Package and publish for easy adoption");
    
    // Demonstrate another transformation scenario
    println!("\n{}", "=".repeat(50));
    println!("\n📝 Another Example: Consecutive Same-Role Messages\n");
    
    let request2 = ChatRequest::new(
        "gpt-4",
        vec![
            Message::user("First question"),
            Message::user("Follow-up question"),
            Message::user("Another follow-up"),
            Message::assistant("I'll answer all three..."),
        ],
    );
    
    println!("Original: {} messages with consecutive user messages", request2.messages.len());
    let result2 = transform_request(request2, "anthropic");
    
    println!("After transformation: {} messages", result2.transformed.messages.len());
    println!("Lossy: {}", result2.lossy);
    if result2.lossy {
        println!("Reasons: {:?}", result2.reasons);
    }
    
    println!("\n🏁 Demo Complete!");
}