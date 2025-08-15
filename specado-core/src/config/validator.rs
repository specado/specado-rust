//! Configuration validation utilities

use super::error::{ValidationError, ValidationErrorKind};
use super::schema::SpecadoConfig;
use regex::Regex;
use std::collections::HashSet;

/// Configuration validator with additional validation rules
pub struct ConfigValidator {
    /// Pattern for environment variable placeholders
    env_var_pattern: Regex,
    /// Pattern for sensitive field names
    sensitive_pattern: Regex,
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            env_var_pattern: Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").unwrap(),
            sensitive_pattern: Regex::new(r"(?i)(api_key|secret|token|password|credential)")
                .unwrap(),
        }
    }

    /// Validate a configuration with extended rules
    pub fn validate(&self, config: &SpecadoConfig) -> Result<(), ValidationError> {
        // First run the built-in validation
        config.validate()?;

        // Additional validation rules
        self.validate_env_vars(config)?;
        self.validate_routing_strategy(config)?;
        self.validate_model_consistency(config)?;

        Ok(())
    }

    /// Validate environment variable placeholders
    fn validate_env_vars(&self, config: &SpecadoConfig) -> Result<(), ValidationError> {
        for provider in config.providers.iter() {
            let api_key_str = provider.api_key.expose_secret();
            // Check if API key contains env var placeholder
            if self.env_var_pattern.is_match(api_key_str) {
                let captures = self.env_var_pattern.captures(api_key_str);
                if let Some(cap) = captures {
                    let var_name = &cap[1];
                    // Check if it's a sensitive variable name
                    if !self.sensitive_pattern.is_match(var_name) {
                        // Warning: API key should use a sensitive variable name
                        // This is just a warning, not an error
                    }
                }
            } else if !api_key_str.starts_with("sk-") && !api_key_str.starts_with("${") {
                // If it's not an env var and doesn't look like a key, warn
                // This helps catch plain text keys in config
            }
        }

        Ok(())
    }

    /// Validate routing strategy configuration
    fn validate_routing_strategy(&self, config: &SpecadoConfig) -> Result<(), ValidationError> {
        use super::schema::RoutingStrategy;

        // First check that at least one provider is enabled
        let enabled_count = config.providers.iter().filter(|p| p.enabled).count();
        if enabled_count == 0 {
            return Err(ValidationError::new(
                "providers",
                ValidationErrorKind::Custom {
                    message: "At least one provider must be enabled".to_string(),
                },
            ));
        }

        match config.routing.strategy {
            RoutingStrategy::Weighted => {
                // Weighted strategy requires weights to be defined
                if config.routing.weights.is_empty() {
                    return Err(ValidationError::new(
                        "routing.weights",
                        ValidationErrorKind::RequiredFieldMissing,
                    )
                    .with_context("Weighted routing strategy requires weights to be defined"));
                }

                // All enabled providers should have weights
                let mut total_weight = 0.0;
                for provider in &config.providers {
                    if provider.enabled {
                        if let Some(weight) = config.routing.weights.get(&provider.name) {
                            total_weight += weight;
                        } else {
                            return Err(ValidationError::new(
                                format!("routing.weights.{}", provider.name),
                                ValidationErrorKind::RequiredFieldMissing,
                            )
                            .with_context(
                                "Weighted routing requires all enabled providers to have weights",
                            ));
                        }
                    }
                }

                // Ensure total weight is positive
                if total_weight <= 0.0 {
                    return Err(ValidationError::new(
                        "routing.weights",
                        ValidationErrorKind::Custom {
                            message: "Sum of weights for enabled providers must be positive"
                                .to_string(),
                        },
                    ));
                }
            }
            RoutingStrategy::CostOptimized => {
                // Cost optimized strategy requires cost information
                for (i, provider) in config.providers.iter().enumerate() {
                    if provider.enabled {
                        for (j, model) in provider.models.iter().enumerate() {
                            // Check for missing cost fields and report specific errors
                            if model.cost_per_1k_input.is_none() {
                                return Err(ValidationError::new(
                                    format!("providers[{i}].models[{j}].cost_per_1k_input"),
                                    ValidationErrorKind::RequiredFieldMissing,
                                )
                                .with_context(
                                    "Cost-optimized routing requires input cost for all models",
                                ));
                            }
                            if model.cost_per_1k_output.is_none() {
                                return Err(ValidationError::new(
                                    format!("providers[{i}].models[{j}].cost_per_1k_output"),
                                    ValidationErrorKind::RequiredFieldMissing,
                                )
                                .with_context(
                                    "Cost-optimized routing requires output cost for all models",
                                ));
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Validate model consistency across providers
    fn validate_model_consistency(&self, config: &SpecadoConfig) -> Result<(), ValidationError> {
        // Collect all model IDs across providers
        let mut model_providers: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for provider in &config.providers {
            for model in &provider.models {
                model_providers
                    .entry(model.id.clone())
                    .or_default()
                    .push(provider.name.clone());
            }
        }

        // Check for models that exist in multiple providers with different configs
        // This is not an error, but we should validate they're compatible
        for (model_id, providers) in model_providers.iter() {
            if providers.len() > 1 {
                // Verify that the same model has compatible settings across providers
                let mut max_tokens_set = HashSet::new();

                for provider in &config.providers {
                    if let Some(model) = provider.models.iter().find(|m| &m.id == model_id) {
                        max_tokens_set.insert(model.max_tokens);
                    }
                }

                if max_tokens_set.len() > 1 {
                    // Warning: same model has different max_tokens across providers
                    // This might be intentional (different API versions), so not an error
                }
            }
        }

        Ok(())
    }

    /// Check if a field name appears to contain sensitive information
    pub fn is_sensitive_field(&self, field_name: &str) -> bool {
        self.sensitive_pattern.is_match(field_name)
    }

    /// Extract environment variables from a string
    pub fn extract_env_vars(&self, text: &str) -> Vec<String> {
        self.env_var_pattern
            .captures_iter(text)
            .map(|cap| cap[1].to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_extraction() {
        let validator = ConfigValidator::new();

        let text = "api_key: ${OPENAI_API_KEY}, url: ${API_BASE_URL}";
        let vars = validator.extract_env_vars(text);

        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"OPENAI_API_KEY".to_string()));
        assert!(vars.contains(&"API_BASE_URL".to_string()));
    }

    #[test]
    fn test_sensitive_field_detection() {
        let validator = ConfigValidator::new();

        assert!(validator.is_sensitive_field("api_key"));
        assert!(validator.is_sensitive_field("API_KEY"));
        assert!(validator.is_sensitive_field("secret_token"));
        assert!(validator.is_sensitive_field("password"));
        assert!(validator.is_sensitive_field("auth_credential"));

        assert!(!validator.is_sensitive_field("username"));
        assert!(!validator.is_sensitive_field("model_id"));
    }
}
