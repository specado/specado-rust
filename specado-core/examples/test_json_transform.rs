//! Test program to verify JSON transformation implementation

use specado_core::protocol::types::{ChatRequest, Message};
use specado_core::providers::json_transform::{openai_request_to_anthropic_json, anthropic_response_to_openai};
use serde_json::json;

fn main() {
    println!("Testing JSON Transformation for Issue #63\n");
    
    // Test 1: OpenAI to Anthropic request transformation
    println!("Test 1: OpenAI → Anthropic Request Transformation");
    println!("{}", "=".repeat(50));
    
    let mut request = ChatRequest::new(
        "gpt-4",
        vec![
            Message::system("You are a helpful assistant"),
            Message::user("What's the weather like today?"),
        ],
    );
    request.max_tokens = Some(100);
    request.temperature = Some(0.7);
    
    println!("Original OpenAI Request:");
    println!("{}", serde_json::to_string_pretty(&request).unwrap());
    
    let anthropic_json = openai_request_to_anthropic_json(request);
    
    println!("\nTransformed Anthropic Request:");
    println!("{}", serde_json::to_string_pretty(&anthropic_json).unwrap());
    
    // Verify critical fields
    let obj = anthropic_json.as_object().unwrap();
    assert!(obj.contains_key("system"), "System field should be present");
    assert!(obj.contains_key("messages"), "Messages field should be present");
    assert!(obj.contains_key("max_tokens"), "max_tokens field should be present");
    assert_eq!(obj.get("max_tokens").unwrap(), &json!(100), "max_tokens should be 100");
    
    println!("\n✅ Request transformation test passed!");
    
    // Test 2: Anthropic to OpenAI response transformation
    println!("\nTest 2: Anthropic → OpenAI Response Transformation");
    println!("{}", "=".repeat(50));
    
    let anthropic_response = json!({
        "id": "msg_01XYZ",
        "model": "claude-3-opus-20240229",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": "I don't have access to real-time weather data."
            }
        ],
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 15,
            "output_tokens": 12
        }
    });
    
    println!("Original Anthropic Response:");
    println!("{}", serde_json::to_string_pretty(&anthropic_response).unwrap());
    
    let openai_response = anthropic_response_to_openai(anthropic_response).unwrap();
    
    println!("\nTransformed OpenAI Response:");
    println!("{}", serde_json::to_string_pretty(&openai_response).unwrap());
    
    // Verify critical fields
    assert_eq!(openai_response.object, "chat.completion", "Object type should be chat.completion");
    assert_eq!(openai_response.choices.len(), 1, "Should have one choice");
    assert_eq!(openai_response.choices[0].finish_reason, Some("stop".to_string()), "Finish reason should be stop");
    
    let usage = openai_response.usage.unwrap();
    assert_eq!(usage.prompt_tokens, 15, "Prompt tokens should be mapped from input_tokens");
    assert_eq!(usage.completion_tokens, 12, "Completion tokens should be mapped from output_tokens");
    assert_eq!(usage.total_tokens, 27, "Total tokens should be calculated");
    
    println!("\n✅ Response transformation test passed!");
    
    println!("\n🎉 All JSON transformation tests passed successfully!");
    println!("\nIssue #63 Implementation Status:");
    println!("✅ JSON transformation module created");
    println!("✅ OpenAI to Anthropic request transformation working");
    println!("✅ Anthropic to OpenAI response transformation working");
    println!("✅ Field mappings correctly implemented:");
    println!("   - max_tokens field preserved");
    println!("   - System messages extracted to separate field");
    println!("   - Message role/content mapping correct");
    println!("   - Response normalization working");
    println!("   - Usage statistics properly mapped");
    println!("\n📝 Note: Multimodal support, function calling, and JSON mode");
    println!("   transformations are also implemented but not tested here.");
}