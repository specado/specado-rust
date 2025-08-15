//! Environment variable interpolation for configuration

use super::error::ConfigError;
use super::secrets::SecretString;
use regex::Regex;
use std::env;

/// Interpolate environment variables in a configuration string
pub fn interpolate_env_vars(content: &str) -> Result<String, ConfigError> {
    let env_var_pattern = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").unwrap();
    let mut result = content.to_string();
    let mut missing_vars = Vec::new();

    // Find all environment variable references
    for cap in env_var_pattern.captures_iter(content) {
        let full_match = cap.get(0).unwrap().as_str();
        let var_name = &cap[1];

        match env::var(var_name) {
            Ok(value) => {
                result = result.replace(full_match, &value);
            }
            Err(_) => {
                missing_vars.push(var_name.to_string());
            }
        }
    }

    // Report the first missing variable (to match the error type)
    if let Some(var) = missing_vars.first() {
        return Err(ConfigError::EnvVarNotFound { var: var.clone() });
    }

    Ok(result)
}

/// Interpolate environment variables in a SpecadoConfig after loading
/// This is a more targeted approach that only interpolates specific fields
pub fn interpolate_config_env_vars(
    config: &mut super::schema::SpecadoConfig,
) -> Result<(), ConfigError> {
    let env_var_pattern = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").unwrap();

    // Interpolate provider API keys
    for provider in &mut config.providers {
        let api_key_str = provider.api_key.expose_secret();
        if env_var_pattern.is_match(api_key_str) {
            let interpolated = interpolate_single_value(api_key_str)?;
            provider.api_key = SecretString::new(interpolated);
        }

        // Also check base_url in case it uses env vars
        if env_var_pattern.is_match(&provider.base_url) {
            provider.base_url = interpolate_single_value(&provider.base_url)?;
        }
    }

    Ok(())
}

/// Interpolate a single value that may contain environment variables
fn interpolate_single_value(value: &str) -> Result<String, ConfigError> {
    let env_var_pattern = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").unwrap();

    if let Some(cap) = env_var_pattern.captures(value) {
        let var_name = &cap[1];
        match env::var(var_name) {
            Ok(env_value) => Ok(value.replace(&cap[0], &env_value)),
            Err(_) => Err(ConfigError::EnvVarNotFound {
                var: var_name.to_string(),
            }),
        }
    } else {
        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_env_vars() {
        env::set_var("TEST_VAR", "test_value");

        let content = "api_key: ${TEST_VAR}";
        let result = interpolate_env_vars(content).unwrap();
        assert_eq!(result, "api_key: test_value");

        env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_missing_env_var() {
        let content = "api_key: ${MISSING_VAR}";
        let result = interpolate_env_vars(content);

        assert!(result.is_err());
        if let Err(ConfigError::EnvVarNotFound { var }) = result {
            assert_eq!(var, "MISSING_VAR");
        } else {
            panic!("Expected EnvVarNotFound error");
        }
    }

    #[test]
    fn test_multiple_env_vars() {
        env::set_var("VAR1", "value1");
        env::set_var("VAR2", "value2");

        let content = "key1: ${VAR1}, key2: ${VAR2}";
        let result = interpolate_env_vars(content).unwrap();
        assert_eq!(result, "key1: value1, key2: value2");

        env::remove_var("VAR1");
        env::remove_var("VAR2");
    }
}
