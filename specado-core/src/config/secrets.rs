//! Secrets management and redaction for configuration
//!
//! This module provides comprehensive secrets handling including:
//! - Field-level sensitivity marking
//! - Automatic redaction in Display/Debug output
//! - Safe logging utilities
//! - Serialization with redaction

use serde::{Deserialize, Serialize};
use std::fmt;

/// A wrapper type for sensitive strings like API keys
#[derive(Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SecretString {
    value: String,
}

impl SecretString {
    /// Create a new secret string
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Get the actual value (use with caution)
    pub fn expose_secret(&self) -> &str {
        &self.value
    }

    /// Check if the secret is empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Get a partially redacted version for debugging
    pub fn partial_redact(&self) -> String {
        if self.value.is_empty() {
            return "[EMPTY]".to_string();
        }

        let len = self.value.len();
        if len <= 8 {
            // Very short secrets get fully redacted
            "[REDACTED]".to_string()
        } else if self.value.starts_with("sk-") || self.value.starts_with("pk-") {
            // API keys with prefixes
            format!("{}...{}", &self.value[..3], &self.value[len - 4..])
        } else {
            // Show first 2 and last 2 characters
            format!(
                "{}...{}",
                &self.value[..2.min(len)],
                &self.value[len.saturating_sub(2)..]
            )
        }
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl PartialEq for SecretString {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecretString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// A trait for types that can be logged safely
pub trait SafeLogging {
    /// Returns a safe version for logging
    fn safe_for_logging(&self) -> String;

    /// Returns a debug version with partial info
    fn safe_debug(&self) -> String {
        self.safe_for_logging()
    }
}

/// Redaction policy configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactionPolicy {
    /// Fully redact all sensitive fields
    Full,
    /// Partially redact showing some info for debugging
    Partial,
    /// No redaction (dangerous - only for secure environments)
    None,
}

impl Default for RedactionPolicy {
    fn default() -> Self {
        Self::Full
    }
}

/// Global redaction policy for the application
static mut REDACTION_POLICY: RedactionPolicy = RedactionPolicy::Full;

/// Set the global redaction policy (use with caution)
///
/// # Safety
/// This function is unsafe because it modifies global state.
/// It should only be called during application initialization.
#[allow(dead_code)]
pub unsafe fn set_redaction_policy(policy: RedactionPolicy) {
    REDACTION_POLICY = policy;
}

/// Get the current redaction policy
pub fn get_redaction_policy() -> RedactionPolicy {
    unsafe { REDACTION_POLICY }
}

/// Redact a string based on field name patterns
pub fn redact_by_field_name(field_name: &str, value: &str) -> String {
    let sensitive_patterns = [
        "api_key",
        "secret",
        "token",
        "password",
        "credential",
        "auth",
        "private",
        "passphrase",
    ];

    let field_lower = field_name.to_lowercase();
    let is_sensitive = sensitive_patterns
        .iter()
        .any(|pattern| field_lower.contains(pattern));

    if is_sensitive {
        match get_redaction_policy() {
            RedactionPolicy::Full => "[REDACTED]".to_string(),
            RedactionPolicy::Partial => {
                if value.len() <= 4 {
                    "[REDACTED]".to_string()
                } else {
                    format!("{}...", &value[..2])
                }
            }
            RedactionPolicy::None => value.to_string(),
        }
    } else {
        value.to_string()
    }
}

/// Helper to create a safe version of any value for logging
pub fn safe_value<T: fmt::Display>(value: &T, is_sensitive: bool) -> String {
    if is_sensitive {
        match get_redaction_policy() {
            RedactionPolicy::Full => "[REDACTED]".to_string(),
            RedactionPolicy::Partial => {
                let s = value.to_string();
                if s.len() > 4 {
                    format!("{}...", &s[..2])
                } else {
                    "[REDACTED]".to_string()
                }
            }
            RedactionPolicy::None => value.to_string(),
        }
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_string_redaction() {
        let secret = SecretString::new("sk-1234567890abcdef");
        assert_eq!(format!("{}", secret), "[REDACTED]");
        assert_eq!(format!("{:?}", secret), "[REDACTED]");
        assert_eq!(secret.partial_redact(), "sk-...cdef");
    }

    #[test]
    fn test_secret_string_expose() {
        let secret = SecretString::new("my-secret-value");
        assert_eq!(secret.expose_secret(), "my-secret-value");
    }

    #[test]
    fn test_redact_by_field_name() {
        // Ensure we're using Full redaction for this test
        unsafe {
            set_redaction_policy(RedactionPolicy::Full);
        }

        assert_eq!(redact_by_field_name("api_key", "sk-123"), "[REDACTED]");
        assert_eq!(redact_by_field_name("name", "John"), "John");
        assert_eq!(redact_by_field_name("password", "pass123"), "[REDACTED]");
        assert_eq!(
            redact_by_field_name("auth_token", "bearer-xyz"),
            "[REDACTED]"
        );

        // Reset to default
        unsafe {
            set_redaction_policy(RedactionPolicy::Full);
        }
    }

    #[test]
    fn test_redaction_policy() {
        // Test Full policy first (since tests may run in parallel, start with Full)
        unsafe {
            set_redaction_policy(RedactionPolicy::Full);
        }
        let result_full = redact_by_field_name("api_key", "sk-1234567890");
        assert_eq!(result_full, "[REDACTED]");

        // Test Partial policy
        unsafe {
            set_redaction_policy(RedactionPolicy::Partial);
        }
        assert_eq!(get_redaction_policy(), RedactionPolicy::Partial);

        let result = redact_by_field_name("api_key", "sk-1234567890");
        assert_eq!(result, "sk...");

        // Test None policy
        unsafe {
            set_redaction_policy(RedactionPolicy::None);
        }
        let result_none = redact_by_field_name("api_key", "sk-1234567890");
        assert_eq!(result_none, "sk-1234567890");

        // Reset to default
        unsafe {
            set_redaction_policy(RedactionPolicy::Full);
        }
    }

    #[test]
    fn test_safe_value() {
        // Ensure we're using Full redaction for this test
        unsafe {
            set_redaction_policy(RedactionPolicy::Full);
        }

        assert_eq!(safe_value(&"sensitive", true), "[REDACTED]");
        assert_eq!(safe_value(&"normal", false), "normal");

        unsafe {
            set_redaction_policy(RedactionPolicy::None);
        }
        assert_eq!(safe_value(&"sensitive", true), "sensitive");

        // Reset
        unsafe {
            set_redaction_policy(RedactionPolicy::Full);
        }
    }
}
