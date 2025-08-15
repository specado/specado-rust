//! Configuration module for Specado
//!
//! This module provides the configuration schema and validation for the Specado
//! spec-driven LLM integration library.

mod env;
mod error;
mod schema;
mod secrets;
mod validator;

pub use error::{ConfigError, ValidationError};
pub use schema::{
    ConnectionConfig, FallbackPolicy, Model, Provider, ProviderType, RateLimitConfig, RetryPolicy,
    RoutingStrategy, SpecadoConfig,
};
pub use secrets::{redact_by_field_name, safe_value, RedactionPolicy, SafeLogging, SecretString};
pub use validator::ConfigValidator;

use std::fs;
use std::path::Path;

/// Load a configuration from a YAML file
pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> Result<SpecadoConfig, ConfigError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path).map_err(|e| ConfigError::IoError {
        path: path.to_string_lossy().to_string(),
        source: e,
    })?;

    // Interpolate environment variables before parsing
    let interpolated = env::interpolate_env_vars(&content)?;

    let mut config: SpecadoConfig =
        serde_yaml::from_str(&interpolated).map_err(|e| ConfigError::ParseError {
            path: path.to_string_lossy().to_string(),
            line: e.location().map(|l| l.line()),
            column: e.location().map(|l| l.column()),
            message: e.to_string(),
        })?;

    // Additional interpolation for any remaining env vars
    env::interpolate_config_env_vars(&mut config)?;

    // Use the validator for extended validation
    let validator = ConfigValidator::new();
    validator.validate(&config)?;
    Ok(config)
}

/// Load a configuration from a JSON file
pub fn load_from_json<P: AsRef<Path>>(path: P) -> Result<SpecadoConfig, ConfigError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path).map_err(|e| ConfigError::IoError {
        path: path.to_string_lossy().to_string(),
        source: e,
    })?;

    // Interpolate environment variables before parsing
    let interpolated = env::interpolate_env_vars(&content)?;

    let mut config: SpecadoConfig =
        serde_json::from_str(&interpolated).map_err(|e| ConfigError::ParseError {
            path: path.to_string_lossy().to_string(),
            line: Some(e.line()),
            column: Some(e.column()),
            message: e.to_string(),
        })?;

    // Additional interpolation for any remaining env vars
    env::interpolate_config_env_vars(&mut config)?;

    // Use the validator for extended validation
    let validator = ConfigValidator::new();
    validator.validate(&config)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_yaml() {
        let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: ${OPENAI_API_KEY}
    base_url: https://api.openai.com/v1
    models:
      - id: gpt-4
        max_tokens: 8192
        default_temperature: 0.7
      - id: gpt-3.5-turbo
        max_tokens: 4096
        default_temperature: 0.7
routing:
  strategy: round_robin
  fallback:
    enabled: true
    max_retries: 3
"#;
        let config: Result<SpecadoConfig, _> = serde_yaml::from_str(yaml);
        assert!(config.is_ok());
    }
}
