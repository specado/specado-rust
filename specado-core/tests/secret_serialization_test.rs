//! Test that SecretString serialization preserves values for persistence

use serde::{Deserialize, Serialize};
use specado_core::config::SecretString;

#[derive(Serialize, Deserialize, Debug)]
struct TestConfig {
    api_key: SecretString,
    name: String,
}

#[test]
fn test_secret_string_serialization_roundtrip() {
    // Create a config with a secret
    let config = TestConfig {
        api_key: SecretString::new("sk-secret-key-123"),
        name: "test".to_string(),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&config).unwrap();

    // Verify the actual value is in the JSON (not redacted)
    assert!(json.contains("sk-secret-key-123"));
    assert!(!json.contains("[REDACTED]"));

    // Deserialize back
    let deserialized: TestConfig = serde_json::from_str(&json).unwrap();

    // Verify round-trip works
    assert_eq!(deserialized.api_key.expose_secret(), "sk-secret-key-123");
    assert_eq!(deserialized.name, "test");

    // Test Debug output is still redacted
    let debug_output = format!("{:?}", deserialized.api_key);
    assert_eq!(debug_output, "[REDACTED]");

    // Test Display output is still redacted
    let display_output = format!("{}", deserialized.api_key);
    assert_eq!(display_output, "[REDACTED]");
}

#[test]
fn test_yaml_serialization_roundtrip() {
    let config = TestConfig {
        api_key: SecretString::new("my-api-key-value"),
        name: "provider".to_string(),
    };

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&config).unwrap();

    // Verify the actual value is in the YAML (not redacted)
    assert!(yaml.contains("my-api-key-value"));
    assert!(!yaml.contains("[REDACTED]"));

    // Deserialize back
    let deserialized: TestConfig = serde_yaml::from_str(&yaml).unwrap();

    // Verify round-trip works
    assert_eq!(deserialized.api_key.expose_secret(), "my-api-key-value");
    assert_eq!(deserialized.name, "provider");
}
