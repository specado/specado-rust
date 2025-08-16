//! Provider capability taxonomy for spec-driven LLM integration
//! 
//! This module defines the structured model for provider capabilities,
//! including multimodal support, function calling, JSON mode, temperature
//! control, and provider-specific constraints.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod modality;
pub mod constraints;
pub mod comparison;
pub mod provider_manifest;
pub mod ffi;

pub use modality::{Modality, ModalitySupport};
pub use constraints::{Constraints, TokenLimits, RateLimits};
pub use comparison::{CapabilityComparison, LossinessReport, LossinessType};
pub use provider_manifest::{ProviderManifest, ProviderInfo};

/// Core capability definition for LLM providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capability {
    /// Version of the capability schema
    pub version: String,
    
    /// Supported input/output modalities
    pub modalities: ModalitySupport,
    
    /// Model features and capabilities
    pub features: ModelFeatures,
    
    /// Control parameters supported
    pub parameters: ControlParameters,
    
    /// Role constraints
    pub roles: RoleSupport,
    
    /// Provider-specific constraints
    pub constraints: Constraints,
    
    /// Custom/experimental capabilities
    pub extensions: Extensions,
}

/// Model features and capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelFeatures {
    /// Function/tool calling support
    pub function_calling: bool,
    
    /// JSON mode for structured output
    pub json_mode: bool,
    
    /// Streaming response support
    pub streaming: bool,
    
    /// Log probabilities support
    pub logprobs: bool,
    
    /// Multiple response generation (n > 1)
    pub multiple_responses: bool,
    
    /// Stop sequences support
    pub stop_sequences: bool,
    
    /// Seed parameter for deterministic output
    pub seed_support: bool,
    
    /// Tool use (Anthropic-style tools)
    pub tool_use: bool,
    
    /// Vision capabilities (for analyzing images)
    pub vision: bool,
}

/// Control parameters for model behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlParameters {
    /// Temperature control (0.0-2.0 typically)
    pub temperature: ParameterSupport<f32>,
    
    /// Top-p nucleus sampling
    pub top_p: ParameterSupport<f32>,
    
    /// Top-k sampling
    pub top_k: ParameterSupport<i32>,
    
    /// Maximum tokens to generate
    pub max_tokens: ParameterSupport<i32>,
    
    /// Frequency penalty (-2.0 to 2.0)
    pub frequency_penalty: ParameterSupport<f32>,
    
    /// Presence penalty (-2.0 to 2.0)
    pub presence_penalty: ParameterSupport<f32>,
    
    /// Repetition penalty (some providers)
    pub repetition_penalty: ParameterSupport<f32>,
}

/// Parameter support with min/max bounds
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParameterSupport<T> {
    pub supported: bool,
    pub min: Option<T>,
    pub max: Option<T>,
    pub default: Option<T>,
}

/// Role support in conversations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleSupport {
    /// System role for instructions
    pub system: bool,
    
    /// User role for input
    pub user: bool,
    
    /// Assistant role for model responses
    pub assistant: bool,
    
    /// Function/tool role for function results
    pub function: bool,
    
    /// Tool role (Anthropic-style)
    pub tool: bool,
    
    /// Custom roles
    pub custom_roles: HashSet<String>,
}

/// Extensions for custom/experimental capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Extensions {
    /// Provider-specific custom capabilities
    pub custom: std::collections::HashMap<String, serde_json::Value>,
    
    /// Experimental features that may not be stable
    pub experimental: HashSet<String>,
}

impl Default for Capability {
    fn default() -> Self {
        Self {
            version: "0.1.0".to_string(),
            modalities: ModalitySupport::default(),
            features: ModelFeatures::default(),
            parameters: ControlParameters::default(),
            roles: RoleSupport::default(),
            constraints: Constraints::default(),
            extensions: Extensions::default(),
        }
    }
}

impl Default for ModelFeatures {
    fn default() -> Self {
        Self {
            function_calling: false,
            json_mode: false,
            streaming: false,
            logprobs: false,
            multiple_responses: false,
            stop_sequences: false,
            seed_support: false,
            tool_use: false,
            vision: false,
        }
    }
}

