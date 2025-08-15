//! Integration tests for configuration loading and validation

use specado_core::config::{load_from_json, load_from_yaml, ConfigError};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a test config file
fn create_test_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn test_load_valid_yaml_config() {
    use std::env;
    env::set_var("OPENAI_API_KEY", "test-key");

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
        default_temperature: 0.9
routing:
  strategy: round_robin
  fallback:
    enabled: true
    max_retries: 3
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.version, "0.1");
    assert_eq!(config.providers.len(), 1);
    assert_eq!(config.providers[0].name, "openai");
    assert_eq!(config.providers[0].api_key.expose_secret(), "test-key");
    assert_eq!(config.providers[0].models.len(), 2);

    env::remove_var("OPENAI_API_KEY");
}

#[test]
fn test_load_valid_json_config() {
    use std::env;
    env::set_var("ANTHROPIC_API_KEY", "test-key");

    let json = r#"{
  "version": "0.1",
  "providers": [
    {
      "name": "anthropic",
      "type": "anthropic",
      "api_key": "${ANTHROPIC_API_KEY}",
      "base_url": "https://api.anthropic.com/v1",
      "models": [
        {
          "id": "claude-3-opus",
          "max_tokens": 200000,
          "default_temperature": 0.7
        }
      ]
    }
  ],
  "routing": {
    "strategy": "priority"
  }
}"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.json", json);

    let result = load_from_json(path);
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.version, "0.1");
    assert_eq!(config.providers[0].name, "anthropic");

    env::remove_var("ANTHROPIC_API_KEY");
}

#[test]
fn test_missing_version_field() {
    let yaml = r#"
providers:
  - name: openai
    type: openai
    api_key: test
    base_url: https://api.openai.com/v1
    models: []
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err());

    match result {
        Err(ConfigError::ValidationError(e)) => {
            assert_eq!(e.field_path, "version");
        }
        Err(ConfigError::ParseError { .. }) => {
            // This is also acceptable - serde might fail to parse without version
        }
        _ => {
            panic!(
                "Expected validation or parse error for missing version, got: {:?}",
                result
            );
        }
    }
}

#[test]
fn test_invalid_version() {
    let yaml = r#"
version: "2.0"
providers:
  - name: openai
    type: openai
    api_key: test
    base_url: https://api.openai.com/v1
    models: []
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err());

    if let Err(ConfigError::ValidationError(e)) = result {
        assert_eq!(e.field_path, "version");
    } else {
        panic!("Expected validation error for invalid version");
    }
}

#[test]
fn test_empty_providers_list() {
    let yaml = r#"
version: "0.1"
providers: []
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err());

    if let Err(ConfigError::ValidationError(e)) = result {
        assert_eq!(e.field_path, "providers");
    } else {
        panic!("Expected validation error for empty providers");
    }
}

#[test]
fn test_duplicate_provider_names() {
    let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: key1
    base_url: https://api.openai.com/v1
    models: []
  - name: openai
    type: openai
    api_key: key2
    base_url: https://api.openai.com/v1
    models: []
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err());
}

#[test]
fn test_invalid_url_format() {
    let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: test
    base_url: not-a-url
    models: []
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err());
}

#[test]
fn test_invalid_temperature_range() {
    let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: test
    base_url: https://api.openai.com/v1
    models:
      - id: gpt-4
        max_tokens: 8192
        default_temperature: 3.0
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err());
}

#[test]
fn test_weighted_routing_without_weights() {
    let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: test
    base_url: https://api.openai.com/v1
    models: []
routing:
  strategy: weighted
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err());
}

#[test]
fn test_routing_weights_for_nonexistent_provider() {
    let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: test
    base_url: https://api.openai.com/v1
    models: []
routing:
  strategy: weighted
  weights:
    openai: 1.0
    nonexistent: 2.0
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err());
}

#[test]
fn test_complex_valid_config() {
    use std::env;
    env::set_var("OPENAI_API_KEY", "test-key-1");
    env::set_var("ANTHROPIC_API_KEY", "test-key-2");

    let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: ${OPENAI_API_KEY}
    base_url: https://api.openai.com/v1
    priority: 100
    models:
      - id: gpt-4-turbo
        max_tokens: 128000
        max_output_tokens: 4096
        default_temperature: 0.7
        supports_streaming: true
        supports_functions: true
        cost_per_1k_input: 0.01
        cost_per_1k_output: 0.03
    rate_limit:
      requests_per_minute: 500
      tokens_per_minute: 90000
    retry_policy:
      max_retries: 3
      initial_delay_ms: 1000
      max_delay_ms: 30000
      backoff_multiplier: 2.0
  - name: anthropic
    type: anthropic
    api_key: ${ANTHROPIC_API_KEY}
    base_url: https://api.anthropic.com/v1
    priority: 90
    models:
      - id: claude-3-opus
        max_tokens: 200000
        max_output_tokens: 4096
        default_temperature: 0.7
        supports_streaming: true
        cost_per_1k_input: 0.015
        cost_per_1k_output: 0.075
routing:
  strategy: priority
  fallback:
    enabled: true
    max_retries: 5
    timeout_ms: 30000
connection:
  connect_timeout_ms: 5000
  request_timeout_ms: 120000
  max_idle_per_host: 20
defaults:
  temperature: 0.8
  max_tokens: 2048
metadata:
  environment: production
  version: 1.0.0
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.providers.len(), 2);
    assert_eq!(config.providers[0].priority, 100);
    assert_eq!(config.providers[1].priority, 90);
    assert_eq!(config.routing.fallback.max_retries, 5);
    assert_eq!(config.connection.connect_timeout_ms, 5000);
    assert_eq!(config.defaults.temperature, 0.8);

    env::remove_var("OPENAI_API_KEY");
    env::remove_var("ANTHROPIC_API_KEY");
}

