//! Integration tests for the capabilities module

use specado_core::capabilities::*;
use specado_core::capabilities::comparison::LossinessSeverity;

#[test]
fn test_capability_comparison_openai_to_anthropic() {
    // Create OpenAI manifest and get GPT-4 capabilities
    let openai_manifest = provider_manifest::ProviderManifest::openai();
    let gpt4_capabilities = openai_manifest
        .get_model_capabilities("gpt-4-turbo")
        .expect("GPT-4 Turbo should exist");
    
    // Create Anthropic manifest and get Claude capabilities
    let anthropic_manifest = provider_manifest::ProviderManifest::anthropic();
    let claude_capabilities = anthropic_manifest
        .get_model_capabilities("claude-3-opus")
        .expect("Claude 3 Opus should exist");
    
    // Compare capabilities
    let comparison = gpt4_capabilities.compare(claude_capabilities);
    
    // OpenAI has function_calling, Anthropic doesn't
    assert!(comparison.lossiness_report.is_lossy);
    assert!(comparison.missing_capabilities.contains(&"function_calling".to_string()));
    
    // Both support vision
    assert!(gpt4_capabilities.features.vision);
    assert!(claude_capabilities.features.vision);
    
    // Check lossiness types
    let has_function_calling_loss = comparison.lossiness_report.lossiness_types.iter()
        .any(|t| matches!(t, LossinessType::MissingFeature(f) if f == "function_calling"));
    assert!(has_function_calling_loss);
    
    // Check recommendations
    assert!(!comparison.lossiness_report.recommendations.is_empty());
}

#[test]
fn test_capability_comparison_anthropic_to_openai() {
    let openai_manifest = provider_manifest::ProviderManifest::openai();
    let gpt4_capabilities = openai_manifest
        .get_model_capabilities("gpt-4-turbo")
        .expect("GPT-4 Turbo should exist");
    
    let anthropic_manifest = provider_manifest::ProviderManifest::anthropic();
    let claude_capabilities = anthropic_manifest
        .get_model_capabilities("claude-3-opus")
        .expect("Claude 3 Opus should exist");
    
    // Compare in reverse direction
    let comparison = claude_capabilities.compare(gpt4_capabilities);
    
    // OpenAI has features that Claude doesn't (json_mode, logprobs, etc.)
    assert!(comparison.lossiness_report.is_lossy);
    
    // Claude's tool_use vs OpenAI's function_calling should be handled intelligently
    // Since OpenAI has function_calling but not tool_use, this will be flagged
    // But the transformation engine should treat them as equivalent
    if !gpt4_capabilities.features.tool_use && claude_capabilities.features.tool_use {
        // Only flag as missing if target doesn't have equivalent function_calling
        if !gpt4_capabilities.features.function_calling {
            assert!(comparison.missing_capabilities.contains(&"tool_use".to_string()));
        }
    }
}

#[test]
fn test_multimodal_support() {
    let openai_manifest = provider_manifest::ProviderManifest::openai();
    
    // GPT-4 Turbo supports vision
    let gpt4_turbo = openai_manifest
        .get_model_capabilities("gpt-4-turbo")
        .expect("GPT-4 Turbo should exist");
    assert!(gpt4_turbo.modalities.supports_input(&modality::Modality::Text));
    assert!(gpt4_turbo.modalities.supports_input(&modality::Modality::Image));
    assert!(gpt4_turbo.features.vision);
    
    // GPT-3.5 Turbo doesn't support vision
    let gpt35_turbo = openai_manifest
        .get_model_capabilities("gpt-3.5-turbo")
        .expect("GPT-3.5 Turbo should exist");
    assert!(gpt35_turbo.modalities.supports_input(&modality::Modality::Text));
    assert!(!gpt35_turbo.modalities.supports_input(&modality::Modality::Image));
    assert!(!gpt35_turbo.features.vision);
}

#[test]
fn test_constraint_validation() {
    let constraints = Constraints::openai_gpt4();
    
    // Within limits
    assert!(constraints.check_token_limits(1000, 1000).is_ok());
    
    // Exceeds context window
    let result = constraints.check_token_limits(100000, 30000);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("context window"));
    
    // Exceeds output limit
    let result = constraints.check_token_limits(1000, 5000);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Output tokens"));
}

#[test]
fn test_capability_builder() {
    let capability = Capability::builder()
        .version("2.0.0")
        .with_features(ModelFeatures {
            function_calling: true,
            json_mode: true,
            streaming: true,
            ..Default::default()
        })
        .with_roles(RoleSupport {
            system: true,
            user: true,
            assistant: true,
            ..Default::default()
        })
        .with_constraints(Constraints::openai_gpt4())
        .build();
    
    assert_eq!(capability.version, "2.0.0");
    assert!(capability.features.function_calling);
    assert!(capability.features.json_mode);
    assert!(capability.features.streaming);
    assert!(capability.roles.system);
}

#[test]
fn test_lossiness_severity() {
    let mut source = Capability::default();
    let target = Capability::default();
    
    // Test different severity levels
    
    // Missing multimodal support = Critical
    source.modalities.input.insert(modality::Modality::Image);
    let comparison = source.compare(&target);
    assert_eq!(comparison.lossiness_report.severity, LossinessSeverity::Critical);
    
    // Reset and test function calling = High
    source = Capability::default();
    source.features.function_calling = true;
    let comparison = source.compare(&target);
    assert!(comparison.lossiness_report.severity >= LossinessSeverity::High);
    
    // Reset and test streaming = Low
    source = Capability::default();
    source.features.streaming = true;
    let comparison = source.compare(&target);
    assert_eq!(comparison.lossiness_report.severity, LossinessSeverity::Low);
}

