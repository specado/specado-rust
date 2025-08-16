//! Example usage of the capability taxonomy

use specado_core::capabilities::{
    Capability, ProviderManifest,
    ModelFeatures, ControlParameters, ParameterSupport,
};

fn main() {
    println!("Specado Capability Taxonomy Demo\n");
    println!("=================================\n");
    
    // Load provider manifests
    let openai = ProviderManifest::openai();
    let anthropic = ProviderManifest::anthropic();
    
    // List available models
    println!("Available OpenAI models:");
    for model in openai.list_models() {
        println!("  - {}", model);
    }
    
    println!("\nAvailable Anthropic models:");
    for model in anthropic.list_models() {
        println!("  - {}", model);
    }
    
    // Check specific capabilities
    println!("\n\nCapability Checks:");
    println!("==================");
    
    if let Some(gpt4) = openai.get_model_capabilities("gpt-4-turbo") {
        println!("\nGPT-4 Turbo capabilities:");
        println!("  - Function calling: {}", gpt4.features.function_calling);
        println!("  - Vision support: {}", gpt4.features.vision);
        println!("  - JSON mode: {}", gpt4.features.json_mode);
        println!("  - Streaming: {}", gpt4.features.streaming);
        println!("  - Max context window: {:?}", gpt4.constraints.tokens.max_context_window);
    }
    
    if let Some(claude) = anthropic.get_model_capabilities("claude-3-opus") {
        println!("\nClaude 3 Opus capabilities:");
        println!("  - Tool use: {}", claude.features.tool_use);
        println!("  - Vision support: {}", claude.features.vision);
        println!("  - JSON mode: {}", claude.features.json_mode);
        println!("  - Streaming: {}", claude.features.streaming);
        println!("  - Max context window: {:?}", claude.constraints.tokens.max_context_window);
    }
    
    // Compare capabilities for lossiness detection
    println!("\n\nCapability Comparison:");
    println!("======================");
    
    let gpt4 = openai.get_model_capabilities("gpt-4-turbo").unwrap();
    let claude = anthropic.get_model_capabilities("claude-3-opus").unwrap();
    
    let comparison = gpt4.compare(claude);
    
    println!("\nComparing GPT-4 Turbo → Claude 3 Opus:");
    println!("  - Transformation is lossy: {}", comparison.lossiness_report.is_lossy);
    println!("  - Severity: {:?}", comparison.lossiness_report.severity);
    
    if !comparison.missing_capabilities.is_empty() {
        println!("\n  Missing capabilities in target:");
        for cap in &comparison.missing_capabilities {
            println!("    - {}", cap);
        }
    }
    
    if !comparison.lossiness_report.details.is_empty() {
        println!("\n  Lossiness details:");
        for detail in &comparison.lossiness_report.details {
            println!("    - {}", detail);
        }
    }
    
    if !comparison.lossiness_report.recommendations.is_empty() {
        println!("\n  Recommendations:");
        for rec in &comparison.lossiness_report.recommendations {
            println!("    - {}", rec);
        }
    }
    
    // Build a custom capability
    println!("\n\nCustom Capability Builder:");
    println!("===========================");
    
    let custom = Capability::builder()
        .version("1.0.0")
        .with_features(ModelFeatures {
            function_calling: true,
            json_mode: true,
            streaming: true,
            vision: false,
            tool_use: false,
            logprobs: false,
            multiple_responses: false,
            stop_sequences: true,
            seed_support: false,
        })
        .with_parameters(ControlParameters {
            temperature: ParameterSupport {
                supported: true,
                min: Some(0.0),
                max: Some(1.5),
                default: Some(0.7),
            },
            max_tokens: ParameterSupport {
                supported: true,
                min: Some(1),
                max: Some(2048),
                default: Some(512),
            },
            ..Default::default()
        })
        .build();
    
    println!("\nCustom model capabilities:");
    println!("  - Version: {}", custom.version);
    println!("  - Function calling: {}", custom.features.function_calling);
    println!("  - Temperature range: {:?} - {:?}", 
        custom.parameters.temperature.min,
        custom.parameters.temperature.max
    );
    println!("  - Max tokens: {:?}", custom.parameters.max_tokens.max);
    
    // Serialize to JSON for FFI
    println!("\n\nSerialization Demo:");
    println!("===================");
    
    let json = serde_json::to_string_pretty(&custom).unwrap();
    println!("Custom capability as JSON (first 500 chars):");
    println!("{}", &json[..500.min(json.len())]);
    
    println!("\n✅ Capability taxonomy demo complete!");
}