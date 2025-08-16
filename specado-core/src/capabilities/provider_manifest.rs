//! Provider manifest definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::capabilities::{Capability, Constraints, ModelFeatures, ModalitySupport, RoleSupport, ControlParameters, ParameterSupport};

/// Complete provider manifest including all capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderManifest {
    /// Provider information
    pub info: ProviderInfo,
    
    /// Available models from this provider
    pub models: HashMap<String, ModelManifest>,
    
    /// Provider-wide capabilities (applies to all models)
    pub provider_capabilities: Option<Capability>,
    
    /// Authentication requirements
    pub authentication: AuthenticationRequirements,
    
    /// Endpoint configuration
    pub endpoints: EndpointConfiguration,
}

/// Provider information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderInfo {
    /// Provider name (e.g., "OpenAI", "Anthropic")
    pub name: String,
    
    /// Provider version
    pub version: String,
    
    /// Provider description
    pub description: Option<String>,
    
    /// Provider website
    pub website: Option<String>,
    
    /// Support contact
    pub support: Option<String>,
}

/// Model-specific manifest
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelManifest {
    /// Model identifier (e.g., "gpt-4", "claude-3-opus")
    pub model_id: String,
    
    /// Model display name
    pub display_name: String,
    
    /// Model description
    pub description: Option<String>,
    
    /// Model capabilities
    pub capabilities: Capability,
    
    /// Model-specific pricing
    pub pricing: Option<PricingInfo>,
    
    /// Model status
    pub status: ModelStatus,
    
    /// Model release date
    pub release_date: Option<String>,
    
    /// Deprecation date if applicable
    pub deprecation_date: Option<String>,
}

/// Authentication requirements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthenticationRequirements {
    /// Required authentication type
    pub auth_type: AuthType,
    
    /// Required headers
    pub required_headers: Vec<String>,
    
    /// Required environment variables
    pub required_env_vars: Vec<String>,
    
    /// OAuth configuration if applicable
    pub oauth_config: Option<OAuthConfig>,
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthType {
    ApiKey,
    BearerToken,
    OAuth2,
    Custom(String),
}

/// OAuth configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OAuthConfig {
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
}

/// Endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EndpointConfiguration {
    /// Base URL for the API
    pub base_url: String,
    
    /// Chat completions endpoint
    pub chat_endpoint: Option<String>,
    
    /// Completions endpoint (legacy)
    pub completions_endpoint: Option<String>,
    
    /// Embeddings endpoint
    pub embeddings_endpoint: Option<String>,
    
    /// Images endpoint
    pub images_endpoint: Option<String>,
    
    /// Audio endpoint
    pub audio_endpoint: Option<String>,
    
    /// Custom endpoints
    pub custom_endpoints: HashMap<String, String>,
}

/// Pricing information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PricingInfo {
    /// Price per 1K input tokens
    pub input_price_per_1k: Option<f64>,
    
    /// Price per 1K output tokens
    pub output_price_per_1k: Option<f64>,
    
    /// Currency (e.g., "USD")
    pub currency: String,
    
    /// Pricing tier if applicable
    pub tier: Option<String>,
}

/// Model status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelStatus {
    Available,
    Preview,
    Beta,
    Deprecated,
    Unavailable,
}

impl ProviderManifest {
    /// Create OpenAI provider manifest
    pub fn openai() -> Self {
        let mut models = HashMap::new();
        
        // GPT-4 Turbo
        models.insert("gpt-4-turbo".to_string(), ModelManifest {
            model_id: "gpt-4-turbo".to_string(),
            display_name: "GPT-4 Turbo".to_string(),
            description: Some("Most capable GPT-4 model with vision and function calling".to_string()),
            capabilities: Capability {
                version: "0.1.0".to_string(),
                modalities: {
                    let mut m = ModalitySupport::text_and_image();
                    m.configs.image = Some(crate::capabilities::modality::ImageConfig::default());
                    m
                },
                features: ModelFeatures {
                    function_calling: true,
                    json_mode: true,
                    streaming: true,
                    logprobs: true,
                    multiple_responses: true,
                    stop_sequences: true,
                    seed_support: true,
                    tool_use: false,
                    vision: true,
                },
                parameters: ControlParameters {
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
                        default: Some(1.0),
                    },
                    max_tokens: ParameterSupport {
                        supported: true,
                        min: Some(1),
                        max: Some(4096),
                        default: None,
                    },
                    frequency_penalty: ParameterSupport {
                        supported: true,
                        min: Some(-2.0),
                        max: Some(2.0),
                        default: Some(0.0),
                    },
                    presence_penalty: ParameterSupport {
                        supported: true,
                        min: Some(-2.0),
                        max: Some(2.0),
                        default: Some(0.0),
                    },
                    ..Default::default()
                },
                roles: RoleSupport {
                    system: true,
                    user: true,
                    assistant: true,
                    function: true,
                    tool: false,
                    custom_roles: Default::default(),
                },
                constraints: Constraints::openai_gpt4(),
                extensions: Default::default(),
            },
            pricing: Some(PricingInfo {
                input_price_per_1k: Some(0.01),
                output_price_per_1k: Some(0.03),
                currency: "USD".to_string(),
                tier: None,
            }),
            status: ModelStatus::Available,
            release_date: Some("2024-04-09".to_string()),
            deprecation_date: None,
        });
        