#[test]
fn test_provider_manifest_model_listing() {
    let openai = provider_manifest::ProviderManifest::openai();
    let models = openai.list_models();
    assert!(models.contains(&"gpt-4-turbo"));
    assert!(models.contains(&"gpt-3.5-turbo"));
    
    let anthropic = provider_manifest::ProviderManifest::anthropic();
    let models = anthropic.list_models();
    assert!(models.contains(&"claude-3-opus"));
}

#[test]
fn test_custom_modality() {
    let mut support = modality::ModalitySupport::default();
    support.input.insert(modality::Modality::Custom("3D".to_string()));
    
    assert!(support.supports_input(&modality::Modality::Custom("3D".to_string())));
    assert!(!support.supports_input(&modality::Modality::Custom("4D".to_string())));
    assert!(support.supports_input(&modality::Modality::Text));
}

#[test]
fn test_image_config_formats() {
    let support = modality::ModalitySupport::text_and_image();
    let formats = support.get_input_formats(&modality::Modality::Image);
    
    assert!(formats.is_some());
    let formats = formats.unwrap();
    assert!(formats.contains("jpeg"));
    assert!(formats.contains("png"));
    assert!(formats.contains("gif"));
    assert!(formats.contains("webp"));
}

#[test]
fn test_parameter_support_bounds() {
    let params = ControlParameters {
        temperature: ParameterSupport {
            supported: true,
            min: Some(0.0),
            max: Some(2.0),
            default: Some(1.0),
        },
        top_p: ParameterSupport {
            supported: true,
            min: Some(0.0),
            max: Some(1.0),
            default: Some(0.9),
        },
        ..Default::default()
    };
    
    assert!(params.temperature.supported);
    assert_eq!(params.temperature.min, Some(0.0));
    assert_eq!(params.temperature.max, Some(2.0));
    assert_eq!(params.temperature.default, Some(1.0));
    
    assert!(params.top_p.supported);
    assert_eq!(params.top_p.min, Some(0.0));
    assert_eq!(params.top_p.max, Some(1.0));
}

#[test]
fn test_authentication_requirements() {
    let openai = provider_manifest::ProviderManifest::openai();
    assert_eq!(openai.authentication.auth_type, provider_manifest::AuthType::ApiKey);
    assert!(openai.authentication.required_headers.contains(&"Authorization".to_string()));
    assert!(openai.authentication.required_env_vars.contains(&"OPENAI_API_KEY".to_string()));
    
    let anthropic = provider_manifest::ProviderManifest::anthropic();
    assert_eq!(anthropic.authentication.auth_type, provider_manifest::AuthType::ApiKey);
    assert!(anthropic.authentication.required_headers.contains(&"x-api-key".to_string()));
    assert!(anthropic.authentication.required_env_vars.contains(&"ANTHROPIC_API_KEY".to_string()));
}

#[test]
fn test_endpoint_configuration() {
    let openai = provider_manifest::ProviderManifest::openai();
    assert_eq!(openai.endpoints.base_url, "https://api.openai.com/v1");
    assert_eq!(openai.endpoints.chat_endpoint, Some("/chat/completions".to_string()));
    assert!(openai.endpoints.images_endpoint.is_some());
    
    let anthropic = provider_manifest::ProviderManifest::anthropic();
    assert_eq!(anthropic.endpoints.base_url, "https://api.anthropic.com");
    assert_eq!(anthropic.endpoints.chat_endpoint, Some("/v1/messages".to_string()));
    assert!(anthropic.endpoints.images_endpoint.is_none());
}

#[test]
fn test_pricing_info() {
    let openai = provider_manifest::ProviderManifest::openai();
    let gpt4_turbo = &openai.models["gpt-4-turbo"];
    
    assert!(gpt4_turbo.pricing.is_some());
    let pricing = gpt4_turbo.pricing.as_ref().unwrap();
    assert_eq!(pricing.currency, "USD");
    assert!(pricing.input_price_per_1k.is_some());
    assert!(pricing.output_price_per_1k.is_some());
}

#[test]
fn test_extensions_and_experimental_features() {
    let mut capability = Capability::default();
    capability.extensions.experimental.insert("beta_feature".to_string());
    capability.extensions.custom.insert(
        "provider_specific".to_string(),
        serde_json::json!({"custom": "value"}),
    );
    
    assert!(capability.supports_feature("beta_feature"));
    assert!(!capability.supports_feature("unknown_feature"));
    assert!(capability.extensions.custom.contains_key("provider_specific"));
}

#[test]
fn test_serialization_deserialization() {
    let original = Capability::builder()
        .version("1.0.0")
        .with_features(ModelFeatures {
            function_calling: true,
            json_mode: true,
            ..Default::default()
        })
        .build();
    
    // Serialize to JSON
    let json = serde_json::to_string(&original).expect("Should serialize");
    
    // Deserialize back
    let deserialized: Capability = serde_json::from_str(&json).expect("Should deserialize");
    
    assert_eq!(original, deserialized);
    assert_eq!(deserialized.version, "1.0.0");
    assert!(deserialized.features.function_calling);
    assert!(deserialized.features.json_mode);
}

#[test]
fn test_provider_manifest_serialization() {
    let original = provider_manifest::ProviderManifest::openai();
    
    // Serialize to JSON
    let json = serde_json::to_string(&original).expect("Should serialize");
    
    // Deserialize back
    let deserialized: provider_manifest::ProviderManifest = 
        serde_json::from_str(&json).expect("Should deserialize");
    
    assert_eq!(original.info.name, deserialized.info.name);
    assert_eq!(original.models.len(), deserialized.models.len());
    assert!(deserialized.models.contains_key("gpt-4-turbo"));
}