impl Default for ControlParameters {
    fn default() -> Self {
        Self {
            temperature: ParameterSupport::default(),
            top_p: ParameterSupport::default(),
            top_k: ParameterSupport::default(),
            max_tokens: ParameterSupport::default(),
            frequency_penalty: ParameterSupport::default(),
            presence_penalty: ParameterSupport::default(),
            repetition_penalty: ParameterSupport::default(),
        }
    }
}

impl<T> Default for ParameterSupport<T> {
    fn default() -> Self {
        Self {
            supported: false,
            min: None,
            max: None,
            default: None,
        }
    }
}

impl Default for RoleSupport {
    fn default() -> Self {
        Self {
            system: false,
            user: false,
            assistant: false,
            function: false,
            tool: false,
            custom_roles: HashSet::new(),
        }
    }
}

impl Default for Extensions {
    fn default() -> Self {
        Self {
            custom: std::collections::HashMap::new(),
            experimental: HashSet::new(),
        }
    }
}

impl Capability {
    /// Create a new capability with defaults
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Builder pattern for capability configuration
    pub fn builder() -> CapabilityBuilder {
        CapabilityBuilder::new()
    }
    
    /// Check if a specific feature is supported
    pub fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "function_calling" => self.features.function_calling,
            "json_mode" => self.features.json_mode,
            "streaming" => self.features.streaming,
            "logprobs" => self.features.logprobs,
            "multiple_responses" => self.features.multiple_responses,
            "stop_sequences" => self.features.stop_sequences,
            "seed" => self.features.seed_support,
            "tool_use" => self.features.tool_use,
            "vision" => self.features.vision,
            _ => self.extensions.experimental.contains(feature),
        }
    }
    
    /// Compare capabilities with another provider
    pub fn compare(&self, other: &Capability) -> CapabilityComparison {
        CapabilityComparison::compare(self, other)
    }
}

/// Builder for constructing capabilities
pub struct CapabilityBuilder {
    capability: Capability,
}

impl CapabilityBuilder {
    pub fn new() -> Self {
        Self {
            capability: Capability::default(),
        }
    }
    
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.capability.version = version.into();
        self
    }
    
    pub fn with_modalities(mut self, modalities: ModalitySupport) -> Self {
        self.capability.modalities = modalities;
        self
    }
    
    pub fn with_features(mut self, features: ModelFeatures) -> Self {
        self.capability.features = features;
        self
    }
    
    pub fn with_parameters(mut self, parameters: ControlParameters) -> Self {
        self.capability.parameters = parameters;
        self
    }
    
    pub fn with_roles(mut self, roles: RoleSupport) -> Self {
        self.capability.roles = roles;
        self
    }
    
    pub fn with_constraints(mut self, constraints: Constraints) -> Self {
        self.capability.constraints = constraints;
        self
    }
    
    pub fn build(self) -> Capability {
        self.capability
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capability_default() {
        let cap = Capability::default();
        assert_eq!(cap.version, "0.1.0");
        assert!(!cap.features.function_calling);
        assert!(!cap.features.json_mode);
    }
    
    #[test]
    fn test_capability_builder() {
        let cap = Capability::builder()
            .version("1.0.0")
            .with_features(ModelFeatures {
                function_calling: true,
                json_mode: true,
                ..Default::default()
            })
            .build();
        
        assert_eq!(cap.version, "1.0.0");
        assert!(cap.features.function_calling);
        assert!(cap.features.json_mode);
    }
    
    #[test]
    fn test_supports_feature() {
        let mut cap = Capability::default();
        cap.features.function_calling = true;
        cap.extensions.experimental.insert("custom_feature".to_string());
        
        assert!(cap.supports_feature("function_calling"));
        assert!(!cap.supports_feature("json_mode"));
        assert!(cap.supports_feature("custom_feature"));
        assert!(!cap.supports_feature("unknown_feature"));
    }
}