        // GPT-3.5 Turbo
        models.insert("gpt-3.5-turbo".to_string(), ModelManifest {
            model_id: "gpt-3.5-turbo".to_string(),
            display_name: "GPT-3.5 Turbo".to_string(),
            description: Some("Fast and efficient model for most tasks".to_string()),
            capabilities: Capability {
                version: "0.1.0".to_string(),
                modalities: ModalitySupport::text_only(),
                features: ModelFeatures {
                    function_calling: true,
                    json_mode: true,
                    streaming: true,
                    logprobs: true,
                    multiple_responses: true,
                    stop_sequences: true,
                    seed_support: true,
                    tool_use: false,
                    vision: false,
                },
                parameters: ControlParameters {
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
                        default: Some(1.0),
                    },
                    max_tokens: ParameterSupport {
                        supported: true,
                        min: Some(1),
                        max: Some(4096),
                        default: None,
                    },
                    ..Default::default()
                },
                roles: RoleSupport {
                    system: true,
                    user: true,
                    assistant: true,
                    function: true,
                    tool: false,
                    custom_roles: Default::default(),
                },
                constraints: Constraints {
                    tokens: crate::capabilities::constraints::TokenLimits {
                        max_context_window: Some(16385),
                        max_output_tokens: Some(4096),
                        encoding: Some("cl100k_base".to_string()),
                        ..Default::default()
                    },
                    ..Constraints::openai_gpt4()
                },
                extensions: Default::default(),
            },
            pricing: Some(PricingInfo {
                input_price_per_1k: Some(0.0005),
                output_price_per_1k: Some(0.0015),
                currency: "USD".to_string(),
                tier: None,
            }),
            status: ModelStatus::Available,
            release_date: Some("2023-06-13".to_string()),
            deprecation_date: None,
        });
        
        Self {
            info: ProviderInfo {
                name: "OpenAI".to_string(),
                version: "1.0.0".to_string(),
                description: Some("OpenAI API provider for GPT models".to_string()),
                website: Some("https://openai.com".to_string()),
                support: Some("https://help.openai.com".to_string()),
            },
            models,
            provider_capabilities: None,
            authentication: AuthenticationRequirements {
                auth_type: AuthType::ApiKey,
                required_headers: vec!["Authorization".to_string()],
                required_env_vars: vec!["OPENAI_API_KEY".to_string()],
                oauth_config: None,
            },
            endpoints: EndpointConfiguration {
                base_url: "https://api.openai.com/v1".to_string(),
                chat_endpoint: Some("/chat/completions".to_string()),
                completions_endpoint: Some("/completions".to_string()),
                embeddings_endpoint: Some("/embeddings".to_string()),
                images_endpoint: Some("/images".to_string()),
                audio_endpoint: Some("/audio".to_string()),
                custom_endpoints: HashMap::new(),
            },
        }
    }
    
    /// Create Anthropic provider manifest
    pub fn anthropic() -> Self {
        let mut models = HashMap::new();
        
        // Claude 3 Opus
        models.insert("claude-3-opus".to_string(), ModelManifest {
            model_id: "claude-3-opus-20240229".to_string(),
            display_name: "Claude 3 Opus".to_string(),
            description: Some("Most capable Claude model for complex tasks".to_string()),
            capabilities: Capability {
                version: "0.1.0".to_string(),
                modalities: ModalitySupport::text_and_image(),
                features: ModelFeatures {
                    function_calling: false,  // Uses tool_use instead
                    json_mode: false,
                    streaming: true,
                    logprobs: false,
                    multiple_responses: false,
                    stop_sequences: true,
                    seed_support: false,
                    tool_use: true,
                    vision: true,
                },
                parameters: ControlParameters {
                    temperature: ParameterSupport {
                        supported: true,
                        min: Some(0.0),
                        max: Some(1.0),
                        default: Some(1.0),
                    },
                    top_p: ParameterSupport {
                        supported: true,
                        min: Some(0.0),
                        max: Some(1.0),
                        default: None,
                    },
                    top_k: ParameterSupport {
                        supported: true,
                        min: Some(1),
                        max: Some(100),
                        default: None,
                    },
                    max_tokens: ParameterSupport {
                        supported: true,
                        min: Some(1),
                        max: Some(4096),
                        default: None,
                    },
                    ..Default::default()
                },
                roles: RoleSupport {
                    system: true,
                    user: true,
                    assistant: true,
                    function: false,
                    tool: true,
                    custom_roles: Default::default(),
                },
                constraints: Constraints::anthropic_claude(),
                extensions: Default::default(),
            },
            pricing: Some(PricingInfo {
                input_price_per_1k: Some(0.015),
                output_price_per_1k: Some(0.075),
                currency: "USD".to_string(),
                tier: None,
            }),
            status: ModelStatus::Available,
            release_date: Some("2024-02-29".to_string()),
            deprecation_date: None,
        });
        
        Self {
            info: ProviderInfo {
                name: "Anthropic".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Anthropic API provider for Claude models".to_string()),
                website: Some("https://anthropic.com".to_string()),
                support: Some("https://support.anthropic.com".to_string()),
            },
            models,
            provider_capabilities: None,
            authentication: AuthenticationRequirements {
                auth_type: AuthType::ApiKey,
                required_headers: vec!["x-api-key".to_string(), "anthropic-version".to_string()],
                required_env_vars: vec!["ANTHROPIC_API_KEY".to_string()],
                oauth_config: None,
            },
            endpoints: EndpointConfiguration {
                base_url: "https://api.anthropic.com".to_string(),
                chat_endpoint: Some("/v1/messages".to_string()),
                completions_endpoint: None,
                embeddings_endpoint: None,
                images_endpoint: None,
                audio_endpoint: None,
                custom_endpoints: HashMap::new(),
            },
        }
    }
    
    /// Get a specific model's capabilities
    pub fn get_model_capabilities(&self, model_id: &str) -> Option<&Capability> {
        self.models.get(model_id).map(|m| &m.capabilities)
    }
    
    /// List all available models
    pub fn list_models(&self) -> Vec<&str> {
        self.models.keys().map(|s| s.as_str()).collect()
    }
    
    /// Check if a model supports a specific feature
    pub fn model_supports_feature(&self, model_id: &str, feature: &str) -> bool {
        self.get_model_capabilities(model_id)
            .map(|cap| cap.supports_feature(feature))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_openai_manifest() {
        let manifest = ProviderManifest::openai();
        assert_eq!(manifest.info.name, "OpenAI");
        assert!(manifest.models.contains_key("gpt-4-turbo"));
        assert!(manifest.models.contains_key("gpt-3.5-turbo"));
        
        let gpt4 = &manifest.models["gpt-4-turbo"];
        assert!(gpt4.capabilities.features.function_calling);
        assert!(gpt4.capabilities.features.vision);
    }
    
    #[test]
    fn test_anthropic_manifest() {
        let manifest = ProviderManifest::anthropic();
        assert_eq!(manifest.info.name, "Anthropic");
        assert!(manifest.models.contains_key("claude-3-opus"));
        
        let claude = &manifest.models["claude-3-opus"];
        assert!(!claude.capabilities.features.function_calling);
        assert!(claude.capabilities.features.tool_use);
        assert!(claude.capabilities.features.vision);
    }
    
    #[test]
    fn test_model_feature_support() {
        let manifest = ProviderManifest::openai();
        assert!(manifest.model_supports_feature("gpt-4-turbo", "function_calling"));
        assert!(manifest.model_supports_feature("gpt-4-turbo", "vision"));
        assert!(!manifest.model_supports_feature("gpt-3.5-turbo", "vision"));
        assert!(!manifest.model_supports_feature("unknown-model", "vision"));
    }
}