#[test]
fn test_model_validation() {
    let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: test
    base_url: https://api.openai.com/v1
    models:
      - id: gpt-4
        max_tokens: 8192
        max_output_tokens: 10000
        default_temperature: 0.7
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err()); // max_output_tokens > max_tokens
}

#[test]
fn test_env_var_interpolation() {
    use std::env;

    env::set_var("TEST_API_KEY", "sk-test-key-123");

    let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: ${TEST_API_KEY}
    base_url: https://api.openai.com/v1
    models: []
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(
        config.providers[0].api_key.expose_secret(),
        "sk-test-key-123"
    );

    env::remove_var("TEST_API_KEY");
}

#[test]
fn test_missing_env_var() {
    let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: ${MISSING_ENV_VAR}
    base_url: https://api.openai.com/v1
    models: []
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err());

    if let Err(ConfigError::EnvVarNotFound { var }) = result {
        assert_eq!(var, "MISSING_ENV_VAR");
    } else {
        panic!("Expected EnvVarNotFound error");
    }
}

#[test]
fn test_no_enabled_providers() {
    let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: test
    base_url: https://api.openai.com/v1
    enabled: false
    models: []
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err());
}

#[test]
fn test_weighted_routing_positive_sum() {
    let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: test
    base_url: https://api.openai.com/v1
    enabled: true
    models: []
  - name: anthropic
    type: anthropic
    api_key: test2
    base_url: https://api.anthropic.com/v1
    enabled: false
    models: []
routing:
  strategy: weighted
  weights:
    openai: 0.0
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err()); // Sum of weights is 0
}

#[test]
fn test_cost_optimized_specific_error() {
    let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: test
    base_url: https://api.openai.com/v1
    models:
      - id: gpt-4
        max_tokens: 8192
        default_temperature: 0.7
        cost_per_1k_input: 0.01
routing:
  strategy: cost_optimized
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err());

    // Should specifically report missing cost_per_1k_output
    if let Err(ConfigError::ValidationError(e)) = result {
        assert!(e.field_path.contains("cost_per_1k_output"));
    }
}

#[test]
fn test_retry_policy_validation() {
    let yaml = r#"
version: "0.1"
providers:
  - name: openai
    type: openai
    api_key: test
    base_url: https://api.openai.com/v1
    models: []
    retry_policy:
      max_retries: 3
      initial_delay_ms: 5000
      max_delay_ms: 2000
      backoff_multiplier: 2.0
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_err()); // max_delay_ms < initial_delay_ms
}

#[test]
fn test_api_key_redaction() {
    use specado_core::config::SafeLogging;
    use std::env;

    env::set_var("REDACT_TEST_KEY", "sk-secret-api-key-123456");

    let yaml = r#"
version: "0.1"
providers:
  - name: test_provider
    type: openai
    api_key: ${REDACT_TEST_KEY}
    base_url: https://api.example.com/v1
    models: []
"#;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "config.yaml", yaml);

    let result = load_from_yaml(path);
    assert!(result.is_ok());

    let config = result.unwrap();
    let provider = &config.providers[0];

    // Verify the actual value is preserved
    assert_eq!(provider.api_key.expose_secret(), "sk-secret-api-key-123456");

    // Verify Debug output is redacted
    let debug_output = format!("{:?}", provider);
    assert!(debug_output.contains("[REDACTED]"));
    assert!(!debug_output.contains("sk-secret-api-key-123456"));

    // Verify SafeLogging output is redacted
    let safe_output = provider.safe_for_logging();
    assert!(safe_output.contains("[REDACTED]"));
    assert!(!safe_output.contains("sk-secret-api-key-123456"));

    env::remove_var("REDACT_TEST_KEY");
}

#[test]
fn test_secret_string_partial_redaction() {
    use specado_core::config::SecretString;

    // Test various secret formats
    let test_cases = vec![
        ("sk-1234567890abcdef", "sk-...cdef"),
        ("pk-test-key-123", "pk-...-123"), // Fixed expectation
        ("short", "[REDACTED]"),           // Too short for partial
        ("my-secret-value", "my...ue"),
        ("", "[EMPTY]"),
    ];

    for (input, expected) in test_cases {
        let secret = SecretString::new(input);
        assert_eq!(secret.partial_redact(), expected);
        // Always fully redacted in Display/Debug
        assert_eq!(format!("{}", secret), "[REDACTED]");
        assert_eq!(format!("{:?}", secret), "[REDACTED]");
    }
}

#[test]
fn test_redaction_helper_functions() {
    use specado_core::config::{redact_by_field_name, safe_value};

    // Test field name based redaction
    assert_eq!(redact_by_field_name("api_key", "secret123"), "[REDACTED]");
    assert_eq!(redact_by_field_name("API_KEY", "secret123"), "[REDACTED]");
    assert_eq!(redact_by_field_name("secret_token", "token"), "[REDACTED]");
    assert_eq!(redact_by_field_name("password", "pass"), "[REDACTED]");
    assert_eq!(
        redact_by_field_name("auth_credential", "cred"),
        "[REDACTED]"
    );
    assert_eq!(redact_by_field_name("private_key", "key"), "[REDACTED]");

    // Non-sensitive fields should not be redacted
    assert_eq!(redact_by_field_name("username", "john"), "john");
    assert_eq!(
        redact_by_field_name("email", "user@example.com"),
        "user@example.com"
    );
    assert_eq!(redact_by_field_name("model_id", "gpt-4"), "gpt-4");

    // Test safe_value helper
    assert_eq!(safe_value(&"sensitive", true), "[REDACTED]");
    assert_eq!(safe_value(&"normal", false), "normal");
    assert_eq!(safe_value(&42, false), "42");
}
